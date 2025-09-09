pub mod calibnet;
pub mod calibnet_usdfc;
pub mod mainnet;

use leptos::prelude::*;
use leptos::{IntoView, component, leptos_dom::helpers::event_target_value, view};
use leptos_meta::{Meta, Title};

use crate::faucet::constants::FaucetInfo;
use crate::faucet::controller::FaucetController;
use crate::faucet::views::components::alert::ErrorMessages;
use crate::faucet::views::components::balance::{FaucetBalance, TargetBalance};
use crate::faucet::views::components::nav::{GotoFaucetList, GotoHome};
use crate::faucet::views::components::transaction::{TransactionHistoryButton, TransactionList};

#[component]
fn FaucetInput(faucet: RwSignal<FaucetController>) -> impl IntoView {
    view! {
        <div class="input-container">
            <input
                type="text"
                placeholder="Enter target address (Filecoin or Ethereum style)"
                prop:value=faucet.get().get_target_address()
                on:input=move |ev| { faucet.get().set_target_address(event_target_value(&ev)) }
                on:keydown=move |ev| {
                    if ev.key() == "Enter" && !faucet.get().is_send_disabled()
                        && faucet.get().get_send_rate_limit_remaining() <= 0
                    {
                        faucet.get().drip();
                    }
                }
                class="input"
            />
            {move || {
                if faucet.get().is_send_disabled() {
                    view! {
                        <button class="btn-disabled" disabled=true>
                            "Claiming..."
                        </button>
                    }
                        .into_any()
                } else if faucet.get().get_send_rate_limit_remaining() > 0 {
                    let duration = faucet.get().get_send_rate_limit_remaining();
                    view! {
                        <button class="btn-disabled" disabled=true>
                            {format!("Rate-limited! {duration}s")}
                        </button>
                    }
                        .into_any()
                } else {
                    let unit = faucet.get().get_fil_unit();
                    let disabled = faucet.get().is_low_balance();
                    let btn_class = if disabled { "btn-disabled" } else { "btn-enabled" };
                    if disabled {
                        view! {
                            <button class=btn_class disabled=true>
                                {format!("Claim {unit}")}
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <button
                                class=btn_class
                                on:click=move |_| {
                                    faucet.get().drip();
                                }
                            >
                                {format!("Claim {unit}")}
                            </button>
                        }
                            .into_any()
                    }
                }
            }}
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn use_faucet_polling(faucet: RwSignal<FaucetController>) {
    use leptos_use::use_interval_fn;
    use leptos_use::utils::Pausable;

    let Pausable {
        pause: pause_rate_limiter,
        ..
    } = use_interval_fn(
        move || {
            let duration = faucet.get().get_send_rate_limit_remaining();
            if duration > 0 {
                faucet.get().set_send_rate_limit_remaining(duration - 1);
            }
        },
        1000,
    );

    let Pausable {
        pause: pause_refetch_balances,
        ..
    } = use_interval_fn(
        move || {
            faucet.get().refetch_balances();
        },
        5000,
    );

    on_cleanup(move || {
        pause_rate_limiter();
        pause_refetch_balances();
    });
}

#[component]
pub fn Faucet(faucet_info: FaucetInfo) -> impl IntoView {
    let faucet = RwSignal::new(FaucetController::new(faucet_info));

    #[cfg(feature = "hydrate")]
    {
        use_faucet_polling(faucet);
    }

    let faucet_tx_base_url = RwSignal::new(faucet_info.transaction_base_url());

    view! {
        {move || {
            let errors = faucet.get().get_error_messages();
            if !errors.is_empty() {
                view! { <ErrorMessages errors=errors faucet=faucet /> }.into_any()
            } else {
                ().into_any()
            }
        }}
        <div class="faucet-section">
            <FaucetInput faucet=faucet />
            <div class="balance-container">
                <FaucetBalance faucet=faucet />
                <TargetBalance faucet=faucet />
            </div>
            <hr class="separator" />
            {move || {
                let messages = faucet.get().get_sent_messages();
                if !messages.is_empty() {
                    view! { <TransactionList messages=messages faucet_tx_base_url=faucet_tx_base_url /> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
        <div class="nav-container">
            <TransactionHistoryButton faucet=faucet faucet_tx_base_url=faucet_tx_base_url />
            <GotoFaucetList />
        </div>
    }
}

#[component]
pub fn Faucets() -> impl IntoView {
    view! {
        <Title text="Filecoin Faucets" />
        <Meta name="description" content="Filecoin Faucet list" />
        <h1 class="header">Filecoin Faucet List</h1>
        <div class="main-container">
            <div class="faucet-list-items">
                <a class="link-text" href="/faucet/calibnet_usdfc">
                    "💰 Calibration Network USDFC Faucet"
                </a>
                <a class="link-text" href="/faucet/calibnet">
                    "🧪 Calibration Network Faucet"
                </a>
                <a class="link-text" href="/faucet/mainnet">
                    "🌐 Mainnet Network Faucet"
                </a>
            </div>
            <GotoHome />
        </div>
    }
}
