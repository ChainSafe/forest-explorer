use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn GotoFaucetList() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded-lg">
                <a href="/faucet">Faucet List</a>
            </button>
        </div>
    }
}

#[component]
pub fn GotoHome() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded-lg">
                <a href="/">Home</a>
            </button>
        </div>
    }
}
