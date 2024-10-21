use anyhow::{Context as _, Result};
use bls_signatures::{
    verify_messages, PrivateKey as BlsPrivate, PublicKey as BlsPubKey, Serialize as _,
    Signature as BlsSignature,
};
use libsecp256k1::{
    recover, Message as SecpMessage, PublicKey as SecpPublic, RecoveryId, SecretKey as SecpPrivate,
    Signature as EcsdaSignature,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};

use fvm_shared::{
    address::{Address, Protocol},
    crypto::signature::{Signature, SignatureType, SECP_SIG_LEN, SECP_SIG_MESSAGE_HASH_SIZE},
};

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

    use base64::engine::{general_purpose::STANDARD, Engine as _};

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

/// Generates BLAKE2b hash of fixed 32 bytes size.
///
/// # Example
/// ```
/// # use forest_filecoin::doctest_private::blake2b_256;
///
/// let ingest: Vec<u8> = vec![];
/// let hash = blake2b_256(&ingest);
/// assert_eq!(hash.len(), 32);
/// ```
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

/// Sign takes in `SignatureType`, private key and message. Returns a Signature
/// for that message
pub fn sign(sig_type: SignatureType, private_key: &[u8], msg: &[u8]) -> Result<Signature> {
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

pub fn verify(signature: &str, address: &str, msg: &str) -> Result<bool> {
    let sig_bytes = hex::decode(signature).context("Signature has to be a hex string")?;
    log::info!("Signature length: {} bytes", sig_bytes.len());
    let address = Address::from_str(address)?;
    log::info!("Address: {:?}", address);
    let signature = match address.protocol() {
        Protocol::Secp256k1 => Signature::new_secp256k1(sig_bytes),
        Protocol::BLS => Signature::new_bls(sig_bytes),
        _ => anyhow::bail!("Invalid signature (must be bls or secp256k1)"),
    };
    log::info!("Signature: {:?}", signature);
    let msg = hex::decode(msg).unwrap_or(msg.as_bytes().to_vec());
    log::info!("Message: {:?}", msg);

    Ok(match signature.sig_type {
        SignatureType::BLS => verify_bls_sig(&signature.bytes, &msg, &address),
        SignatureType::Secp256k1 => verify_secp256k1_sig(&signature.bytes, &msg, &address),
    }
    .is_ok())
}

fn verify_bls_sig(signature: &[u8], data: &[u8], addr: &Address) -> Result<(), String> {
    if addr.protocol() != Protocol::BLS {
        return Err(format!(
            "cannot validate a BLS signature against a {} address",
            addr.protocol()
        ));
    }

    let pub_k = addr.payload_bytes();

    // generate public key object from bytes
    let pk = BlsPubKey::from_bytes(&pub_k).map_err(|e| e.to_string())?;

    // generate signature struct from bytes
    let sig = BlsSignature::from_bytes(signature).map_err(|e| e.to_string())?;

    // BLS verify hash against key
    if verify_messages(&sig, &[data], &[pk]) {
        Ok(())
    } else {
        Err(format!(
            "bls signature verification failed for addr: {}",
            addr
        ))
    }
}

/// Returns `String` error if a secp256k1 signature is invalid.
fn verify_secp256k1_sig(signature: &[u8], data: &[u8], addr: &Address) -> Result<(), String> {
    if addr.protocol() != Protocol::Secp256k1 {
        return Err(format!(
            "cannot validate a secp256k1 signature against a {} address",
            addr.protocol()
        ));
    }

    if signature.len() != SECP_SIG_LEN {
        return Err(format!(
            "Invalid Secp256k1 signature length. Was {}, must be 65",
            signature.len()
        ));
    }

    // blake2b 256 hash
    let hash = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(data)
        .finalize();

    // Ecrecover with hash and signature
    let mut sig = [0u8; SECP_SIG_LEN];
    sig[..].copy_from_slice(signature);
    let rec_addr = ecrecover(hash.as_bytes().try_into().expect("fixed array size"), &sig)
        .map_err(|e| e.to_string())?;

    // check address against recovered address
    if &rec_addr == addr {
        Ok(())
    } else {
        Err("Secp signature verification failed".to_owned())
    }
}

/// Return the public key used for signing a message given it's signing bytes hash and signature.
fn recover_secp_public_key(
    hash: &[u8; SECP_SIG_MESSAGE_HASH_SIZE],
    signature: &[u8; SECP_SIG_LEN],
) -> Result<SecpPublic> {
    // generate types to recover key from
    let rec_id = RecoveryId::parse(signature[64])?;
    let message = SecpMessage::parse(hash);

    // Signature value without recovery byte
    let mut s = [0u8; 64];
    s.clone_from_slice(signature[..64].as_ref());

    // generate Signature
    let sig = EcsdaSignature::parse_standard(&s)?;
    Ok(recover(&message, &sig, &rec_id)?)
}

/// Return Address for a message given it's signing bytes hash and signature.
fn ecrecover(hash: &[u8; 32], signature: &[u8; SECP_SIG_LEN]) -> Result<Address> {
    // recover public key from a message hash and secp signature.
    let key = recover_secp_public_key(hash, signature)?;
    let ret = key.serialize();
    let addr = Address::new_secp256k1(&ret)?;
    Ok(addr)
}

pub fn secret_key() -> Key {
    let key_info = KeyInfo::from_str("7b2254797065223a312c22507269766174654b6579223a22446e72546a6d6e6e367468724a7365596a58482b6b6855313052647047513558392b64306e784f575148633d227d").unwrap();
    Key::try_from(key_info).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_info_from_str() {
        let key_info = KeyInfo::from_str("7b2254797065223a312c22507269766174654b6579223a22446e72546a6d6e6e367468724a7365596a58482b6b6855313052647047513558392b64306e784f575148633d227d").unwrap();
        assert_eq!(key_info.r#type, SignatureType::Secp256k1);
    }
}
