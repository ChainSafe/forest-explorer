use crate::utils::format::format_balance;
use fvm_shared::econ::TokenAmount;
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn FaucetDescription(
    /// Amount of tokens to be dripped per request
    drip_amount: TokenAmount,
    /// Maximum amount of tokens a wallet can receive from the faucet
    wallet_cap: TokenAmount,
    /// Unit of the token (e.g., "FIL", "tFIL", "USDFC")
    token_unit: String,
    /// Time in seconds between allowed faucet requests
    rate_limit_seconds: i64,
    /// Time in seconds after which the wallet cap resets
    wallet_limit_seconds: i64,
) -> impl IntoView {
    view! {
        <div class="description">
            <p>
                "This faucet distributes " {format_balance(&drip_amount, &token_unit)}
                " per request and is rate-limited to 1 request per " {rate_limit_seconds}
                " seconds per address. Each wallet address is subject to receive "
                {format_balance(&wallet_cap, &token_unit)} " every "{wallet_limit_seconds / 3600}
                " hours, and exceeding this limit may result in temporary restrictions."
            </p>
            <p>
                "Farming, abuse, or automated requests are strictly prohibited and will lead to stricter rate limits, temporary suspensions, or permanent bans."
            </p>
            <p>
                "Faucet funds are limited and may run out. Refills are not guaranteed and occur periodically based on availability."
            </p>
        </div>
    }
}
