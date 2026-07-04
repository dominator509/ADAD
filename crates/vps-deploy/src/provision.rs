use std::time::{Duration, Instant};

use adad_core::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionTarget {
    pub host: String,
    pub user: String,
    pub port: u16,
}

impl ProvisionTarget {
    #[must_use]
    pub fn new(host: impl Into<String>, user: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            user: user.into(),
            port,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SshOutput {
    pub exit_status: i32,
    pub stdout: String,
}

impl SshOutput {
    #[must_use]
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            exit_status: 0,
            stdout: stdout.into(),
        }
    }
}

pub trait SshSession {
    fn run_setup_script(
        &mut self,
        target: &ProvisionTarget,
        setup_script: &str,
    ) -> Result<SshOutput, Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionHandle {
    pub target: ProvisionTarget,
    pub stdout: String,
    pub elapsed: Duration,
}

pub fn provision(
    session: &mut impl SshSession,
    target: ProvisionTarget,
    setup_script: &str,
) -> Result<ProvisionHandle, Error> {
    if target.host.trim().is_empty()
        || target.user.trim().is_empty()
        || target.port == 0
        || setup_script.trim().is_empty()
    {
        return Err(Error::VpsProvision);
    }

    let started = Instant::now();
    let output = session.run_setup_script(&target, setup_script)?;
    if output.exit_status != 0 {
        return Err(Error::VpsProvision);
    }

    Ok(ProvisionHandle {
        target,
        stdout: output.stdout,
        elapsed: started.elapsed(),
    })
}
