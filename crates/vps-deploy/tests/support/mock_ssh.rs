use adad_core::Error;
use vps_deploy::{ProvisionTarget, SshOutput, SshSession};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SshCall {
    pub target: ProvisionTarget,
    pub setup_script: String,
}

#[derive(Clone, Debug)]
pub struct MockSsh {
    pub calls: Vec<SshCall>,
    output: SshOutput,
}

impl MockSsh {
    #[allow(dead_code)]
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            calls: Vec::new(),
            output: SshOutput::success(stdout),
        }
    }

    pub fn failure(exit_status: i32, stdout: impl Into<String>) -> Self {
        Self {
            calls: Vec::new(),
            output: SshOutput {
                exit_status,
                stdout: stdout.into(),
            },
        }
    }
}

impl SshSession for MockSsh {
    fn run_setup_script(
        &mut self,
        target: &ProvisionTarget,
        setup_script: &str,
    ) -> Result<SshOutput, Error> {
        self.calls.push(SshCall {
            target: target.clone(),
            setup_script: setup_script.to_owned(),
        });
        Ok(self.output.clone())
    }
}
