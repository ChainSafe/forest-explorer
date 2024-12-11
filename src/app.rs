use leptos::{component, create_local_resource, event_target_value, view, IntoView, SignalGet};
use leptos_meta::*;
use leptos_router::*;

use crate::rpc_context::RpcContext;

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
                <Route path="/faucet" view=crate::faucet::views::Faucets />
                <Route path="/faucet/calibnet" view=crate::faucet::views::Faucet_Calibnet />
                <Route path="/faucet/mainnet" view=crate::faucet::views::Faucet_Mainnet />
            </Routes>
        </Router>
    }
}
