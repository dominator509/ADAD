use std::sync::{Arc, Mutex};

use adad_core::Error;
use serde_json::{json, Value};
use xmr_wallet::WalletRpcTransport;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RecordedRpc {
    pub url: String,
    pub body: Value,
}

#[derive(Clone, Debug, Default)]
pub struct MockWalletRpc {
    records: Arc<Mutex<Vec<RecordedRpc>>>,
    force_error: bool,
}

impl MockWalletRpc {
    pub fn with_error() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
            force_error: true,
        }
    }

    pub fn records(&self) -> Arc<Mutex<Vec<RecordedRpc>>> {
        Arc::clone(&self.records)
    }
}

impl WalletRpcTransport for MockWalletRpc {
    fn post_json(&mut self, url: &str, body: &str) -> Result<String, Error> {
        let body: Value = serde_json::from_str(body).map_err(|_| Error::WalletRpc)?;
        self.records
            .lock()
            .expect("records should lock")
            .push(RecordedRpc {
                url: url.to_owned(),
                body: body.clone(),
            });

        if self.force_error {
            return Ok(json!({
                "jsonrpc": "2.0",
                "id": "0",
                "error": { "code": -1, "message": "mock error" }
            })
            .to_string());
        }

        match body["method"].as_str() {
            Some("get_balance") => Ok(json!({
                "jsonrpc": "2.0",
                "id": "0",
                "result": {
                    "balance": 100000,
                    "unlocked_balance": 90000
                }
            })
            .to_string()),
            Some("get_address") => Ok(json!({
                "jsonrpc": "2.0",
                "id": "0",
                "result": {
                    "address": "55mockPrimary"
                }
            })
            .to_string()),
            Some("transfer") => Ok(json!({
                "jsonrpc": "2.0",
                "id": "0",
                "result": {
                    "amount": 42000,
                    "fee": 1000,
                    "tx_hash": "mock_tx_hash",
                    "tx_metadata": "mock_tx_metadata"
                }
            })
            .to_string()),
            _ => Err(Error::WalletRpc),
        }
    }
}
