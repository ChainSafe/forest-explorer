use alloy::primitives::TxHash;
use alloy::providers::{Provider as AlloyProvider, ProviderBuilder as AlloyProviderBuilder};
use alloy::sol;
use anyhow::Context as _;
use cid::Cid;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use leptos::prelude::*;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::LazyLock;
use url::Url;

use crate::faucet::constants::TokenType;
use crate::utils::address::AddressAlloyExt as _;
use crate::utils::conversions::TokenAmountAlloyExt as _;

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

    /// Returns the balance of a wallet address in the specified token type.
    pub async fn wallet_balance(
        &self,
        wallet_address: Address,
        token_type: &TokenType,
    ) -> anyhow::Result<TokenAmount> {
        match token_type {
            TokenType::Native => self.wallet_balance_native(wallet_address).await,
            TokenType::Erc20(contract_address) => {
                self.wallet_balance_erc20(wallet_address, *contract_address)
                    .await
            }
        }
    }

    /// Returns the balance of a wallet address in native Filecoin token.
    async fn wallet_balance_native(&self, wallet_address: Address) -> anyhow::Result<TokenAmount> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.WalletBalance",
            &[serde_json::to_value(LotusJson(wallet_address))?],
        )
        .await
    }

    /// Returns the balance of a wallet address in an ERC-20 token.
    async fn wallet_balance_erc20(
        &self,
        wallet_address: Address,
        contract_address: alloy::primitives::Address,
    ) -> anyhow::Result<TokenAmount> {
        sol! {
           #[sol(rpc)]
           contract ERC20 {
                function balanceOf(address owner) public view returns (uint256);
           }
        }

        let eth_address = wallet_address.into_eth_address()?;
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        let erc20 = ERC20::new(contract_address, provider);

        let balance = erc20.balanceOf(eth_address).call().await?;
        Ok(TokenAmount::from_alloy_amount(&balance))
    }

    pub async fn send_eth_transaction_signed(&self, signed_tx: &[u8]) -> anyhow::Result<TxHash> {
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        Ok(provider
            .send_raw_transaction(signed_tx)
            .await?
            .tx_hash()
            .to_owned())
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
    /// expect the gas price to exceed 1 FIL (`1e18` attoFIL) in the foreseeable future.
    pub async fn gas_price(&self) -> anyhow::Result<u64> {
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        provider
            .get_gas_price()
            .await
            .map(|price| price as u64)
            .context("Failed to get gas price")
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

    /// Checks if an Ethereum transaction is confirmed by checking if it is included in any block.
    pub async fn check_eth_transaction_confirmed(&self, tx_hash: TxHash) -> anyhow::Result<bool> {
        let provider = AlloyProviderBuilder::new().connect_http(self.url.clone());
        match provider.get_transaction_receipt(tx_hash).await? {
            Some(receipt) => Ok(receipt.block_number.is_some() && receipt.status()),
            None => Ok(false),
        }
    }
}
