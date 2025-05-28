use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn GotoFaucetList() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="btn">
                <a href="/faucet">Faucet List</a>
            </button>
        </div>
    }
}

#[component]
pub fn GotoHome() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="btn">
                <a href="/">Home</a>
            </button>
        </div>
    }
}
