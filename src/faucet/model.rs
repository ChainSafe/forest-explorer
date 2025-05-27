// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_shared::{address::Network, econ::TokenAmount};
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct FaucetModel {
    pub network: Network,
    pub send_disabled: RwSignal<bool>,
    pub send_limited: RwSignal<i32>,
    pub sent_messages: RwSignal<Vec<(Cid, bool)>>,
    pub error_messages: RwSignal<Vec<(Uuid, String)>>,
    pub balance_trigger: Trigger,
    pub faucet_balance: LocalResource<TokenAmount>,
    pub target_balance: LocalResource<TokenAmount>,
    pub sender_address: RwSignal<String>,
    pub target_address: RwSignal<String>,
}
