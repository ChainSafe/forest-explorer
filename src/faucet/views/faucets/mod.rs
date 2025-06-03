pub mod calibnet;
pub mod mainnet;

use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{component, leptos_dom::helpers::event_target_value, view, IntoView};
use leptos_meta::{Meta, Title};
use url::Url;

use crate::faucet::constants::FaucetInfo;
use crate::faucet::controller::FaucetController;
use crate::faucet::views::components::alert::ErrorMessages;
use crate::faucet::views::components::balance::{FaucetBalance, TargetBalance};
use crate::faucet::views::components::nav::{GotoFaucetList, GotoHome};
use crate::faucet::views::components::transaction::{TransactionHistoryButton, TransactionList};
use crate::utils::address::is_valid_prefix;

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
                            "Sending..."
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
                } else if faucet.get().is_low_balance() {
                    view! {
                        <button class="btn-disabled" disabled=true>
                            "Send"
                        </button>
                    }
                        .into_any()
                } else {
                    view! {
                        <button
                            class="btn-enabled"
                            on:click=move |_| {
                                faucet.get().drip();
                            }
                        >
                            Send
                        </button>
                    }
                        .into_any()
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

    let target_network = faucet_info.network();

    let faucet_tx_base_url = match target_network {
        Network::Mainnet => {
            RwSignal::new(option_env!("FAUCET_TX_URL_MAINNET").and_then(|url| Url::parse(url).ok()))
        }
        Network::Testnet => RwSignal::new(
            option_env!("FAUCET_TX_URL_CALIBNET").and_then(|url| Url::parse(url).ok()),
        ),
    };

    view! {
        {move || {
            let errors = faucet.get().get_error_messages();
            if !errors.is_empty() {
                view! { <ErrorMessages errors=errors faucet=faucet /> }.into_any()
            } else {
                ().into_any()
            }
        }}
        <div class="faucet-container">
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
            <Show when=move || is_valid_prefix(&faucet.get().get_sender_address(), target_network)>
                <TransactionHistoryButton faucet=faucet faucet_tx_base_url=faucet_tx_base_url />
            </Show>
            <GotoFaucetList />
        </div>
    }
}

#[component]
pub fn Faucets() -> impl IntoView {
    view! {
        <Title text="Filecoin Faucets" />
        <Meta name="description" content="Filecoin Faucet list" />
        <div class="faucet-list-container">
            <h1 class="header">Filecoin Faucet List</h1>
            <a class="link-text-hover" href="/faucet/calibnet">
                Calibration Network Faucet
            </a>
            <br />
            <a class="link-text-hover" href="/faucet/mainnet">
                Mainnet Network Faucet
            </a>
            <GotoHome />
        </div>
    }
}
