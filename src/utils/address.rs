use anyhow::{bail, ensure};
use fvm_shared::address::{Address, DelegatedAddress, Network, Protocol};
use fvm_shared::ActorID;
use leptos::logging::error;

// '0x' + 20bytes
const ETH_ADDRESS_LENGTH: usize = 42;
const EAM_NAMESPACE: ActorID = 10;

fn is_valid_prefix(s: &str, n: Network) -> bool {
    if s.len() < 2 {
        return false;
    }

    match n {
        Network::Mainnet => s.starts_with("f") || s.starts_with("0x"),
        Network::Testnet => s.starts_with("t") || s.starts_with("0x"),
    }
}

pub fn parse_address(raw: &str, n: Network) -> anyhow::Result<Address> {
    let s = raw.trim().to_lowercase();

    ensure!(is_valid_prefix(&s, n), "Not a valid {:?} address", n);

    if s.len() > 2 && s.starts_with("0x") {
        // Expecting an eth address, perform further validation
        ensure!(
            s.len() == ETH_ADDRESS_LENGTH,
            "Expected address length {}, got {}",
            ETH_ADDRESS_LENGTH,
            s.len(),
        );
        ensure!(
            s.chars().skip(2).all(|c| c.is_ascii_hexdigit()),
            "Invalid characters in address"
        );

        let addr = hex::decode(&s[2..])?;
        Ok(Address::new_delegated(EAM_NAMESPACE, &addr)?)
    } else {
        Ok(n.parse_address(&s)?)
    }
}

/// Extensions around [`fvm_shared::address::Address`] to convert it into an
/// Ethereum address, usable by the `alloy` crate.
pub trait AddressAlloyExt {
    /// Converts the FVM address into an `alloy`-compatible Ethereum address. This is possible only
    /// for ID and Delegated addresses.
    fn into_eth_address(self) -> anyhow::Result<alloy::primitives::Address>;
    /// Converts an ActorID into an `alloy`-compatible Ethereum address. The implementation is
    fn from_actor_id(id: ActorID) -> anyhow::Result<alloy::primitives::Address>;
}

// Implementation is mostly taken from Forest. See
// [here](https://github.com/ChainSafe/forest/blob/ddcdfbfd93dc21fa61544f80222c2ede6f1ee21a/src/rpc/methods/eth/types.rs).
// TODO: migrate tests if any
impl AddressAlloyExt for Address {
    fn into_eth_address(self) -> anyhow::Result<alloy::primitives::Address> {
        match self.protocol() {
            Protocol::ID => Self::from_actor_id(self.id()?),
            Protocol::Delegated => {
                let payload = self.payload();
                let result: Result<DelegatedAddress, _> = payload.try_into();
                if let Ok(f4_addr) = result {
                    let namespace = f4_addr.namespace();
                    ensure!(
                        namespace == EAM_NAMESPACE,
                        "Invalid address namespace {namespace} != {EAM_NAMESPACE}"
                    );
                    return Ok(alloy::primitives::Address::from_slice(f4_addr.subaddress()));
                }
                bail!("invalid delegated address namespace in: {self}")
            }
            _ => {
                error!("Cannot convert address {self} to Ethereum address. Only ID and Delegated addresses are supported.");
                bail!("invalid address {self}");
            }
        }
    }

    fn from_actor_id(id: ActorID) -> anyhow::Result<alloy::primitives::Address> {
        static MASKED_ID_PREFIX: [u8; 12] = [0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let pfx = MASKED_ID_PREFIX;
        let arr = id.to_be_bytes();
        let payload = [
            pfx[0], pfx[1], pfx[2], pfx[3], pfx[4], pfx[5], pfx[6], pfx[7], //
            pfx[8], pfx[9], pfx[10], pfx[11], //
            arr[0], arr[1], arr[2], arr[3], arr[4], arr[5], arr[6], arr[7],
        ];

        Ok(alloy::primitives::Address::from_slice(&payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Whenever we change the network in tests, we need to fork the test to avoid
    // changing the network for other tests. This is because the network is a global
    // variable. This is not a problem when run with `cargo nextest` because each test
    // is run separately.
    use fvm_shared::address::set_current_network;
    use rusty_fork::rusty_fork_test;

    #[test]
    fn test_check_address_prefix() {
        // Valid cases
        assert!(is_valid_prefix("f123...", Network::Mainnet));
        assert!(is_valid_prefix("0x123...", Network::Mainnet));
        assert!(is_valid_prefix("t456...", Network::Testnet));
        assert!(is_valid_prefix("0x789...", Network::Testnet));

        // Wrong network
        assert!(!is_valid_prefix("f123...", Network::Testnet));
        assert!(!is_valid_prefix("t456...", Network::Mainnet));

        // Bad length
        assert!(!is_valid_prefix("f", Network::Mainnet));
        assert!(!is_valid_prefix("t", Network::Testnet));
        assert!(!is_valid_prefix("", Network::Mainnet)); // Empty string
        assert!(!is_valid_prefix("abc", Network::Mainnet)); // Short address

        // Invalid prefixes
        assert!(!is_valid_prefix("g123...", Network::Mainnet));
        assert!(!is_valid_prefix("h456...", Network::Testnet));
        assert!(!is_valid_prefix("123...", Network::Mainnet));
        assert!(!is_valid_prefix("456...", Network::Testnet));
    }

    rusty_fork_test! {
    #[test]
    fn test_parse_mainnet_address() {
        let addr_str = "f1alg2sxw32ns3ech2w7r3dmp2gl2fputkl7x7jta";
        let addr = parse_address(addr_str, Network::Mainnet).unwrap();

        set_current_network(Network::Mainnet); // Required to correctly stringify address
        assert_eq!(addr.to_string(), addr_str);
    }
    }

    rusty_fork_test! {
    #[test]
    fn test_parse_testnet_address() {
        let addr_str = "t410f2oekwcmo2pueydmaq53eic2i62crtbeyuzx2gmy";
        let addr = parse_address(addr_str, Network::Testnet).unwrap();

        set_current_network(Network::Testnet); // Required to correctly stringify address
        assert_eq!(addr.to_string(), addr_str);
    }
    }

    #[test]
    fn test_parse_wrong_network() {
        let m_addr_str = "f1alg2sxw32ns3ech2w7r3dmp2gl2fputkl7x7jta";
        let err = parse_address(m_addr_str, Network::Testnet).unwrap_err();
        assert_eq!(err.to_string(), "Not a valid Testnet address");

        let t_addr_str = "t410f2oekwcmo2pueydmaq53eic2i62crtbeyuzx2gmy";
        let err = parse_address(t_addr_str, Network::Mainnet).unwrap_err();
        assert_eq!(err.to_string(), "Not a valid Mainnet address");
    }

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

        assert_eq!(
            e.to_string(),
            format!(
                "Expected address length {}, got {}",
                ETH_ADDRESS_LENGTH,
                addr_str.len()
            )
        );
    }

    #[test]
    fn test_parse_eth_address_too_long() {
        let addr_str = "0xd388ab098ed3e84c0d808776440b48f68519849812";
        let e = parse_address(addr_str, Network::Mainnet).err().unwrap();

        assert_eq!(
            e.to_string(),
            format!(
                "Expected address length {}, got {}",
                ETH_ADDRESS_LENGTH,
                addr_str.len()
            )
        );
    }

    #[test]
    fn test_parse_eth_address_invalid_chars() {
        let addr_str = "0xd3!8ab098ed3e84c0d808776440b48f685198498";
        let e = parse_address(addr_str, Network::Mainnet).err().unwrap();

        assert_eq!(e.to_string(), "Invalid characters in address");
    }
}
