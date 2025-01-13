use cid::Cid;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use leptos::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::LazyLock;

use crate::lotus_json::{HasLotusJson, LotusJson};
use crate::message::SignedMessage;

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

const GLIF_CALIBNET: &str = "https://api.calibration.node.glif.io";
const GLIF_MAINNET: &str = "https://api.node.glif.io";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkWrapper {
    Mainnet,
    Testnet,
}

impl From<Network> for NetworkWrapper {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkWrapper::Mainnet,
            Network::Testnet => NetworkWrapper::Testnet,
        }
    }
}

impl From<NetworkWrapper> for Network {
    fn from(serde_network: NetworkWrapper) -> Self {
        match serde_network {
            NetworkWrapper::Mainnet => Network::Mainnet,
            NetworkWrapper::Testnet => Network::Testnet,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
pub struct RpcContext {
    #[allow(unused)]
    #[serde(skip)]
    network: LocalResource<NetworkWrapper>,
    provider: RwSignal<Provider>,
}

impl RpcContext {
    pub fn new() -> Self {
        let provider = RwSignal::new(Provider::new(GLIF_CALIBNET.to_string()));
        let network = LocalResource::new(move || {
            let provider = provider.get();
            async move {
                if provider.network_name().await.ok() != Some("mainnet".to_string()) {
                    NetworkWrapper::Testnet
                } else {
                    NetworkWrapper::Mainnet
                }
            }
        });
        Effect::new(move |_| {
            log::info!("Updating network: {:?}", network.get());
            set_current_network(
                network
                    .get()
                    .as_deref()
                    .cloned()
                    .unwrap_or(NetworkWrapper::Testnet)
                    .into(),
            );
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

    pub fn set(&self, provider: String) {
        self.provider.set(Provider::new(provider));
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    url: String,
}

async fn invoke_rpc_method<T: HasLotusJson + Clone>(
    url: &str,
    method: &str,
    params: &[Value],
) -> anyhow::Result<T> {
    let res = CLIENT
        .post(url)
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
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub fn calibnet() -> Self {
        Self {
            url: GLIF_CALIBNET.to_string(),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            url: GLIF_MAINNET.to_string(),
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

    pub async fn wallet_balance(&self, address: Address) -> anyhow::Result<TokenAmount> {
        invoke_rpc_method(
            &self.url,
            "Filecoin.WalletBalance",
            &[serde_json::to_value(LotusJson(address))?],
        )
        .await
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
    ) -> anyhow::Result<Option<crate::lotus_json::MessageLookup>> {
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
}
