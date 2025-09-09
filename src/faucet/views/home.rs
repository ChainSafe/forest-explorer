use crate::faucet::views::components::icons::{CheckIcon, LightningIcon, Loader};
use crate::faucet::views::components::layout::Header;
use crate::faucet::views::components::nav::GotoFaucetList;
use crate::utils::rpc_context::{Provider, RpcContext};
use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{IntoView, component, leptos_dom::helpers::event_target_value, view};
use leptos_meta::Title;

#[component]
fn FaucetOverview() -> impl IntoView {
    view! {
        <div class="overview-container">
            <div class="card">
                <h2 class="card-title">What does the faucet offer?</h2>
                <ul class="list">
                    <li class="list-element">
                        <CheckIcon />
                        <span class="list-text">Free calibnet tFIL for experimentation and development.</span>
                    </li>
                    <li class="list-element">
                        <CheckIcon />
                        <span class="list-text">
                            Real mainnet FIL for contributors engaging with the Filecoin ecosystem.
                        </span>
                    </li>
                    <li class="list-element">
                        <CheckIcon />
                        <span class="list-text">
                            A Quick and Easy way to request free tFIL and FIL - Just enter your wallet address.
                        </span>
                    </li>
                </ul>
            </div>

            <div class="card">
                <h2 class="card-title">Why use this faucet?</h2>
                <ul class="list">
                    <li class="list-element">
                        <LightningIcon />
                        <span class="list-text">Supports both calibnet and mainnet, unlike typical faucets.</span>
                    </li>
                    <li class="list-element">
                        <LightningIcon />
                        <span class="list-text">
                            Enables testing of smart contracts, storage deals, and blockchain interactions without financial risk.
                        </span>
                    </li>
                    <li class="list-element">
                        <LightningIcon />
                        <span class="list-text">Easy access to FIL for developers and users building on Filecoin.</span>
                    </li>
                    <li class="list-element">
                        <LightningIcon />
                        <span class="list-text">
                            Need help? Visit the <a class="link-text" href="https://filecoin.io/slack" target="_blank">
                                {" "}
                                Filecoin Slack
                            </a>{" "}or <a class="link-text" href="https://docs.filecoin.io" target="_blank">
                                {" "}
                                documentation
                            </a>.
                        </span>
                    </li>
                </ul>
            </div>
        </div>
    }
}

#[component]
fn NetworkSelection(
    rpc_context: RpcContext,
    network_name: LocalResource<Option<String>>,
    network_version: LocalResource<Option<u64>>,
) -> impl IntoView {
    view! {
        <div class="network-selector">
            <div class="dropdown">
                <label for="network-select" class="sr-only">
                    Select Network
                </label>
                <select
                    id="network-select"
                    on:change=move |ev| {
                        rpc_context.set(event_target_value(&ev).parse().expect("predefined values, must succeed"))
                    }
                    class="dropdown-items"
                >
                    <option value=Provider::get_network_url(Network::Testnet).to_string()>Glif.io Calibnet</option>
                    <option value=Provider::get_network_url(Network::Mainnet).to_string()>Glif.io Mainnet</option>
                </select>
                <div class="dropdown-icon">
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </div>
            </div>

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
        </div>
    }
}

#[component]
pub fn Explorer() -> impl IntoView {
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
        <main class="main-container">
            <Title text="Filecoin Forest Explorer Faucet" />
            <Header />
            <FaucetOverview />
            <NetworkSelection rpc_context=rpc_context network_name=network_name network_version=network_version />
            <GotoFaucetList />
        </main>
    }
}
