use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::faucet::views::components::faucet_description::FaucetDescription;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Mainnet() -> impl IntoView {
    let drip_amount = FaucetInfo::MainnetFIL.drip_amount();
    let token_unit = FaucetInfo::MainnetFIL.unit();
    let rate_limit_seconds = FaucetInfo::MainnetFIL.rate_limit_seconds();
    let wallet_cap = FaucetInfo::MainnetFIL.wallet_cap();
    let rpc_context = RpcContext::use_context();
    let wallet_limit_seconds = FaucetInfo::MainnetFIL.wallet_limit_seconds();
    // Set rpc context to mainnet url
    rpc_context.set(Provider::get_network_url(FaucetInfo::MainnetFIL.network()));

    view! {
        <Title text="🌐 Filecoin Faucet - Mainnet" />
        <Meta name="description" content="Filecoin Mainnet Faucet dispensing tokens for testing purposes." />

        <h1 class="header">"🌐 Filecoin Mainnet Faucet"</h1>
        <div class="main-container">
            <Faucet faucet_info=FaucetInfo::MainnetFIL />
            <FaucetDescription
                drip_amount=drip_amount.clone()
                wallet_cap=wallet_cap
                token_unit=token_unit.to_string()
                rate_limit_seconds=rate_limit_seconds
                wallet_limit_seconds=wallet_limit_seconds
            />
        </div>
    }
}
