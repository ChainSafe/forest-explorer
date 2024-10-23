use cid::Cid;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use leptos::*;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::LazyLock;

#[cfg(feature = "hydrate")]
use crate::lotus_json::MessageLookup;
use crate::lotus_json::{HasLotusJson, LotusJson};
use crate::message::SignedMessage;

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

const GLIF_CALIBNET: &str = "https://api.calibration.node.glif.io";

#[derive(Clone, Copy)]
pub struct RpcContext {
    #[allow(unused)]
    network: Resource<Provider, Network>,
    provider: RwSignal<Provider>,
}

impl RpcContext {
    pub fn new() -> Self {
        let provider = create_rw_signal(Provider::new(GLIF_CALIBNET.to_string()));
        let network = create_local_resource(
            move || provider.get(),
            move |provider| async move {
                if provider.network_name().await.ok() != Some("mainnet".to_string()) {
                    Network::Testnet
                } else {
                    Network::Mainnet
                }
            },
        );
        create_effect(move |_| {
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

    pub fn get_untracked(&self) -> Provider {
        self.provider.get_untracked()
    }

    pub fn set(&self, provider: String) {
        self.provider.set(Provider::new(provider));
    }
}

#[derive(Clone, PartialEq, Eq)]
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

    #[cfg(feature = "hydrate")]
    pub async fn state_search_msg(&self, msg: Cid) -> anyhow::Result<MessageLookup> {
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