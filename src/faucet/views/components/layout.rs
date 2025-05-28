use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="space-y-6 flex flex-col items-center">
            <h1 class="header-large">Filecoin Forest Explorer Faucet</h1>
            <p class="max-w-2xl text-center">
                The Filecoin Forest Explorer Faucet provides developers and users with free calibnet(tFil) and mainnet(FIL) to support their exploration, testing and development on the Filecoin network.
            </p>
        </header>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
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
