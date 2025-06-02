#![cfg(feature = "ssr")]

use crate::utils::key::{sign, Key};
use crate::{
    faucet::constants::TokenType,
    utils::{
        address::AddressAlloyExt as _,
        lotus_json::{signed_message::SignedMessage, LotusJson},
    },
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Uint, U128},
    rpc::types::TransactionRequest,
};
use alloy::{sol, sol_types::SolCall};
use anyhow::Result;
use fvm_shared::{address::Address, message::Message};
use leptos::{leptos_dom::logging::console_log, prelude::ServerFnError, server};

use super::constants::FaucetInfo;

/// Reads the faucet secret key from the CloudFlare Worker secrets.
pub async fn read_faucet_secret(faucet_info: FaucetInfo) -> Result<String, ServerFnError> {
    use axum::Extension;
    use leptos::server_fn::error;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    env.secret(faucet_info.secret_key_name())
        .map(|s| s.to_string())
        .map_err(|e| ServerFnError::new(e))
        .and_then(|s| {
            if s.is_empty() {
                Err(ServerFnError::ServerError(
                    "Faucet secret key is empty".to_string(),
                ))
            } else {
                Ok(s)
            }
        })
}

pub async fn secret_key(faucet_info: FaucetInfo) -> Result<Key, ServerFnError> {
    use crate::utils::key::KeyInfo;
    use axum::Extension;
    use leptos::server_fn::error;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let key_info = KeyInfo::from_str(&env.secret(faucet_info.secret_key_name())?.to_string())
        .map_err(|e| ServerFnError::new(e))?;
    Key::try_from(key_info).map_err(|e| ServerFnError::ServerError(e.to_string()))
}

pub async fn sign_with_eth_secret_key(
    tx_request: TransactionRequest,
    faucet_info: FaucetInfo,
) -> Result<Vec<u8>, ServerFnError> {
    use alloy::signers::local::PrivateKeySigner;
    use leptos::server_fn::error;
    use send_wrapper::SendWrapper;

    SendWrapper::new(async move {
        check_rate_limit(faucet_info).await?;
        let key = read_faucet_secret(faucet_info).await?;
        let pk_signer: PrivateKeySigner = std::str::FromStr::from_str(&key)?;
        let wallet = alloy::network::EthereumWallet::new(pk_signer);
        let tx_envolope = tx_request.build(&wallet).await;
        if tx_envolope.is_err() {
            return Err(ServerFnError::ServerError(format!(
                "Failed to build transaction envelope: {:?}",
                tx_envolope.err()
            )));
        }
        let tx_envolope = tx_envolope.unwrap();

        let rlp = alloy::eips::Encodable2718::encoded_2718(&tx_envolope);
        Ok(rlp)
    })
    .await
}

pub async fn query_rate_limiter(faucet_info: FaucetInfo) -> Result<bool, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::{Env, Method, Request};

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let rate_limiter = env
        .durable_object("RATE_LIMITER")?
        .id_from_name(&faucet_info.to_string())?
        .get_stub()?;
    Ok(rate_limiter
        .fetch_with_request(Request::new(
            &format!("http://do/rate_limiter/{faucet_info}"),
            Method::Get,
        )?)
        .await?
        .json::<bool>()
        .await?)
}

pub async fn check_rate_limit(faucet_info: FaucetInfo) -> Result<(), ServerFnError> {
    let axum::Extension(env): axum::Extension<std::sync::Arc<worker::Env>> =
        leptos_axum::extract().await?;
    let rate_limiter_disabled = env
        .secret("RATE_LIMITER_DISABLED")
        .map(|v| v.to_string().to_lowercase() == "true")
        .unwrap_or(false);
    let may_sign = rate_limiter_disabled || query_rate_limiter(faucet_info).await?;
    let rate_limit_seconds = faucet_info.rate_limit_seconds();
    if !may_sign {
        return Err(ServerFnError::ServerError(format!(
            "Rate limit exceeded - wait {rate_limit_seconds} seconds"
        )));
    }
    Ok(())
}
