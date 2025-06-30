use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::faucet::views::components::faucet_description::FaucetDescription;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Calibnet_USDFC() -> impl IntoView {
    let faucet_info = FaucetInfo::CalibnetUSDFC;
    let rpc_context = RpcContext::use_context();
    rpc_context.set(Provider::get_network_url(
        FaucetInfo::CalibnetUSDFC.network(),
    ));

    view! {
        <Title text="💰 Filecoin USDFC Faucet - Calibration Network" />
        <Meta
            name="description"
            content="Filecoin USDFC Calibration Network Faucet dispensing USDFC tokens for testing purposes."
        />
        <h1 class="header">"💰 Filecoin Calibnet USDFC Faucet"</h1>
        <div class="main-container">
            <Faucet faucet_info=faucet_info />
            <FaucetDescription faucet_info=faucet_info />
            <div class="description">
                <p>
                    "You can also obtain testnet USDFC by minting it and using tFIL as collateral with the "
                    <a class="text-blue-600" rel="noopener noreferrer" href="https://stg.usdfc.net/#/" target="_blank">
                        "USDFC testnet application."
                    </a> " For more information, visit the "
                    <a
                        class="text-blue-600"
                        rel="noopener noreferrer"
                        href="https://docs.secured.finance/usdfc-stablecoin/getting-started/getting-test-usdfc-on-testnet"
                        target="_blank"
                    >
                        "USDFC documentation"
                    </a>.
                </p>
            </div>
        </div>
    }
}
