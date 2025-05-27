// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod views;

use fvm_shared::econ::TokenAmount;
use std::sync::LazyLock;

pub const MAINNET_RATE_LIMIT_SECONDS: i64 = 600;
pub static FIL_MAINNET_UNIT: &str = "FIL";
/// The amount of mainnet FIL to be dripped to the user. This corresponds to 0.01 FIL.
pub static MAINNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_nano(10_000_000));
