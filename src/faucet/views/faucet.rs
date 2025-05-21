use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{component, leptos_dom::helpers::event_target_value, view, IntoView};
use leptos_meta::{Meta, Title};
#[cfg(feature = "hydrate")]
use leptos_use::use_interval_fn;
use url::Url;

use crate::faucet::views::alert::ErrorMessages;
use crate::faucet::views::balance::{FaucetBalance, TargetBalance};
use crate::faucet::views::nav::{GotoFaucetList, GotoHome};
use crate::faucet::views::transaction::{TransactionHistoryButton, TransactionList};
use crate::faucet::FaucetController;

#[component]
fn FaucetInput(faucet: RwSignal<FaucetController>) -> impl IntoView {
    view! {
        <div class="my-4 flex">
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
                class="flex-grow border border-gray-300 p-2 rounded-l"
            />
            {move || {
                if faucet.get().is_send_disabled() {
                    view! {
                        <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                            "Sending..."
                        </button>
                    }
                        .into_any()
                } else if faucet.get().get_send_rate_limit_remaining() > 0 {
                    let duration = faucet.get().get_send_rate_limit_remaining();
                    view! {
                        <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                            {format!("Rate-limited! {duration}s")}
                        </button>
                    }
                        .into_any()
                } else if faucet.get().is_low_balance() {
                    view! {
                        <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                            "Send"
                        </button>
                    }
                        .into_any()
                } else {
                    view! {
                        <button
                            class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded-r"
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
    let _ = use_interval_fn(
        move || {
            let duration = faucet.get().get_send_rate_limit_remaining();
            if duration > 0 {
                faucet.get().set_send_rate_limit_remaining(duration - 1);
            }
        },
        1000,
    );

    let _ = use_interval_fn(
        move || {
            faucet.get().refetch_balances();
        },
        5000,
    );
}

#[component]
pub fn Faucet(target_network: Network) -> impl IntoView {
    let faucet = RwSignal::new(FaucetController::new(target_network));

    #[cfg(feature = "hydrate")]
    {
        use_faucet_polling(faucet);
    }

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
        <div class="max-w-2xl mx-auto">
            <FaucetInput faucet=faucet />
            <div class="flex justify-between my-4">
                <FaucetBalance faucet=faucet />
                <TargetBalance faucet=faucet />
            </div>
            <hr class="my-4 border-t border-gray-300" />
            {move || {
                let messages = faucet.get().get_sent_messages();
                if !messages.is_empty() {
                    view! { <TransactionList messages=messages faucet_tx_base_url=faucet_tx_base_url /> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
        <div class="flex justify-center space-x-4">
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
        <div class="text-center">
            <h1 class="text-4xl font-bold mb-6 text-center">Filecoin Faucet List</h1>
            <a class="text-blue-600" href="/faucet/calibnet">
                Calibration Network Faucet
            </a>
            <br />
            <a class="text-blue-600" href="/faucet/mainnet">
                Mainnet Network Faucet
            </a>
        </div>
        <GotoHome />
    }
}
