#![cfg(feature = "ssr")]
use std::str::FromStr as _;

use crate::faucet::constants::FaucetInfo;
use crate::utils::lotus_json::LotusJson;
use chrono::{DateTime, Duration, Utc};
use fvm_shared::econ::TokenAmount;
use worker::*;

#[cfg(test)]
use mockall::automock;

/// Abstraction for storage backends used by the rate limiter.
/// This trait allows the rate limiter logic to be decoupled from the underlying storage implementation.
/// Implementations may use [`DurableObjectStorage`], in-memory mocks, or other storage systems.
#[cfg_attr(test, automock)]
#[async_trait::async_trait(?Send)]
trait RateLimiterStorage {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + 'static;
    async fn put<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + 'static;
    async fn get_alarm(&self) -> Result<Option<i64>>;
    async fn set_alarm(&self, duration: std::time::Duration) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

/// Storage backend for the rate limiter using Durable Objects.
/// This struct implements the [`RateLimiterStorage`] trait and is used in production to persist rate limiting state.
#[cfg(not(test))]
struct DurableObjectStorage<'a> {
    state: &'a State,
}

#[cfg(not(test))]
impl<'a> DurableObjectStorage<'a> {
    fn new(state: &'a State) -> Self {
        Self { state }
    }
}

#[cfg(not(test))]
#[async_trait::async_trait(?Send)]
impl RateLimiterStorage for DurableObjectStorage<'_> {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.state.storage().get(key).await
    }
    async fn put<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.state.storage().put(key, value).await
    }
    async fn get_alarm(&self) -> Result<Option<i64>> {
        self.state.storage().get_alarm().await
    }
    async fn set_alarm(&self, duration: std::time::Duration) -> Result<()> {
        self.state.storage().set_alarm(duration).await
    }
    async fn delete_all(&self) -> Result<()> {
        self.state.storage().delete_all().await
    }
}

/// Core logic for rate limiting, generic over a storage backend.
/// This struct encapsulates all rate limiting logic and can be used with any storage backend that implements [`RateLimiterStorage`].
/// It is used by the [`RateLimiter`] durable object handler in production and by mocks in tests.
struct RateLimiterCore<S: RateLimiterStorage> {
    storage: S,
}

impl<S: RateLimiterStorage> RateLimiterCore<S> {
    fn new(storage: S) -> Self {
        Self { storage }
    }

    fn parse_request_path(path: &str) -> Result<(FaucetInfo, String)> {
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
    ) -> Result<(bool, Option<i64>, TokenAmount, TokenAmount)> {
        let dripped = self
            .storage
            .get::<LotusJson<TokenAmount>>("dripped")
            .await
            .ok()
            .flatten()
            .map(|v| v.into_inner())
            .unwrap_or_default();
        let claimed = self
            .storage
            .get::<LotusJson<TokenAmount>>(&format!("claimed_{id}"))
            .await
            .ok()
            .flatten()
            .map(|v| v.into_inner())
            .unwrap_or_default();
        if dripped >= faucet_info.drip_cap() {
            let retry_after = self
                .storage
                .get_alarm()
                .await
                .ok()
                .flatten()
                .map(|alarm| Duration::milliseconds(alarm - now.timestamp_millis()).num_seconds())
                .unwrap_or(0);
            log::info!("{faucet_info} Rate limiter for {id} invoked: Drip capped now={now:?}, dripped={dripped:?}, retry_after={retry_after:?}");
            return Ok((false, Some(retry_after), claimed, dripped));
        }
        if claimed >= faucet_info.wallet_cap() {
            let retry_after = self
                .storage
                .get_alarm()
                .await
                .ok()
                .flatten()
                .map(|alarm| Duration::milliseconds(alarm - now.timestamp_millis()).num_seconds())
                .unwrap_or(0);
            log::info!("{faucet_info} Rate limiter for {id} invoked: Wallet capped now={now:?}, claimed={claimed:?}, retry_after={retry_after:?}");
            return Ok((false, Some(retry_after), claimed, dripped));
        }
        let block_until = self
            .storage
            .get::<i64>("block_until")
            .await
            .ok()
            .flatten()
            .and_then(|secs| DateTime::<Utc>::from_timestamp(secs, 0))
            .unwrap_or(now);
        if block_until > now {
            let retry_after = block_until.signed_duration_since(now).num_seconds();
            log::info!(
                "{faucet_info} Rate limiter for {id} invoked: now={now:?}, claimed={claimed:?}, retry_after={retry_after:?}"
            );
            return Ok((false, Some(retry_after), claimed, dripped));
        }
        Ok((true, None, claimed, dripped))
    }

    async fn update_rate_limit(
        &self,
        faucet_info: &FaucetInfo,
        id: &str,
        now: DateTime<Utc>,
        claimed: TokenAmount,
        dripped: TokenAmount,
    ) -> Result<()> {
        let update_dripped = dripped + faucet_info.drip_amount();
        let updated_claimed = claimed + faucet_info.drip_amount();
        let next_block = now + Duration::seconds(faucet_info.rate_limit_seconds());
        self.storage
            .put("dripped", LotusJson(update_dripped.clone()))
            .await?;
        self.storage
            .put(&format!("claimed_{id}"), LotusJson(updated_claimed.clone()))
            .await?;
        self.storage
            .put("block_until", next_block.timestamp())
            .await?;
        if self.storage.get_alarm().await?.is_none() {
            self.storage
                .set_alarm(std::time::Duration::from_secs(
                    faucet_info.reset_limiter_seconds() as u64,
                ))
                .await?;
        }
        log::info!("{faucet_info} Rate limiter for {id} set: now={now:?}, block_until={next_block:?}, claimed={updated_claimed:?}, dripped={update_dripped:?}");
        Ok(())
    }

    #[allow(dead_code)]
    async fn handle_request(&self, path: &str, now: DateTime<Utc>) -> Result<Option<i64>> {
        let (faucet_info, id) = Self::parse_request_path(path)?;
        let (is_allowed, retry_after, claimed, dripped) =
            self.get_rate_limit(&faucet_info, &id, now).await?;
        if is_allowed {
            self.update_rate_limit(&faucet_info, &id, now, claimed, dripped)
                .await?;
        }
        Ok(retry_after)
    }

    #[allow(dead_code)]
    async fn handle_alarm(&self) -> Result<()> {
        log::info!("Rate limiter alarm triggered. DurableObject will be deleted.");
        self.storage.delete_all().await
    }
}

#[cfg(not(test))]
#[durable_object]
pub struct RateLimiter {
    #[cfg(not(test))]
    state: State,
}

#[cfg(not(test))]
impl RateLimiter {
    fn parse_request_path(path: &str) -> Result<(FaucetInfo, String)> {
        RateLimiterCore::<DurableObjectStorage>::parse_request_path(path)
    }
    fn create_core(&self) -> RateLimiterCore<DurableObjectStorage> {
        RateLimiterCore::new(DurableObjectStorage::new(&self.state))
    }
    async fn get_rate_limit(
        &self,
        faucet_info: &FaucetInfo,
        id: &str,
        now: DateTime<Utc>,
    ) -> Result<(bool, Option<i64>, TokenAmount, TokenAmount)> {
        self.create_core()
            .get_rate_limit(faucet_info, id, now)
            .await
    }
    async fn update_rate_limit(
        &self,
        faucet_info: &FaucetInfo,
        id: &str,
        now: DateTime<Utc>,
        claimed: TokenAmount,
        dripped: TokenAmount,
    ) -> Result<()> {
        self.create_core()
            .update_rate_limit(faucet_info, id, now, claimed, dripped)
            .await
    }
}

#[cfg(not(test))]
impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let now = Utc::now();
        let (faucet_info, id) = Self::parse_request_path(&req.path())?;
        let (is_allowed, retry_after, claimed, dripped) =
            self.get_rate_limit(&faucet_info, &id, now).await?;

        if is_allowed {
            self.update_rate_limit(&faucet_info, &id, now, claimed, dripped)
                .await?;
        }
        Response::from_json(&retry_after)
    }

    async fn alarm(&self) -> Result<Response> {
        console_log!("Rate limiter alarm triggered. DurableObject will be deleted.");
        self.state.storage().delete_all().await.ok();
        Response::ok("OK")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::cast::ToPrimitive;
    use std::ops::AddAssign;

    /// Configuration for mock storage used in rate limiter tests.
    /// This struct allows tests to specify the initial state and expected behavior of the mock storage backend implementing [`RateLimiterStorage`].
    struct MockStorageConfig<'a> {
        dripped: Option<TokenAmount>,
        claimed: Option<TokenAmount>,
        block_until: Option<i64>,
        alarm: Option<i64>,
        wallet_id: &'a str,
        // If true, expect puts and set_alarm (for allowed requests)
        expect_puts: bool,
    }

    fn new_mock_storage(config: MockStorageConfig) -> MockRateLimiterStorage {
        let mut mock_storage = MockRateLimiterStorage::new();
        mock_storage
            .expect_get::<LotusJson<TokenAmount>>()
            .with(mockall::predicate::eq("dripped"))
            .returning(move |_| Ok(config.dripped.clone().map(LotusJson)));
        mock_storage
            .expect_get::<LotusJson<TokenAmount>>()
            .with(mockall::predicate::eq(format!(
                "claimed_{}",
                config.wallet_id
            )))
            .returning(move |_| Ok(config.claimed.clone().map(LotusJson)));
        mock_storage
            .expect_get::<i64>()
            .with(mockall::predicate::eq("block_until"))
            .returning(move |_| Ok(config.block_until));
        mock_storage
            .expect_get_alarm()
            .returning(move || Ok(config.alarm));
        if config.expect_puts {
            mock_storage
                .expect_put::<LotusJson<TokenAmount>>()
                .with(
                    mockall::predicate::eq("dripped"),
                    mockall::predicate::always(),
                )
                .returning(|_, _| Ok(()));
            mock_storage
                .expect_put::<LotusJson<TokenAmount>>()
                .with(
                    mockall::predicate::eq(format!("claimed_{}", config.wallet_id)),
                    mockall::predicate::always(),
                )
                .returning(|_, _| Ok(()));
            mock_storage
                .expect_put::<i64>()
                .with(
                    mockall::predicate::eq("block_until"),
                    mockall::predicate::always(),
                )
                .returning(|_, _| Ok(()));
            mock_storage.expect_set_alarm().returning(|_| Ok(()));
        }
        mock_storage
    }

    /// Checks that the initial request is allowed when storage is empty (no rate limiting).
    #[tokio::test]
    async fn test_rate_limiter_initial_request() {
        let wallet_id = "test_wallet";
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: None,
            alarm: None,
            wallet_id,
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let now = Utc::now();
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet";
        let result = core.handle_request(path, now).await.unwrap();
        assert!(result.is_none());
    }

    /// Checks that a request is rate limited.
    #[tokio::test]
    async fn test_rate_limiter_cooldown_period() {
        let wallet_id = "test_wallet";
        let now = Utc::now();
        let future_time = now + Duration::seconds(30);
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: Some(future_time.timestamp()),
            alarm: None,
            wallet_id,
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet";
        let result = core.handle_request(path, now).await.unwrap();
        assert!(result.is_some());
        let retry_after = result.unwrap();
        assert!(retry_after > 0);
        assert!(retry_after <= 30);
    }

    /// Checks that a request is rate limited if the wallet cap is exceeded.
    #[tokio::test]
    async fn test_rate_limiter_wallet_cap_exceeded() {
        let now = Utc::now();
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet";
        let faucet_info = FaucetInfo::CalibnetFIL;
        let exceeded_amount = faucet_info.wallet_cap() + TokenAmount::from_whole(1);
        let alarm_time = now.timestamp_millis() + 3600 * 1000; // 1 hour from now
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: Some(exceeded_amount.clone()),
            block_until: None,
            alarm: Some(alarm_time),
            wallet_id: "test_wallet",
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result = core.handle_request(path, now).await.unwrap();
        assert!(result.is_some());
        let retry_after = result.unwrap();
        assert!(retry_after > 0);
        assert!(retry_after <= 3600);
    }

    /// Checks that a request is rate limited if the global drip cap is exceeded.
    #[tokio::test]
    async fn test_rate_limiter_drip_cap_exceeded() {
        let now = Utc::now();
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet";
        let faucet_info = FaucetInfo::CalibnetFIL;
        let exceeded_amount = faucet_info.drip_cap() + TokenAmount::from_whole(1);
        let alarm_time = now.timestamp_millis() + 7200 * 1000; // 2 hours from now
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: Some(exceeded_amount.clone()),
            claimed: None,
            block_until: None,
            alarm: Some(alarm_time),
            wallet_id: "test_wallet",
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result = core.handle_request(path, now).await.unwrap();
        assert!(result.is_some());
        let retry_after = result.unwrap();
        assert!(retry_after > 0);
        assert!(retry_after <= 7200);
    }

    /// Checks that a successful request updates storage as expected.
    #[tokio::test]
    async fn test_rate_limiter_successful_request_updates_storage() {
        let now = Utc::now();
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet";
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: None,
            alarm: None,
            wallet_id: "test_wallet",
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result = core.handle_request(path, now).await.unwrap();
        assert!(result.is_none());
    }

    /// Checks path parsing with a valid path.
    #[tokio::test]
    async fn test_parse_request_path() {
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet_123";
        let (faucet_info, id) =
            RateLimiterCore::<MockRateLimiterStorage>::parse_request_path(path).unwrap();
        assert_eq!(faucet_info, FaucetInfo::CalibnetFIL);
        assert_eq!(id, "test_wallet_123");
    }

    /// Checks path parsing with an invalid path.
    #[tokio::test]
    async fn test_parse_request_path_invalid_faucet() {
        let path = "http://do/rate_limiter/InvalidFaucet/test_wallet";
        let result = RateLimiterCore::<MockRateLimiterStorage>::parse_request_path(path);
        assert!(result.is_err());
    }

    /// Checks that the alarm handler resets storage.
    #[tokio::test]
    async fn test_alarm_handler() {
        let mut mock_storage = MockRateLimiterStorage::new();
        mock_storage.expect_delete_all().returning(|| Ok(()));
        let core = RateLimiterCore::new(mock_storage);
        core.handle_alarm().await.unwrap();
    }

    /// Simulates a single user's journey to the wallet cap, ensuring rate limiting is enforced at the cap.
    #[tokio::test]
    async fn test_user_journey_to_wallet_cap() {
        let faucet_info = FaucetInfo::CalibnetFIL;
        let wallet_id = "test_wallet_123";
        let path = format!("http://do/rate_limiter/{faucet_info}/{wallet_id}");
        let wallet_cap_requests = (faucet_info.wallet_cap().atto()
            / faucet_info.drip_amount().atto())
        .to_u64()
        .unwrap() as usize;

        let mut claimed = TokenAmount::default();
        let mut dripped = TokenAmount::default();

        // For each request up to wallet cap
        for _ in 0..wallet_cap_requests {
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                block_until: None,
                alarm: None,
                wallet_id,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core.handle_request(&path, now).await.unwrap();
            assert!(result.is_none());
            claimed += faucet_info.drip_amount();
            dripped += faucet_info.drip_amount();
        }

        // Final request should hit wallet cap
        let mut mock_storage = MockRateLimiterStorage::new();
        let dripped_clone = dripped.clone();
        mock_storage
            .expect_get::<LotusJson<TokenAmount>>()
            .with(mockall::predicate::eq("dripped"))
            .returning(move |_| Ok(Some(LotusJson(dripped_clone.clone()))));
        let claimed_clone = claimed.clone();
        mock_storage
            .expect_get::<LotusJson<TokenAmount>>()
            .with(mockall::predicate::eq(format!("claimed_{}", wallet_id)))
            .returning(move |_| Ok(Some(LotusJson(claimed_clone.clone()))));
        let alarm_time =
            chrono::Utc::now().timestamp_millis() + faucet_info.reset_limiter_seconds() * 1000;
        mock_storage
            .expect_get_alarm()
            .returning(move || Ok(Some(alarm_time)));
        let core = RateLimiterCore::new(mock_storage);
        let now = chrono::Utc::now();
        let result = core.handle_request(&path, now).await.unwrap();
        assert!(result.is_some());
        let retry_after = result.unwrap();
        assert!(retry_after > 0 && retry_after <= faucet_info.reset_limiter_seconds());
    }

    /// Simulates multiple users claiming up to the global drip cap, ensuring all are rate limited at the cap.
    #[tokio::test]
    async fn test_multiple_user_journey_to_drip_cap() {
        let faucet_info = FaucetInfo::CalibnetFIL;
        let wallet_1 = "wallet_1";
        let wallet_2 = "wallet_2";
        let wallet_3 = "wallet_3";
        let wallet_cap_requests = (faucet_info.wallet_cap().atto()
            / faucet_info.drip_amount().atto())
        .to_u64()
        .unwrap() as usize;
        let mut dripped = TokenAmount::default();
        let mut claimed_1 = TokenAmount::default();
        let mut claimed_2 = TokenAmount::default();
        let mut claimed_3 = TokenAmount::default();

        // Wallet 1: Up to wallet cap
        for _ in 0..wallet_cap_requests {
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                block_until: None,
                alarm: None,
                wallet_id: wallet_1,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet_1}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_none());
            claimed_1 += faucet_info.drip_amount();
            dripped += faucet_info.drip_amount();
        }
        // Wallet 2: Up to wallet cap
        for _ in 0..wallet_cap_requests {
            if dripped >= faucet_info.drip_cap() {
                break;
            }
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                block_until: None,
                alarm: None,
                wallet_id: wallet_2,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet_2}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_none());
            claimed_2 += faucet_info.drip_amount();
            dripped += faucet_info.drip_amount();
        }
        // Wallet 3: Up to drip cap
        while dripped < faucet_info.drip_cap() {
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                block_until: None,
                alarm: None,
                wallet_id: wallet_3,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet_3}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_none());
            claimed_3 += faucet_info.drip_amount();
            dripped += faucet_info.drip_amount();
        }
        // Now all wallets should be rate limited due to drip cap
        for wallet in [wallet_1, wallet_2, wallet_3] {
            let mut mock_storage = MockRateLimiterStorage::new();
            let dripped_clone = dripped.clone();
            let claimed_clone = match wallet {
                w if w == wallet_1 => claimed_1.clone(),
                w if w == wallet_2 => claimed_2.clone(),
                _ => claimed_3.clone(),
            };
            mock_storage
                .expect_get::<LotusJson<TokenAmount>>()
                .with(mockall::predicate::eq("dripped"))
                .returning(move |_| Ok(Some(LotusJson(dripped_clone.clone()))));
            mock_storage
                .expect_get::<LotusJson<TokenAmount>>()
                .with(mockall::predicate::eq(format!("claimed_{}", wallet)))
                .returning(move |_| Ok(Some(LotusJson(claimed_clone.clone()))));
            let alarm_time =
                chrono::Utc::now().timestamp_millis() + faucet_info.reset_limiter_seconds() * 1000;
            mock_storage
                .expect_get_alarm()
                .returning(move || Ok(Some(alarm_time)));
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_some());
        }
    }

    /// Simulates reaching the drip cap, triggering the alarm reset, and verifies new requests are allowed after reset.
    #[tokio::test]
    async fn test_alarm_reset_cycle() {
        let faucet_info = FaucetInfo::CalibnetFIL;
        let wallets = ["wallet_1", "wallet_2", "wallet_3", "wallet_4", "wallet_5"];
        let mut dripped = TokenAmount::default();
        let mut claimed = std::collections::HashMap::new();
        for &wallet in &wallets {
            claimed.insert(wallet, TokenAmount::default());
        }
        // Use multiple wallets to reach the global cap
        'outer: for &wallet in &wallets {
            for _ in 0..2 {
                if dripped >= faucet_info.drip_cap() {
                    break 'outer;
                }
                let mock_storage = new_mock_storage(MockStorageConfig {
                    dripped: None,
                    claimed: None,
                    block_until: None,
                    alarm: None,
                    wallet_id: wallet,
                    expect_puts: true,
                });
                let core = RateLimiterCore::new(mock_storage);
                let now = chrono::Utc::now();
                let result = core
                    .handle_request(
                        &format!("http://do/rate_limiter/{faucet_info}/{}", wallet),
                        now,
                    )
                    .await
                    .unwrap();
                assert!(result.is_none());
                claimed
                    .get_mut(wallet)
                    .unwrap()
                    .add_assign(faucet_info.drip_amount());
                dripped += faucet_info.drip_amount();
            }
        }
        // Verify drip cap is reached
        let mut mock_storage = MockRateLimiterStorage::new();
        let dripped_clone = dripped.clone();
        let claimed_clone = claimed["wallet_1"].clone();
        mock_storage
            .expect_get::<LotusJson<TokenAmount>>()
            .with(mockall::predicate::eq("dripped"))
            .returning(move |_| Ok(Some(LotusJson(dripped_clone.clone()))));
        mock_storage
            .expect_get::<LotusJson<TokenAmount>>()
            .with(mockall::predicate::eq(format!("claimed_{}", "wallet_1")))
            .returning(move |_| Ok(Some(LotusJson(claimed_clone.clone()))));
        let alarm_time =
            chrono::Utc::now().timestamp_millis() + faucet_info.reset_limiter_seconds() * 1000;
        mock_storage
            .expect_get_alarm()
            .returning(move || Ok(Some(alarm_time)));
        let core = RateLimiterCore::new(mock_storage);
        let now = chrono::Utc::now();
        let result = core
            .handle_request(
                &format!("http://do/rate_limiter/{faucet_info}/wallet_1"),
                now,
            )
            .await
            .unwrap();
        assert!(result.is_some());
        // Simulate alarm handler
        let mut mock_storage = MockRateLimiterStorage::new();
        mock_storage.expect_delete_all().returning(|| Ok(()));
        let core = RateLimiterCore::new(mock_storage);
        core.handle_alarm().await.unwrap();
        // After alarm, new request should be allowed (storage is reset)
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: None,
            alarm: None,
            wallet_id: "wallet_1",
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let now = chrono::Utc::now();
        let result = core
            .handle_request(
                &format!("http://do/rate_limiter/{faucet_info}/wallet_1"),
                now,
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    /// Simulates the cool-down stages: allowed, blocked, partially blocked, then allowed again.
    #[tokio::test]
    async fn test_cooldown_period_progression() {
        let faucet_info = FaucetInfo::CalibnetFIL;
        let wallet_id = "cooldown_test_wallet";
        let path = format!("http://do/rate_limiter/{faucet_info}/{wallet_id}");
        let now = chrono::Utc::now();
        // Step 1: First request (should succeed)
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: None,
            alarm: None,
            wallet_id,
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result_1 = core.handle_request(&path, now).await.unwrap();
        assert!(result_1.is_none());
        // Step 2: Immediate retry (should be rate limited)
        let block_until = now + chrono::Duration::seconds(faucet_info.rate_limit_seconds());
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: Some(block_until.timestamp()),
            alarm: None,
            wallet_id,
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result_2 = core.handle_request(&path, now).await.unwrap();
        assert!(result_2.is_some());
        let retry_2 = result_2.unwrap();
        assert!((0..=faucet_info.rate_limit_seconds()).contains(&retry_2));
        // Step 3: Partial cooldown (should still be rate limited, less time left)
        let partial_block_until = now + chrono::Duration::seconds(1); // 1 second in the future
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: Some(partial_block_until.timestamp()),
            alarm: None,
            wallet_id,
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result_3 = core.handle_request(&path, now).await.unwrap();
        assert!(result_3.is_some());
        let retry_3 = result_3.unwrap();
        assert!((0..=1).contains(&retry_3));
        assert!(retry_3 < retry_2);
        let past_block_until = now - chrono::Duration::seconds(1);
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            block_until: Some(past_block_until.timestamp()),
            alarm: None,
            wallet_id,
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let result_4 = core.handle_request(&path, now).await.unwrap();
        assert!(result_4.is_none());
    }
}
