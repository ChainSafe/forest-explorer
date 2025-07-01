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

impl RateLimiter {
    fn parse_request_path(path: &str) -> Result<(FaucetInfo, String)> {
        // Request path format: http://do/rate_limiter/{faucet_info}/{id}
        let mut path_info = path.split('/');
        let id = path_info.next_back().unwrap_or_default().to_string();
        let faucet_info = FaucetInfo::from_str(path_info.next_back().unwrap_or_default())
            .map_err(|e| Error::RustError(e.to_string()))?;
        Ok((faucet_info, id))
    }

    async fn get_rate_limit(
        &self,
        faucet_info: &FaucetInfo,
        id: &str,
        now: DateTime<Utc>,
    ) -> Result<(bool, Option<i64>, TokenAmount)> {
        let claimed_key = format!("claimed_{id}");
        let claimed: TokenAmount = self
            .state
            .storage()
            .get(&claimed_key)
            .await
            .unwrap_or(TokenAmount::default());

        if claimed >= faucet_info.wallet_cap() {
            let wallet_block_until = self.state.storage().get_alarm().await?.unwrap_or_default();
            let retry_after =
                Duration::milliseconds(wallet_block_until - now.timestamp_millis()).num_seconds();
            console_log!("{faucet_info} Rate limiter for {id} invoked: Wallet capped now={now:?}, claimed={claimed:?}, retry_after={retry_after:?}");
            return Ok((false, Some(retry_after), claimed));
        }

        let block_until_key = format!("block_until_{id}");
        let block_until = self
            .state
            .storage()
            .get(&block_until_key)
            .await
            .map(|v| DateTime::<Utc>::from_timestamp(v, 0).unwrap_or_default())
            .unwrap_or(now);

        if block_until > now {
            let retry_after = block_until.signed_duration_since(now).num_seconds();
            console_log!(
                "{faucet_info} Rate limiter for {id} invoked: now={now:?}, claimed={claimed:?}, retry_after={retry_after:?}"
            );
            return Ok((false, Some(retry_after), claimed));
        }

        Ok((true, None, claimed))
    }

    async fn update_rate_limit(
        &self,
        faucet_info: &FaucetInfo,
        id: &str,
        now: DateTime<Utc>,
        claimed: TokenAmount,
    ) -> Result<()> {
        let updated_claimed = claimed + faucet_info.drip_amount();
        let next_block = now + Duration::seconds(faucet_info.rate_limit_seconds());

        self.state
            .storage()
            .put(&format!("claimed_{id}"), updated_claimed.clone())
            .await?;
        self.state
            .storage()
            .put(&format!("block_until_{id}"), next_block.timestamp())
            .await?;

        if self.state.storage().get_alarm().await?.is_none() {
            self.state
                .storage()
                .set_alarm(std::time::Duration::from_secs(
                    faucet_info.wallet_limit_seconds() as u64,
                ))
                .await?;
        }
        console_log!("{faucet_info} Rate limiter for {id} set: now={now:?}, block_until={next_block:?}, claimed={updated_claimed:?}");
        Ok(())
    }
}

impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let now = Utc::now();
        let (faucet_info, id) = Self::parse_request_path(&req.path())?;
        let (is_allowed, retry_after, claimed) =
            self.get_rate_limit(&faucet_info, &id, now).await?;

        if is_allowed {
            self.update_rate_limit(&faucet_info, &id, now, claimed)
                .await?;
        }
        return Response::from_json(&retry_after);
    }

    async fn alarm(&self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. DurableObject will be deleted.");
        self.state.storage().delete_all().await.ok();
        Response::ok("OK")
    }
}
