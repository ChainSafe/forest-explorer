// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct CidLotusJson {
    #[serde(rename = "/", with = "crate::utils::lotus_json::stringify")]
    slash: ::cid::Cid,
}

impl HasLotusJson for ::cid::Cid {
    type LotusJson = CidLotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        Self::LotusJson { slash: self }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        let Self::LotusJson { slash } = lotus_json;
        slash
    }
}
