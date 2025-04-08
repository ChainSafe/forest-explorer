use crate::constants::{CALIBNET_RATE_LIMIT_SECONDS, MAINNET_RATE_LIMIT_SECONDS};
use chrono::{DateTime, Duration, Utc};
use worker::*;

#[durable_object]
pub struct RateLimiter {
    state: State,
    #[allow(unused)]
    block_until: DateTime<Utc>,
}

#[durable_object]
impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            block_until: Utc::now(),
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let now = Utc::now();
        let path = req.path();
        let (network, rate_limit_seconds) = if path.contains("mainnet") {
            ("mainnet", MAINNET_RATE_LIMIT_SECONDS)
        } else {
            ("calibnet", CALIBNET_RATE_LIMIT_SECONDS)
        };
        let storage_key = format!("block_until_{}", network);
        let block_until = self
            .state
            .storage()
            .get(&storage_key)
            .await
            .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
            .unwrap_or(Utc::now());
        console_log!(
            "Rate limiter invoked: now={:?}, block_until={:?}, may_sign={:?}",
            now,
            block_until,
            block_until <= now
        );
        if block_until <= now {
            // This Durable Object will be deleted after the alarm is triggered
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    rate_limit_seconds as u64 + 1,
                ))
                .await?;
            let block_until = now + Duration::seconds(rate_limit_seconds);
            self.state
                .storage()
                .put(&storage_key, block_until.timestamp())
                .await?;

            Response::from_json(&true)
        } else {
            Response::from_json(&false)
        }
    }

    async fn alarm(&mut self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. DurableObject will be deleted.");
        Response::ok("OK")
    }
}
