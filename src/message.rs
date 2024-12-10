use cid::Cid;
use fvm_ipld_encoding::Error;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Network;
pub use fvm_shared::message::Message;
use fvm_shared::{address, address::Address, crypto::signature::{Signature, SignatureType}, econ::TokenAmount, METHOD_SEND};
use multihash_codetable::{Code, MultihashDigest as _};
use serde::{Deserialize, Serialize};

fn from_cbor_blake2b256<S: serde::ser::Serialize>(obj: &S) -> Result<Cid, Error> {
    let bytes = fvm_ipld_encoding::to_vec(obj)?;
    Ok(Cid::new_v1(
        fvm_ipld_encoding::DAG_CBOR,
        Code::Blake2b256.digest(&bytes),
    ))
}

pub fn parse_address(s: &str, network: Network) -> anyhow::Result<Address> {
    match network {
        Network::Mainnet => {
            if let Ok(addr) = Network::Mainnet.parse_address(s) {
                return Ok(addr)
            }
        }
        Network::Testnet => {
            if let Ok(addr) = Network::Testnet.parse_address(s) {
                return Ok(addr)
            }
        }
    }
    // Try parsing as 0x ethereum address
    if s.len() != 42 {
        return Err(anyhow::Error::from(address::Error::InvalidLength))
    }

    let addr = hex::decode(&s[2..])?;
    Ok(Address::new_delegated(10, &addr)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eth_address_testnet() {
        let addr_str = "0xd388ab098ed3e84c0d808776440b48f685198498";
        let addr = parse_address(addr_str, Network::Testnet).unwrap();

        let exp_addr_str = "t410f2oekwcmo2pueydmaq53eic2i62crtbeyuzx2gmy";
        let exp_addr = parse_address(exp_addr_str, Network::Testnet).unwrap();

        assert_eq!(exp_addr, addr);
    }

    #[test]
    fn test_parse_eth_address_mainnet() {
        let addr_str = "0xd388ab098ed3e84c0d808776440b48f685198498";
        let addr = parse_address(addr_str, Network::Mainnet).unwrap();

        let exp_addr_str = "f410f2oekwcmo2pueydmaq53eic2i62crtbeyuzx2gmy";
        let exp_addr = parse_address(exp_addr_str, Network::Mainnet).unwrap();

        assert_eq!(exp_addr, addr);
    }

    #[test]
    fn test_parse_eth_address_too_short() {
        let addr_str = "0xd3";
        let e = parse_address(addr_str, Network::Mainnet).err().unwrap();
        let addr_err = e.downcast_ref::<address::Error>().unwrap();

        assert_eq!(*addr_err, address::Error::InvalidLength);
    }

    #[test]
    fn test_parse_eth_address_too_long() {
        let addr_str = "0xd388ab098ed3e84c0d808776440b48f68519849812";
        let e = parse_address(addr_str, Network::Mainnet).err().unwrap();
        let addr_err = e.downcast_ref::<address::Error>().unwrap();

        assert_eq!(*addr_err, address::Error::InvalidLength);
    }

    #[test]
    fn test_parse_mainnet_address() {
        let addr_str = "f1alg2sxw32ns3ech2w7r3dmp2gl2fputkl7x7jta";
        let addr = parse_address(addr_str, Network::Mainnet).unwrap();

        assert_eq!(addr.to_string(), addr_str);
    }
}