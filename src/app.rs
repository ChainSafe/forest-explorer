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
        <html lang="en">
            <head>
                <title>Forest Filecoin Explorer</title>
                <meta charset="utf-8"/>
                <meta name="robots" content="index, follow" />
                <meta name="viewport" content="width=device-width, initial-scale=1"/>

                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
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
        <div class="flex flex-col items-center">
        <h1 class="mb-4 text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">
            Forest Explorer
        </h1>
        <select on:change=move |ev| { rpc_context.set(event_target_value(&ev)) }>
            <option value=Provider::get_network_url(Network::Testnet)>Glif.io Calibnet</option>
            <option value=Provider::get_network_url(Network::Mainnet)>Glif.io Mainnet</option>
        </select>
        <p>StateNetworkName</p>
        <Transition fallback={move || view!{ <p>Loading network name...</p> }}>
            <p class="px-8">
                <span>{move || network_name.get().as_deref().flatten().cloned()}</span>
                <Loader loading={move || network_name.get().is_none()} />
            </p>
        </Transition>

        <p>StateNetworkVersion</p>
        <Transition fallback={move || view!{ <p>Loading network version...</p> }}>
            <p class="px-8">
                <span>{move || network_version.get().as_deref().flatten().cloned()}</span>
                <Loader loading={move || network_version.get().is_none()} />
            </p>
        </Transition>
        <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded-full">
          <a href="/faucet">To faucet list</a>
        </button>
        </div>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="p-4 text-center">
            <a class="text-green-600" target="_blank" rel="noopener noreferrer" href="https://github.com/ChainSafe/forest-explorer">Forest Explorer</a>", built with ❤️ by " <a class="text-blue-600" target="_blank" rel="noopener noreferrer" href="https://chainsafe.io">ChainSafe Systems</a>
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
            <Routes fallback=|| "Not found.">
                <Route path=path!("/") view=BlockchainExplorer />
                <Route path=path!("/faucet") view=crate::faucet::views::Faucets />
                <Route path=path!("/faucet/calibnet") view=crate::faucet::views::Faucet_Calibnet />
                <Route path=path!("/faucet/mainnet") view=crate::faucet::views::Faucet_Mainnet />
            </Routes>
            <Footer />
        </Router>
    }
}
