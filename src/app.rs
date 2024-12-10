use std::future::Future;

use cid::Cid;
use fvm_shared::address::Network;
use fvm_shared::econ::TokenAmount;
use leptos::{component, create_local_resource, event_target_value, view, IntoView, SignalGet};
use leptos_meta::*;

use leptos::*;
use leptos_router::*;
#[cfg(feature = "hydrate")]
use leptos_use::*;

use crate::faucet::sign_with_secret_key;
use crate::{faucet::faucet_address, message::message_transfer, rpc_context::Provider, message::parse_address};
use crate::{lotus_json::LotusJson, rpc_context::RpcContext};

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static) -> impl IntoView {
    view! { <span class:loader=loading /> }
}

#[component]
pub fn BlockchainExplorer() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    let network_name = create_local_resource(
        move || rpc_context.get(),
        move |provider| async move { provider.network_name().await.ok() },
    );

    let network_version = create_local_resource(
        move || rpc_context.get(),
        move |provider| async move { provider.network_version().await.ok() },
    );

    view! {
        <h1 class="mb-4 text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">
            Forest Explorer
        </h1>
        <select on:change=move |ev| { rpc_context.set(event_target_value(&ev)) }>
            <option value="https://api.calibration.node.glif.io">Glif.io Calibnet</option>
            <option value="https://api.node.glif.io/">Glif.io Mainnet</option>
        </select>
        <p>StateNetworkName</p>
        <p class="px-8">
            <span>{move || network_name.get()}</span>
            <Loader loading=move || network_name.loading().get() />
        </p>
        <p>StateNetworkVersion</p>
        <p class="px-8">
            <span>{move || network_version.get()}</span>
            <Loader loading=move || network_name.loading().get() />
        </p>
    }
}

async fn catch_all(
    errors: RwSignal<Vec<String>>,
    cb: impl Future<Output = Result<(), anyhow::Error>>,
) {
    match cb.await {
        Ok(_) => (),
        Err(e) => errors.update(|errors| errors.push(e.to_string())),
    }
}

#[component]
pub fn Faucet(target_network: Network) -> impl IntoView {
    let is_mainnet = target_network == Network::Mainnet;
    let error_messages = create_rw_signal(vec![]);

    // Disable the send button while we're asking the RPC provider for the nonce
    let send_button_disabled = create_rw_signal(false);
    // Disable the send button if rate-limited
    let send_button_limited = create_rw_signal(0);

    #[cfg(feature = "hydrate")]
    let _ = use_interval_fn(
        move || {
            let duration = send_button_limited.get();
            if duration > 0 {
                send_button_limited.set(duration - 1);
            }
        },
        1000,
    );

    let sender_address = create_local_resource(
        move || (),
        move |()| async move {
            faucet_address(is_mainnet)
                .await
                .map(|LotusJson(addr)| addr)
                .ok()
        },
    );
    let faucet_balance = create_local_resource_with_initial_value(
        move || sender_address.get(),
        move |addr| async move {
            if let Some(Some(addr)) = addr {
                Provider::from_network(target_network)
                    .wallet_balance(addr)
                    .await
                    .ok()
                    .unwrap_or(TokenAmount::from_atto(0))
            } else {
                TokenAmount::from_atto(0)
            }
        },
        Some(TokenAmount::from_atto(0)),
    );
    let target_address = create_rw_signal(String::new());
    let target_balance = create_local_resource_with_initial_value(
        move || target_address.get(),
        move |address| async move {
            if let Ok(address) = parse_address(&address) {
                Provider::from_network(target_network)
                    .wallet_balance(address)
                    .await
                    .ok()
                    .unwrap_or(TokenAmount::from_atto(0))
            } else {
                TokenAmount::from_atto(0)
            }
        },
        Some(TokenAmount::from_atto(0)),
    );

    let sent_messages: RwSignal<Vec<(Cid, bool)>> = create_rw_signal(Vec::new());

    #[cfg(feature = "hydrate")]
    let _ = use_interval_fn(
        move || {
            log::info!("Checking for new transactions");
            target_balance.refetch();
            faucet_balance.refetch();
            let pending = sent_messages
                .get_untracked()
                .into_iter()
                .filter_map(|(cid, sent)| if !sent { Some(cid) } else { None })
                .collect::<Vec<_>>();
            spawn_local(catch_all(error_messages, async move {
                for cid in pending {
                    if let Some(lookup) = Provider::from_network(target_network)
                        .state_search_msg(cid)
                        .await?
                    {
                        sent_messages.update(|messages| {
                            for (cid, sent) in messages {
                                if cid == &lookup.message {
                                    *sent = true;
                                }
                            }
                        });
                    }
                }
                Ok(())
            }));
        },
        5000,
    );

    view! {
        {move || {
            let errors = error_messages.get();
            if !errors.is_empty() {
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
                                                    error_messages
                                                        .update(|msgs| {
                                                            msgs.remove(index);
                                                        });
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
            } else {
                view! {}.into_view()
            }
        }}
        <h1 class="text-4xl font-bold mb-6 text-center">Faucet</h1>

        <div class="max-w-2xl mx-auto">
            <div class="my-4 flex">
                <input
                    type="text"
                    placeholder="Enter target address"
                    prop:value=target_address
                    on:input=move |ev| { target_address.set(event_target_value(&ev)) }
                    class="flex-grow border border-gray-300 p-2 rounded-l"
                />
                {move || {
                    if send_button_disabled.get() {
                        view! {
                            <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                                "Sending..."
                            </button>
                        }
                    } else if send_button_limited.get() > 0 {
                        let duration = send_button_limited.get();
                        view! {
                            <button class="bg-gray-400 text-white font-bold py-2 px-4 rounded-r" disabled=true>
                                {format!("Rate-limited! {duration}s")}
                            </button>
                        }
                    } else {
                        view! {
                            <button
                                class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded-r"
                                on:click=move |_| {
                                    match parse_address(&target_address.get()) {
                                        Ok(addr) => {
                                            spawn_local(async move {
                                                catch_all(
                                                        error_messages,
                                                        async move {
                                                            let rpc = Provider::from_network(target_network);
                                                            let LotusJson(from) =
                                                                faucet_address(is_mainnet)
                                                                    .await
                                                                    .map_err(|e| {
                                                                    anyhow::anyhow!("Error getting faucet address: {}", e)
                                                                })?;
                                                            send_button_disabled.set(true);
                                                            let nonce = rpc.mpool_get_nonce(from).await?;
                                                            let mut msg = message_transfer(
                                                                from,
                                                                addr,
                                                                if is_mainnet {
                                                                    TokenAmount::from_nano(1_000_000)
                                                                } else {
                                                                    TokenAmount::from_whole(1)
                                                                },
                                                            );
                                                            msg.sequence = nonce;
                                                            let msg = rpc.estimate_gas(msg).await?;
                                                            match sign_with_secret_key(
                                                                    LotusJson(msg.clone()),
                                                                    is_mainnet,
                                                                )
                                                                .await
                                                            {
                                                                Ok(LotusJson(smsg)) => {
                                                                    let cid = rpc.mpool_push(smsg).await?;
                                                                    sent_messages
                                                                        .update(|messages| {
                                                                            messages.push((cid, false));
                                                                        });
                                                                    log::info!("Sent message: {:?}", cid);
                                                                }
                                                                Err(e) => {
                                                                    log::error!("Failed to sign message: {}", e);
                                                                    send_button_limited.set(30);
                                                                }
                                                            }
                                                            Ok(())
                                                        },
                                                    )
                                                    .await;
                                                send_button_disabled.set(false);
                                            });
                                        }
                                        Err(e) => {
                                            error_messages.update(|errors| errors.push("Invalid address".to_string()));
                                            log::error!("Error parsing address: {}", e);
                                        }
                                    }
                                }
                            >
                                Send
                            </button>
                        }
                    }
                }}

            </div>
            <div class="flex justify-between my-4">
                <div>
                    <h3 class="text-lg font-semibold">Faucet Balance:</h3>
                    <p class="text-xl">{move || faucet_balance.get().unwrap_or_default().to_string()}</p>
                </div>
                <div>
                    <h3 class="text-lg font-semibold">Target Balance:</h3>
                    <p class="text-xl">{move || target_balance.get().unwrap_or_default().to_string()}</p>
                </div>
            </div>
            <hr class="my-4 border-t border-gray-300" />
            {move || {
                let messages = sent_messages.get();
                if !messages.is_empty() {
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
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}

#[component]
pub fn Faucets() -> impl IntoView {
    view! {
        <div>
            <h2 class="text-2xl font-bold mb-4">Calibration Network Faucet</h2>
            <Faucet target_network=Network::Testnet />

            <h2 class="text-2xl font-bold mb-4 mt-8">Mainnet Network Faucet</h2>
            <Faucet target_network=Network::Mainnet />
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    RpcContext::provide_context();

    view! {
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Router>
            <Routes>
                <Route path="/" view=BlockchainExplorer />
                <Route path="/faucet" view=Faucets />
            </Routes>
        </Router>
    }
}
