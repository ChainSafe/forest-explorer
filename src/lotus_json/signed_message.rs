// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::message::{Message, SignedMessage};
use ::cid::Cid;
use fvm_shared::crypto::signature::Signature;

use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignedMessageLotusJson {
    #[serde(with = "crate::lotus_json")]
    message: Message,
    #[serde(with = "crate::lotus_json")]
    signature: Signature,
    #[serde(
        with = "crate::lotus_json",
        rename = "CID",
        skip_serializing_if = "Option::is_none",
        default
    )]
    cid: Option<Cid>,
}

impl HasLotusJson for SignedMessage {
    type LotusJson = SignedMessageLotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        let cid = Some(self.cid());
        let Self { message, signature } = self;
        Self::LotusJson {
            message,
            signature,
            cid,
        }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        let Self::LotusJson {
            message,
            signature,
            cid: _ignored, // See notes on Message
        } = lotus_json;
        Self { message, signature }
    }
}
