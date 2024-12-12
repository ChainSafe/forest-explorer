use anyhow::bail;
use fvm_shared::address::{Address, Network};
use fvm_shared::ActorID;

// '0x' + 20bytes
const ETH_ADDRESS_LENGTH: usize = 42;
const EAM_NAMESPACE: ActorID = 10;

fn check_address_prefix(s: &str, n: Network) -> bool {
    if s.len() < 2 {
        return false;
    }

    match n {
        Network::Mainnet => s[0..1].eq("f") || s[0..2].eq("0x"),
        Network::Testnet => s[0..1].eq("t") || s[0..2].eq("0x"),
    }
}

pub fn parse_address(raw: &str, n: Network) -> anyhow::Result<Address> {
    let s = raw.trim().to_lowercase();

    if !check_address_prefix(&s, n) {
        bail!("Not a valid {:?} address", n);
    }

    if s.len() > 2 && s.starts_with("0x") {
        // Expecting an eth address, perform further validation
        if s.len() != ETH_ADDRESS_LENGTH {
            bail!("Invalid address length")
        }

        if !s.chars().skip(2).all(|c| c.is_ascii_hexdigit()) {
            bail!("Invalid characters in address")
        }

        let addr = hex::decode(&s[2..])?;
        Ok(Address::new_delegated(EAM_NAMESPACE, &addr)?)
    } else {
        Ok(n.parse_address(&s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fvm_shared::address::set_current_network;

    #[test]
    fn test_check_address_prefix() {
        // Valid cases
        assert!(check_address_prefix("f123...", Network::Mainnet));
        assert!(check_address_prefix("0x123...", Network::Mainnet));
        assert!(check_address_prefix("t456...", Network::Testnet));
        assert!(check_address_prefix("0x789...", Network::Testnet));

        // Wrong network
        assert!(!check_address_prefix("f123...", Network::Testnet));
        assert!(!check_address_prefix("t456...", Network::Mainnet));

        // Bad length
        assert!(!check_address_prefix("f", Network::Mainnet));
        assert!(!check_address_prefix("t", Network::Testnet));
        assert!(!check_address_prefix("", Network::Mainnet)); // Empty string
        assert!(!check_address_prefix("abc", Network::Mainnet)); // Short address

        // Invalid prefixes
        assert!(!check_address_prefix("g123...", Network::Mainnet));
        assert!(!check_address_prefix("h456...", Network::Testnet));
        assert!(!check_address_prefix("123...", Network::Mainnet));
        assert!(!check_address_prefix("456...", Network::Testnet));
    }

    #[test]
    fn test_parse_mainnet_address() {
        let addr_str = "f1alg2sxw32ns3ech2w7r3dmp2gl2fputkl7x7jta";
        let addr = parse_address(addr_str, Network::Mainnet).unwrap();

        set_current_network(Network::Mainnet); // Required to correctly stringify address
        assert_eq!(addr.to_string(), addr_str);
    }

    #[test]
    fn test_parse_testnet_address() {
        let addr_str = "t410f2oekwcmo2pueydmaq53eic2i62crtbeyuzx2gmy";
        let addr = parse_address(addr_str, Network::Testnet).unwrap();

        set_current_network(Network::Testnet); // Required to correctly stringify address
        assert_eq!(addr.to_string(), addr_str);
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

        assert_eq!(e.to_string(), "Invalid address length");
    }

    #[test]
    fn test_parse_eth_address_too_long() {
        let addr_str = "0xd388ab098ed3e84c0d808776440b48f68519849812";
        let e = parse_address(addr_str, Network::Mainnet).err().unwrap();

        assert_eq!(e.to_string(), "Invalid address length");
    }

    #[test]
    fn test_parse_eth_address_invalid_chars() {
        let addr_str = "0xd3!8ab098ed3e84c0d808776440b48f685198498";
        let e = parse_address(addr_str, Network::Mainnet).err().unwrap();

        assert_eq!(e.to_string(), "Invalid characters in address");
    }
}
