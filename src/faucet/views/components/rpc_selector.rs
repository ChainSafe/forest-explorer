use crate::faucet::views::components::icons::Loader;
use crate::utils::rpc_context::{RpcContext, providers_for};
use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{IntoView, component, leptos_dom::helpers::event_target_value, view};

fn parse_network(value: &str) -> Network {
    match value {
        "calibnet" => Network::Testnet,
        "mainnet" => Network::Mainnet,
        _ => panic!("predefined network values, must succeed"),
    }
}

#[component]
pub fn NetworkSelection(rpc_context: RpcContext) -> impl IntoView {
    view! {
        <div class="dropdown">
            <label for="network-select" class="sr-only">
                Select Network
            </label>
            <select
                id="network-select"
                on:change=move |ev| { rpc_context.set_network(parse_network(&event_target_value(&ev))) }
                class="dropdown-items"
            >
                <option value="calibnet" selected=move || rpc_context.network().get() == Network::Testnet>
                    Calibnet
                </option>
                <option value="mainnet" selected=move || rpc_context.network().get() == Network::Mainnet>
                    Mainnet
                </option>
            </select>
            <div class="dropdown-icon">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn ProviderSelection(rpc_context: RpcContext) -> impl IntoView {
    view! {
        <div class="dropdown">
            <label for="provider-select" class="sr-only">
                Select RPC Provider
            </label>
            <select
                id="provider-select"
                on:change=move |ev| {
                    rpc_context
                        .set_provider_url(
                            event_target_value(&ev).parse().expect("predefined provider URLs, must succeed"),
                        )
                }
                class="dropdown-items"
            >
                {move || {
                    let network = rpc_context.network().get();
                    let current_url = rpc_context.get().url.to_string();
                    providers_for(network)
                        .iter()
                        .map(|endpoint| {
                            let url = endpoint.url.to_string();
                            let selected = url == current_url;
                            view! {
                                <option value=url selected=selected>
                                    {endpoint.label}
                                </option>
                            }
                        })
                        .collect_view()
                }}
            </select>
            <div class="dropdown-icon">
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn RpcStatus(
    rpc_context: RpcContext,
    network_name: LocalResource<Option<String>>,
    network_version: LocalResource<Option<u64>>,
) -> impl IntoView {
    let _ = rpc_context;
    view! {
        <div class="network-info">
            <p>Network:</p>
            <Transition fallback=move || view! { <p>Loading network name...</p> }>
                <p>
                    <span>{move || network_name.get().flatten()}</span>
                    <Loader loading=move || network_name.get().is_none() />
                </p>
            </Transition>
        </div>

        <div class="network-info">
            <p>Version:</p>
            <Transition fallback=move || view! { <p>Loading network version...</p> }>
                <p>
                    <span>{move || network_version.get().flatten()}</span>
                    <Loader loading=move || network_version.get().is_none() />
                </p>
            </Transition>
        </div>
    }
}

#[component]
pub fn RpcSelectors(
    rpc_context: RpcContext,
    network_name: LocalResource<Option<String>>,
    network_version: LocalResource<Option<u64>>,
) -> impl IntoView {
    view! {
        <div class="selector-group">
            <NetworkSelection rpc_context=rpc_context />
            <RpcStatus rpc_context=rpc_context network_name=network_name network_version=network_version />
        </div>
    }
}
