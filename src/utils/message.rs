// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::RawBytes;
use fvm_shared::message::Message;
use fvm_shared::{address::Address, econ::TokenAmount, METHOD_SEND};

pub fn message_transfer(from: Address, to: Address, value: TokenAmount) -> Message {
    Message {
        from,
        to,
        value,
        method_num: METHOD_SEND,
        params: RawBytes::new(vec![]),
        gas_limit: 0,
        gas_fee_cap: TokenAmount::from_atto(0),
        gas_premium: TokenAmount::from_atto(0),
        version: 0,
        sequence: 0,
    }
}
