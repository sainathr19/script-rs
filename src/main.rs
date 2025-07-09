use std::str::FromStr;

use alloy::{
    hex::FromHex, 
    primitives::{Address, FixedBytes, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
    transports::http::Http
};
use reqwest::Url;
use alloy::{
    network::EthereumWallet,
    providers::{
        fillers::{
            ChainIdFiller, GasFiller, JoinFill, NonceFiller, SimpleNonceManager, WalletFiller,
        },
        Identity, RootProvider,
    }
};

pub type AlloyProvider = alloy::providers::fillers::FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<Identity, GasFiller>, NonceFiller<SimpleNonceManager>>,
            ChainIdFiller,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<reqwest::Client>>,
>;

sol!(
    #[sol(rpc)]
    ERC20,
    "src/abi/erc20.json",
);

const PRIVATE_KEY: &str = "04f51c632b4ac7619e5f33e5ddce7edda88a278d013aaaf807eac326e0a1b276";
const RPC_URL: &str = "https://rpc.hashira.io/arbitrum_sepolia";
const WBTC_ADDRESS: &str = "0xD8a6E3FCA403d79b6AD6216b60527F51cc967D39";
const AMOUNT: u128 = 1000;
const RECIPIENT_ADDRESS: &str = "0x92df3Da2B4B0a76A89401e779A6c5F8458E7fF1d";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer = PrivateKeySigner::from_bytes(
        &FixedBytes::from_hex(PRIVATE_KEY).expect("Invalid executor private key"),
    )?;

    let wallet = EthereumWallet::from(signer.clone());
    let provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .with_gas_estimation()
        .with_simple_nonce_management()
        .fetch_chain_id()
        .wallet(wallet)
        .connect_http(Url::from_str(&RPC_URL).expect("Invalid RPC URL"));

    let wbtc_address = Address::from_str(WBTC_ADDRESS).expect("Invalid WBTC address");
    let recipient_address = Address::from_str(RECIPIENT_ADDRESS).expect("Invalid recipient address");
    let sender_address = signer.address();

    let wbtc = ERC20::new(wbtc_address, &provider);

    let mut transaction_count = 0;

    loop {
        transaction_count += 1;
        println!("\nTransaction {}", transaction_count);

        let amount = U256::from(AMOUNT);

        let balance_before = wbtc.balanceOf(sender_address).call().await;
        match balance_before {
            Ok(balance_before) => {
                if balance_before < amount {
                    println!("❌ Insufficient balance! Need {} but have {}", 
                             amount,
                             balance_before);
                    break;
                }
            },
            Err(e) => {
                println!("❌ Error getting balance: {}", e);
            }
        }
        
        let tx_hash = wbtc
            .transfer(recipient_address, amount)
            .send()
            .await;
        match tx_hash {
            Ok(tx_hash) => {
                let tx_hash = tx_hash.watch().await;
                match tx_hash {
                    Ok(tx_hash) => {
                        println!("✅ Transaction confirmed! Hash: {}", tx_hash);
                    },
                    Err(e) => {
                        println!("❌ Error confirming transaction: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("❌ Error submitting transaction: {}", e);
            }
        }
    }
    Ok(())
}