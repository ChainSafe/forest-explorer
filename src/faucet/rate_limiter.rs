use std::str::FromStr as _;

use crate::faucet::constants::FaucetInfo;
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
        let faucet_info = FaucetInfo::from_str(path.split('/').last().unwrap_or_default())
            .map_err(|e| worker::Error::RustError(e.to_string()))?;

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
            let next_block = now + Duration::seconds(faucet_info.rate_limit_seconds());
            self.state
                .storage()
                .put("block_until", next_block.timestamp())
                .await?;
            console_log!("Rate limiter for {faucet_info} set: block_until={next_block:?}");
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    faucet_info.rate_limit_seconds() as u64 + 1,
                ))
                .await?;
        } else {
            console_log!(
                "Rate limiter for {faucet_info} invoked: now={now:?}, block_until={block_until:?}, may_sign={is_allowed:?}"
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
