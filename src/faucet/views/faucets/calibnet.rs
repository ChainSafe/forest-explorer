use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::faucet::views::components::faucet_description::FaucetDescription;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Calibnet() -> impl IntoView {
    let drip_amount = FaucetInfo::CalibnetFIL.drip_amount();
    let token_unit = FaucetInfo::CalibnetFIL.unit();
    let rate_limit_seconds = FaucetInfo::CalibnetFIL.rate_limit_seconds();
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
            <Faucet faucet_info=FaucetInfo::CalibnetFIL />
            <FaucetDescription
                drip_amount=drip_amount.clone()
                token_unit=token_unit.to_string()
                rate_limit_seconds=rate_limit_seconds
            />
        </div>
    }
}
