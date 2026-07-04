use std::time::Duration;

use adad_core::Error;
use vps_deploy::{
    run_headless, ProvisionHandle, ProvisionTarget, VpsAction, VpsEvent, VpsViewState,
};

#[test]
fn vps_empty_state_renders_keyboard_actions() {
    let log = run_headless(&[VpsEvent::Render]).expect("VPS TUI should render");

    assert_eq!(log.state, VpsViewState::Empty);
    assert!(log.frames[0].contains("ADAD VPS Deploy"));
    assert!(log.frames[0].contains("State: Empty"));
    assert!(log.frames[0].contains("Keys: p provision"));
}

#[test]
fn vps_provision_action_is_keyboard_reachable() {
    let target = ProvisionTarget::new("mock-hidden-service.onion", "debian", 22);
    let handle = ProvisionHandle {
        target: target.clone(),
        stdout: "forgejo ready".to_owned(),
        elapsed: Duration::from_secs(1),
    };
    let log = run_headless(&[
        VpsEvent::SetTarget(target),
        VpsEvent::Key('p'),
        VpsEvent::Provisioned(handle),
    ])
    .expect("VPS provision script should render");

    assert_eq!(log.actions, vec![VpsAction::Provision]);
    assert_eq!(log.state, VpsViewState::Ready);
    assert!(log.frames[1].contains("State: Loading"));
    assert!(log.frames[2].contains("Provisioned: mock-hidden-service.[REDACTED]"));
    assert!(log.frames[2].contains("Output: forgejo ready"));
    assert!(!log.frames[2].contains("mock-hidden-service.onion"));
}

#[test]
fn vps_error_state_uses_redacted_message() {
    let log =
        run_headless(&[VpsEvent::Error(Error::VpsProvision)]).expect("VPS error should render");

    assert_eq!(log.state, VpsViewState::Error);
    assert!(log.frames[0].contains("State: Error - Provisioning failed"));
}
