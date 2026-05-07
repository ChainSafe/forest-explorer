use fvm_shared::{econ::TokenAmount, sector::StoragePower};
use num_traits::Zero as _;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DripAmount {
    Token(#[serde(with = "crate::utils::lotus_json")] TokenAmount),
    Storage(#[serde(with = "crate::utils::lotus_json")] StoragePower),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    /// Filecoin native token
    Native,
    /// ERC-20 token, e.g., `USDFC`
    Erc20(ContractAddress),
    /// Datacap token, e.g., `MiB`
    Datacap,
}

pub type ContractAddress = alloy::primitives::Address;

impl DripAmount {
    pub fn zero(token_type: TokenType) -> DripAmount {
        match token_type {
            TokenType::Native | TokenType::Erc20(_) => DripAmount::Token(TokenAmount::zero()),
            TokenType::Datacap => DripAmount::Storage(StoragePower::zero()),
        }
    }

    /// Subtract rhs from self, saturating at the appropriate zero representation.
    pub fn saturating_sub(&self, rhs: &DripAmount) -> DripAmount {
        match (self, rhs) {
            (DripAmount::Token(a), DripAmount::Token(b)) => {
                DripAmount::Token(if *a <= *b {
                    TokenAmount::zero()
                } else {
                    a - b
                })
            }
            (DripAmount::Storage(a), DripAmount::Storage(b)) => DripAmount::Storage(
                if a <= b {
                    StoragePower::zero()
                } else {
                    a - b
                },
            ),
            _ => unreachable!("DripAmount variant mismatch"),
        }
    }
}

impl Add<&DripAmount> for &DripAmount {
    type Output = DripAmount;

    fn add(self, rhs: &DripAmount) -> DripAmount {
        match (&self, rhs) {
            (DripAmount::Token(x), DripAmount::Token(y)) => DripAmount::Token(x + y),
            (DripAmount::Storage(x), DripAmount::Storage(y)) => DripAmount::Storage(x + y),
            _ => unreachable!("DripAmount variant mismatch"),
        }
    }
}

impl AddAssign<&DripAmount> for DripAmount {
    fn add_assign(&mut self, rhs: &DripAmount) {
        *self = self.add(rhs);
    }
}

impl Mul<i64> for DripAmount {
    type Output = DripAmount;

    fn mul(self, mult: i64) -> DripAmount {
        match self {
            DripAmount::Token(t) => DripAmount::Token(t * mult),
            DripAmount::Storage(s) => DripAmount::Storage(s * mult),
        }
    }
}
