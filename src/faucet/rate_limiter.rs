// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::faucet::calibnet::CALIBNET_RATE_LIMIT_SECONDS;
use crate::faucet::mainnet::MAINNET_RATE_LIMIT_SECONDS;
use chrono::{DateTime, Duration, Utc};
use worker::*;

#[durable_object]
pub struct RateLimiter {
    state: State,
}

#[durable_object]
impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let now = Utc::now();
        let path = req.path();
        let (network, rate_limit_seconds) = if path.contains("mainnet") {
            ("mainnet", MAINNET_RATE_LIMIT_SECONDS)
        } else {
            ("calibnet", CALIBNET_RATE_LIMIT_SECONDS)
        };

        let block_until = self
            .state
            .storage()
            .get("block_until")
            .await
            .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
            .unwrap_or(now);

        let is_allowed = block_until <= now;

        if is_allowed {
            // This Durable Object will be deleted after the alarm is triggered
            let next_block = now + Duration::seconds(rate_limit_seconds);
            self.state
                .storage()
                .put("block_until", next_block.timestamp())
                .await?;
            console_log!(
                "Rate limiter for {} set: block_until={:?}",
                network,
                next_block
            );
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    rate_limit_seconds as u64 + 1,
                ))
                .await?;
        } else {
            console_log!(
                "Rate limiter for {} invoked: now={:?}, block_until={:?}, may_sign={:?}",
                network,
                now,
                block_until,
                is_allowed
            );
        }
        Response::from_json(&is_allowed)
    }

    async fn alarm(&mut self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. DurableObject will be deleted.");
        self.state.storage().delete_all().await.ok();
        Response::ok("OK")
    }
}
