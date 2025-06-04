//! This module provides conversions between foreign types.

use fvm_shared::{bigint::BigInt, econ::TokenAmount};

pub trait TokenAmountAlloyExt {
    /// Converts an [`alloy::primitives::U256`] amount to a [`TokenAmount`].
    ///
    /// Warning! The assumption here is that the decimals are the same for both Filecoin
    /// and given ERC20 token. This holds true for USDFC, but may not hold for other
    /// tokens.
    fn from_alloy_amount(amount: &alloy::primitives::U256) -> Self;

    /// Converts a [`TokenAmount`] to an [`alloy::primitives::U256`] amount.
    ///
    /// Warning! The assumption here is that the decimals are the same for both Filecoin
    /// and given ERC20 token. This holds true for USDFC, but may not hold for other
    /// tokens.
    #[allow(dead_code)]
    fn to_alloy_amount(&self) -> alloy::primitives::U256;
}

impl TokenAmountAlloyExt for fvm_shared::econ::TokenAmount {
    fn from_alloy_amount(amount: &alloy::primitives::U256) -> Self {
        TokenAmount::from_atto(BigInt::from_bytes_be(
            fvm_shared::bigint::Sign::Plus,
            &amount.to_be_bytes_trimmed_vec(),
        ))
    }
    fn to_alloy_amount(&self) -> alloy::primitives::U256 {
        let atto = self.atto();
        alloy::primitives::U256::from_be_slice(&atto.to_signed_bytes_be())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_from_alloy_amount() {
        let amount = U256::from(1000000000000000000u64); // 1e18, so 1 FIL
        let token_amount = TokenAmount::from_alloy_amount(&amount);
        assert_eq!(token_amount.atto(), &BigInt::from(1000000000000000000u64));
    }

    #[test]
    fn test_to_alloy_amount() {
        let token_amount = TokenAmount::from_whole(1);
        let alloy_amount = token_amount.to_alloy_amount();
        assert_eq!(alloy_amount, U256::from(1000000000000000000u64));
    }

    #[test]
    fn test_alloy_conversion_round_trip() {
        let original_amount = TokenAmount::from_whole(42);
        let alloy_amount = original_amount.to_alloy_amount();
        let converted_back = TokenAmount::from_alloy_amount(&alloy_amount);
        assert_eq!(original_amount, converted_back);
    }
}
