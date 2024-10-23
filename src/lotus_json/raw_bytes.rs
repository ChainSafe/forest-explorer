// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::{vec_u8::VecU8LotusJson, *};
use fvm_ipld_encoding::RawBytes;

impl HasLotusJson for RawBytes {
    type LotusJson = VecU8LotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        Vec::from(self).into_lotus_json()
    }

    fn from_lotus_json(value: Self::LotusJson) -> Self {
        Self::from(Vec::from_lotus_json(value))
    }
}
