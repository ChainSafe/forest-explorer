use super::{model::FaucetModel, utils::sign_with_secret_key};
use cid::Cid;
use fvm_shared::{address::Network, econ::TokenAmount};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    address::parse_address, lotus_json::LotusJson, message::message_transfer,
    rpc_context::Provider, utils::catch_all,
};

use super::utils::faucet_address;

#[derive(Clone)]
pub(super) struct FaucetController {
    faucet: FaucetModel,
}

impl FaucetController {
    pub fn new(network: Network) -> Self {
        let is_mainnet = network == Network::Mainnet;
        let target_address = RwSignal::new(String::new());
        let target_balance = Resource::new(
            move || target_address.get(),
            move |address| async move {
                if let Ok(address) = parse_address(&address, network) {
                    Provider::from_network(network)
                        .wallet_balance(address)
                        .await
                        .ok()
                        .unwrap_or(TokenAmount::from_atto(0))
                } else {
                    TokenAmount::from_atto(0)
                }
            },
        );
        let sender_address = Resource::new(
            move || (),
            move |()| async move {
                faucet_address(is_mainnet)
                    .await
                    .map(|LotusJson(addr)| addr)
                    .ok()
            },
        );
        let faucet_balance = Resource::new(
            move || sender_address.get().flatten(),
            move |addr| async move {
                if let Some(addr) = addr {
                    Provider::from_network(network)
                        .wallet_balance(addr)
                        .await
                        .ok()
                        .unwrap_or(TokenAmount::from_atto(0))
                } else {
                    TokenAmount::from_atto(0)
                }
            },
        );
        let faucet = FaucetModel {
            network,
            send_disabled: RwSignal::new(false),
            send_limited: RwSignal::new(0),
            sent_messages: RwSignal::new(Vec::new()),
            error_messages: RwSignal::new(Vec::new()),
            target_balance,
            faucet_balance,
            target_address,
        };
        Self { faucet }
    }

    #[allow(dead_code)]
    pub fn refetch_balances(&self) {
        use leptos::prelude::GetUntracked;

        log::info!("Checking for new transactions");
        self.faucet.target_balance.refetch();
        self.faucet.faucet_balance.refetch();
        let pending = self
            .faucet
            .sent_messages
            .get_untracked()
            .into_iter()
            .filter_map(|(cid, sent)| if !sent { Some(cid) } else { None })
            .collect::<Vec<_>>();

        let network = self.faucet.network;
        let messages = self.faucet.sent_messages;
        spawn_local(catch_all(self.faucet.error_messages, async move {
            for cid in pending {
                if let Some(lookup) = Provider::from_network(network)
                    .state_search_msg(cid)
                    .await?
                {
                    messages.update(|messages| {
                        for (cid, sent) in messages {
                            if cid == &lookup.message {
                                *sent = true;
                            }
                        }
                    });
                }
            }
            Ok(())
        }));
    }
    pub fn get_target_balance(&self) -> TokenAmount {
        self.faucet.target_balance.get().unwrap_or_default()
    }

    pub fn get_target_address(&self) -> String {
        self.faucet.target_address.get()
    }

    pub fn set_target_address(&self, address: String) {
        self.faucet.target_address.set(address);
    }

    pub fn get_faucet_balance(&self) -> TokenAmount {
        self.faucet.faucet_balance.get().unwrap_or_default()
    }

    pub fn get_error_messages(&self) -> Vec<String> {
        self.faucet.error_messages.get().clone()
    }

    pub fn add_error_message(&self, message: String) {
        self.faucet.error_messages.update(|messages| {
            messages.push(message);
        });
    }

    pub fn remove_error_message(&self, index: usize) {
        self.faucet.error_messages.update(|messages| {
            messages.remove(index);
        });
    }

    pub fn get_sent_messages(&self) -> Vec<(Cid, bool)> {
        self.faucet.sent_messages.get().clone()
    }

    pub fn is_send_disabled(&self) -> bool {
        self.faucet.send_disabled.get()
    }

    pub fn get_send_rate_limit_remaining(&self) -> i32 {
        self.faucet.send_limited.get()
    }

    #[allow(dead_code)]
    pub fn set_send_rate_limit_remaining(&self, remaining: i32) {
        self.faucet.send_limited.set(remaining);
    }

    pub fn drip(&self) {
        let is_mainnet = self.faucet.network == Network::Mainnet;
        let faucet = self.faucet.clone();
        match parse_address(&self.faucet.target_address.get(), self.faucet.network) {
            Ok(addr) => {
                spawn_local(async move {
                    catch_all(faucet.error_messages, async move {
                        let rpc = Provider::from_network(faucet.network);
                        let LotusJson(from) = faucet_address(is_mainnet)
                            .await
                            .map_err(|e| anyhow::anyhow!("Error getting faucet address: {}", e))?;
                        faucet.send_disabled.set(true);
                        let nonce = rpc.mpool_get_nonce(from).await?;
                        let mut msg = message_transfer(
                            from,
                            addr,
                            if is_mainnet {
                                TokenAmount::from_nano(1_000_000)
                            } else {
                                TokenAmount::from_whole(1)
                            },
                        );
                        msg.sequence = nonce;
                        let msg = rpc.estimate_gas(msg).await?;
                        match sign_with_secret_key(LotusJson(msg.clone()), is_mainnet).await {
                            Ok(LotusJson(smsg)) => {
                                let cid = rpc.mpool_push(smsg).await?;
                                faucet.sent_messages.update(|messages| {
                                    messages.push((cid, false));
                                });
                                log::info!("Sent message: {:?}", cid);
                            }
                            Err(e) => {
                                log::error!("Failed to sign message: {}", e);
                                faucet.send_limited.set(30);
                            }
                        }
                        Ok(())
                    })
                    .await;
                    faucet.send_disabled.set(false);
                });
            }
            Err(e) => {
                self.add_error_message("Invalid address".to_string());
                log::error!("Error parsing address: {}", e);
            }
        }
    }
}
