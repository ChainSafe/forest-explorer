use fvm_shared::{address::Network, econ::TokenAmount};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strum::EnumString;

/// The amount of mainnet FIL to be dripped to the user. This corresponds to 1 tFIL.
static CALIBNET_DRIP_AMOUNT: LazyLock<TokenAmount> = LazyLock::new(|| TokenAmount::from_whole(1));

/// The amount of mainnet FIL to be dripped to the user. This corresponds to 0.01 FIL.
static MAINNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_nano(10_000_000));

/// The amount of calibnet `USDFC` to be dripped to the user. This corresponds to 1 `tUSDFC`.
static CALIBNET_USDFC_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_whole(1));

#[derive(strum::Display, EnumString, Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum FaucetInfo {
    MainnetFIL,
    CalibnetFIL,
    CalibnetUSDFC,
}

impl FaucetInfo {
    /// Return the drip amount for the given faucet in the defined token unit.
    pub fn drip_amount(&self) -> &TokenAmount {
        match self {
            FaucetInfo::MainnetFIL => &MAINNET_DRIP_AMOUNT,
            FaucetInfo::CalibnetFIL => &CALIBNET_DRIP_AMOUNT,
            FaucetInfo::CalibnetUSDFC => &CALIBNET_USDFC_DRIP_AMOUNT,
        }
    }

    /// Returns the rate limit in seconds for the given faucet. The rate limit defines period after
    /// which the faucet is temporarily disabled and no more drips can be sent.
    pub fn rate_limit_seconds(&self) -> i64 {
        match self {
            FaucetInfo::MainnetFIL => 600,
            FaucetInfo::CalibnetFIL => 60,
            FaucetInfo::CalibnetUSDFC => 60,
        }
    }

    /// Returns the unit of the token for the given faucet.
    pub fn unit(&self) -> &str {
        match self {
            FaucetInfo::MainnetFIL => "FIL",
            FaucetInfo::CalibnetFIL => "tFIL",
            FaucetInfo::CalibnetUSDFC => "tUSDFC",
        }
    }

    /// Returns the the secret key label as configured in the CloudFlare Worker secrets.
    #[cfg(any(test, feature = "ssr"))]
    pub fn secret_key_name(&self) -> &str {
        match self {
            FaucetInfo::CalibnetFIL => "SECRET_WALLET",
            FaucetInfo::MainnetFIL => "SECRET_MAINNET_WALLET",
            FaucetInfo::CalibnetUSDFC => "SECRET_CALIBNET_USDFC_WALLET",
        }
    }

    /// Returns the network type for the given faucet. Note that there might be multiple faucets on
    /// a given network, e.g., for ERC-20 tokens.
    pub fn network(&self) -> Network {
        match self {
            FaucetInfo::MainnetFIL => Network::Mainnet,
            FaucetInfo::CalibnetFIL | FaucetInfo::CalibnetUSDFC => Network::Testnet,
        }
    }

    /// Returns the base URL for transactions on the given faucet. This is used to link to
    /// transaction details in the block explorer.
    pub fn transaction_base_url(&self) -> Option<url::Url> {
        match self {
            FaucetInfo::MainnetFIL => {
                option_env!("FAUCET_TX_URL_MAINNET").and_then(|url| url::Url::parse(url).ok())
            }
            FaucetInfo::CalibnetFIL => {
                option_env!("FAUCET_TX_URL_CALIBNET").and_then(|url| url::Url::parse(url).ok())
            }
            FaucetInfo::CalibnetUSDFC => {
                None // USDFC does not have a transaction base URL for now - to investigate later.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faucet_info() {
        // these tests are not exactly useful, but they give coverage and ensure that some warts with
        // lazily initializing constants are caught.
        let mainnet_faucet = FaucetInfo::MainnetFIL;
        assert_eq!(mainnet_faucet.drip_amount(), &*MAINNET_DRIP_AMOUNT);
        assert_eq!(mainnet_faucet.rate_limit_seconds(), 600);
        assert_eq!(mainnet_faucet.unit(), "FIL");
        assert_eq!(mainnet_faucet.network(), Network::Mainnet);
        assert_eq!(mainnet_faucet.secret_key_name(), "SECRET_MAINNET_WALLET");
        assert!(mainnet_faucet.transaction_base_url().is_none());

        let calibnet_fil_faucet = FaucetInfo::CalibnetFIL;
        assert_eq!(calibnet_fil_faucet.drip_amount(), &*CALIBNET_DRIP_AMOUNT);
        assert_eq!(calibnet_fil_faucet.rate_limit_seconds(), 60);
        assert_eq!(calibnet_fil_faucet.unit(), "tFIL");
        assert_eq!(calibnet_fil_faucet.network(), Network::Testnet);
        assert_eq!(calibnet_fil_faucet.secret_key_name(), "SECRET_WALLET");
        assert!(calibnet_fil_faucet.transaction_base_url().is_none());

        let calibnet_usdfc_faucet = FaucetInfo::CalibnetUSDFC;
        assert_eq!(
            calibnet_usdfc_faucet.drip_amount(),
            &*CALIBNET_USDFC_DRIP_AMOUNT
        );
        assert_eq!(calibnet_usdfc_faucet.rate_limit_seconds(), 60);
        assert_eq!(calibnet_usdfc_faucet.unit(), "tUSDFC");
        assert_eq!(calibnet_usdfc_faucet.network(), Network::Testnet);
        assert_eq!(
            calibnet_usdfc_faucet.secret_key_name(),
            "SECRET_CALIBNET_USDFC_WALLET"
        );
        assert!(calibnet_usdfc_faucet.transaction_base_url().is_none());
    }
}
