use alloy::primitives::address;
use fvm_shared::{address::Network, econ::TokenAmount};
use serde::{Deserialize, Serialize};
use std::{str::FromStr as _, sync::LazyLock};
use strum::EnumString;

/// The amount of calibnet FIL to be dripped to the user.
static CALIBNET_DRIP_AMOUNT: LazyLock<TokenAmount> = LazyLock::new(|| TokenAmount::from_whole(5));

/// The amount of mainnet FIL to be dripped to the user. This corresponds to 0.01 FIL.
static MAINNET_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_nano(10_000_000));

/// The amount of calibnet `USDFC` to be dripped to the user.
static CALIBNET_USDFC_DRIP_AMOUNT: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_whole(5));

/// Multiplier to determine the maximum amount of tokens that can be dripped per wallet every [`FaucetInfo::reset_limiter_seconds`].
const MAINNET_PER_WALLET_DRIP_MULTIPLIER: i64 = 1;
const CALIBNET_PER_WALLET_DRIP_MULTIPLIER: i64 = 2;

/// Multiplier used to determine the maximum amount of tokens that can be dripped globally every [`FaucetInfo::reset_limiter_seconds`].
const MAINNET_GLOBAL_DRIP_MULTIPLIER: i64 = 2;
const CALIBNET_GLOBAL_DRIP_MULTIPLIER: i64 = 200;

/// Cool-down duration in seconds between faucet requests on mainnet.
const MAINNET_COOLDOWN_SECONDS: i64 = 600; // 10 minutes
/// Cool-down duration in seconds between faucet requests on calibnet.
const CALIBNET_COOLDOWN_SECONDS: i64 = 60; // 1 minute
/// Time in seconds after which the wallet drip cap resets.
const DRIP_CAP_RESET_SECONDS: i64 = 86400; // 24 hours

/// Maximum gas limit for mainnet including buffer.
const MAX_MAINNET_GAS_LIMIT: u64 = 10_000_000;
/// Maximum gas limit for calibnet including buffer.
const MAX_CALIBNET_GAS_LIMIT: u64 = 30_000_000;

/// Maximum gas fee cap for mainnet including buffer.
static MAX_MAINNET_GAS_FEE_CAP: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_atto(100_000_000));
/// Maximum gas fee cap for calibnet including buffer.
static MAX_CALIBNET_GAS_FEE_CAP: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_atto(200_000));

/// Maximum gas premium for mainnet including buffer.
static MAX_MAINNET_GAS_PREMIUM: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_atto(1_000_000));
/// Maximum gas premium for calibnet including buffer.
static MAX_CALIBNET_GAS_PREMIUM: LazyLock<TokenAmount> =
    LazyLock::new(|| TokenAmount::from_atto(200_000));

pub type ContractAddress = alloy::primitives::Address;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    /// Filecoin native token
    Native,
    /// ERC-20 token, e.g., `USDFC`
    Erc20(ContractAddress),
}

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
            FaucetInfo::MainnetFIL => MAINNET_COOLDOWN_SECONDS,
            FaucetInfo::CalibnetFIL | FaucetInfo::CalibnetUSDFC => CALIBNET_COOLDOWN_SECONDS,
        }
    }

    /// Returns the maximum amount of tokens that can be dripped by the wallet per [`FaucetInfo::reset_limiter_seconds`].
    /// This is used to prevent the wallet from being drained completely and to ensure that the
    /// faucet can continue to operate.
    pub fn drip_cap(&self) -> TokenAmount {
        match self {
            FaucetInfo::MainnetFIL => self.drip_amount() * MAINNET_GLOBAL_DRIP_MULTIPLIER,
            FaucetInfo::CalibnetFIL => self.drip_amount() * CALIBNET_GLOBAL_DRIP_MULTIPLIER,
            FaucetInfo::CalibnetUSDFC => self.drip_amount() * CALIBNET_GLOBAL_DRIP_MULTIPLIER,
        }
    }

    /// Returns the maximum amount of tokens that can be claimed by the wallet per [`FaucetInfo::reset_limiter_seconds`].
    /// This is used to prevent the wallet from being drained completely and to ensure that the
    /// faucet can continue to operate.
    pub fn wallet_cap(&self) -> TokenAmount {
        match self {
            FaucetInfo::MainnetFIL => self.drip_amount() * MAINNET_PER_WALLET_DRIP_MULTIPLIER,
            FaucetInfo::CalibnetFIL => self.drip_amount() * CALIBNET_PER_WALLET_DRIP_MULTIPLIER,
            FaucetInfo::CalibnetUSDFC => self.drip_amount() * CALIBNET_PER_WALLET_DRIP_MULTIPLIER,
        }
    }

    /// Returns the number of seconds after which the all drip cap resets for the faucet.
    pub fn reset_limiter_seconds(&self) -> i64 {
        DRIP_CAP_RESET_SECONDS
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
                option_env!("FAUCET_TX_URL_CALIBNET").and_then(|url| url::Url::parse(url).ok())
                //None // USDFC does not have a transaction base URL for now - to investigate later.
            }
        }
    }

    /// Returns the type of token for the given faucet. This is used to determine how the token
    /// is represented in the interface and how it is handled in the backend.
    pub fn token_type(&self) -> TokenType {
        match self {
            FaucetInfo::MainnetFIL | FaucetInfo::CalibnetFIL => TokenType::Native,
            FaucetInfo::CalibnetUSDFC => TokenType::Erc20(
                option_env!("CALIBNET_USDFC_CONTRACT_ADDRESS")
                    .and_then(|addr| alloy::primitives::Address::from_str(addr).ok())
                    // Default, as present in: https://stg.usdfc.net/#/
                    .unwrap_or_else(|| address!("0xb3042734b608a1B16e9e86B374A3f3e389B4cDf0")),
            ),
        }
    }

    /// Returns the Ethereum chain ID for the given network. We could query the provider for this,
    /// but since we know the chain ID for the networks we support, we can just return it directly
    /// and avoid the overhead of a network request.
    #[cfg(any(test, feature = "ssr"))]
    pub fn chain_id(&self) -> u64 {
        match self.network() {
            Network::Mainnet => 314,    // https://chainlist.org/chain/314
            Network::Testnet => 314159, // https://chainlist.org/chain/314159
        }
    }

    /// Returns the maximum allowed gas limit.
    #[allow(dead_code)]
    pub fn max_gas_limit(&self) -> u64 {
        match self {
            FaucetInfo::MainnetFIL => MAX_MAINNET_GAS_LIMIT,
            FaucetInfo::CalibnetFIL | FaucetInfo::CalibnetUSDFC => MAX_CALIBNET_GAS_LIMIT,
        }
    }

    /// Returns the maximum allowed gas fee cap (in attoFIL).
    #[allow(dead_code)]
    pub fn max_gas_fee_cap(&self) -> TokenAmount {
        match self {
            FaucetInfo::MainnetFIL => MAX_MAINNET_GAS_FEE_CAP.clone(),
            FaucetInfo::CalibnetFIL | FaucetInfo::CalibnetUSDFC => MAX_CALIBNET_GAS_FEE_CAP.clone(),
        }
    }

    /// Returns the maximum allowed gas premium (in attoFIL).
    #[allow(dead_code)]
    pub fn max_gas_premium(&self) -> TokenAmount {
        match self {
            FaucetInfo::MainnetFIL => MAX_MAINNET_GAS_PREMIUM.clone(),
            FaucetInfo::CalibnetFIL | FaucetInfo::CalibnetUSDFC => MAX_CALIBNET_GAS_PREMIUM.clone(),
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
        assert_eq!(mainnet_faucet.token_type(), TokenType::Native);
        assert_eq!(mainnet_faucet.chain_id(), 314);
        assert_eq!(mainnet_faucet.max_gas_limit(), 10_000_000);
        assert_eq!(
            mainnet_faucet.max_gas_fee_cap(),
            TokenAmount::from_atto(1_000_000)
        );
        assert_eq!(
            mainnet_faucet.max_gas_premium(),
            TokenAmount::from_atto(1_000_000)
        );
        assert_eq!(
            mainnet_faucet.wallet_cap(),
            MAINNET_PER_WALLET_DRIP_MULTIPLIER * &*MAINNET_DRIP_AMOUNT
        );
        assert_eq!(
            mainnet_faucet.drip_cap(),
            MAINNET_GLOBAL_DRIP_MULTIPLIER * &*MAINNET_DRIP_AMOUNT
        );

        let calibnet_fil_faucet = FaucetInfo::CalibnetFIL;
        assert_eq!(calibnet_fil_faucet.drip_amount(), &*CALIBNET_DRIP_AMOUNT);
        assert_eq!(calibnet_fil_faucet.rate_limit_seconds(), 60);
        assert_eq!(calibnet_fil_faucet.unit(), "tFIL");
        assert_eq!(calibnet_fil_faucet.network(), Network::Testnet);
        assert_eq!(calibnet_fil_faucet.secret_key_name(), "SECRET_WALLET");
        assert!(calibnet_fil_faucet.transaction_base_url().is_none());
        assert_eq!(calibnet_fil_faucet.token_type(), TokenType::Native);
        assert_eq!(calibnet_fil_faucet.chain_id(), 314159);
        assert_eq!(calibnet_fil_faucet.max_gas_limit(), 30_000_000);
        assert_eq!(
            calibnet_fil_faucet.max_gas_fee_cap(),
            TokenAmount::from_atto(200_000)
        );
        assert_eq!(
            calibnet_fil_faucet.max_gas_premium(),
            TokenAmount::from_atto(200_000)
        );
        assert_eq!(
            calibnet_fil_faucet.wallet_cap(),
            CALIBNET_PER_WALLET_DRIP_MULTIPLIER * &*CALIBNET_DRIP_AMOUNT
        );
        assert_eq!(
            calibnet_fil_faucet.drip_cap(),
            CALIBNET_GLOBAL_DRIP_MULTIPLIER * &*CALIBNET_DRIP_AMOUNT
        );

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
        assert_eq!(
            calibnet_usdfc_faucet.token_type(),
            TokenType::Erc20(
                alloy::primitives::Address::from_str("0xb3042734b608a1B16e9e86B374A3f3e389B4cDf0")
                    .unwrap()
            )
        );
        assert_eq!(calibnet_usdfc_faucet.chain_id(), 314159);
        assert_eq!(calibnet_usdfc_faucet.max_gas_limit(), 30_000_000);
        assert_eq!(
            calibnet_usdfc_faucet.max_gas_premium(),
            TokenAmount::from_atto(200_000)
        );
        assert_eq!(
            calibnet_usdfc_faucet.max_gas_fee_cap(),
            TokenAmount::from_atto(200_000)
        );
        assert_eq!(
            calibnet_usdfc_faucet.wallet_cap(),
            CALIBNET_PER_WALLET_DRIP_MULTIPLIER * &*CALIBNET_USDFC_DRIP_AMOUNT
        );
        assert_eq!(
            calibnet_usdfc_faucet.drip_cap(),
            CALIBNET_GLOBAL_DRIP_MULTIPLIER * &*CALIBNET_USDFC_DRIP_AMOUNT
        );
    }
}
