use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::faucet::views::components::faucet_description::FaucetDescription;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::{Meta, Title};

/// Displays the Calibnet Faucet page.
/// Sets the RPC context to calibnet and renders the faucet and its description.
#[component]
pub fn Faucet_Calibnet() -> impl IntoView {
    let faucet_info = FaucetInfo::CalibnetFIL;
    let rpc_context = RpcContext::use_context();
    // Set rpc context to calibnet url
    rpc_context.set(Provider::get_network_url(FaucetInfo::CalibnetFIL.network()));

    view! {
        <Title text="🧪 Filecoin Faucet - Calibration Network" />
        <Meta
            name="description"
            content="Filecoin Calibration Network Faucet dispensing tokens for testing purposes."
        />
        <h1 class="header">"🧪 Filecoin Calibnet Faucet"</h1>
        <div class="main-container">
            <Faucet faucet_info=faucet_info />
            <FaucetDescription faucet_info=faucet_info />
        </div>
    }
}
