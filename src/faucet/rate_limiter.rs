#![cfg(feature = "ssr")]
use std::str::FromStr as _;

use crate::faucet::constants::{FaucetInfo, WALLET_CAP_RESET_IN_SECONDS};
use crate::utils::lotus_json::LotusJson;
use chrono::{DateTime, Duration, Utc};
use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};
use worker::*;

/// Response from the rate limiter indicating whether a request can proceed
/// and providing context about rate limiting status.
#[derive(Serialize, Deserialize)]
pub struct RateLimiterResponse {
    /// Unix timestamp (in seconds) until which requests are blocked.
    /// A value of 0 indicates no blocking is in effect.
    pub block_until: i64,
    /// The total amount of tokens claimed by the requester so far.
    /// Used to track usage against wallet cap.
    pub claimed: LotusJson<TokenAmount>,
    /// Indicates whether the request is allowed to proceed with signing.
    /// If false, the request should be rejected due to rate limiting.
    pub may_sign: bool,
}

#[durable_object]
pub struct RateLimiter {
    state: State,
}

impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let now = Utc::now();
        let path = req.path();
        let mut path_info = path.split('/');
        let id = path_info.next_back().unwrap_or_default();
        let faucet_info = FaucetInfo::from_str(path_info.next_back().unwrap_or_default())
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        let block_until_key = format!("block_until_{}", id);
        let claimed_key = format!("claimed_{}", id);
        let block_until = self
            .state
            .storage()
            .get(&block_until_key)
            .await
            .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
            .unwrap_or(now);
        let claimed = self
            .state
            .storage()
            .get(&claimed_key)
            .await
            .unwrap_or(TokenAmount::default());

        let is_allowed = block_until <= now && claimed < faucet_info.wallet_cap();

        if is_allowed {
            let claimed = claimed.clone() + faucet_info.drip_amount();
            let next_block = now + Duration::seconds(faucet_info.rate_limit_seconds());
            self.state
                .storage()
                .put(&block_until_key, next_block.timestamp())
                .await?;
            self.state
                .storage()
                .put(&claimed_key, claimed.clone())
                .await?;
            console_log!("Rate limiter for {faucet_info} set: block_until={next_block:?} claimed={claimed:?}");
        } else {
            console_log!(
                "Rate limiter for {faucet_info} invoked: now={now:?}, block_until={block_until:?}, claimed={claimed:?}, may_sign={is_allowed:?}"
            );
        }
        if self.state.storage().get_alarm().await?.is_none() {
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    WALLET_CAP_RESET_IN_SECONDS as u64,
                ))
                .await?;
        }
        let response = RateLimiterResponse {
            block_until: block_until.timestamp(),
            claimed: LotusJson(claimed),
            may_sign: is_allowed,
        };
        Response::from_json(&response)
    }

    async fn alarm(&self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. DurableObject will be deleted.");
        self.state.storage().delete_all().await.ok();

        Response::ok("OK")
    }
}
