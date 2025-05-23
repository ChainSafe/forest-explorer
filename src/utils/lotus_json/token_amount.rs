// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)] // name the field for clarity
pub struct TokenAmountLotusJson {
    #[serde(with = "crate::utils::lotus_json")]
    attos: BigInt,
}

impl HasLotusJson for TokenAmount {
    type LotusJson = TokenAmountLotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        Self::LotusJson {
            attos: self.atto().clone(),
        }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        let Self::LotusJson { attos } = lotus_json;
        Self::from_atto(attos)
    }
}
