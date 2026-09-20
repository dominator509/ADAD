use std::env;

use adad_core::Error;
use xmr_wallet::{UreqWalletRpcTransport, WalletRpcClient};

const DEFAULT_RPC_URL: &str = "http://127.0.0.1:18082/json_rpc";

fn main() {
    if let Err(error) = run() {
        eprintln!("xmr-wallet: {error}");
        std::process::exit(error.exit_code());
    }
}

fn run() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "--help".to_owned());
    match command.as_str() {
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "--version" | "-V" => {
            println!("xmr-wallet {}", adad_core::version());
            Ok(())
        }
        "balance" => {
            let mut client = client()?;
            let balance = client.balance()?;
            println!(
                "balance_atomic={} unlocked_balance_atomic={}",
                balance.balance, balance.unlocked_balance
            );
            Ok(())
        }
        "address" => {
            let mut client = client()?;
            println!("{}", client.address()?.address);
            Ok(())
        }
        "prepare-transfer" => {
            let address = args.next().ok_or(Error::WalletRpc)?;
            let amount = args
                .next()
                .ok_or(Error::WalletRpc)?
                .parse::<u64>()
                .map_err(|_| Error::WalletRpc)?;
            if args.next().is_some() {
                return Err(Error::WalletRpc);
            }
            let mut client = client()?;
            let prepared = client.prepare_transfer(address, amount)?;
            println!(
                "prepared amount_atomic={} fee_atomic={} tx_hash={} tx_metadata={}",
                prepared.amount, prepared.fee, prepared.tx_hash, prepared.tx_metadata
            );
            Ok(())
        }
        _ => Err(Error::WalletRpc),
    }
}

fn client() -> Result<WalletRpcClient<UreqWalletRpcTransport>, Error> {
    let url = env::var("MONERO_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_owned());
    Ok(WalletRpcClient::new(url, UreqWalletRpcTransport::new()))
}

fn print_help() {
    println!(
        "xmr-wallet {}\n\nUsage:\n  xmr-wallet balance\n  xmr-wallet address\n  xmr-wallet prepare-transfer <address> <amount-atomic>\n\nMONERO_RPC_URL defaults to the loopback wallet RPC. Transfers are prepared with do_not_relay=true; this binary does not spend funds.",
        adad_core::version()
    );
}
