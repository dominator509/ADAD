use adad_core::Error;
use xmr_wallet::{
    run_headless, Balance, WalletAction, WalletAddress, WalletEvent, WalletViewState,
};

#[test]
fn wallet_empty_state_renders_keyboard_actions() {
    let log = run_headless(&[WalletEvent::Render]).expect("wallet TUI should render");

    assert_eq!(log.state, WalletViewState::Empty);
    assert!(log.frames[0].contains("ADAD Wallet"));
    assert!(log.frames[0].contains("State: Empty"));
    assert!(log.frames[0].contains("Keys: b balance, a address"));
}

#[test]
fn wallet_balance_action_is_keyboard_reachable() {
    let log = run_headless(&[
        WalletEvent::Key('b'),
        WalletEvent::BalanceLoaded(Balance {
            balance: 100_000,
            unlocked_balance: 90_000,
        }),
    ])
    .expect("wallet balance script should render");

    assert_eq!(log.actions, vec![WalletAction::Balance]);
    assert_eq!(log.state, WalletViewState::Ready);
    assert!(log.frames[0].contains("State: Loading"));
    assert!(log.frames[1].contains("Balance: 100000"));
    assert!(log.frames[1].contains("Unlocked: 90000"));
}

#[test]
fn wallet_address_action_redacts_address() {
    let log = run_headless(&[
        WalletEvent::Key('a'),
        WalletEvent::AddressLoaded(WalletAddress {
            address: "55mockPrimaryAddressShouldNotRenderFully".to_owned(),
        }),
    ])
    .expect("wallet address script should render");

    assert_eq!(log.actions, vec![WalletAction::Address]);
    assert!(log.frames[1].contains("55mo...[REDACTED]"));
    assert!(!log.frames[1].contains("55mockPrimaryAddressShouldNotRenderFully"));
}

#[test]
fn wallet_error_state_uses_redacted_message() {
    let log =
        run_headless(&[WalletEvent::Error(Error::WalletRpc)]).expect("wallet error should render");

    assert_eq!(log.state, WalletViewState::Error);
    assert!(log.frames[0].contains("State: Error - Wallet operation failed"));
}
