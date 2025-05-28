use super::HasLotusJson;
use ::cid::Cid;
use fvm_ipld_encoding::Error;
use fvm_shared::crypto::signature::Signature;
use fvm_shared::crypto::signature::SignatureType;
use fvm_shared::message::Message;
use multihash_codetable::{Code, MultihashDigest as _};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct SignedMessage {
    pub message: Message,
    pub signature: Signature,
}

fn from_cbor_blake2b256<S: serde::ser::Serialize>(obj: &S) -> Result<Cid, Error> {
    let bytes = fvm_ipld_encoding::to_vec(obj)?;
    Ok(Cid::new_v1(
        fvm_ipld_encoding::DAG_CBOR,
        Code::Blake2b256.digest(&bytes),
    ))
}
pub fn message_cid(msg: &Message) -> Cid {
    from_cbor_blake2b256(msg).expect("message serialization is infallible")
}

impl SignedMessage {
    /// Checks if the signed message is a BLS message.
    pub fn is_bls(&self) -> bool {
        self.signature.signature_type() == SignatureType::BLS
    }

    // Important note: `msg.cid()` is different from
    // `Cid::from_cbor_blake2b256(msg)`. The behavior comes from Lotus, and
    // Lotus, by, definition, is correct.
    pub fn cid(&self) -> Cid {
        if self.is_bls() {
            message_cid(&self.message)
        } else {
            from_cbor_blake2b256(self).expect("message serialization is infallible")
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignedMessageLotusJson {
    #[serde(with = "crate::utils::lotus_json")]
    message: Message,
    #[serde(with = "crate::utils::lotus_json")]
    signature: Signature,
    #[serde(
        with = "crate::utils::lotus_json",
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
