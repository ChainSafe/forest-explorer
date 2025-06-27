#![cfg(feature = "ssr")]

use super::constants::{FaucetInfo, WALLET_CAP_RESET_IN_SECONDS};
use super::rate_limiter::RateLimiterResponse;
use crate::utils::format::format_balance;
use crate::utils::key::KeyInfo;
use crate::utils::key::{sign, Key};
use crate::utils::lotus_json::{
    signed_message::{message_cid, SignedMessage},
    LotusJson,
};
use alloy::{network::TransactionBuilder, rpc::types::TransactionRequest};
use anyhow::Result;
use axum::Extension;
use chrono::{DateTime, Utc};
use fvm_shared::message::Message;
use leptos::prelude::ServerFnError;
use leptos_axum::extract;
use send_wrapper::SendWrapper;
use std::str::FromStr as _;
use std::sync::Arc;
use worker::*;

/// Reads the faucet secret key from the CloudFlare Worker secrets.
pub async fn read_faucet_secret(faucet_info: FaucetInfo) -> Result<String, ServerFnError> {
    let Extension(env): Extension<Arc<Env>> = extract().await?;
    env.secret(faucet_info.secret_key_name())
        .map(|s| s.to_string())
        .map_err(ServerFnError::new)
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
    let axum::Extension(env): axum::Extension<Arc<worker::Env>> = extract().await?;
    let key_info = KeyInfo::from_str(&env.secret(faucet_info.secret_key_name())?.to_string())
        .map_err(ServerFnError::new)?;
    Key::try_from(key_info).map_err(ServerFnError::new)
}

/// Signs a message using the faucet's secret key.
/// Note: it is important to ensure that the `Message` is fully controlled by the server
/// not exposed to the client, as it might be modified by the client, leading to potential
/// security issues.
pub async fn sign_with_secret_key(
    msg: Message,
    faucet_info: FaucetInfo,
) -> Result<LotusJson<SignedMessage>, ServerFnError> {
    SendWrapper::new(async move {
        check_rate_limit(faucet_info, msg.to.id()?).await?;
        let cid = message_cid(&msg);
        let key = secret_key(faucet_info).await?;
        let sig = sign(
            key.key_info.r#type,
            &key.key_info.private_key,
            cid.to_bytes().as_slice(),
        )
        .map_err(ServerFnError::new)?;
        Ok(LotusJson(SignedMessage {
            message: msg,
            signature: sig,
        }))
    })
    .await
}

/// Signs a transaction request using the faucet's secret key.
///
/// Note: it is important to ensure that the `TransactionRequest` is fully controlled by the server
/// and not exposed to the client, as it might be modified by the client, leading to potential
/// security issues.
pub async fn sign_with_eth_secret_key(
    tx_request: TransactionRequest,
    faucet_info: FaucetInfo,
    id: u64,
) -> Result<Vec<u8>, ServerFnError> {
    SendWrapper::new(async move {
        check_rate_limit(faucet_info, id).await?;
        let key = read_faucet_secret(faucet_info).await?;
        let pk_signer: alloy::signers::local::PrivateKeySigner = std::str::FromStr::from_str(&key)?;
        let wallet = alloy::network::EthereumWallet::new(pk_signer);
        let tx_envolope = tx_request.build(&wallet).await;
        let rlp =
            alloy::eips::Encodable2718::encoded_2718(&tx_envolope.map_err(ServerFnError::new)?);
        Ok(rlp)
    })
    .await
}

/// Internal. Queries the rate limiter Durable Object to check if the request can proceed.
async fn query_rate_limiter(
    faucet_info: FaucetInfo,
    id: u64,
) -> Result<RateLimiterResponse, ServerFnError> {
    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let rate_limiter = env
        .durable_object("RATE_LIMITER")?
        .id_from_name(&faucet_info.to_string())?
        .get_stub()?;
    Ok(rate_limiter
        .fetch_with_request(Request::new(
            &format!("http://do/rate_limiter/{faucet_info}/{id}"),
            Method::Get,
        )?)
        .await?
        .json()
        .await?)
}

/// Checks if the request can proceed based on the rate limit for the given faucet.
async fn check_rate_limit(faucet_info: FaucetInfo, id: u64) -> Result<(), ServerFnError> {
    let axum::Extension(env): axum::Extension<std::sync::Arc<worker::Env>> =
        leptos_axum::extract().await?;
    let rate_limiter_disabled = env
        .secret("RATE_LIMITER_DISABLED")
        .map(|v| v.to_string().to_lowercase() == "true")
        .unwrap_or(false);
    let rate_limiter = query_rate_limiter(faucet_info, id).await?;
    let may_sign = rate_limiter_disabled || rate_limiter.may_sign;
    if !may_sign {
        let LotusJson(claimed) = rate_limiter.claimed;
        let block_until = if claimed >= faucet_info.wallet_cap() {
            rate_limiter.block_until + WALLET_CAP_RESET_IN_SECONDS
        } else {
            rate_limiter.block_until
        };

        return Err(ServerFnError::ServerError(format!(
            "Rate limiter invoked for {faucet_info}: claimed={}, blocked until {}, id={id}",
            format_balance(&claimed, faucet_info.unit()),
            DateTime::<Utc>::from_timestamp(block_until, 0).unwrap_or_default()
        )));
    }
    Ok(())
}
