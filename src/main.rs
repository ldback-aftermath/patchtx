use sui_types::{
    base_types::{SuiAddress, ObjectID},
    transaction::TransactionData,
};
use sui_sdk::{SuiClientBuilder, rpc_types::SuiObjectDataOptions};

use clap::Parser;
use fastcrypto::encoding::{Encoding, Base64};

use std::{str::FromStr, sync::Arc};

const NODE: &str= &"https://fullnode.testnet.sui.io:443";

/// Patch serialized transaction to make it possible to run from multisig account.
#[derive(Parser, Debug)]
struct Args {

    /// multisig address (MultiSig address).
   #[arg(short, long)]
    multisig: String,

    /// node url.
   #[arg(short, long, default_value_t = String::from(NODE))]
    node: String,

    /// gas coin which belongs to the multisig account.
   #[arg(short, long)]
    coin: ObjectID,

   /// serialized transaction.
   #[arg(short, long)]
    bytes: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let client = SuiClientBuilder::default().build(args.node).await?;
    let client = Arc::new(client);

    // Get gas coin
    let resp = client
        .read_api()
        .get_object_with_options(
                        args.coin,
                        SuiObjectDataOptions::new().with_bcs(),
        ).await?;
    let coin_ref = resp.object()?.object_ref();

    let multisig_address =
        SuiAddress::from_str(&args.multisig).
        expect("Incorrect multisig address provided");

    let mut tx_data: TransactionData =
                    bcs::from_bytes(&Base64::decode(&args.bytes).unwrap()).
                    expect(&"Transaction deserialization is failed.");

    let TransactionData::V1(ref mut tx_data_inner) = tx_data;

    // Patch transaction.
    tx_data_inner.sender = multisig_address;
    tx_data_inner.gas_data.owner = multisig_address;
    tx_data_inner.gas_data.payment = vec![coin_ref];

    // Printout patched serialized transaction.
    println!("{}",Base64::encode(bcs::to_bytes(&tx_data).unwrap()));
    Ok(())
}
