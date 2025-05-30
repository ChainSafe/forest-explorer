use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::utils::format::format_balance;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Mainnet() -> impl IntoView {
    let drip_amount = FaucetInfo::MainnetFIL.drip_amount();
    let token_unit = FaucetInfo::MainnetFIL.unit();
    let rate_limit_seconds = FaucetInfo::MainnetFIL.rate_limit_seconds();
    let rpc_context = RpcContext::use_context();
    // Set rpc context to mainnet url
    rpc_context.set(Provider::get_network_url(FaucetInfo::MainnetFIL.network()));

    view! {
        <Title text="🌐 Filecoin Faucet - Mainnet" />
        <Meta name="description" content="Filecoin Mainnet Faucet dispensing tokens for testing purposes." />
        <div>
            <h1 class="header">"🌐 Filecoin Mainnet Faucet"</h1>
            <Faucet faucet_info=FaucetInfo::MainnetFIL />
            <div class="description">
                "This faucet distributes " {format_balance(drip_amount, token_unit)}
                " per request. It is rate-limited to 1 request per " {rate_limit_seconds}
                " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans or service termination. Faucet funds are limited and may run out. They are replenished periodically."
            </div>
        </div>
    }
}
