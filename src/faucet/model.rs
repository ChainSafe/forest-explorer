use cid::Cid;
use fvm_shared::{
    address::{Address, Network},
    econ::TokenAmount,
};
use leptos::{Resource, RwSignal};

#[derive(Clone)]
pub(super) struct FaucetModel {
    pub network: Network,
    pub send_disabled: RwSignal<bool>,
    pub send_limited: RwSignal<i32>,
    pub sent_messages: RwSignal<Vec<(Cid, bool)>>,
    pub error_messages: RwSignal<Vec<String>>,
    pub faucet_balance: Resource<Option<Address>, TokenAmount>,
    pub target_balance: Resource<String, TokenAmount>,
    pub target_address: RwSignal<String>,
}
