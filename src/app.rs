use crate::faucet::views::faucets::{
    Faucets, calibnet::Faucet_Calibnet, calibnet_usdfc::Faucet_Calibnet_USDFC,
    mainnet::Faucet_Mainnet,
};
use crate::faucet::views::{components::layout::Footer, home::Explorer};
use crate::utils::rpc_context::RpcContext;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    RpcContext::provide_context();

    view! {
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Router>
            <div class="app-container">
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Explorer />
                    <Route path=path!("/faucet") view=Faucets />
                    <Route path=path!("/faucet/calibnet") view=Faucet_Calibnet />
                    <Route path=path!("/faucet/mainnet") view=Faucet_Mainnet />
                    <Route path=path!("/faucet/calibnet_usdfc") view=Faucet_Calibnet_USDFC />
                </Routes>
                <Footer />
            </div>
        </Router>
    }
}
