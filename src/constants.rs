use std::sync::LazyLock;

use fvm_shared::econ::TokenAmount;

/// The rate limit imposed by the CloudFlare's rate limiter, and also reflected in the user
/// interface.
pub const CALIBNET_RATE_LIMIT_SECONDS: i64 = 60;
pub const MAINNET_RATE_LIMIT_SECONDS: i64 = 600;
/// The amount of mainnet FIL to be dripped to the user. This corresponds to 0.01 FIL.
pub static MAINNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_nano(10_000_000));
/// The amount of calibnet tFIL to be dripped to the user.
pub static CALIBNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_whole(1));
pub static FIL_MAINNET_UNIT: &str = "FIL";
pub static FIL_CALIBNET_UNIT: &str = "tFIL";
