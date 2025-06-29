use fvm_shared::econ::TokenAmount;
use leptos::prelude::*;
use uuid::Uuid;

use crate::utils::transaction_id::TransactionId;

#[derive(Clone)]
pub(super) struct FaucetModel {
    pub send_disabled: RwSignal<bool>,
    pub send_limited: RwSignal<i32>,
    pub sent_messages: RwSignal<Vec<(TransactionId, bool)>>,
    pub error_messages: RwSignal<Vec<(Uuid, String)>>,
    pub balance_trigger: Trigger,
    pub faucet_balance: LocalResource<TokenAmount>,
    pub target_balance: LocalResource<TokenAmount>,
    pub sender_address: RwSignal<String>,
    pub target_address: RwSignal<String>,
}
