use crate::utils::format::format_balance;
use fvm_shared::econ::TokenAmount;
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn FaucetDescription(
    drip_amount: TokenAmount,
    wallet_cap: TokenAmount,
    token_unit: String,
    rate_limit_seconds: i64,
    wallet_cap_reset: i64,
) -> impl IntoView {
    view! {
        <div class="description">
            <p>
                "This faucet distributes " {format_balance(&drip_amount, &token_unit)}
                " per request and is rate-limited to one request per " {rate_limit_seconds}
                " seconds per address. Each wallet address is subject to receive "
                {format_balance(&wallet_cap, &token_unit)} " every " {wallet_cap_reset}
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
