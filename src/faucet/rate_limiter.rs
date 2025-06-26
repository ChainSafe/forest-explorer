#![cfg(feature = "ssr")]
use std::str::FromStr as _;

use crate::faucet::constants::FaucetInfo;
use chrono::{DateTime, Duration, Utc};
use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize)]
pub struct RateLimiterResponse {
    pub block_until: i64,
    pub wallet_cap_reached: bool,
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
        let wallet_key = format!("wallet_{}", id);
        let id_tracker_key = "id_tracker";
        let mut id_tracker: Vec<String> = self
            .state
            .storage()
            .get(id_tracker_key)
            .await
            .unwrap_or_default();
        if !id_tracker.contains(&id.to_string()) {
            id_tracker.push(id.to_string());
        }
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
            .get(&wallet_key)
            .await
            .unwrap_or(TokenAmount::default());

        let is_allowed = block_until <= now && claimed < faucet_info.wallet_cap();

        if is_allowed {
            // This Durable Object will be deleted after the alarm is triggered
            let claimed = claimed.clone() + faucet_info.drip_amount();
            let next_block = if claimed == faucet_info.wallet_cap() {
                block_until + Duration::hours(faucet_info.wallet_cap_reset())
            } else {
                now + Duration::seconds(faucet_info.rate_limit_seconds())
            };
            self.state
                .storage()
                .put(&block_until_key, next_block.timestamp())
                .await?;
            self.state.storage().put(id_tracker_key, id_tracker).await?;
            self.state
                .storage()
                .put(&wallet_key, claimed.clone())
                .await?;
            console_log!("Rate limiter for {faucet_info} set: block_until={next_block:?} claimed={claimed:?}");
        } else {
            console_log!(
                "Rate limiter for {faucet_info} invoked: now={now:?}, block_until={block_until:?}, claimed={claimed:?}, may_sign={is_allowed:?}"
            );
        }
        self.state
            .storage()
            .set_alarm(std::time::Duration::from_secs(
                faucet_info.rate_limit_seconds() as u64 + 1,
            ))
            .await?;
        let response = RateLimiterResponse {
            block_until: block_until.timestamp(),
            wallet_cap_reached: claimed >= faucet_info.wallet_cap(),
            may_sign: is_allowed,
        };
        Response::from_json(&response)
    }

    async fn alarm(&self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. Cleaning up expired keys...");
        let mut ids_to_remove = Vec::new();
        let storage = self.state.storage();
        let now = Utc::now();
        let id_tracker_key = "id_tracker";
        let mut id_tracker: Vec<String> = self
            .state
            .storage()
            .get(id_tracker_key)
            .await
            .unwrap_or_default();

        for id in id_tracker.iter() {
            let block_until_key = format!("block_until_{}", id);
            let wallet_key = format!("wallet_{}", id);
            let block_until = self
                .state
                .storage()
                .get(&block_until_key)
                .await
                .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
                .unwrap_or(now);
            if block_until <= now {
                console_log!("Deleting expired {block_until_key}");
                storage.delete(&block_until_key).await.ok();
                if now - block_until >= Duration::days(1) {
                    console_log!("Deleting expired {wallet_key}");
                    storage.delete(&wallet_key).await.ok();
                    ids_to_remove.push(id.to_string());
                }
            }
        }
        id_tracker.retain(|x| !ids_to_remove.contains(x));
        self.state
            .storage()
            .put(id_tracker_key, id_tracker)
            .await
            .ok();

        Response::ok("OK")
    }
}
