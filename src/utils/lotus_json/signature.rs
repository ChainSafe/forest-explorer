// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;
use fvm_shared::crypto::signature::{Signature, SignatureType};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureLotusJson {
    #[serde(with = "crate::utils::lotus_json")]
    r#type: SignatureType,
    #[serde(with = "crate::utils::lotus_json")]
    data: Vec<u8>,
}

impl HasLotusJson for Signature {
    type LotusJson = SignatureLotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        let Self { sig_type, bytes } = self;
        Self::LotusJson {
            r#type: sig_type,
            data: bytes,
        }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        let Self::LotusJson { r#type, data } = lotus_json;
        Self {
            sig_type: r#type,
            bytes: data,
        }
    }
}
