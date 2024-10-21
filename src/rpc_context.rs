use cid::Cid;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use leptos::*;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::LazyLock;

use crate::lotus_json::LotusJson;
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

    pub fn set(&self, provider: String) {
        self.provider.set(Provider::new(provider));
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Provider {
    url: String,
}

impl Provider {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn network_name(&self) -> anyhow::Result<String> {
        let res = CLIENT
            .post(&self.url)
            .json(&json! {
                {
                    "jsonrpc": "2.0",
                    "method": "Filecoin.StateNetworkName",
                    "params": [],
                    "id": 0
                }
            })
            .send()
            .await?;
        log::info!("Got response: {:?}", res);
        let LotusJson(name) = serde_json::from_value(
            res.json::<Value>()
                .await?
                .get("result")
                .ok_or(anyhow::anyhow!("No result"))?
                .clone(),
        )?;
        Ok(name)
    }

    pub async fn network_version(&self) -> anyhow::Result<u64> {
        let res = CLIENT
            .post(&self.url)
            .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.StateNetworkVersion",
                        "params": [[]],
                        "id": 0
                }
            })
            .send()
            .await?;
        log::info!("Got response: {:?}", res);
        let LotusJson(version) = serde_json::from_value(
            res.json::<Value>()
                .await?
                .get("result")
                .ok_or(anyhow::anyhow!("No result"))?
                .clone(),
        )?;
        Ok(version)
    }

    pub async fn wallet_balance(&self, address: Address) -> anyhow::Result<TokenAmount> {
        let res = CLIENT
            .post(&self.url)
            .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.WalletBalance",
                        "params": [LotusJson(address)],
                        "id": 0
                }
            })
            .send()
            .await?;
        log::info!("Got response: {:?}", res);
        let LotusJson(balance) = serde_json::from_value(
            res.json::<Value>()
                .await?
                .get("result")
                .ok_or(anyhow::anyhow!("No result"))?
                .clone(),
        )?;
        Ok(balance)
    }

    pub async fn estimate_gas(&self, msg: Message) -> anyhow::Result<Message> {
        let res = CLIENT
            .post(&self.url)
            .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.GasEstimateMessageGas",
                        "params": [LotusJson(msg), null, null],
                        "id": 0
                }
            })
            .send()
            .await?;
        log::info!("Got response: {:?}", res);
        let LotusJson(msg) = serde_json::from_value(
            res.json::<Value>()
                .await?
                .get("result")
                .ok_or(anyhow::anyhow!("No result"))?
                .clone(),
        )?;
        Ok(msg)
    }

    pub async fn mpool_get_nonce(&self, addr: Address) -> anyhow::Result<u64> {
        let res = CLIENT
            .post(&self.url)
            .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.MpoolGetNonce",
                        "params": [LotusJson(addr)],
                        "id": 0
                }
            })
            .send()
            .await?;
        log::info!("Got response: {:?}", res);
        let LotusJson(nonce) = serde_json::from_value(
            res.json::<Value>()
                .await?
                .get("result")
                .ok_or(anyhow::anyhow!("No result"))?
                .clone(),
        )?;
        Ok(nonce)
    }

    pub async fn mpool_push(&self, smsg: SignedMessage) -> anyhow::Result<Cid> {
        log::info!(
            "Pushing json: {}",
            json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.MpoolPush",
                        "params": [LotusJson(smsg.clone())],
                        "id": 0
                }
            }
        );
        let res = CLIENT
            .post(&self.url)
            .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.MpoolPush",
                        "params": [LotusJson(smsg)],
                        "id": 0
                }
            })
            .send()
            .await?;
        log::info!("Got response: {:?}", res);
        let LotusJson(cid) = serde_json::from_value(
            res.json::<Value>()
                .await?
                .get("result")
                .ok_or(anyhow::anyhow!("No result"))?
                .clone(),
        )?;
        Ok(cid)
    }
}
