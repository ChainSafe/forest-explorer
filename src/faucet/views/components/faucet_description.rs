use crate::faucet::constants::FaucetInfo;
use crate::utils::format::format_balance;
use chrono::Duration;
use leptos::prelude::*;
use leptos::{IntoView, component, view};

/// This component renders a user-friendly description of the faucet's token distribution rules and limitations.
#[component]
pub fn FaucetDescription(faucet_info: FaucetInfo) -> impl IntoView {
    let drip_amount = faucet_info.drip_amount();
    let token_unit = faucet_info.unit();
    let wallet_cap = faucet_info.wallet_cap();
    let drip_cap = faucet_info.drip_cap();
    let rate_limit_seconds = faucet_info.rate_limit_seconds();
    let reset_limiter_hours = Duration::seconds(faucet_info.reset_limiter_seconds()).num_hours();
    view! {
        <div class="description">
            <p>
                "This faucet distributes " {format_balance(drip_amount, token_unit)}
                " per request and is rate-limited to 1 request per " {rate_limit_seconds}
                " seconds. Each wallet address is subject to receive " {format_balance(&wallet_cap, token_unit)}
                " every " {reset_limiter_hours} " hours, and exceeding this limit may result in temporary restrictions."
            </p>
            <p>
                "Farming, abuse, or automated requests are strictly prohibited and will lead to stricter rate limits, temporary suspensions, or permanent bans."
            </p>
            <p>
                "Faucet funds are limited, so there is a maximum distribution cap of "
                {format_balance(&drip_cap, token_unit)} " for all users combined every " {reset_limiter_hours}
                " hours. Refills are not guaranteed and occur periodically based on availability."
            </p>
        </div>
    }
}
