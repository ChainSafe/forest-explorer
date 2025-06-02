use std::fmt::Display;

/// Represents a transaction ID that can be either a native CID or an Ethereum transaction hash.
/// Used for a unified handling of transactions across different blockchain types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
