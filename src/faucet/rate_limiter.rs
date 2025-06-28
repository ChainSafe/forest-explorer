#![cfg(feature = "ssr")]
use std::str::FromStr as _;

use crate::faucet::constants::FaucetInfo;
use chrono::{DateTime, Duration, Utc};
use fvm_shared::econ::TokenAmount;
use worker::*;

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
        let mut retry_after = None;
        let mut path_info = path.split('/');
        let id = path_info.next_back().unwrap_or_default();
        let faucet_info = FaucetInfo::from_str(path_info.next_back().unwrap_or_default())
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        let claimed_key = format!("claimed_{}", id);
        let claimed = self
            .state
            .storage()
            .get(&claimed_key)
            .await
            .unwrap_or(TokenAmount::default());
        let is_wallet_capped = claimed >= faucet_info.wallet_cap();
        if !is_wallet_capped {
            let block_until_key = format!("block_until_{}", id);
            let block_until = self
                .state
                .storage()
                .get(&block_until_key)
                .await
                .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
                .unwrap_or(now);
            let is_allowed = block_until <= now;

            if is_allowed {
                let next_block = now + Duration::seconds(faucet_info.rate_limit_seconds());
                let updated_claimed = claimed.clone() + faucet_info.drip_amount();
                self.state
                    .storage()
                    .put(&block_until_key, next_block.timestamp())
                    .await?;
                self.state
                    .storage()
                    .put(&claimed_key, updated_claimed.clone())
                    .await?;
                if self.state.storage().get_alarm().await?.is_none() {
                    // This Durable Object will be deleted after the alarm is triggered
                    self.state
                        .storage()
                        .set_alarm(std::time::Duration::from_secs(
                            faucet_info.wallet_limit_seconds() as u64,
                        ))
                        .await?;
                }
                console_log!("{faucet_info} Rate limiter for {id} set: now={now:?}, block_until={next_block:?}, claimed={updated_claimed:?}");
            } else {
                console_log!(
                "{faucet_info} Rate limiter for {id} invoked: now={now:?}, block_until={block_until:?}, claimed={claimed:?}, may_sign={is_allowed:?}"
            );
                retry_after = Some(block_until.signed_duration_since(&now).num_seconds());
            }
            return Response::from_json(&Some(retry_after));
        }
        console_log!("{faucet_info} Rate limiter for {id} invoked: Wallet capped now={now:?}, claimed={claimed:?}");
        let wallet_block_until = self.state.storage().get_alarm().await?.unwrap_or_default();
        retry_after =
            Some(Duration::milliseconds(wallet_block_until - now.timestamp_millis()).num_seconds());
        Response::from_json(&retry_after)
    }

    async fn alarm(&self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. DurableObject will be deleted.");
        self.state.storage().delete_all().await.ok();
        Response::ok("OK")
    }
}
