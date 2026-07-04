#[path = "support/mock_rpc.rs"]
mod mock_rpc;

use adad_core::Error;
use mock_rpc::MockWalletRpc;
use xmr_wallet::WalletRpcClient;

#[test]
fn failure_wallet_rpc_error_returns_typed_error_without_success_payload() {
    let transport = MockWalletRpc::with_error();
    let records = transport.records();
    let mut client = WalletRpcClient::new("http://127.0.0.1:18082/json_rpc", transport);

    let error = client.balance().expect_err("RPC error should fail");

    assert_eq!(error, Error::WalletRpc);
    let records = records.lock().expect("records should lock");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].body["method"], "get_balance");
}
