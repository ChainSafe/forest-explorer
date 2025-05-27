// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::faucet::views::{faucet::Faucets, home::Explorer, layout::Footer};
use crate::faucet::{calibnet::views::Faucet_Calibnet, mainnet::views::Faucet_Mainnet};
use crate::utils::rpc_context::RpcContext;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    RpcContext::provide_context();

    view! {
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Router>
            <div class="flex flex-col min-h-screen space-y-8 py-10 px-6 min-h-screen">
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Explorer />
                    <Route path=path!("/faucet") view=Faucets />
                    <Route path=path!("/faucet/calibnet") view=Faucet_Calibnet />
                    <Route path=path!("/faucet/mainnet") view=Faucet_Mainnet />
                </Routes>
                <Footer />
            </div>
        </Router>
    }
}
