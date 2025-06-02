use alloy::primitives::TxHash;
use alloy::providers::{Provider as AlloyProvider, ProviderBuilder as AlloyProviderBuilder};
use alloy::sol;
use cid::Cid;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use reqwest::Client;
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::LazyLock;
use url::Url;

use crate::faucet::constants::TokenType;
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

    /// Returns the current gas price in attoFIL.
    ///
    /// Internally, it prunes the result from `u128` to `u64` but it should be safe as we don't
    /// expect the gas price to exceed 1 FIL (1e18 attoFIL) in the foreseeable future.
    pub async fn gas_price(&self) -> anyhow::Result<u64> {
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        provider
            .get_gas_price()
            .await
            .map(|price| price as u64)
            .map_err(|e| anyhow::anyhow!("Failed to get gas price: {e}"))
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
