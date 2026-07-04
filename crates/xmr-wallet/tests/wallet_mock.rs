#[path = "support/mock_rpc.rs"]
mod mock_rpc;

use adad_core::Error;
use mock_rpc::MockWalletRpc;
use xmr_wallet::WalletRpcClient;

#[test]
fn wallet_methods_use_monero_json_rpc_shape_against_mock() {
    let transport = MockWalletRpc::default();
    let records = transport.records();
    let mut client = WalletRpcClient::new("http://127.0.0.1:18082/json_rpc", transport);

    let balance = client.balance().expect("balance should parse");
    let address = client.address().expect("address should parse");
    let prepared = client
        .prepare_transfer("55mockDestination", 42_000)
        .expect("prepared transfer should parse");

    assert_eq!(balance.balance, 100_000);
    assert_eq!(balance.unlocked_balance, 90_000);
    assert_eq!(address.address, "55mockPrimary");
    assert_eq!(prepared.tx_hash, "mock_tx_hash");
    assert_eq!(prepared.tx_metadata, "mock_tx_metadata");

    let records = records.lock().expect("records should lock");
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].body["jsonrpc"], "2.0");
    assert_eq!(records[0].body["method"], "get_balance");
    assert_eq!(records[0].body["params"]["account_index"], 0);
    assert_eq!(records[1].body["method"], "get_address");
    assert_eq!(records[2].body["method"], "transfer");
    assert_eq!(
        records[2].body["params"]["destinations"][0]["amount"],
        42_000
    );
    assert_eq!(
        records[2].body["params"]["destinations"][0]["address"],
        "55mockDestination"
    );
    assert_eq!(records[2].body["params"]["do_not_relay"], true);
    assert_eq!(records[2].body["params"]["get_tx_metadata"], true);
}

#[test]
fn wallet_rpc_error_maps_to_typed_error() {
    let transport = MockWalletRpc::with_error();
    let mut client = WalletRpcClient::new("http://127.0.0.1:18082/json_rpc", transport);

    let error = client.balance().expect_err("mock RPC error should fail");

    assert_eq!(error, Error::WalletRpc);
}
