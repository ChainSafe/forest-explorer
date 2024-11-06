use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use fvm_ipld_encoding::Error;
use fvm_ipld_encoding::RawBytes;
pub use fvm_shared::message::Message;
use fvm_shared::{
    address::Address,
    crypto::signature::{Signature, SignatureType},
    econ::TokenAmount,
    METHOD_SEND,
};
use serde::{Deserialize, Serialize};

fn from_cbor_blake2b256<S: serde::ser::Serialize>(obj: &S) -> Result<Cid, Error> {
    let bytes = fvm_ipld_encoding::to_vec(obj)?;
    Ok(Cid::new_v1(
        fvm_ipld_encoding::DAG_CBOR,
        Code::Blake2b256.digest(&bytes),
    ))
}

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

pub fn message_cid(msg: &Message) -> cid::Cid {
    from_cbor_blake2b256(msg).expect("message serialization is infallible")
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct SignedMessage {
    pub message: Message,
    pub signature: Signature,
}

impl SignedMessage {
    /// Checks if the signed message is a BLS message.
    pub fn is_bls(&self) -> bool {
        self.signature.signature_type() == SignatureType::BLS
    }

    // Important note: `msg.cid()` is different from
    // `Cid::from_cbor_blake2b256(msg)`. The behavior comes from Lotus, and
    // Lotus, by, definition, is correct.
    pub fn cid(&self) -> cid::Cid {
        if self.is_bls() {
            message_cid(&self.message)
        } else {
            from_cbor_blake2b256(self).expect("message serialization is infallible")
        }
    }
}
