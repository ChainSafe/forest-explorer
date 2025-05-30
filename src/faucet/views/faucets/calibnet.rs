use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::utils::format::format_balance;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
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
        <div>
            <h1 class="header">"🧪 Filecoin Calibnet Faucet"</h1>
            <Faucet faucet_info=FaucetInfo::CalibnetFIL />
        </div>
        <div class="description">
            "This faucet distributes " {format_balance(drip_amount, token_unit)}
            " per request. It is rate-limited to 1 request per " {rate_limit_seconds}
            " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans."
        </div>
    }
}
