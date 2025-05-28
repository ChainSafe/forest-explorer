use leptos::prelude::*;
use leptos::{component, view, IntoView};

use crate::faucet::controller::FaucetController;
use crate::utils::format::format_balance;

#[component]
pub fn FaucetBalance(faucet: RwSignal<FaucetController>) -> impl IntoView {
    view! {
        <div>
            <h3 class="title">Faucet Balance:</h3>
            <Transition fallback=move || {
                view! { <p>Loading faucet balance...</p> }
            }>
                {move || {
                    if faucet.get().is_low_balance() {
                        let topup_req_url = option_env!("FAUCET_TOPUP_REQ_URL");
                        view! {
                            <a class="btn-topup" target="_blank" rel="noopener noreferrer" href=topup_req_url>
                                "Request Faucet Top-up"
                            </a>
                        }
                            .into_any()
                    } else {
                        view! {
                            <p class="balance">
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
            <h3 class="title">Target Balance:</h3>
            <Transition fallback=move || view! { <p>Loading target balance...</p> }>
                <p class="balance">
                    {move || format_balance(&faucet.get().get_target_balance(), &faucet.get().get_fil_unit())}
                </p>
            </Transition>
        </div>
    }
}
