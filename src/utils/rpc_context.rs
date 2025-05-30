use alloy::network::TransactionBuilder as _;
use alloy::primitives::{TxHash, Uint};
use alloy::providers::{Provider as AlloyProvider, ProviderBuilder as AlloyProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::k256::sha2::digest::typenum::UInt;
use alloy::{sol, sol_types::SolCall};
use cid::Cid;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use num_traits::Zero;
use reqwest::Client;
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::LazyLock;
use url::Url;

use crate::faucet::constants::{ContractAddress, FaucetInfo, TokenType};
use crate::utils::address::AddressAlloyExt as _;

use super::lotus_json::{signed_message::SignedMessage, HasLotusJson, LotusJson};

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

static GLIF_CALIBNET: LazyLock<Url> = LazyLock::new(|| {
    "https://api.calibration.node.glif.io"
        .parse()
        .expect("Invalid URL for Filecoin calibration network")
});
static GLIF_MAINNET: LazyLock<Url> = LazyLock::new(|| {
    "https://api.node.glif.io"
        .parse()
        .expect("Invalid URL for Filecoin mainnet")
});

#[derive(Clone, Copy)]
pub struct RpcContext {
    #[allow(unused)]
    network: LocalResource<Network>,
    provider: RwSignal<Provider>,
}

impl RpcContext {
    pub fn new() -> Self {
        let provider = RwSignal::new(Provider::new(GLIF_CALIBNET.clone()));
        let network = LocalResource::new(move || {
            let provider = provider.get();
            async move {
                if provider.network_name().await.ok() != Some("mainnet".to_string()) {
                    Network::Testnet
                } else {
                    Network::Mainnet
                }
            }
        });
        Effect::new(move |_| {
            log::info!("Updating network: {:?}", network.get());
            set_current_network(network.get().unwrap_or(Network::Testnet));
        });
        Self { network, provider }
    }

    pub fn provide_context() {
        provide_context(RpcContext::new());
    }

    pub fn use_context() -> Self {
        use_context::<Self>().expect("RpcContext should be provided")
    }

    pub fn get(&self) -> Provider {
        self.provider.get()
    }

    pub fn set(&self, provider: Url) {
        self.provider.set(Provider::new(provider));
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Provider {
    pub url: Url,
}

async fn invoke_rpc_method<T: HasLotusJson + Clone>(
    url: &Url,
    method: &str,
    params: &[Value],
) -> anyhow::Result<T> {
    let res = CLIENT
        .post(url.as_ref())
        .json(&json! {
            {
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 0
            }
        })
        .send()
        .await?;
    let LotusJson(ret) = serde_json::from_value(
        res.json::<Value>()
            .await?
            .get("result")
            .ok_or(anyhow::anyhow!("No result"))?
            .clone(),
    )?;
    Ok(ret)
}

impl Provider {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn get_network_url(network: Network) -> Url {
        match network {
            Network::Testnet => GLIF_CALIBNET.to_owned(),
            Network::Mainnet => GLIF_MAINNET.to_owned(),
        }
    }

    pub fn calibnet() -> Self {
        Self {
            url: GLIF_CALIBNET.to_owned(),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            url: GLIF_MAINNET.to_owned(),
        }
    }

    pub fn from_network(network: Network) -> Self {
        match network {
            Network::Testnet => Self::calibnet(),
            Network::Mainnet => Self::mainnet(),
        }
    }

    pub async fn network_name(&self) -> anyhow::Result<String> {
        invoke_rpc_method(&self.url, "Filecoin.StateNetworkName", &[]).await
    }

    pub async fn network_version(&self) -> anyhow::Result<u64> {
        invoke_rpc_method(&self.url, "Filecoin.StateNetworkVersion", &[Value::Null]).await
    }

    pub async fn wallet_balance(
        &self,
        wallet_address: Address,
        token_type: &TokenType,
    ) -> anyhow::Result<TokenAmount> {
        match token_type {
            TokenType::Native => {
                invoke_rpc_method(
                    &self.url,
                    "Filecoin.WalletBalance",
                    &[serde_json::to_value(LotusJson(wallet_address))?],
                )
                .await
            }
            TokenType::Erc20(contract_address) => {
                //// TODO: hide it
                sol! {
                   #[sol(rpc)]
                   contract ERC20 {
                        function balanceOf(address owner) public view returns (uint256);
                   }
                }

                let eth_address = wallet_address.into_eth_address()?;
                let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
                let erc20 = ERC20::new(*contract_address, provider);

                let balance = erc20.balanceOf(eth_address).call().await?;
                // Warning! The assumption here is that the decimals are the same for both Filecoin
                // and given ERC20 token. This holds true for USDFC, but may not hold for other
                // tokens.
                let token_amount = TokenAmount::from_atto(BigInt::from_bytes_be(
                    fvm_shared::bigint::Sign::Plus,
                    &balance.to_be_bytes_trimmed_vec(),
                ));

                console_log(&format!(
                    "Balance of {eth_address} in contract {contract_address}: {token_amount}"
                ));

                Ok(token_amount)
            }
        }
    }

    pub async fn erc20_transfer_transaction(
        &self,
        from: Address,
        to: Address,
        faucet_info: FaucetInfo,
    ) -> anyhow::Result<TransactionRequest> {
        let contract_address = match faucet_info.token_type() {
            TokenType::Erc20(addr) => addr,
            _ => {
                return Err(anyhow::anyhow!(
                    "Cannot create ERC20 transfer transaction for non-ERC20 token"
                ))
            }
        };
        let amount = faucet_info.drip_amount();
        sol! {
            #[sol(rpc)]
            contract ERC20 {
                function transfer(address to, uint256 amount) public returns (bool);
            }
        }

        // let eth_from = from.into_eth_address()?;
        let eth_to = to.into_eth_address()?;
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        //let erc20 = ERC20::new(contract_address, provider.clone());

        let amount = Uint::from_be_slice(&amount.atto().to_signed_bytes_be());

        //let transfer_call = erc20.transfer(eth_to, amount).into_transaction_request();

        //let gas_estimate = provider.estimate_gas(transfer_call).await?;
        // let nonce = provider.get_transaction_count(eth_from).await?;
        let nonce = self.mpool_get_nonce(from).await?;
        let chain_id = provider.get_chain_id().await?;
        // let gas_limit = gas_estimate + 10_000;
        let gas_limit = 50_000_000; // Set a reasonable gas limit for the transaction
        let gas_price = provider.get_gas_price().await?; // Add a buffer to the gas
                                                         // price

        let calldata = ERC20::transferCall::new((eth_to, amount)).abi_encode();

        // === Build EIP-1559 Transaction ===
        let tx = alloy::rpc::types::TransactionRequest::default()
            .with_to(contract_address)
            .with_chain_id(chain_id)
            .with_nonce(nonce)
            .with_gas_limit(gas_limit)
            .with_gas_price(gas_price)
            .with_input(calldata);

        Ok(tx)
    }

    pub async fn send_eth_transaction_signed(&self, signed_tx: &[u8]) -> anyhow::Result<String> {
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        let tx = provider.send_raw_transaction(signed_tx).await?;
        Ok(tx.tx_hash().to_string())
    }

    pub async fn estimate_gas(&self, msg: Message) -> anyhow::Result<Message> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.GasEstimateMessageGas",
            &[
                serde_json::to_value(LotusJson(msg))?,
                Value::Null,
                Value::Null,
            ],
        )
        .await
    }

    pub async fn mpool_get_nonce(&self, addr: Address) -> anyhow::Result<u64> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.MpoolGetNonce",
            &[serde_json::to_value(LotusJson(addr))?],
        )
        .await
    }

    pub async fn mpool_push(&self, smsg: SignedMessage) -> anyhow::Result<Cid> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.MpoolPush",
            &[serde_json::to_value(LotusJson(smsg))?],
        )
        .await
    }

    pub async fn state_search_msg(
        &self,
        msg: Cid,
    ) -> anyhow::Result<Option<crate::utils::lotus_json::MessageLookup>> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.StateSearchMsg",
            &[
                Value::Null,
                serde_json::to_value(LotusJson(msg))?,
                Value::Number(10.into()),
                Value::Bool(false),
            ],
        )
        .await
    }

    pub async fn check_eth_transaction_confirmed(&self, eth_hash: &str) -> anyhow::Result<bool> {
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        // TODO: strongly type this
        let tx_hash = TxHash::from_str(eth_hash)?;
        match provider.get_transaction_receipt(tx_hash).await? {
            Some(receipt) => Ok(receipt.block_number.is_some()),
            None => Ok(false),
        }
    }
}
