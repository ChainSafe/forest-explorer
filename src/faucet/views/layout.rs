use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="space-y-6 flex flex-col items-center">
            <h1 class="text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">
                Filecoin Forest Explorer Faucet
            </h1>
            <p class="max-w-2xl text-center">
                The Filecoin Forest Explorer Faucet provides developers and users with free calibnet(tFil) and mainnet(FIL) to support their exploration, testing and development on the Filecoin network.
            </p>
        </header>
    }
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
