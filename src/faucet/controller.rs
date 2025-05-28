// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::calibnet::{CALIBNET_DRIP_AMOUNT, CALIBNET_RATE_LIMIT_SECONDS, FIL_CALIBNET_UNIT};
use super::mainnet::{FIL_MAINNET_UNIT, MAINNET_DRIP_AMOUNT, MAINNET_RATE_LIMIT_SECONDS};

use super::server::{faucet_address, sign_with_secret_key};
use crate::faucet::model::FaucetModel;
use crate::utils::lotus_json::LotusJson;
use crate::utils::rpc_context::Provider;
use crate::utils::{address::parse_address, error::catch_all, message::message_transfer};
use cid::Cid;
use fvm_shared::{address::Network, econ::TokenAmount};
use leptos::prelude::*;
use leptos::task::spawn_local;
use uuid::Uuid;

#[derive(Clone)]
pub struct FaucetController {
    faucet: FaucetModel,
}

impl FaucetController {
    pub fn new(network: Network) -> Self {
        let is_mainnet = network == Network::Mainnet;
        let balance_trigger = Trigger::new();
        let sender_address = RwSignal::new(String::new());
        let target_address = RwSignal::new(String::new());
        let target_balance = LocalResource::new(move || {
            let target_address = target_address.get();
            balance_trigger.track();
            async move {
                if let Ok(address) = parse_address(&target_address, network) {
                    Provider::from_network(network)
                        .wallet_balance(address)
                        .await
                        .ok()
                        .unwrap_or(TokenAmount::from_atto(0))
                } else {
                    TokenAmount::from_atto(0)
                }
            }
        });
        let faucet_address = LocalResource::new(move || async move {
            faucet_address(is_mainnet)
                .await
                .map(|LotusJson(addr)| addr)
                .ok()
        });
        let faucet_balance = LocalResource::new(move || {
            balance_trigger.track();
            async move {
                if let Some(addr) = faucet_address.await {
                    sender_address.set(addr.to_string());
                    Provider::from_network(network)
                        .wallet_balance(addr)
                        .await
                        .ok()
                        .unwrap_or(TokenAmount::from_atto(0))
                } else {
                    TokenAmount::from_atto(0)
                }
            }
        });
        let faucet = FaucetModel {
            network,
            send_disabled: RwSignal::new(false),
            send_limited: RwSignal::new(0),
            sent_messages: RwSignal::new(Vec::new()),
            error_messages: RwSignal::new(Vec::new()),
            balance_trigger,
            target_balance,
            faucet_balance,
            sender_address,
            target_address,
        };
        Self { faucet }
    }

    #[allow(dead_code)]
    pub fn refetch_balances(&self) {
        use leptos::prelude::GetUntracked;

        log::info!("Checking for new transactions");
        self.faucet.balance_trigger.notify();
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

    pub fn get_sender_address(&self) -> String {
        self.faucet.sender_address.get()
    }

    pub fn get_target_address(&self) -> String {
        self.faucet.target_address.get()
    }

    pub fn get_fil_unit(&self) -> String {
        match self.faucet.network {
            Network::Mainnet => FIL_MAINNET_UNIT,
            _ => FIL_CALIBNET_UNIT,
        }
        .to_string()
    }

    pub fn set_target_address(&self, address: String) {
        self.faucet.target_address.set(address);
    }

    pub fn get_faucet_balance(&self) -> TokenAmount {
        self.faucet.faucet_balance.get().unwrap_or_default()
    }

    pub fn get_error_messages(&self) -> Vec<(Uuid, String)> {
        self.faucet.error_messages.get().clone()
    }

    pub fn add_error_message(&self, message: String) {
        self.faucet.error_messages.update(|messages| {
            messages.push((Uuid::new_v4(), message));
        });
    }

    pub fn remove_error_message(&self, id: Uuid) {
        self.faucet.error_messages.update(|messages| {
            messages.retain(|(x, _)| *x != id);
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

    fn get_drip_amount(&self) -> TokenAmount {
        if self.faucet.network == Network::Mainnet {
            MAINNET_DRIP_AMOUNT.clone()
        } else {
            CALIBNET_DRIP_AMOUNT.clone()
        }
    }

    pub fn is_low_balance(&self) -> bool {
        let target_balance = self.get_faucet_balance();
        let drip_amount = self.get_drip_amount();
        target_balance < drip_amount
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
                                MAINNET_DRIP_AMOUNT.clone()
                            } else {
                                CALIBNET_DRIP_AMOUNT.clone()
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
                                let rate_limit_seconds = if is_mainnet {
                                    MAINNET_RATE_LIMIT_SECONDS
                                } else {
                                    CALIBNET_RATE_LIMIT_SECONDS
                                };
                                faucet.send_limited.set(rate_limit_seconds as i32);
                            }
                        }
                        Ok(())
                    })
                    .await;
                    faucet.send_disabled.set(false);
                });
            }
            Err(e) => {
                self.add_error_message(format!(
                    "Invalid address: {}",
                    &self.faucet.target_address.get()
                ));
                log::error!("Error parsing address: {}", e);
            }
        }
    }
}
