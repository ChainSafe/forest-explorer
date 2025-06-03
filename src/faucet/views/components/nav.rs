use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn GotoFaucetList() -> impl IntoView {
    view! {
        <div class="text-center">
            <button>
                <a class="btn" href="/faucet">
                    Faucet List
                </a>
            </button>
        </div>
    }
}

#[component]
pub fn GotoHome() -> impl IntoView {
    view! {
        <div class="text-center">
            <button>
                <a class="btn" href="/">
                    Home
                </a>
            </button>
        </div>
    }
}
