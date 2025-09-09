use anyhow::{Context as _, Result};
use bls_signatures::{PrivateKey as BlsPrivate, Serialize as _};
use libsecp256k1::{PublicKey as SecpPublic, SecretKey as SecpPrivate};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};

use fvm_shared::{address::Address, crypto::signature::SignatureType};

/// Return the public key for a given private key and `SignatureType`
pub fn to_public(sig_type: SignatureType, private_key: &[u8]) -> Result<Vec<u8>> {
    match sig_type {
        SignatureType::BLS => Ok(BlsPrivate::from_bytes(private_key)?.public_key().as_bytes()),
        SignatureType::Secp256k1 => {
            let private_key = SecpPrivate::parse_slice(private_key)?;
            let public_key = SecpPublic::from_secret_key(&private_key);
            Ok(public_key.serialize().to_vec())
        }
    }
}

/// Return a new Address that is of a given `SignatureType` and uses the
/// supplied public key
pub fn new_address(sig_type: SignatureType, public_key: &[u8]) -> Result<Address> {
    match sig_type {
        SignatureType::BLS => Ok(Address::new_bls(public_key)?),
        SignatureType::Secp256k1 => Ok(Address::new_secp256k1(public_key)?),
    }
}

pub mod base64_standard {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use base64::engine::{Engine as _, general_purpose::STANDARD};

    pub fn serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        STANDARD.encode(value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        STANDARD
            .decode(String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyInfo {
    pub r#type: SignatureType,
    #[serde(with = "base64_standard")]
    pub private_key: Vec<u8>,
}

/// A key, this contains a `KeyInfo` and an address
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Key {
    pub key_info: KeyInfo,
    pub address: Address,
}

impl TryFrom<KeyInfo> for Key {
    type Error = anyhow::Error;

    fn try_from(key_info: KeyInfo) -> Result<Self, Self::Error> {
        let public_key = to_public(key_info.r#type, &key_info.private_key)?;
        let address = new_address(key_info.r#type, &public_key)?;
        Ok(Key { key_info, address })
    }
}

impl FromStr for KeyInfo {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded_key = hex::decode(s).context("Key must be hex encoded")?;

        let key_str = std::str::from_utf8(&decoded_key)?;

        let key_info = serde_json::from_str::<KeyInfo>(key_str).context("invalid key format")?;
        Ok(key_info)
    }
}

#[cfg(feature = "ssr")]
/// Generates BLAKE2b hash of fixed 32 bytes size.
pub fn blake2b_256(ingest: &[u8]) -> [u8; 32] {
    use blake2b_simd::Params;

    let digest = Params::new()
        .hash_length(32)
        .to_state()
        .update(ingest)
        .finalize();

    let mut ret = [0u8; 32];
    ret.clone_from_slice(digest.as_bytes());
    ret
}

#[cfg(feature = "ssr")]
/// Sign takes in `SignatureType`, private key and message. Returns a Signature
/// for that message
pub fn sign(
    sig_type: SignatureType,
    private_key: &[u8],
    msg: &[u8],
) -> Result<fvm_shared::crypto::signature::Signature> {
    use fvm_shared::crypto::signature::Signature;
    use libsecp256k1::Message as SecpMessage;
    match sig_type {
        SignatureType::BLS => {
            let priv_key = BlsPrivate::from_bytes(private_key)?;
            // this returns a signature from bls-signatures, so we need to convert this to a
            // crypto signature
            let sig = priv_key.sign(msg);
            let crypto_sig = Signature::new_bls(sig.as_bytes());
            Ok(crypto_sig)
        }
        SignatureType::Secp256k1 => {
            let priv_key = SecpPrivate::parse_slice(private_key)?;
            let msg_hash = blake2b_256(msg);
            let message = SecpMessage::parse(&msg_hash);
            let (sig, recovery_id) = libsecp256k1::sign(&message, &priv_key);
            let mut new_bytes = [0; 65];
            new_bytes[..64].copy_from_slice(&sig.serialize());
            new_bytes[64] = recovery_id.serialize();
            let crypto_sig = Signature::new_secp256k1(new_bytes.to_vec());
            Ok(crypto_sig)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_info_from_str() {
        let key_info = KeyInfo::from_str("7b2254797065223a312c22507269766174654b6579223a2272744f75762f386664316d72535570313970487064645479392b67756e7376656a786e317950356b6869493d227d").unwrap();
        assert_eq!(key_info.r#type, SignatureType::Secp256k1);
    }
}
