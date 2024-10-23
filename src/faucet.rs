#[cfg(feature = "ssr")]
use crate::key::{sign, Key};
use crate::lotus_json::LotusJson;
#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};
use cid::Cid;
use fvm_shared::{address::Address, crypto::signature::Signature};
use leptos::{server, ServerFnError};

#[server]
pub async fn sign_with_secret_key(
    cid: LotusJson<Cid>,
) -> Result<LotusJson<Signature>, ServerFnError> {
    use send_wrapper::SendWrapper;
    SendWrapper::new(async move {
        let now = Utc::now();
        let last_sign = query_last_sign().await?;
        if now - last_sign < chrono::Duration::seconds(30) {
            return Err(ServerFnError::ServerError(
                "Rate limit exceeded - wait 30 seconds".to_string(),
            ));
        }
        set_last_sign(now).await?;

        let cid = cid.0;
        let key = secret_key().await?;
        sign(
            key.key_info.r#type,
            &key.key_info.private_key,
            cid.to_bytes().as_slice(),
        )
        .map(LotusJson)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
    })
    .await
}

#[server]
pub async fn faucet_address() -> Result<LotusJson<Address>, ServerFnError> {
    let key = secret_key().await?;
    Ok(LotusJson(key.address))
}

#[cfg(feature = "ssr")]
pub async fn query_last_sign() -> Result<DateTime<Utc>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let kv = env.kv("RATE_LIMIT")?;
    let timestamp_last_request = kv
        .get("GLOBAL_RATE_LIMIT")
        .json::<i64>()
        .await?
        .unwrap_or_default();
    DateTime::<Utc>::from_timestamp(timestamp_last_request, 0)
        .ok_or_else(|| ServerFnError::ServerError("Invalid timestamp".to_string()))
}

#[cfg(feature = "ssr")]
pub async fn set_last_sign(at: DateTime<Utc>) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let kv = env.kv("RATE_LIMIT")?;
    kv.put("GLOBAL_RATE_LIMIT", at.timestamp())?
        .execute()
        .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn secret_key() -> Result<Key, ServerFnError> {
    use crate::key::KeyInfo;
    use axum::Extension;
    use leptos::server_fn::error::NoCustomError;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let key_info = KeyInfo::from_str(&env.secret("SECRET_WALLET")?.to_string())
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
    Key::try_from(key_info).map_err(|e| ServerFnError::ServerError(e.to_string()))
}
