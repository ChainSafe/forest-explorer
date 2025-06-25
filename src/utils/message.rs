use fvm_ipld_encoding::RawBytes;
use fvm_shared::message::Message;
use fvm_shared::{address::Address, econ::TokenAmount, METHOD_SEND};

/// Creates a new transfer message with default values for gas and sequence.
pub fn message_transfer(from: Address, to: Address, value: TokenAmount) -> Message {
    message_transfer_native(
        from,
        to,
        value,
        0,
        TokenAmount::from_atto(0),
        TokenAmount::from_atto(0),
        0,
    )
}

/// Creates a new transfer message with specified values for gas and sequence.
pub fn message_transfer_native(
    from: Address,
    to: Address,
    value: TokenAmount,
    gas_limit: u64,
    gas_fee_cap: TokenAmount,
    gas_premium: TokenAmount,
    sequence: u64,
) -> Message {
    Message {
        from,
        to,
        value,
        method_num: METHOD_SEND,
        params: RawBytes::new(vec![]),
        gas_limit,
        gas_fee_cap,
        gas_premium,
        version: 0,
        sequence,
    }
}
