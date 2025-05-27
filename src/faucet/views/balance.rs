// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use leptos::prelude::*;
use leptos::{component, view, IntoView};

use crate::faucet::FaucetController;
use crate::utils::format::format_balance;

#[component]
pub fn FaucetBalance(faucet: RwSignal<FaucetController>) -> impl IntoView {
    view! {
        <div>
            <h3 class="text-lg font-semibold">Faucet Balance:</h3>
            <Transition fallback=move || {
                view! { <p>Loading faucet balance...</p> }
            }>
                {move || {
                    if faucet.get().is_low_balance() {
                        let topup_req_url = option_env!("FAUCET_TOPUP_REQ_URL");
                        view! {
                            <a
                                class="bg-orange-500 hover:bg-orange-600 text-white font-bold text-sm py-1 px-2 rounded"
                                target="_blank"
                                rel="noopener noreferrer"
                                href=topup_req_url
                            >
                                "Request Faucet Top-up"
                            </a>
                        }
                            .into_any()
                    } else {
                        view! {
                            <p class="text-xl">
                                {format_balance(&faucet.get().get_faucet_balance(), &faucet.get().get_fil_unit())}
                            </p>
                        }
                            .into_any()
                    }
                }}
            </Transition>
        </div>
    }
}

#[component]
pub fn TargetBalance(faucet: RwSignal<FaucetController>) -> impl IntoView {
    view! {
        <div>
            <h3 class="text-lg font-semibold">Target Balance:</h3>
            <Transition fallback=move || view! { <p>Loading target balance...</p> }>
                <p class="text-xl">
                    {move || format_balance(&faucet.get().get_target_balance(), &faucet.get().get_fil_unit())}
                </p>
            </Transition>
        </div>
    }
}
