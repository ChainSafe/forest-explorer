// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::faucet::mainnet::{FIL_MAINNET_UNIT, MAINNET_DRIP_AMOUNT, MAINNET_RATE_LIMIT_SECONDS};
use crate::faucet::views::faucet::Faucet;
use crate::utils::format::format_balance;
use crate::utils::rpc_context::{Provider, RpcContext};
use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Mainnet() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    // Set rpc context to mainnet url
    rpc_context.set(Provider::get_network_url(Network::Mainnet));

    view! {
        <Title text="Filecoin Faucet - Mainnet" />
        <Meta name="description" content="Filecoin Mainnet Faucet dispensing tokens for testing purposes." />
        <div>
            <h1 class="text-4xl font-bold mb-6 text-center">Filecoin Mainnet Faucet</h1>
            <Faucet target_network=Network::Mainnet />
            <div class="text-center mt-4">
                "This faucet distributes " {format_balance(&MAINNET_DRIP_AMOUNT, FIL_MAINNET_UNIT)}
                " per request. It is rate-limited to 1 request per " {MAINNET_RATE_LIMIT_SECONDS}
                " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans or service termination. Faucet funds are limited and may run out. They are replenished periodically."
            </div>
        </div>
    }
}
