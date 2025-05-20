// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{address::Address, econ::TokenAmount, message::Message};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageLotusJson {
    #[serde(default)]
    version: u64,
    #[serde(with = "crate::lotus_json")]
    to: Address,
    #[serde(with = "crate::lotus_json")]
    from: Address,
    #[serde(default)]
    nonce: u64,
    #[serde(with = "crate::lotus_json", default)]
    value: TokenAmount,
    #[serde(default)]
    gas_limit: u64,
    #[serde(with = "crate::lotus_json", default)]
    gas_fee_cap: TokenAmount,
    #[serde(with = "crate::lotus_json", default)]
    gas_premium: TokenAmount,
    #[serde(default)]
    method: u64,
    #[serde(
        with = "crate::lotus_json",
        skip_serializing_if = "Option::is_none",
        default
    )]
    params: Option<RawBytes>,
}

impl HasLotusJson for Message {
    type LotusJson = MessageLotusJson;

    fn into_lotus_json(self) -> Self::LotusJson {
        let Self {
            version,
            from,
            to,
            sequence,
            value,
            method_num,
            params,
            gas_limit,
            gas_fee_cap,
            gas_premium,
        } = self;
        Self::LotusJson {
            version,
            to,
            from,
            nonce: sequence,
            value,
            gas_limit,
            gas_fee_cap,
            gas_premium,
            method: method_num,
            params: Some(params),
        }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        let Self::LotusJson {
            version,
            to,
            from,
            nonce,
            value,
            gas_limit,
            gas_fee_cap,
            gas_premium,
            method,
            params,
        } = lotus_json;
        Self {
            version,
            from,
            to,
            sequence: nonce,
            value,
            method_num: method,
            params: params.unwrap_or_default(),
            gas_limit,
            gas_fee_cap,
            gas_premium,
        }
    }
}
