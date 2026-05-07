#![cfg(feature = "ssr")]

use super::constants::FaucetInfo;
use crate::utils::address::AnyAddress;
use crate::utils::key::KeyInfo;
use crate::utils::key::{Key, sign};
use crate::utils::lotus_json::{
    LotusJson,
    signed_message::{SignedMessage, message_cid},
};
use alloy::{network::NetworkTransactionBuilder, rpc::types::TransactionRequest};
use anyhow::Result;
use axum::Extension;
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
) -> Result<Vec<u8>, ServerFnError> {
    SendWrapper::new(async move {
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

/// Queries the rate limiter for a specific faucet and wallet address.
/// Returns:
/// - `None` if no rate limit is set.
/// - `Some(i32)` containing the remaining cool-down time in seconds.
async fn query_rate_limiter(
    faucet_info: FaucetInfo,
    wallet_addr: AnyAddress,
) -> Result<Option<i32>, ServerFnError> {
    SendWrapper::new(async move {
        let Extension(env): Extension<Arc<Env>> = extract().await?;
        let rate_limiter = env
            .durable_object("RATE_LIMITER")?
            .id_from_name(&faucet_info.to_string())?
            .get_stub()?;
        rate_limiter
            .fetch_with_request(Request::new(
                &format!("http://do/rate_limiter/{faucet_info}/{wallet_addr}"),
                Method::Get,
            )?)
            .await?
            .json::<Option<i32>>()
            .await
            .map_err(ServerFnError::new)
    })
    .await
}

/// Undoes one drip allocation in the rate limiter DO after a failed on-chain submission.
/// Server-internal: never call from a public server fn (would defeat rate limiting).
pub async fn refund_rate_limit_by_key(
    faucet_info: FaucetInfo,
    wallet_addr: AnyAddress,
) -> Result<(), ServerFnError> {
    SendWrapper::new(async move {
        let Extension(env): Extension<Arc<Env>> = extract().await?;
        if env
            .secret("RATE_LIMITER_DISABLED")
            .map(|v| v.to_string().to_lowercase() == "true")
            .unwrap_or(false)
        {
            return Ok(());
        }
        let token = match env.secret("RATE_LIMITER_REFUND_SECRET") {
            Ok(s) if !s.to_string().trim().is_empty() => s.to_string(),
            _ => {
                log::warn!("RATE_LIMITER_REFUND_SECRET unset/empty: skipping rate limit refund");
                return Ok(());
            }
        };
        let stub = env
            .durable_object("RATE_LIMITER")?
            .id_from_name(&faucet_info.to_string())?
            .get_stub()?;
        let headers = Headers::new();
        headers.set("Authorization", &format!("Bearer {}", token.trim()))?;
        let mut init = RequestInit::new();
        init.with_method(Method::Post).with_headers(headers);
        let request = Request::new_with_init(
            &format!("http://do/rate_limiter/{faucet_info}/{wallet_addr}/refund"),
            &init,
        )?;
        let status = stub
            .fetch_with_request(request)
            .await
            .map_err(ServerFnError::new)?
            .status_code();
        if !(200..300).contains(&status) {
            log::warn!("rate limit refund DO returned HTTP {status} (faucet={faucet_info})");
        }
        Ok(())
    })
    .await
}

/// Checks if the request can proceed based on the rate limit for the given faucet.
pub async fn check_rate_limit(
    faucet_info: FaucetInfo,
    wallet_addr: AnyAddress,
) -> Result<Option<i32>, ServerFnError> {
    let axum::Extension(env): axum::Extension<std::sync::Arc<worker::Env>> =
        leptos_axum::extract().await?;
    let mut rate_limit = None;
    let rate_limiter_disabled = env
        .secret("RATE_LIMITER_DISABLED")
        .map(|v| v.to_string().to_lowercase() == "true")
        .unwrap_or(false);
    if !rate_limiter_disabled {
        rate_limit = query_rate_limiter(faucet_info, wallet_addr).await?;
    }
    Ok(rate_limit)
}
