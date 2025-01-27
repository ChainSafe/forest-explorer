use chrono::{DateTime, Duration, Utc};
use worker::*;

#[durable_object]
pub struct RateLimiter {
    state: State,
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

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        let now = Utc::now();
        console_log!(
            "Rate limiter invoked: now={:?}, block_until={:?}, may_sign={:?}",
            now,
            self.block_until,
            self.block_until <= now
        );
        if self.block_until <= now {
            // This Durable Object will be deleted after the alarm is triggered
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    crate::constants::RATE_LIMIT_SECONDS as u64 + 1,
                ))
                .await?;
            self.block_until = now + Duration::seconds(crate::constants::RATE_LIMIT_SECONDS);

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
