use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use adad_core::{ConfigField, Error};

use crate::TunnelHealth;

pub const WG_INTERFACE: &str = "wg0";
const DEFAULT_CONFIG_PATH: &str = "/run/adad/wg0.conf";

/// Result of running one of the small, fixed system commands used by the
/// WireGuard adapter. The adapter intentionally discards stdout/stderr so
/// that a private key or endpoint cannot enter application logs.
pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&OsStr]) -> Result<bool, Error>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[&OsStr]) -> Result<bool, Error> {
        Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .map_err(|_| Error::Killswitch)
    }
}

/// Production WireGuard lifecycle adapter.
///
/// The vault/runtime is responsible for materializing the private config at
/// `ADAD_WG_CONF`. This adapter only passes that path to `wg-quick`; it never
/// parses, copies, logs, or persists the configuration contents.
#[derive(Debug)]
pub struct WireGuardController<R = SystemCommandRunner> {
    config_path: PathBuf,
    runner: R,
}

impl WireGuardController<SystemCommandRunner> {
    #[must_use]
    pub fn default_config() -> Self {
        Self::with_runner(PathBuf::from(DEFAULT_CONFIG_PATH), SystemCommandRunner)
    }

    pub fn from_env() -> Result<Self, Error> {
        let path = std::env::var_os("ADAD_WG_CONF").ok_or(Error::Config {
            field: ConfigField::WgConf,
        })?;
        Ok(Self::with_runner(PathBuf::from(path), SystemCommandRunner))
    }
}

impl Default for WireGuardController<SystemCommandRunner> {
    fn default() -> Self {
        Self::default_config()
    }
}

impl<R: CommandRunner> WireGuardController<R> {
    #[must_use]
    pub fn with_runner(config_path: PathBuf, runner: R) -> Self {
        Self {
            config_path,
            runner,
        }
    }

    /// Return the observed interface state without assuming that an absent
    /// tool or an incomplete observation is safe.
    #[must_use]
    pub fn status(&self) -> TunnelHealth {
        let interface_up = match self.runner.run(
            "ip",
            &[
                OsStr::new("link"),
                OsStr::new("show"),
                OsStr::new("dev"),
                OsStr::new(WG_INTERFACE),
                OsStr::new("up"),
            ],
        ) {
            Ok(active) => active,
            Err(_) => return TunnelHealth::Unknown,
        };
        if !interface_up {
            return TunnelHealth::Inactive;
        }

        match self
            .runner
            .run("wg", &[OsStr::new("show"), OsStr::new(WG_INTERFACE)])
        {
            Ok(true) => TunnelHealth::Active,
            Ok(false) => TunnelHealth::Inactive,
            Err(_) => TunnelHealth::Unknown,
        }
    }

    /// Bring up the vault-supplied configuration and verify the resulting
    /// interface before reporting success. A failed verification triggers a
    /// best-effort teardown so a half-created interface is not left behind.
    pub fn up(&self) -> Result<(), Error> {
        self.validate_config_path()?;

        match self.status() {
            TunnelHealth::Active => return Ok(()),
            TunnelHealth::Unknown => return Err(Error::Killswitch),
            TunnelHealth::Inactive => {}
        }

        let succeeded = self.runner.run(
            "wg-quick",
            &[OsStr::new("up"), self.config_path.as_os_str()],
        )?;
        if !succeeded || self.status() != TunnelHealth::Active {
            let _ = self
                .runner
                .run("wg-quick", &[OsStr::new("down"), OsStr::new(WG_INTERFACE)]);
            return Err(Error::Killswitch);
        }
        Ok(())
    }

    /// Tear down the named interface and verify that it is gone. The down
    /// operation uses the fixed interface name so teardown still works after
    /// the vault-backed config has been removed from the runtime filesystem.
    pub fn down(&self) -> Result<(), Error> {
        match self.status() {
            TunnelHealth::Inactive => return Ok(()),
            TunnelHealth::Unknown => return Err(Error::Killswitch),
            TunnelHealth::Active => {}
        }

        if !self
            .runner
            .run("wg-quick", &[OsStr::new("down"), OsStr::new(WG_INTERFACE)])?
        {
            return Err(Error::Killswitch);
        }
        if self.status() == TunnelHealth::Inactive {
            Ok(())
        } else {
            Err(Error::Killswitch)
        }
    }

    fn validate_config_path(&self) -> Result<(), Error> {
        let path = &self.config_path;
        if !path.is_absolute()
            || path.file_name() != Some(OsStr::new("wg0.conf"))
            || path.parent() != Some(Path::new("/run/adad"))
        {
            return Err(Error::Config {
                field: ConfigField::WgConf,
            });
        }

        let metadata = fs::symlink_metadata(path).map_err(|_| Error::Config {
            field: ConfigField::WgConf,
        })?;
        if !metadata.file_type().is_file() {
            return Err(Error::Config {
                field: ConfigField::WgConf,
            });
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};

            let parent_metadata = fs::symlink_metadata("/run/adad").map_err(|_| Error::Config {
                field: ConfigField::WgConf,
            })?;
            let file_mode = metadata.permissions().mode() & 0o777;
            let parent_mode = parent_metadata.permissions().mode() & 0o777;
            if file_mode != 0o600
                || metadata.uid() != 0
                || parent_mode & 0o022 != 0
                || parent_metadata.uid() != 0
                || !parent_metadata.file_type().is_dir()
            {
                return Err(Error::Config {
                    field: ConfigField::WgConf,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, ffi::OsStr, path::PathBuf};

    use super::{CommandRunner, WireGuardController, WG_INTERFACE};
    use crate::TunnelHealth;
    use adad_core::Error;

    struct FakeRunner {
        responses: RefCell<VecDeque<Result<bool, Error>>>,
        calls: RefCell<Vec<(String, Vec<String>)>>,
    }

    impl FakeRunner {
        fn new(responses: impl IntoIterator<Item = Result<bool, Error>>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().collect()),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, program: &str, args: &[&OsStr]) -> Result<bool, Error> {
            self.calls.borrow_mut().push((
                program.to_owned(),
                args.iter()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect(),
            ));
            self.responses
                .borrow_mut()
                .pop_front()
                .expect("test response available")
        }
    }

    #[test]
    fn status_requires_both_ip_and_wireguard_observations() {
        let runner = FakeRunner::new([Ok(true), Ok(true)]);
        let controller =
            WireGuardController::with_runner(PathBuf::from("/run/adad/wg0.conf"), runner);

        assert_eq!(controller.status(), TunnelHealth::Active);
        let calls = controller.runner.calls.borrow();
        assert_eq!(calls[0].0, "ip");
        assert_eq!(calls[0].1, ["link", "show", "dev", WG_INTERFACE, "up"]);
        assert_eq!(calls[1].0, "wg");
        assert_eq!(calls[1].1, ["show", WG_INTERFACE]);
    }

    #[test]
    fn missing_observer_is_unknown_not_inactive_or_active() {
        let runner = FakeRunner::new([Err(Error::Killswitch)]);
        let controller =
            WireGuardController::with_runner(PathBuf::from("/run/adad/wg0.conf"), runner);

        assert_eq!(controller.status(), TunnelHealth::Unknown);
    }

    #[test]
    fn status_does_not_claim_a_down_interface_is_ready() {
        let runner = FakeRunner::new([Ok(false)]);
        let controller =
            WireGuardController::with_runner(PathBuf::from("/run/adad/wg0.conf"), runner);

        assert_eq!(controller.status(), TunnelHealth::Inactive);
    }

    #[test]
    fn lifecycle_rejects_any_config_path_other_than_runtime_wg0() {
        let runner = FakeRunner::new([]);
        let controller =
            WireGuardController::with_runner(PathBuf::from("/tmp/private-wg0.conf"), runner);

        assert_eq!(
            controller.validate_config_path(),
            Err(Error::Config {
                field: adad_core::ConfigField::WgConf,
            })
        );
    }
}
