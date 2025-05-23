use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn GotoFaucetList() -> impl IntoView {
    view! {
        <div class="text-center">
            <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded-lg">
                <a href="/faucet">View Faucet List</a>
            </button>
        </div>
    }
}
