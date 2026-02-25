use leptos::prelude::*;
use uuid::Uuid;

use crate::faucet::constants::DripAmount;
use crate::utils::transaction_id::TransactionId;

#[derive(Clone)]
pub(super) struct FaucetModel {
    pub send_disabled: RwSignal<bool>,
    pub send_limited: RwSignal<i32>,
    pub sent_messages: RwSignal<Vec<(TransactionId, bool)>>,
    pub error_messages: RwSignal<Vec<(Uuid, String)>>,
    pub balance_trigger: Trigger,
    pub faucet_balance: LocalResource<DripAmount>,
    pub target_balance: LocalResource<DripAmount>,
    pub sender_address: RwSignal<String>,
    pub target_address: RwSignal<String>,
}
