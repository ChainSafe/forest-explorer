use std::collections::HashSet;
use std::time::Duration;

use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::{component, leptos_dom::helpers::event_target_value, view, IntoView};
use leptos_meta::{Meta, Title};
#[cfg(feature = "hydrate")]
use leptos_use::*;
use url::Url;

use crate::faucet::controller::FaucetController;
use crate::faucet::utils::{format_balance, format_tx_url};
use crate::rpc_context::{Provider, RpcContext};

const MESSAGE_FADE_AFTER: Duration = Duration::new(3, 0);
const MESSAGE_REMOVAL_AFTER: Duration = Duration::new(3, 500_000_000);

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

    let (fading_messages, set_fading_messages) = signal(HashSet::new());
    let (drip_amount, faucet_tx_base_url) = match target_network {
        Network::Mainnet => (
            crate::constants::MAINNET_DRIP_AMOUNT.clone(),
            option_env!("FAUCET_TX_URL_MAINNET").and_then(|url| Url::parse(url).ok()),
        ),
        Network::Testnet => (
            crate::constants::CALIBNET_DRIP_AMOUNT.clone(),
            option_env!("FAUCET_TX_URL_CALIBNET").and_then(|url| Url::parse(url).ok()),
        ),
    };
    let topup_req_url = option_env!("FAUCET_TOPUP_REQ_URL");
    view! {
        {move || {
            let errors = faucet.get().get_error_messages();
            if !errors.is_empty() {
                view! {
                    <div class="fixed top-4 left-1/2 transform -translate-x-1/2 z-50">
                        {errors
                            .into_iter()
                            .map(|(id, error)| {
                                spawn_local(async move {
                                    // Start fading message after 3 seconds
                                    set_timeout(
                                        move || {
                                            set_fading_messages.update(|fading| { fading.insert(id); });
                                        },
                                        MESSAGE_FADE_AFTER,
                                    );

                                    // Remove message after 3.5 seconds
                                    set_timeout(
                                        move || {
                                            set_fading_messages.update(|fading| {
                                                fading.remove(&id);
                                            });

                                            faucet.get().remove_error_message(id);
                                        },
                                        MESSAGE_REMOVAL_AFTER,
                                    );
                                });

                                view! {
                                    <div
                                    class=move || {
                                        if fading_messages.get().contains(&id) {
                                            "opacity-0 transition-opacity bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-2 w-96"
                                        } else {
                                            "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-2 w-96"
                                        }
                                    }
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
                                                    faucet.get().remove_error_message(id);
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
                    .into_any()
            } else {
                ().into_any()
            }
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
                    if faucet.get().is_send_disabled() {
                        view! {
                            <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                                "Sending..."
                            </button>
                        }.into_any()
                    } else if faucet.get().get_send_rate_limit_remaining() > 0 {
                        let duration = faucet.get().get_send_rate_limit_remaining();
                        view! {
                            <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                                {format!("Rate-limited! {duration}s")}
                            </button>
                        }.into_any()
                    } else if faucet.get().get_faucet_balance() < drip_amount {
                        view! {
                            <a href={topup_req_url} target="_blank" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-r">
                                "Request Faucet Top-up"
                            </a>
                        }.into_any()
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
                        }.into_any()
                    }
                }}

            </div>
            <div class="flex justify-between my-4">
                <div>
                    <h3 class="text-lg font-semibold">Faucet Balance:</h3>
                    <Transition fallback={move || view!{ <p>Loading faucet balance...</p> }}>
                        <p class="text-xl">{ move || format_balance(&faucet.get().get_faucet_balance(), &faucet.get().get_fil_unit()) }</p>
                    </Transition>
                </div>
                <div>
                    <h3 class="text-lg font-semibold">Target Balance:</h3>
                    <Transition fallback={move || view!{ <p>Loading target balance...</p> }}>
                        <p class="text-xl">{ move || format_balance(&faucet.get().get_target_balance(), &faucet.get().get_fil_unit()) }</p>
                    </Transition>
                </div>
            </div>
            <hr class="my-4 border-t border-gray-300" />
            {move || {
                let messages = faucet.get().get_sent_messages();
                if !messages.is_empty() {
                    view! {
                        <div class="mt-4">
                            <h3 class="text-lg font-semibold">Transactions:</h3>
                            <ul class="list-disc pl-5">
                                {messages
                                    .into_iter()
                                    .map(|(msg, sent)| {
                                        let mut cid = view! {{msg.to_string()}}.into_any();
                                        let status;
                                        if sent {
                                            status = "(confirmed)";
                                            if let Some(ref base_url) = faucet_tx_base_url {
                                                if let Ok(tx_url) = format_tx_url(base_url, &msg.to_string()) {
                                                    cid = view! {
                                                        <a href=tx_url.to_string() target="_blank" class="text-blue-600 hover:underline">
                                                            {msg.to_string()}
                                                        </a>
                                                    }.into_any();
                                                }
                                            }
                                        } else {
                                            status = "(pending)";
                                        }
                                        view! {
                                            <li>
                                                "CID:" {cid} {status}
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </ul>
                        </div>
                    }
                    .into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
        <div class="flex flex-col items-center">
            <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded-full">
              <a href="/faucet">Back to faucet list</a>
            </button>
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
                <a class="text-blue-600" href="/faucet/calibnet">Calibration Network Faucet</a><br />
                <a class="text-blue-600" href="/faucet/mainnet">Mainnet Network Faucet</a>
        </div>
    }
}

#[component]
pub fn Faucet_Calibnet() -> impl IntoView {
    let rpc_context = RpcContext::use_context();

    // update the rpc context to the calibnet url
    Effect::new(move |_| {
        rpc_context.set(Provider::get_network_url(Network::Testnet));
    });

    view! {
        <Title text="Filecoin Faucet - Calibration Network" />
        <Meta name="description" content="Filecoin Calibration Network Faucet dispensing tokens for testing purposes." />
        <div>
            <h1 class="text-4xl font-bold mb-6 text-center">Filecoin Calibnet Faucet</h1>
            <Faucet target_network=Network::Testnet />
        </div>
        <div class="text-center mt-4">
            "This faucet distributes " { format_balance(&crate::constants::CALIBNET_DRIP_AMOUNT, crate::constants::FIL_CALIBNET_UNIT) } " per request. It is rate-limited to 1 request per " {crate::constants::RATE_LIMIT_SECONDS} " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans."
        </div>
    }
}

#[component]
pub fn Faucet_Mainnet() -> impl IntoView {
    let rpc_context = RpcContext::use_context();

    // update the rpc context to the mainnet url
    Effect::new(move |_| {
        rpc_context.set(Provider::get_network_url(Network::Mainnet));
    });

    view! {
        <Title text="Filecoin Faucet - Mainnet" />
        <Meta name="description" content="Filecoin Mainnet Faucet dispensing tokens for testing purposes." />
        <div>
            <h1 class="text-4xl font-bold mb-6 text-center">Filecoin Mainnet Faucet</h1>
            <Faucet target_network=Network::Mainnet />
        <div class="text-center mt-4">
            "This faucet distributes " { format_balance(&crate::constants::MAINNET_DRIP_AMOUNT, crate::constants::FIL_MAINNET_UNIT) } " per request. It is rate-limited to 1 request per " {crate::constants::RATE_LIMIT_SECONDS} " seconds. Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans or service termination. Faucet funds are limited and may run out. They are replenished periodically."
        </div>
        </div>
    }
}
