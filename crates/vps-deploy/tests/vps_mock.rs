#[path = "support/mock_ssh.rs"]
mod mock_ssh;

use std::time::Duration;

use adad_core::Error;
use mock_ssh::MockSsh;
use vps_deploy::{provision, ProvisionTarget};

#[test]
fn provision_runs_setup_script_against_mock_ssh_under_two_minutes() {
    let mut ssh = MockSsh::success("forgejo ready");
    let target = ProvisionTarget::new("mock-hidden-service.onion", "debian", 22);

    let handle = provision(&mut ssh, target.clone(), "install-forgejo")
        .expect("mock provisioning should succeed");

    assert_eq!(handle.target, target);
    assert_eq!(handle.stdout, "forgejo ready");
    assert!(handle.elapsed < Duration::from_secs(120));
    assert_eq!(ssh.calls.len(), 1);
    assert_eq!(ssh.calls[0].target, target);
    assert_eq!(ssh.calls[0].setup_script, "install-forgejo");
}

#[test]
fn provision_failure_maps_to_typed_error() {
    let mut ssh = MockSsh::failure(1, "setup failed");
    let target = ProvisionTarget::new("mock-hidden-service.onion", "debian", 22);

    let error =
        provision(&mut ssh, target, "install-forgejo").expect_err("nonzero SSH result should fail");

    assert_eq!(error, Error::VpsProvision);
}
