use crate::faucet::views::icons::{CheckIcon, LightningIcon, Loader};
use crate::faucet::views::layout::Header;
use crate::faucet::views::nav::GotoFaucetList;
use crate::utils::rpc_context::{Provider, RpcContext};
use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{component, leptos_dom::helpers::event_target_value, view, IntoView};

#[component]
fn FaucetOverview() -> impl IntoView {
    view! {
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
                        <span class="text-gray-600">Supports both calibnet and mainnet, unlike typical faucets.</span>
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
    }
}

#[component]
fn NetworkSelection(
    rpc_context: RpcContext,
    network_name: LocalResource<Option<String>>,
    network_version: LocalResource<Option<u64>>,
) -> impl IntoView {
    view! {
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
        <main class="min-h-screen flex flex-col flex-grow space-y-8">
            <Header />
            <FaucetOverview />
            <NetworkSelection rpc_context=rpc_context network_name=network_name network_version=network_version />
            <GotoFaucetList />
        </main>
    }
}
