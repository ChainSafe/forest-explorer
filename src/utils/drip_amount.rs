use fvm_shared::{econ::TokenAmount, sector::StoragePower};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, Mul};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
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
    /// Default (zero) in the same variant as `self`.
    pub fn default(token_type: TokenType) -> DripAmount {
        match token_type {
            TokenType::Native | TokenType::Erc20(_) => DripAmount::Token(TokenAmount::default()),
            TokenType::Datacap => DripAmount::Storage(StoragePower::default()),
        }
    }
}

impl Add<&DripAmount> for DripAmount {
    type Output = DripAmount;

    fn add(self, rhs: &DripAmount) -> DripAmount {
        match (&self, rhs) {
            (DripAmount::Token(x), DripAmount::Token(y)) => DripAmount::Token(x + y),
            (DripAmount::Storage(x), DripAmount::Storage(y)) => DripAmount::Storage(x + y),
            _ => panic!("DripAmount variant mismatch"),
        }
    }
}

impl AddAssign<&DripAmount> for DripAmount {
    fn add_assign(&mut self, rhs: &DripAmount) {
        *self = self.clone().add(rhs);
    }
}

impl Div<&DripAmount> for DripAmount {
    type Output = DripAmount;

    fn div(self, rhs: &DripAmount) -> DripAmount {
        match (&self, rhs) {
            (DripAmount::Token(x), DripAmount::Token(y)) => {
                DripAmount::Token(TokenAmount::from_atto(x.atto() / y.atto()))
            }
            (DripAmount::Storage(x), DripAmount::Storage(y)) => DripAmount::Storage(x / y),
            _ => panic!("DripAmount variant mismatch"),
        }
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
