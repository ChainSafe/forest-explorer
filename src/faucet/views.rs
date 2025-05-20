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

use crate::faucet::FaucetController;
use crate::rpc_context::{Provider, RpcContext};
use crate::utils::format::{format_balance, format_url, SearchPath};
use crate::utils::icons::{CheckIcon, LightningIcon};

const MESSAGE_FADE_AFTER: Duration = Duration::new(3, 0);
const MESSAGE_REMOVAL_AFTER: Duration = Duration::new(3, 500_000_000);

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static + Send) -> impl IntoView {
    view! { <span class:loader=loading /> }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="py-4 text-center">
            <a
                class="text-green-600"
                target="_blank"
                rel="noopener noreferrer"
                href="https://github.com/ChainSafe/forest-explorer"
            >
                Forest Explorer
            </a>
            ", built with ❤️ by "
            <a class="text-blue-600" target="_blank" rel="noopener noreferrer" href="https://chainsafe.io">
                ChainSafe Systems
            </a>
        </footer>
    }
}

#[component]
pub fn BlockchainExplorer() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    let network_name = LocalResource::new(move || {
        let provider = rpc_context.get();
        async move { provider.network_name().await.ok() }
    });

    let network_version = LocalResource::new(move || {
        let provider = rpc_context.get();
        async move { provider.network_version().await.ok() }
    });

    view! {
        <main class="min-h-screen flex flex-col flex-grow space-y-8">
            <div class="space-y-6 flex flex-col items-center">
                <h1 class="text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">
                    Filecoin Forest Explorer Faucet
                </h1>
                <p class="max-w-2xl text-center">
                    The Filecoin Forest Explorer Faucet provides developers and users with free calibnet(tFil) and mainnet(FIL) to support their exploration, testing and development on the Filecoin network.
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl w-full m-auto">
                <div class="bg-white p-6 rounded-lg border border-gray-100">
                    <h2 class="text-lg font-semibold text-gray-900 mb-4">What does the faucet offer?</h2>
                    <ul class="space-y-3">
                        <li class="flex items-start">
                            <CheckIcon />
                            <span class="text-gray-600">Free calibnet tFIL for experimentation and development.</span>
                        </li>
                        <li class="flex items-start">
                            <CheckIcon />
                            <span class="text-gray-600">
                                Real mainnet FIL for contributors engaging with the Filecoin ecosystem.
                            </span>
                        </li>
                        <li class="flex items-start">
                            <CheckIcon />
                            <span class="text-gray-600">
                                A Quick and Easy way to request free tFIL and FIL - Just enter your wallet address.
                            </span>
                        </li>
                    </ul>
                </div>

                <div class="bg-white p-6 rounded-lg border border-gray-100">
                    <h2 class="text-lg font-semibold text-gray-900 mb-4">Why use this faucet?</h2>
                    <ul class="space-y-3">
                        <li class="flex items-start">
                            <LightningIcon class="text-blue-500".to_string() />
                            <span class="text-gray-600">
                                Supports both calibnet and mainnet, unlike typical faucets.
                            </span>
                        </li>
                        <li class="flex items-start">
                            <LightningIcon class="text-blue-500".to_string() />
                            <span class="text-gray-600">
                                Enables testing of smart contracts, storage deals, and blockchain interactions without financial risk.
                            </span>
                        </li>
                        <li class="flex items-start">
                            <LightningIcon class="text-blue-500".to_string() />
                            <span class="text-gray-600">
                                Easy access to FIL for developers and users building on Filecoin.
                            </span>
                        </li>
                        <li class="flex items-start">
                            <LightningIcon class="text-blue-500".to_string() />
                            <span class="text-gray-600">
                                Need help? Visit the
                                <a class="text-blue-500" href="https://filecoin.io/slack" target="_blank">
                                    {" "}
                                    Filecoin Slack
                                </a>{" "}or <a class="text-blue-500" href="https://docs.filecoin.io" target="_blank">
                                    {" "}
                                    documentation
                                </a>.
                            </span>
                        </li>
                    </ul>
                </div>
            </div>

            <div class="space-y-6 flex flex-col items-center">
                <div class="relative w-64">
                    <select
                        on:change=move |ev| { rpc_context.set(event_target_value(&ev)) }
                        class="w-full px-4 py-2 text-sm text-gray-700 bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 hover:border-gray-400 transition-colors cursor-pointer appearance-none"
                    >
                        <option value=Provider::get_network_url(Network::Testnet)>Glif.io Calibnet</option>
                        <option value=Provider::get_network_url(Network::Mainnet)>Glif.io Mainnet</option>
                    </select>
                    <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </div>
                </div>
                <div class="flex items-center w-[300px] justify-between">
                    <p>Network:</p>
                    <Transition fallback=move || view! { <p>Loading network name...</p> }>
                        <p>
                            <span>{move || network_name.get().flatten()}</span>
                            <Loader loading=move || network_name.get().is_none() />
                        </p>
                    </Transition>
                </div>
                <div class="flex items-center w-[300px] justify-between">
                    <p>Version:</p>
                    <Transition fallback=move || view! { <p>Loading network version...</p> }>
                        <p>
                            <span>{move || network_version.get().flatten()}</span>
                            <Loader loading=move || network_version.get().is_none() />
                        </p>
                    </Transition>
                </div>
                <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded-lg">
                    <a href="/faucet">To faucet list</a>
                </button>
            </div>

        </main>
    }
}

#[component]
fn FaucetBalance(faucet: RwSignal<FaucetController>) -> impl IntoView {
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
fn TargetBalance(faucet: RwSignal<FaucetController>) -> impl IntoView {
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
                view! {
                    <div class="fixed top-4 left-1/2 transform -translate-x-1/2 z-50">
                        {errors
                            .into_iter()
                            .map(|(id, error)| {
                                spawn_local(async move {
                                    set_timeout(
                                        move || {
                                            set_fading_messages
                                                .update(|fading| {
                                                    fading.insert(id);
                                                });
                                        },
                                        MESSAGE_FADE_AFTER,
                                    );
                                    set_timeout(
                                        move || {
                                            set_fading_messages
                                                .update(|fading| {
                                                    fading.remove(&id);
                                                });
                                            faucet.get().remove_error_message(id);
                                        },
                                        MESSAGE_REMOVAL_AFTER,
                                    );
                                });
                                // Start fading message after 3 seconds

                                // Remove message after 3.5 seconds

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
            <div class="flex justify-between my-4">
                <FaucetBalance faucet=faucet />
                <TargetBalance faucet=faucet />
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
                                        let (cid, status) = if sent {
                                            let cid = faucet_tx_base_url
                                                .get()
                                                .as_ref()
                                                .and_then(|base_url| {
                                                    format_url(base_url, SearchPath::Transaction, &msg.to_string()).ok()
                                                })
                                                .map(|tx_url| {
                                                    view! {
                                                        <a
                                                            href=tx_url.to_string()
                                                            target="_blank"
                                                            class="text-blue-600 hover:underline"
                                                        >
                                                            {msg.to_string()}
                                                        </a>
                                                    }
                                                        .into_any()
                                                })
                                                .unwrap_or_else(|| view! { {msg.to_string()} }.into_any());
                                            (cid, "(confirmed)")
                                        } else {
                                            let cid = view! { {msg.to_string()} }.into_any();
                                            (cid, "(pending)")
                                        };
                                        view! { <li>"CID:" {cid} {status}</li> }
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
        <div class="flex justify-center space-x-4">
            {move || {
                match faucet_tx_base_url.get() {
                    Some(ref base_url) => {
                        match format_url(base_url, SearchPath::Address, &faucet.get().get_sender_address()) {
                            Ok(addr_url) => {
                                view! {
                                    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded-full">
                                        <a href=addr_url.to_string() target="_blank" rel="noopener noreferrer">
                                            "Transaction History"
                                        </a>
                                    </button>
                                }
                                    .into_any()
                            }
                            Err(_) => ().into_any(),
                        }
                    }
                    None => ().into_any(),
                }
            }} <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded-full">
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
            <a class="text-blue-600" href="/faucet/calibnet">
                Calibration Network Faucet
            </a>
            <br />
            <a class="text-blue-600" href="/faucet/mainnet">
                Mainnet Network Faucet
            </a>
        </div>
    }
}
