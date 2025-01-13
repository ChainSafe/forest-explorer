use fvm_shared::address::Network;
use leptos::{component, leptos_dom::helpers::event_target_value, view, IntoView};

use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos_use::*;

use crate::faucet::controller::FaucetController;
use crate::faucet::utils::format_balance;

#[component]
pub fn Faucet(target_network: Network) -> impl IntoView {
    let faucet = RwSignal::new(FaucetController::new(target_network));

    #[cfg(feature = "hydrate")]
    let _ = use_interval_fn(
        move || {
            let duration = faucet.get().get_send_rate_limit_remaining();
            if duration > 0 {
                faucet.get().set_send_rate_limit_remaining(duration - 1);
            }
        },
        1000,
    );

    #[cfg(feature = "hydrate")]
    let _ = use_interval_fn(
        move || {
            faucet.get().refetch_balances();
        },
        5000,
    );

    view! {
        {move || {
            let errors = faucet.get().get_error_messages();
                view! {
                    <div class="fixed top-4 left-1/2 transform -translate-x-1/2 z-50">
                        {errors
                            .into_iter()
                            .enumerate()
                            .map(|(index, error)| {
                                view! {
                                    <div
                                        class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-2 w-96"
                                        role="alert"
                                    >
                                        <span class="block sm:inline">{error}</span>
                                        <span class="absolute top-0 bottom-0 right-0 px-4 py-3">
                                            <svg
                                                class="fill-current h-6 w-6 text-red-500"
                                                role="button"
                                                xmlns="http://www.w3.org/2000/svg"
                                                viewBox="0 0 20 20"
                                                on:click=move |_| {
                                                    faucet.get().remove_error_message(index);
                                                }
                                            >
                                                <title>Close</title>
                                                <path d="M14.348 14.849a1.2 1.2 0 0 1-1.697 0L10 11.819l-2.651 3.029a1.2 1.2 0 1 1-1.697-1.697l2.758-3.15-2.759-3.152a1.2 1.2 0 1 1 1.697-1.697L10 8.183l2.651-3.031a1.2 1.2 0 1 1 1.697 1.697l-2.758 3.152 2.758 3.15a1.2 1.2 0 0 1 0 1.698z" />
                                            </svg>
                                        </span>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                }
                    .into_view()
        }}
        <div class="max-w-2xl mx-auto">
            <div class="my-4 flex">
                <input
                    type="text"
                    placeholder="Enter target address (Filecoin or Ethereum style)"
                    prop:value=faucet.get().get_target_address()
                    on:input=move |ev| { faucet.get().set_target_address(event_target_value(&ev)) }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" && !faucet.get().is_send_disabled() && faucet.get().get_send_rate_limit_remaining() <= 0 {
                            faucet.get().drip();
                        }
                    }
                    class="flex-grow border border-gray-300 p-2 rounded-l"
                />
                {move || {
                    let is_disabled = faucet.get().is_send_disabled() || faucet.get().get_send_rate_limit_remaining() > 0;
                    let button_text = if faucet.get().is_send_disabled() {
                        "Sending...".to_string()
                    } else if faucet.get().get_send_rate_limit_remaining() > 0 {
                        let duration = faucet.get().get_send_rate_limit_remaining();
                        format!("Rate-limited! {duration}s")
                    } else {
                        "Send".to_string()
                    };
                    view! {
                        <button
                            class={if is_disabled {
                                "bg-gray-400 text-white font-bold py-2 px-4 rounded-r"
                            } else {
                                "bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded-r"
                            }}
                            disabled={is_disabled}
                            on:click=move |_| {
                                if !is_disabled {
                                    faucet.get().drip();
                                }
                            }
                        >
                            {button_text}
                        </button>
                    }
                }}

            </div>
            <div class="flex justify-between my-4">
                <div>
                    <h3 class="text-lg font-semibold">Faucet Balance:</h3>
                    <p class="text-xl">{ move || format_balance(&faucet.get().get_faucet_balance(), &faucet.get().get_fil_unit()) }</p>
                </div>
                <div>
                    <h3 class="text-lg font-semibold">Target Balance:</h3>
                    <p class="text-xl">{ move || format_balance(&faucet.get().get_target_balance(), &faucet.get().get_fil_unit()) }</p>
                </div>
            </div>
            <hr class="my-4 border-t border-gray-300" />
            {move || {
                let messages = faucet.get().get_sent_messages();
                    view! {
                        <div class="mt-4">
                            <h3 class="text-lg font-semibold">Transactions:</h3>
                            <ul class="list-disc pl-5">
                                {messages
                                    .into_iter()
                                    .map(|(msg, sent)| {
                                        view! {
                                            <li>
                                                "CID: " {msg.to_string()}
                                                {move || if sent { " (confirmed)" } else { " (pending)" }}
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </ul>
                        </div>
                    }
                        .into_view()
            }}
        </div>
    }
}

#[component]
pub fn Faucets() -> impl IntoView {
    view! {
        <div class="text-center">
            <h2 class="text-2xl font-bold mb-4">Faucet List</h2>
                <a class="text-blue-600" href="/faucet/calibnet">Calibration Network Faucet</a><br />
                <a class="text-blue-600" href="/faucet/mainnet">Mainnet Network Faucet</a>
        </div>
    }
}

#[component]
pub fn Faucet_Calibnet() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-4xl font-bold mb-6 text-center">Calibnet Faucet</h1>
            <Faucet target_network=Network::Testnet />
        </div>
        <div class="text-center mt-4">
            "This faucet distributes " { format_balance(&crate::constants::CALIBNET_DRIP_AMOUNT, crate::constants::FIL_CALIBNET_UNIT) } " per request. It is rate-limited to 1 request per " {crate::constants::RATE_LIMIT_SECONDS} " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans."
        </div>
    }
}

#[component]
pub fn Faucet_Mainnet() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-4xl font-bold mb-6 text-center">Mainnet Faucet</h1>
            <Faucet target_network=Network::Mainnet />
        <div class="text-center mt-4">
            "This faucet distributes " { format_balance(&crate::constants::MAINNET_DRIP_AMOUNT, crate::constants::FIL_MAINNET_UNIT) } " per request. It is rate-limited to 1 request per " {crate::constants::RATE_LIMIT_SECONDS} " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans or service termination. Faucet funds are limited and may run out. They are replenished periodically."
        </div>
        </div>
    }
}
