use anyhow::{anyhow, Result};
use fvm_shared::econ::TokenAmount;
use url::Url;

/// Formats FIL balance to a human-readable string with two decimal places and a unit.
pub fn format_balance(balance: &TokenAmount, unit: &str) -> String {
    format!(
        "{:.2} {unit}",
        balance.to_string().parse::<f32>().unwrap_or_default(),
    )
}

/// Types of search paths in Filecoin explorer.
#[derive(Copy, Clone)]
pub enum SearchPath {
    Transaction,
    Address,
}

impl SearchPath {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchPath::Transaction => "txs/",
            SearchPath::Address => "address/",
        }
    }
}

/// Constructs a URL combining base URL, search path, and an identifier.
pub fn format_url(base_url: &Url, path: SearchPath, identifier: &str) -> Result<Url> {
    base_url
        .join(path.as_str())?
        .join(identifier)
        .map_err(|e| anyhow!("Failed to join URL: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fvm_shared::econ::TokenAmount;

    #[test]
    fn test_format_balance() {
        let cases = [
            (TokenAmount::from_whole(1), "1.00 FIL"),
            (TokenAmount::from_whole(0), "0.00 FIL"),
            (TokenAmount::from_nano(10e6 as i64), "0.01 FIL"),
            (TokenAmount::from_nano(999_999_999), "1.00 FIL"),
        ];
        for (balance, expected) in cases.iter() {
            assert_eq!(format_balance(balance, "FIL"), *expected);
        }
    }

    #[test]
    fn test_format_url() {
        let base = Url::parse("https://test.com/").unwrap();
        let cases = [
            (
                SearchPath::Transaction,
                "0xdef456",
                "https://test.com/txs/0xdef456",
            ),
            (
                SearchPath::Address,
                "0xabc123",
                "https://test.com/address/0xabc123",
            ),
        ];

        for (path, query, expected) in cases.iter() {
            let result = format_url(&base, *path, query).unwrap();
            assert_eq!(result.as_str(), *expected);
        }
    }
}
