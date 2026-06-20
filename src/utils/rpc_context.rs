use alloy::primitives::TxHash;
use alloy::providers::{Provider as AlloyProvider, ProviderBuilder as AlloyProviderBuilder};
use alloy::sol;
use anyhow::Context as _;
use cid::Cid;
use fvm_shared::address::{Address, Network, set_current_network};
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::sector::StoragePower;
use leptos::prelude::*;
use reqwest::Client;
use serde_json::{Value, json};
use std::sync::LazyLock;
use url::Url;

use crate::utils::address::AddressAlloyExt as _;
use crate::utils::conversions::TokenAmountAlloyExt as _;
use crate::utils::drip_amount::{DripAmount, TokenType};

use super::lotus_json::{HasLotusJson, LotusJson, signed_message::SignedMessage};

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub struct RpcEndpoint {
    pub label: &'static str,
    pub url: &'static str,
}

const CALIBNET_PROVIDERS: &[RpcEndpoint] = &[
    RpcEndpoint {
        label: "Glif",
        url: "https://api.calibration.node.glif.io",
    },
    RpcEndpoint {
        label: "Ankr",
        url: "https://rpc.ankr.com/filecoin_testnet",
    },
    RpcEndpoint {
        label: "Filfox",
        url: "https://calibration.filfox.info/rpc/v1",
    },
];

const MAINNET_PROVIDERS: &[RpcEndpoint] = &[
    RpcEndpoint {
        label: "Glif",
        url: "https://api.node.glif.io",
    },
    RpcEndpoint {
        label: "Ankr",
        url: "https://rpc.ankr.com/filecoin",
    },
    RpcEndpoint {
        label: "Filfox",
        url: "https://filfox.info/rpc/v1",
    },
];

pub fn providers_for(network: Network) -> &'static [RpcEndpoint] {
    match network {
        Network::Testnet => CALIBNET_PROVIDERS,
        Network::Mainnet => MAINNET_PROVIDERS,
    }
}

pub fn default_provider(network: Network) -> Provider {
    let endpoint = providers_for(network)
        .first()
        .expect("each network has at least one provider");
    Provider::new(endpoint.url.parse().expect("invalid default provider URL"))
}

#[derive(Clone, Copy)]
pub struct RpcContext {
    network: RwSignal<Network>,
    provider: RwSignal<Provider>,
}

impl RpcContext {
    pub fn new() -> Self {
        let network = RwSignal::new(Network::Testnet);
        let provider = RwSignal::new(default_provider(Network::Testnet));
        Effect::new(move |_| {
            log::info!("Updating network: {:?}", network.get());
            set_current_network(network.get());
        });
        Self { network, provider }
    }

    pub fn provide_context() {
        provide_context(RpcContext::new());
    }

    pub fn use_context() -> Self {
        use_context::<Self>().expect("RpcContext should be provided")
    }

    pub fn network(&self) -> RwSignal<Network> {
        self.network
    }

    pub fn provider(&self) -> RwSignal<Provider> {
        self.provider
    }

    pub fn get(&self) -> Provider {
        self.provider.get()
    }

    pub fn set_network(&self, network: Network) {
        if self.network.get_untracked() != network {
            self.network.set(network);
            self.provider.set(default_provider(network));
        }
    }

    pub fn set_provider_url(&self, url: Url) {
        self.provider.set(Provider::new(url));
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

    pub fn default_for(network: Network) -> Self {
        default_provider(network)
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
    ) -> anyhow::Result<DripAmount> {
        match token_type {
            TokenType::Native => {
                let balance = self.wallet_balance_native(wallet_address).await?;
                Ok(DripAmount::Token(balance))
            }
            TokenType::Erc20(contract_address) => {
                let balance = self
                    .wallet_balance_erc20(wallet_address, *contract_address)
                    .await?;
                Ok(DripAmount::Token(balance))
            }
            TokenType::Datacap => {
                let balance = match self.wallet_balance_verifier_datacap(wallet_address).await {
                    Ok(val) => val,
                    Err(_) => {
                        self.wallet_balance_verified_client_datacap(wallet_address)
                            .await?
                    }
                };
                Ok(DripAmount::Storage(balance))
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

    /// Returns the remaining storage power of a verifier address.
    async fn wallet_balance_verifier_datacap(
        &self,
        verifier_address: Address,
    ) -> anyhow::Result<StoragePower> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.StateVerifierStatus",
            &[
                serde_json::to_value(LotusJson(verifier_address))?,
                Value::Null,
            ],
        )
        .await
    }

    /// Returns the remaining storage power of a verified client address.
    async fn wallet_balance_verified_client_datacap(
        &self,
        verified_client_address: Address,
    ) -> anyhow::Result<StoragePower> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.StateVerifiedClientStatus",
            &[
                serde_json::to_value(LotusJson(verified_client_address))?,
                Value::Null,
            ],
        )
        .await
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

    /// Looks up the ID address of a given Filecoin address.
    /// This method makes an RPC call to the Filecoin node to convert a Filecoin address
    /// to its corresponding ID address in the current state tree.
    pub async fn lookup_id(&self, addr: Address) -> anyhow::Result<Address> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.StateLookupID",
            &[serde_json::to_value(LotusJson(addr))?, Value::Null],
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

    pub async fn eth_get_transaction_hash_by_cid(&self, cid: Cid) -> anyhow::Result<TxHash> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.EthGetTransactionHashByCid",
            &[serde_json::to_value(LotusJson(cid))?],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_providers_for_network_testnet() {
        let providers = providers_for(Network::Testnet);
        assert_eq!(providers.len(), CALIBNET_PROVIDERS.len());
        for (actual, expected) in providers.iter().zip(CALIBNET_PROVIDERS) {
            assert_eq!(actual.label, expected.label);
            assert_eq!(actual.url, expected.url);
        }
    }

    #[test]
    fn test_providers_for_network_mainnet() {
        let providers = providers_for(Network::Mainnet);
        assert_eq!(providers.len(), MAINNET_PROVIDERS.len());
        for (actual, expected) in providers.iter().zip(MAINNET_PROVIDERS) {
            assert_eq!(actual.label, expected.label);
            assert_eq!(actual.url, expected.url);
        }
    }

    #[test]
    fn test_default_provider_per_network() {
        let testnet = default_provider(Network::Testnet);
        let mainnet = default_provider(Network::Mainnet);

        assert_eq!(
            testnet.url,
            CALIBNET_PROVIDERS[0].url.parse().unwrap()
        );
        assert_eq!(
            mainnet.url,
            MAINNET_PROVIDERS[0].url.parse().unwrap()
        );
        assert_ne!(testnet.url, mainnet.url);
    }
}
