use crate::faucet::views::components::icons::{CheckIcon, LightningIcon};
use crate::faucet::views::components::layout::Header;
use crate::faucet::views::components::nav::GotoFaucetList;
use crate::faucet::views::components::rpc_selector::RpcSelectors;
use crate::utils::rpc_context::RpcContext;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
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
pub fn Explorer() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    let provider = rpc_context.provider();
    let network_name = LocalResource::new(move || {
        provider.track();
        let provider = rpc_context.get();
        async move { provider.network_name().await.ok() }
    });

    let network_version = LocalResource::new(move || {
        provider.track();
        let provider = rpc_context.get();
        async move { provider.network_version().await.ok() }
    });

    view! {
        <main class="main-container">
            <Title text="Filecoin Forest Explorer Faucet" />
            <Header />
            <FaucetOverview />
            <RpcSelectors rpc_context=rpc_context network_name=network_name network_version=network_version />
            <GotoFaucetList />
        </main>
    }
}
