use crate::constants::{CALIBNET_RATE_LIMIT_SECONDS, MAINNET_RATE_LIMIT_SECONDS};
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
        let network = path.split('/').last().unwrap_or("mainnet");

        let rate_limit_seconds = if network == "mainnet" {
            MAINNET_RATE_LIMIT_SECONDS
        } else {
            CALIBNET_RATE_LIMIT_SECONDS
        };

        let block_until = self
            .state
            .storage()
            .get("block_until")
            .await
            .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
            .unwrap_or(now);

        let is_allowed = now >= block_until;

        console_log!(
            "[RateLimiter:{}] now = {:?}, block_until = {:?}, allowed = {}",
            network,
            now,
            block_until,
            is_allowed
        );

        if is_allowed {
            let next_block = now + Duration::seconds(rate_limit_seconds);

            self.state
                .storage()
                .put("block_until", next_block.timestamp())
                .await?;

            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    rate_limit_seconds as u64 + 1,
                ))
                .await?;
        }

        Response::from_json(&is_allowed)
    }

    async fn alarm(&mut self) -> Result<Response> {
        // Clear the block_until key when cooldown ends
        self.state.storage().delete("block_until").await.ok();
        console_log!("[RateLimiter] Alarm triggered. block_until cleared.");
        Response::ok("Rate limiter reset.")
    }
}
