#[path = "support/mock_ssh.rs"]
mod mock_ssh;

use adad_core::Error;
use mock_ssh::MockSsh;
use vps_deploy::{provision, ProvisionTarget};

#[test]
fn failure_ssh_nonzero_exit_returns_typed_error_after_one_attempt() {
    let mut ssh = MockSsh::failure(1, "setup failed");
    let target = ProvisionTarget::new("mock-hidden-service.onion", "debian", 22);

    let error =
        provision(&mut ssh, target.clone(), "install-forgejo").expect_err("SSH should fail");

    assert_eq!(error, Error::VpsProvision);
    assert_eq!(ssh.calls.len(), 1);
    assert_eq!(ssh.calls[0].target, target);
}
