#[cfg(feature = "ssr")]
use crate::utils::key::{sign, Key};
use crate::utils::lotus_json::{signed_message::SignedMessage, LotusJson};
use anyhow::Result;
use fvm_shared::{address::Address, message::Message};
use leptos::{prelude::ServerFnError, server};

use super::constants::FaucetInfo;

#[server]
pub async fn faucet_address(faucet_info: FaucetInfo) -> Result<LotusJson<Address>, ServerFnError> {
    let key = secret_key(faucet_info).await?;
    Ok(LotusJson(key.address))
}

#[server]
pub async fn sign_with_secret_key(
    msg: LotusJson<Message>,
    faucet_info: FaucetInfo,
) -> Result<LotusJson<SignedMessage>, ServerFnError> {
    use crate::utils::lotus_json::signed_message::message_cid;
    use leptos::server_fn::error;
    use send_wrapper::SendWrapper;
    let LotusJson(msg) = msg;
    let cid = message_cid(&msg);
    let amount_limit = faucet_info.drip_amount();
    if &msg.value > amount_limit {
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
        let may_sign = rate_limiter_disabled || query_rate_limiter(faucet_info).await?;
        let rate_limit_seconds = faucet_info.rate_limit_seconds();
        if !may_sign {
            return Err(ServerFnError::ServerError(format!(
                "Rate limit exceeded - wait {rate_limit_seconds} seconds"
            )));
        }

        let key = secret_key(faucet_info).await?;
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
pub async fn secret_key(faucet_info: FaucetInfo) -> Result<Key, ServerFnError> {
    use crate::utils::key::KeyInfo;
    use axum::Extension;
    use leptos::server_fn::error;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    #[allow(deprecated)]
    let key_info = KeyInfo::from_str(&env.secret(faucet_info.secret_key_name())?.to_string())
        .map_err(|e| ServerFnError::<error::NoCustomError>::ServerError(e.to_string()))?;
    Key::try_from(key_info).map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[cfg(feature = "ssr")]
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
