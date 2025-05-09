#[cfg(feature = "ssr")]
use crate::key::{sign, Key};
use crate::{lotus_json::LotusJson, message::SignedMessage};
use anyhow::{anyhow, Result};
#[cfg(feature = "ssr")]
use fvm_shared::address::Network;
use fvm_shared::{address::Address, econ::TokenAmount, message::Message};
use leptos::{prelude::ServerFnError, server};
use url::Url;

#[server]
pub async fn faucet_address(is_mainnet: bool) -> Result<LotusJson<Address>, ServerFnError> {
    let network = if is_mainnet {
        Network::Mainnet
    } else {
        Network::Testnet
    };
    let key = secret_key(network).await?;
    Ok(LotusJson(key.address))
}

#[server]
pub async fn sign_with_secret_key(
    msg: LotusJson<Message>,
    is_mainnet: bool,
) -> Result<LotusJson<SignedMessage>, ServerFnError> {
    use crate::message::message_cid;
    use leptos::server_fn::error;
    use send_wrapper::SendWrapper;
    let LotusJson(msg) = msg;
    let cid = message_cid(&msg);
    let amount_limit = match is_mainnet {
        true => crate::constants::MAINNET_DRIP_AMOUNT.clone(),
        false => crate::constants::CALIBNET_DRIP_AMOUNT.clone(),
    };
    if msg.value > amount_limit {
        return Err(ServerFnError::ServerError(
            "Amount limit exceeded".to_string(),
        ));
    }
    SendWrapper::new(async move {
        use axum::Extension;
        use leptos_axum::extract;
        use std::sync::Arc;
        use worker::Env;
        let Extension(env): Extension<Arc<Env>> = extract().await?;
        let rate_limiter_disabled = env
            .secret("RATE_LIMITER_DISABLED")
            .map(|v| v.to_string().to_lowercase() == "true")
            .unwrap_or(false);
        let network = if is_mainnet { "mainnet" } else { "calibnet" };
        let may_sign = rate_limiter_disabled || query_rate_limiter(network).await?;
        let rate_limit_seconds = if is_mainnet {
            crate::constants::MAINNET_RATE_LIMIT_SECONDS
        } else {
            crate::constants::CALIBNET_RATE_LIMIT_SECONDS
        };
        if !may_sign {
            return Err(ServerFnError::ServerError(format!(
                "Rate limit exceeded - wait {} seconds",
                rate_limit_seconds
            )));
        }

        let network = if is_mainnet {
            Network::Mainnet
        } else {
            Network::Testnet
        };
        let key = secret_key(network).await?;
        #[allow(deprecated)]
        let sig = sign(
            key.key_info.r#type,
            &key.key_info.private_key,
            cid.to_bytes().as_slice(),
        )
        .map_err(|e| ServerFnError::<error::NoCustomError>::ServerError(e.to_string()))?;
        Ok(LotusJson(SignedMessage {
            message: msg,
            signature: sig,
        }))
    })
    .await
}

#[cfg(feature = "ssr")]
pub async fn secret_key(network: Network) -> Result<Key, ServerFnError> {
    use crate::key::KeyInfo;
    use axum::Extension;
    use leptos::server_fn::error;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let secret_key_name = match network {
        Network::Testnet => "SECRET_WALLET",
        Network::Mainnet => "SECRET_MAINNET_WALLET",
    };

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    #[allow(deprecated)]
    let key_info = KeyInfo::from_str(&env.secret(secret_key_name)?.to_string())
        .map_err(|e| ServerFnError::<error::NoCustomError>::ServerError(e.to_string()))?;
    Key::try_from(key_info).map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[cfg(feature = "ssr")]
pub async fn query_rate_limiter(network: &str) -> Result<bool, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::{Env, Method, Request};

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let rate_limiter = env
        .durable_object("RATE_LIMITER")?
        .id_from_name(network)?
        .get_stub()?;
    Ok(rate_limiter
        .fetch_with_request(Request::new(
            &format!("http://do/rate_limiter/{}", network),
            Method::Get,
        )?)
        .await?
        .json::<bool>()
        .await?)
}

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
