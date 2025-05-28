use fvm_shared::econ::TokenAmount;
use std::sync::LazyLock;

// Calibnet constants
pub const CALIBNET_RATE_LIMIT_SECONDS: i64 = 60;
pub static FIL_CALIBNET_UNIT: &str = "tFIL";
/// The amount of mainnet FIL to be dripped to the user. This corresponds to 1 tFIL.
pub static CALIBNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_whole(1));

// Mainnet constants
pub const MAINNET_RATE_LIMIT_SECONDS: i64 = 600;
pub static FIL_MAINNET_UNIT: &str = "FIL";
/// The amount of mainnet FIL to be dripped to the user. This corresponds to 0.01 FIL.
pub static MAINNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_nano(10_000_000));
