pub mod views;

use fvm_shared::econ::TokenAmount;
use std::sync::LazyLock;

pub const CALIBNET_RATE_LIMIT_SECONDS: i64 = 60;
pub static FIL_CALIBNET_UNIT: &str = "tFIL";
/// The amount of mainnet FIL to be dripped to the user. This corresponds to 1 tFIL.
pub static CALIBNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_whole(1));
