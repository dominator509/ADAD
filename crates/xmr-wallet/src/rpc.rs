use adad_core::Error;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};

pub trait WalletRpcTransport {
    fn post_json(&mut self, url: &str, body: &str) -> Result<String, Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletRpcClient<T> {
    rpc_url: String,
    transport: T,
}

impl<T> WalletRpcClient<T>
where
    T: WalletRpcTransport,
{
    #[must_use]
    pub fn new(rpc_url: impl Into<String>, transport: T) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            transport,
        }
    }

    pub fn balance(&mut self) -> Result<Balance, Error> {
        self.call("get_balance", json!({ "account_index": 0 }))
    }

    pub fn address(&mut self) -> Result<WalletAddress, Error> {
        self.call("get_address", json!({ "account_index": 0 }))
    }

    pub fn prepare_transfer(
        &mut self,
        address: impl Into<String>,
        amount_atomic: u64,
    ) -> Result<PreparedTransfer, Error> {
        self.call(
            "transfer",
            json!({
                "destinations": [{
                    "amount": amount_atomic,
                    "address": address.into(),
                }],
                "account_index": 0,
                "do_not_relay": true,
                "get_tx_metadata": true,
            }),
        )
    }

    fn call<R>(&mut self, method: &str, params: Value) -> Result<R, Error>
    where
        R: DeserializeOwned,
    {
        let body = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params,
        })
        .to_string();
        let response = self.transport.post_json(&self.rpc_url, &body)?;
        let envelope: JsonRpcResponse<R> =
            serde_json::from_str(&response).map_err(|_| Error::WalletRpc)?;

        match (envelope.result, envelope.error) {
            (Some(result), None) => Ok(result),
            _ => Err(Error::WalletRpc),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Balance {
    pub balance: u64,
    pub unlocked_balance: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WalletAddress {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PreparedTransfer {
    pub amount: u64,
    pub fee: u64,
    pub tx_hash: String,
    pub tx_metadata: String,
}

#[derive(Deserialize)]
struct JsonRpcResponse<R> {
    result: Option<R>,
    error: Option<Value>,
}
