use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::faucet::views::components::faucet_description::FaucetDescription;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::{Meta, Title};

/// Displays the Mainnet Faucet page.
/// Sets the RPC context to mainnet and renders the faucet and its description.
#[component]
pub fn Faucet_Mainnet() -> impl IntoView {
    let faucet_info = FaucetInfo::MainnetFIL;
    let rpc_context = RpcContext::use_context();
    // Set rpc context to mainnet url
    rpc_context.set(Provider::get_network_url(FaucetInfo::MainnetFIL.network()));

    view! {
        <Title text="🌐 Filecoin Faucet - Mainnet" />
        <Meta name="description" content="Filecoin Mainnet Faucet dispensing tokens for testing purposes." />

        <h1 class="header">"🌐 Filecoin Mainnet Faucet"</h1>
        <div class="main-container">
            <Faucet faucet_info=faucet_info />
            <FaucetDescription faucet_info=faucet_info />
        </div>
    }
}
