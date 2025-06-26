use super::Faucet;
use crate::faucet::constants::FaucetInfo;
use crate::utils::format::format_balance;
use crate::utils::rpc_context::{Provider, RpcContext};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{Meta, Title};

#[component]
pub fn Faucet_Calibnet_USDFC() -> impl IntoView {
    let drip_amount = FaucetInfo::CalibnetUSDFC.drip_amount();
    let token_unit = FaucetInfo::CalibnetUSDFC.unit();
    let rate_limit_seconds = FaucetInfo::CalibnetUSDFC.rate_limit_seconds();
    let wallet_cap = FaucetInfo::CalibnetUSDFC.wallet_cap();
    let wallet_cap_reset = FaucetInfo::CalibnetUSDFC.wallet_cap_reset();
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
            <Faucet faucet_info=FaucetInfo::CalibnetUSDFC />
            <div class="description">
                <p>
                    "This faucet distributes " {format_balance(drip_amount, token_unit)}
                    " per request. It is rate-limited to 1 request per " {rate_limit_seconds}
                    " seconds. Each wallet address is subject to receive " {format_balance(&wallet_cap, token_unit)}
                    " every " {wallet_cap_reset}
                    " hours, and exceeding this limit may result in temporary restrictions."
                </p>
                <p>
                    "Farming is discouraged and will result in more stringent rate limiting in the future and/or permanent bans."
                </p>
            </div>
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
