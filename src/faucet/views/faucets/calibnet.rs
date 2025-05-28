use super::Faucet;
use crate::faucet::constants::{
    CALIBNET_DRIP_AMOUNT, CALIBNET_RATE_LIMIT_SECONDS, FIL_CALIBNET_UNIT,
};
use crate::utils::format::format_balance;
use crate::utils::rpc_context::{Provider, RpcContext};
use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Calibnet() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    // Set rpc context to calibnet url
    rpc_context.set(Provider::get_network_url(Network::Testnet));

    view! {
        <Title text="Filecoin Faucet - Calibration Network" />
        <Meta
            name="description"
            content="Filecoin Calibration Network Faucet dispensing tokens for testing purposes."
        />
        <div>
            <h1 class="header">Filecoin Calibnet Faucet</h1>
            <Faucet target_network=Network::Testnet />
        </div>
        <div class="description">
            "This faucet distributes " {format_balance(&CALIBNET_DRIP_AMOUNT, FIL_CALIBNET_UNIT)}
            " per request. It is rate-limited to 1 request per " {CALIBNET_RATE_LIMIT_SECONDS}
            " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans."
        </div>
    }
}
