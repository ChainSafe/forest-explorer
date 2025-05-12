use crate::rpc_context::{Provider, RpcContext};
use fvm_shared::address::Network;
use leptos::prelude::*;
use leptos::{component, leptos_dom::helpers::event_target_value, view, IntoView};
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

#[allow(dead_code)]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="py-10 px-6 min-h-screen">
            <head>
                <title>Filecoin Forest Explorer Faucet - Get Free tFIL and FIL</title>
                <meta charset="utf-8" />
                <meta name="robots" content="index, follow" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta
                    name="description"
                    content="Get free tFIL and FIL on the Filecoin Forest Explorer Faucet by ChainSafe. Quickly connect your wallet, request tokens, and start building or experimenting on the Filecoin testnet or mainnet with ease."
                />

                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
        </html>
    }
}

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static + Send) -> impl IntoView {
    view! { <span class:loader=loading /> }
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
                    Forest Explorer
                </h1>
                <p class="max-w-2xl">The Filecoin Forest Explorer Faucet provides developers and users with free calibnet(tFil) and mainnet(FIL) to support their exploration, testing and development on the Filecoin network.
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl w-full m-auto">
                <div class="bg-white p-6 rounded-lg border border-gray-100">
                    <h2 class="text-lg font-semibold text-gray-900 mb-4">What does the faucet offer?</h2>
                    <ul class="space-y-3">
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-green-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                            </svg>
                            <span class="text-gray-600">Free testnet FIL for experimentation and development.</span>
                        </li>
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-green-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                            </svg>
                            <span class="text-gray-600">Real mainnet FIL for contributors engaging with the Filecoin ecosystem.</span>
                        </li>
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-green-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                            </svg>
                            <span class="text-gray-600">A simple and fast way to request FIL by entering your wallet address.</span>
                        </li>
                    </ul>
                </div>

                <div class="bg-white p-6 rounded-lg border border-gray-100">
                    <h2 class="text-lg font-semibold text-gray-900 mb-4">Why use this faucet?</h2>
                    <ul class="space-y-3">
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-blue-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                            </svg>
                            <span class="text-gray-600">Supports both testnet and mainnet, unlike typical faucets.</span>
                        </li>
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-blue-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                            </svg>
                            <span class="text-gray-600">Enables testing of smart contracts, storage deals, and blockchain interactions without financial risk.</span>
                        </li>
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-blue-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                            </svg>
                            <span class="text-gray-600">Easy access to FIL for developers and users building on Filecoin.</span>
                        </li>
                        <li class="flex items-start">
                            <svg class="h-5 w-5 text-blue-500 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                            </svg>
                            <span class="text-gray-600">Need help? Check the documentation or connect with the community for support.</span>
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
                    <p>StateNetworkName</p>
                    <Transition fallback=move || view! { <p>Loading network name...</p> }>
                        <p>
                            <span>{move || network_name.get().flatten()}</span>
                            <Loader loading=move || network_name.get().is_none() />
                        </p>
                    </Transition>
                </div>
                <div class="flex items-center w-[300px] justify-between">
                    <p>StateNetworkVersion</p>
                    <Transition fallback=move || view! { <p>Loading network version...</p> }>
                        <p>
                        <span>{move || network_version.get().flatten()}</span>
                        <Loader loading=move || network_version.get().is_none() />
                        </p>
                    </Transition>
                </div>
                <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded-full">
                <a href="/faucet">To faucet list</a>
                </button>
            </div>
           
        </main>
    }
}

#[component]
fn Footer() -> impl IntoView {
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
pub fn App() -> impl IntoView {
    provide_meta_context();
    RpcContext::provide_context();

    view! {
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Router>
            <div class="flex flex-col min-h-screen space-y-8">
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=BlockchainExplorer />
                    <Route path=path!("/faucet") view=crate::faucet::views::Faucets />
                    <Route path=path!("/faucet/calibnet") view=crate::faucet::views::Faucet_Calibnet />
                    <Route path=path!("/faucet/mainnet") view=crate::faucet::views::Faucet_Mainnet />
                </Routes>
                <Footer />
            </div>
        </Router>
    }
}
