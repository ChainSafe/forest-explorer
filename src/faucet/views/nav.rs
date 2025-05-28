// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn GotoFaucetList() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="btn">
                <a href="/faucet">Faucet List</a>
            </button>
        </div>
    }
}

#[component]
pub fn GotoHome() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="btn">
                <a href="/">Home</a>
            </button>
        </div>
    }
}
