use fvm_ipld_encoding::RawBytes;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::bigint::bigint_ser;
use fvm_shared::message::Message;
use fvm_shared::sector::StoragePower;
use fvm_shared::{METHOD_SEND, address::Address, econ::TokenAmount};

const VERIFIED_REGISTRY_ACTOR: Address = Address::new_id(6);

/// Params for the verified registry's AddVerifiedClient method.
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct AddVerifiedClientParams {
    pub address: Address,
    #[serde(with = "bigint_ser")]
    pub allowance: StoragePower,
}

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

/// Creates a new datacap allocation message with default values for gas and sequence.
pub fn message_grant_datacap(from: Address, params: RawBytes) -> Message {
    message_grant_datacap_native(
        from,
        params,
        0,
        TokenAmount::default(),
        TokenAmount::default(),
        0,
    )
}

/// Creates a new datacap allocation message with specified values for gas and sequence.
pub fn message_grant_datacap_native(
    from: Address,
    params: RawBytes,
    gas_limit: u64,
    gas_fee_cap: TokenAmount,
    gas_premium: TokenAmount,
    sequence: u64,
) -> Message {
    Message {
        from,
        to: VERIFIED_REGISTRY_ACTOR,
        value: TokenAmount::default(),
        method_num: frc42_dispatch::method_hash!("AddVerifiedClient"),
        params,
        gas_limit,
        gas_fee_cap,
        gas_premium,
        sequence,
        version: 0,
    }
}
