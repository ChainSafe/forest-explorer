#![cfg(feature = "ssr")]
use std::net::{AddrParseError, IpAddr, Ipv6Addr};
use std::str::FromStr as _;

use crate::faucet::constants::FaucetInfo;
use crate::utils::drip_amount::DripAmount;
use chrono::{DateTime, Duration, Utc};
use worker::*;

#[cfg(test)]
use mockall::automock;

/// Abstraction for storage backend used by the rate limiter.
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

/// Amounts dripped so far in the current reset window.
#[derive(Debug)]
struct Claims {
    faucet: DripAmount,
    wallet: DripAmount,
    ip: DripAmount,
}

/// A client IP normalized into its rate limiter key. IPv6 is keyed by its /64 prefix, since a
/// single client typically controls a whole /64 and could otherwise rotate addresses freely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub(super) struct IpKey(IpAddr);

impl std::str::FromStr for IpKey {
    type Err = AddrParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ip = s.trim().parse::<IpAddr>()?;
        Ok(Self(match ip {
            IpAddr::V4(_) => ip,
            IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
                Some(v4) => IpAddr::V4(v4),
                None => IpAddr::V6(Ipv6Addr::from(u128::from(v6) & (u128::MAX << 64))),
            },
        }))
    }
}

/// A rate limiter query, parsed from `.../{faucet_info}/{wallet_id}/{ip_key}`.
#[derive(Debug)]
struct RateLimitRequest {
    faucet_info: FaucetInfo,
    wallet_id: String,
    ip_key: IpKey,
}

impl RateLimitRequest {
    fn parse(path: &str) -> Result<Self> {
        let mut path_info = path.split('/');
        let ip_key = path_info.next_back().unwrap_or_default();
        let wallet_id = path_info.next_back().unwrap_or_default().to_string();
        let faucet_info = FaucetInfo::from_str(path_info.next_back().unwrap_or_default())
            .map_err(|e| Error::RustError(e.to_string()))?;
        if wallet_id.is_empty() {
            return Err(Error::RustError(format!(
                "Missing wallet in rate limiter path: {path}"
            )));
        }
        let ip_key = ip_key.parse().map_err(|e| {
            Error::RustError(format!("Invalid IP in rate limiter path {path}: {e}"))
        })?;
        Ok(Self {
            faucet_info,
            wallet_id,
            ip_key,
        })
    }

    fn wallet_storage_key(&self) -> String {
        format!("claimed_{}", self.wallet_id)
    }

    fn ip_storage_key(&self) -> String {
        format!("claimed_ip_{}", self.ip_key)
    }
}

impl<S: RateLimiterStorage> RateLimiterCore<S> {
    fn new(storage: S) -> Self {
        Self { storage }
    }

    async fn read_claimed(&self, key: &str, faucet_info: &FaucetInfo) -> Result<DripAmount> {
        Ok(self
            .storage
            .get::<DripAmount>(key)
            .await?
            .unwrap_or_else(|| DripAmount::zero(faucet_info.token_type())))
    }

    /// Returns the retry-after in seconds if the request is rate limited, along with the current claims.
    async fn get_rate_limit(
        &self,
        req: &RateLimitRequest,
        now: DateTime<Utc>,
    ) -> Result<(Option<i64>, Claims)> {
        let RateLimitRequest {
            faucet_info,
            wallet_id,
            ..
        } = req;
        let claims = Claims {
            faucet: self.read_claimed("dripped", faucet_info).await?,
            wallet: self
                .read_claimed(&req.wallet_storage_key(), faucet_info)
                .await?,
            ip: self
                .read_claimed(&req.ip_storage_key(), faucet_info)
                .await?,
        };
        let cap = if claims.faucet >= faucet_info.drip_cap() {
            Some("Drip")
        } else if claims.wallet >= faucet_info.wallet_cap() {
            Some("Wallet")
        } else if claims.ip >= faucet_info.ip_cap() {
            Some("IP")
        } else {
            None
        };
        if let Some(cap) = cap {
            let retry_after = self
                .storage
                .get_alarm()
                .await
                .ok()
                .flatten()
                .map(|alarm| Duration::milliseconds(alarm - now.timestamp_millis()).num_seconds())
                .unwrap_or(0);
            log::info!(
                "{faucet_info} Rate limiter for {wallet_id} invoked: {cap} capped now={now:?}, claims={claims:?}, retry_after={retry_after:?}"
            );
            return Ok((Some(retry_after), claims));
        }
        let block_until = self
            .storage
            .get::<i64>("block_until")
            .await?
            .and_then(|secs| DateTime::<Utc>::from_timestamp(secs, 0))
            .unwrap_or(now);
        if block_until > now {
            let retry_after = block_until.signed_duration_since(now).num_seconds();
            log::info!(
                "{faucet_info} Rate limiter for {wallet_id} invoked: now={now:?}, claims={claims:?}, retry_after={retry_after:?}"
            );
            return Ok((Some(retry_after), claims));
        }
        Ok((None, claims))
    }

    async fn update_rate_limit(
        &self,
        req: &RateLimitRequest,
        now: DateTime<Utc>,
        claims: &Claims,
    ) -> Result<()> {
        let RateLimitRequest {
            faucet_info,
            wallet_id,
            ..
        } = req;
        let drip_amount = faucet_info.drip_amount();
        let updated = Claims {
            faucet: &claims.faucet + &drip_amount,
            wallet: &claims.wallet + &drip_amount,
            ip: &claims.ip + &drip_amount,
        };
        let next_block = now + Duration::seconds(faucet_info.rate_limit_seconds());
        self.storage.put("dripped", updated.faucet.clone()).await?;
        self.storage
            .put(&req.wallet_storage_key(), updated.wallet.clone())
            .await?;
        self.storage
            .put(&req.ip_storage_key(), updated.ip.clone())
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
        log::info!(
            "{faucet_info} Rate limiter for {wallet_id} set: now={now:?}, block_until={next_block:?}, claims={updated:?}"
        );
        Ok(())
    }

    async fn handle_request(&self, path: &str, now: DateTime<Utc>) -> Result<Option<i64>> {
        let req = RateLimitRequest::parse(path)?;
        let (retry_after, claims) = self.get_rate_limit(&req, now).await?;
        if retry_after.is_none() {
            self.update_rate_limit(&req, now, &claims).await?;
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
impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let retry_after = RateLimiterCore::new(DurableObjectStorage::new(&self.state))
            .handle_request(&req.path(), Utc::now())
            .await?;
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
    use crate::utils::drip_amount::DripAmount;
    use fvm_shared::econ::TokenAmount;

    const CALIBNET_PER_WALLET_DRIP_MULTIPLIER: i64 = 2;
    const TEST_IP: &str = "203.0.113.7";

    /// Configuration for mock storage used in rate limiter tests.
    /// This struct allows tests to specify the initial state and expected behavior of the mock storage backend implementing [`RateLimiterStorage`].
    struct MockStorageConfig<'a> {
        dripped: Option<DripAmount>,
        claimed: Option<DripAmount>,
        claimed_ip: Option<DripAmount>,
        block_until: Option<i64>,
        alarm: Option<i64>,
        wallet_id: &'a str,
        // If true, expect puts and set_alarm (for allowed requests)
        expect_puts: bool,
    }

    fn new_mock_storage(config: MockStorageConfig) -> MockRateLimiterStorage {
        let mut mock_storage = MockRateLimiterStorage::new();
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq("dripped"))
            .returning(move |_| Ok(config.dripped.clone()));
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq(format!(
                "claimed_{}",
                config.wallet_id
            )))
            .returning(move |_| Ok(config.claimed.clone()));
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq(format!("claimed_ip_{TEST_IP}")))
            .returning(move |_| Ok(config.claimed_ip.clone()));
        mock_storage
            .expect_get::<i64>()
            .with(mockall::predicate::eq("block_until"))
            .returning(move |_| Ok(config.block_until));
        mock_storage
            .expect_get_alarm()
            .returning(move || Ok(config.alarm));
        if config.expect_puts {
            mock_storage
                .expect_put::<DripAmount>()
                .with(
                    mockall::predicate::eq("dripped"),
                    mockall::predicate::always(),
                )
                .returning(|_, _| Ok(()));
            mock_storage
                .expect_put::<DripAmount>()
                .with(
                    mockall::predicate::eq(format!("claimed_{}", config.wallet_id)),
                    mockall::predicate::always(),
                )
                .returning(|_, _| Ok(()));
            mock_storage
                .expect_put::<DripAmount>()
                .with(
                    mockall::predicate::eq(format!("claimed_ip_{TEST_IP}")),
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
            claimed_ip: None,
            block_until: None,
            alarm: None,
            wallet_id,
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let now = Utc::now();
        let path = &format!("http://do/rate_limiter/CalibnetFIL/test_wallet/{TEST_IP}");
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
            claimed_ip: None,
            block_until: Some(future_time.timestamp()),
            alarm: None,
            wallet_id,
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let path = &format!("http://do/rate_limiter/CalibnetFIL/test_wallet/{TEST_IP}");
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
        let path = &format!("http://do/rate_limiter/CalibnetFIL/test_wallet/{TEST_IP}");
        let faucet_info = FaucetInfo::CalibnetFIL;
        let exceeded_amount =
            &faucet_info.wallet_cap() + &DripAmount::Token(TokenAmount::from_whole(1));
        let alarm_time = now.timestamp_millis() + 3600 * 1000; // 1 hour from now
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: Some(exceeded_amount),
            claimed_ip: None,
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
        let path = &format!("http://do/rate_limiter/CalibnetFIL/test_wallet/{TEST_IP}");
        let faucet_info = FaucetInfo::CalibnetFIL;
        let exceeded_amount =
            &faucet_info.drip_cap() + &DripAmount::Token(TokenAmount::from_whole(1));
        let alarm_time = now.timestamp_millis() + 7200 * 1000; // 2 hours from now
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: Some(exceeded_amount),
            claimed: None,
            claimed_ip: None,
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

    /// Simulates a single IP claiming to a fresh wallet each time, ensuring the per-IP cap stops it.
    #[tokio::test]
    async fn test_single_ip_journey_to_ip_cap_with_fresh_wallets() {
        let faucet_info = FaucetInfo::CalibnetFIL;
        let drip_amount = faucet_info.drip_amount();
        let mut claimed_ip = DripAmount::zero(faucet_info.token_type());
        let mut i = 0;
        while claimed_ip < faucet_info.ip_cap() {
            i += 1;
            let wallet_id = format!("fresh_wallet_{i}");
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                claimed_ip: Some(claimed_ip.clone()),
                block_until: None,
                alarm: None,
                wallet_id: &wallet_id,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let path = format!("http://do/rate_limiter/{faucet_info}/{wallet_id}/{TEST_IP}");
            let result = core.handle_request(&path, Utc::now()).await.unwrap();
            assert!(result.is_none(), "claim {i} should be allowed");
            claimed_ip += &drip_amount;
        }
        assert_eq!(claimed_ip, faucet_info.ip_cap());
        assert!(i > 1, "IP cap should allow more than one wallet's claim");

        let alarm_time = Utc::now().timestamp_millis() + faucet_info.reset_limiter_seconds() * 1000;
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            claimed_ip: Some(claimed_ip),
            block_until: None,
            alarm: Some(alarm_time),
            wallet_id: "another_fresh_wallet",
            expect_puts: false,
        });
        let core = RateLimiterCore::new(mock_storage);
        let path = format!("http://do/rate_limiter/{faucet_info}/another_fresh_wallet/{TEST_IP}");
        let retry_after = core
            .handle_request(&path, Utc::now())
            .await
            .unwrap()
            .unwrap();
        assert!(retry_after > 0 && retry_after <= faucet_info.reset_limiter_seconds());
    }

    /// Checks that a successful request writes the faucet, wallet and IP counters incremented by one drip.
    #[tokio::test]
    async fn test_rate_limiter_successful_request_updates_storage() {
        let faucet_info = FaucetInfo::CalibnetFIL;
        let drip_amount = faucet_info.drip_amount();
        let claimed = drip_amount.clone();
        let claimed_ip = &drip_amount + &drip_amount;
        let mut mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: Some(claimed.clone()),
            claimed_ip: Some(claimed_ip.clone()),
            block_until: None,
            alarm: None,
            wallet_id: "test_wallet",
            expect_puts: false,
        });
        mock_storage
            .expect_put::<DripAmount>()
            .with(
                mockall::predicate::eq("dripped"),
                mockall::predicate::eq(drip_amount.clone()),
            )
            .times(1)
            .returning(|_, _| Ok(()));
        mock_storage
            .expect_put::<DripAmount>()
            .with(
                mockall::predicate::eq("claimed_test_wallet"),
                mockall::predicate::eq(&claimed + &drip_amount),
            )
            .times(1)
            .returning(|_, _| Ok(()));
        mock_storage
            .expect_put::<DripAmount>()
            .with(
                mockall::predicate::eq(format!("claimed_ip_{TEST_IP}")),
                mockall::predicate::eq(&claimed_ip + &drip_amount),
            )
            .times(1)
            .returning(|_, _| Ok(()));
        mock_storage
            .expect_put::<i64>()
            .with(
                mockall::predicate::eq("block_until"),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(|_, _| Ok(()));
        mock_storage.expect_set_alarm().returning(|_| Ok(()));
        let core = RateLimiterCore::new(mock_storage);
        let path = format!("http://do/rate_limiter/{faucet_info}/test_wallet/{TEST_IP}");
        let result = core.handle_request(&path, Utc::now()).await.unwrap();
        assert!(result.is_none());
    }

    /// Checks that a storage read failure denies the request instead of treating the key as unset.
    #[tokio::test]
    async fn test_rate_limiter_storage_read_error_denies_request() {
        fn read<T>(key: &str, failing_key: &str) -> Result<Option<T>> {
            if key == failing_key {
                Err(Error::RustError("storage unavailable".to_string()))
            } else {
                Ok(None)
            }
        }
        let path = &format!("http://do/rate_limiter/CalibnetFIL/test_wallet/{TEST_IP}");
        for failing_key in [
            "dripped",
            "claimed_test_wallet",
            "claimed_ip_203.0.113.7",
            "block_until",
        ] {
            let mut mock_storage = MockRateLimiterStorage::new();
            mock_storage
                .expect_get::<DripAmount>()
                .returning(move |key| read(key, failing_key));
            mock_storage
                .expect_get::<i64>()
                .returning(move |key| read(key, failing_key));
            let core = RateLimiterCore::new(mock_storage);
            let result = core.handle_request(path, Utc::now()).await;
            assert!(result.is_err(), "read error on {failing_key} was ignored");
        }
    }

    #[test]
    fn test_ip_key() {
        let cases = [
            ("203.0.113.7", "203.0.113.7"),
            (" 203.0.113.7 ", "203.0.113.7"),
            ("2001:db8:1:2::1", "2001:db8:1:2::"),
            ("2001:db8:1:2:ffff:eeee:dddd:cccc", "2001:db8:1:2::"),
            ("2001:db8:1:3::1", "2001:db8:1:3::"),
            ("::ffff:203.0.113.7", "203.0.113.7"),
        ];
        for (raw, expected) in cases {
            assert_eq!(raw.parse::<IpKey>().unwrap().to_string(), expected, "{raw}");
        }
    }

    #[test]
    fn test_ip_key_invalid() {
        for raw in [
            "",
            "not-an-ip",
            "203.0.113",
            "2001:db8::/64",
            "203.0.113.7, 10.0.0.1",
        ] {
            assert!(raw.parse::<IpKey>().is_err(), "{raw}");
        }
    }

    /// Checks path parsing with a valid path, re-normalizing the IP segment.
    #[tokio::test]
    async fn test_parse_request_path() {
        let path = "http://do/rate_limiter/CalibnetFIL/test_wallet_123/2001:db8:1:2::1";
        let req = RateLimitRequest::parse(path).unwrap();
        assert_eq!(req.faucet_info, FaucetInfo::CalibnetFIL);
        assert_eq!(req.wallet_id, "test_wallet_123");
        assert_eq!(req.ip_storage_key(), "claimed_ip_2001:db8:1:2::");
    }

    /// Checks path parsing rejects a path without the wallet or a valid IP segment.
    #[tokio::test]
    async fn test_parse_request_path_missing_segment() {
        for path in [
            "http://do/rate_limiter/CalibnetFIL/test_wallet/",
            "http://do/rate_limiter/CalibnetFIL/test_wallet/not-an-ip",
            "http://do/rate_limiter/CalibnetFIL//203.0.113.7",
        ] {
            let result = RateLimitRequest::parse(path);
            assert!(result.is_err(), "{path}");
        }
    }

    /// Checks path parsing with an invalid path.
    #[tokio::test]
    async fn test_parse_request_path_invalid_faucet() {
        let path = "http://do/rate_limiter/InvalidFaucet/test_wallet/203.0.113.7";
        let result = RateLimitRequest::parse(path);
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
        let path = format!("http://do/rate_limiter/{faucet_info}/{wallet_id}/{TEST_IP}");
        let drip_amount = faucet_info.drip_amount();
        let mut claimed = DripAmount::zero(faucet_info.token_type());
        let mut dripped = DripAmount::zero(faucet_info.token_type());

        // For each request up to wallet cap
        for _ in 0..CALIBNET_PER_WALLET_DRIP_MULTIPLIER {
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                claimed_ip: None,
                block_until: None,
                alarm: None,
                wallet_id,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core.handle_request(&path, now).await.unwrap();
            assert!(result.is_none());
            claimed += &drip_amount;
            dripped += &drip_amount;
        }

        // Final request should hit wallet cap
        let mut mock_storage = MockRateLimiterStorage::new();
        let dripped_clone = dripped.clone();
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq("dripped"))
            .returning(move |_| Ok(Some(dripped_clone.clone())));
        let claimed_clone = claimed.clone();
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq(format!("claimed_{}", wallet_id)))
            .returning(move |_| Ok(Some(claimed_clone.clone())));
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq(format!("claimed_ip_{TEST_IP}")))
            .returning(|_| Ok(None));
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
        let drip_amount = faucet_info.drip_amount();
        let drip_cap = faucet_info.drip_cap();
        let mut dripped = DripAmount::zero(faucet_info.token_type());
        let mut claimed_1 = DripAmount::zero(faucet_info.token_type());
        let mut claimed_2 = DripAmount::zero(faucet_info.token_type());
        let mut claimed_3 = DripAmount::zero(faucet_info.token_type());

        // Wallet 1: Up to wallet cap
        for _ in 0..CALIBNET_PER_WALLET_DRIP_MULTIPLIER {
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                claimed_ip: None,
                block_until: None,
                alarm: None,
                wallet_id: wallet_1,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet_1}/{TEST_IP}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_none());
            claimed_1 += &drip_amount;
            dripped += &drip_amount;
        }
        // Wallet 2: Up to wallet cap
        for _ in 0..CALIBNET_PER_WALLET_DRIP_MULTIPLIER {
            if dripped >= faucet_info.drip_cap() {
                break;
            }
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                claimed_ip: None,
                block_until: None,
                alarm: None,
                wallet_id: wallet_2,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet_2}/{TEST_IP}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_none());
            claimed_2 += &drip_amount;
            dripped += &drip_amount;
        }
        // Wallet 3: Up to drip cap
        while dripped < drip_cap {
            let mock_storage = new_mock_storage(MockStorageConfig {
                dripped: None,
                claimed: None,
                claimed_ip: None,
                block_until: None,
                alarm: None,
                wallet_id: wallet_3,
                expect_puts: true,
            });
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet_3}/{TEST_IP}"),
                    now,
                )
                .await
                .unwrap();
            assert!(result.is_none());
            claimed_3 += &drip_amount;
            dripped += &drip_amount;
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
                .expect_get::<DripAmount>()
                .with(mockall::predicate::eq("dripped"))
                .returning(move |_| Ok(Some(dripped_clone.clone())));
            mock_storage
                .expect_get::<DripAmount>()
                .with(mockall::predicate::eq(format!("claimed_{}", wallet)))
                .returning(move |_| Ok(Some(claimed_clone.clone())));
            mock_storage
                .expect_get::<DripAmount>()
                .with(mockall::predicate::eq(format!("claimed_ip_{TEST_IP}")))
                .returning(|_| Ok(None));
            let alarm_time =
                chrono::Utc::now().timestamp_millis() + faucet_info.reset_limiter_seconds() * 1000;
            mock_storage
                .expect_get_alarm()
                .returning(move || Ok(Some(alarm_time)));
            let core = RateLimiterCore::new(mock_storage);
            let now = chrono::Utc::now();
            let result = core
                .handle_request(
                    &format!("http://do/rate_limiter/{faucet_info}/{wallet}/{TEST_IP}"),
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
        let drip_amount = faucet_info.drip_amount();
        let drip_cap = faucet_info.drip_cap();
        let mut dripped = DripAmount::zero(faucet_info.token_type());
        let mut claimed = std::collections::HashMap::new();
        for &wallet in &wallets {
            claimed.insert(wallet, DripAmount::zero(faucet_info.token_type()));
        }
        // Use multiple wallets to reach the global cap
        'outer: for &wallet in &wallets {
            for _ in 0..2 {
                if dripped >= drip_cap {
                    break 'outer;
                }
                let mock_storage = new_mock_storage(MockStorageConfig {
                    dripped: None,
                    claimed: None,
                    claimed_ip: None,
                    block_until: None,
                    alarm: None,
                    wallet_id: wallet,
                    expect_puts: true,
                });
                let core = RateLimiterCore::new(mock_storage);
                let now = chrono::Utc::now();
                let result = core
                    .handle_request(
                        &format!("http://do/rate_limiter/{faucet_info}/{}/{TEST_IP}", wallet),
                        now,
                    )
                    .await
                    .unwrap();
                assert!(result.is_none());
                let claimed_amount = claimed.get_mut(wallet).unwrap();
                *claimed_amount += &drip_amount;
                dripped += &drip_amount;
            }
        }
        // Verify drip cap is reached
        let mut mock_storage = MockRateLimiterStorage::new();
        let dripped_clone = dripped.clone();
        let claimed_clone = claimed["wallet_1"].clone();
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq("dripped"))
            .returning(move |_| Ok(Some(dripped_clone.clone())));
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq(format!("claimed_{}", "wallet_1")))
            .returning(move |_| Ok(Some(claimed_clone.clone())));
        mock_storage
            .expect_get::<DripAmount>()
            .with(mockall::predicate::eq(format!("claimed_ip_{TEST_IP}")))
            .returning(|_| Ok(None));
        let alarm_time =
            chrono::Utc::now().timestamp_millis() + faucet_info.reset_limiter_seconds() * 1000;
        mock_storage
            .expect_get_alarm()
            .returning(move || Ok(Some(alarm_time)));
        let core = RateLimiterCore::new(mock_storage);
        let now = chrono::Utc::now();
        let result = core
            .handle_request(
                &format!("http://do/rate_limiter/{faucet_info}/wallet_1/{TEST_IP}"),
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
            claimed_ip: None,
            block_until: None,
            alarm: None,
            wallet_id: "wallet_1",
            expect_puts: true,
        });
        let core = RateLimiterCore::new(mock_storage);
        let now = chrono::Utc::now();
        let result = core
            .handle_request(
                &format!("http://do/rate_limiter/{faucet_info}/wallet_1/{TEST_IP}"),
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
        let path = format!("http://do/rate_limiter/{faucet_info}/{wallet_id}/{TEST_IP}");
        let now = chrono::Utc::now();
        // Step 1: First request (should succeed)
        let mock_storage = new_mock_storage(MockStorageConfig {
            dripped: None,
            claimed: None,
            claimed_ip: None,
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
            claimed_ip: None,
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
            claimed_ip: None,
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
            claimed_ip: None,
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
