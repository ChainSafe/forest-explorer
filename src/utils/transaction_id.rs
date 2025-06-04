use std::fmt::Display;

/// Represents a transaction ID that can be either a native CID or an Ethereum transaction hash.
/// Used for a unified handling of transactions across different blockchain types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionId {
    Native(cid::Cid),
    Eth(alloy::primitives::TxHash),
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionId::Native(cid) => write!(f, "{}", cid),
            TransactionId::Eth(tx_hash) => write!(f, "{}", tx_hash),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::primitives::TxHash;
    use cid::Cid;

    #[test]
    fn test_display_native() {
        let cid = Cid::from_str("bafy2bzaceawvht75twtb7jbw262yi5am5oiu5jpzfxndkiyormggdyljxrr6e")
            .unwrap();
        let transaction_id = TransactionId::Native(cid);
        assert_eq!(
            transaction_id.to_string(),
            "bafy2bzaceawvht75twtb7jbw262yi5am5oiu5jpzfxndkiyormggdyljxrr6e"
        );
    }

    #[test]
    fn test_display_eth() {
        let tx_hash = TxHash::from([1; 32]);
        let transaction_id = TransactionId::Eth(tx_hash);
        assert_eq!(
            transaction_id.to_string(),
            "0x0101010101010101010101010101010101010101010101010101010101010101"
        );
    }
}
