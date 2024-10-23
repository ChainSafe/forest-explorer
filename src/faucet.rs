use crate::{
    key::{sign, Key},
    lotus_json::LotusJson,
};
use chrono::{DateTime, Utc};
use cid::Cid;
use fvm_shared::{address::Address, crypto::signature::Signature};
use leptos::{server, ServerFnError};

#[server]
pub async fn sign_with_secret_key(
    cid: LotusJson<Cid>,
) -> Result<LotusJson<Signature>, ServerFnError<String>> {
    use send_wrapper::SendWrapper;
    SendWrapper::new(async move {
        let now = Utc::now();
        let last_sign = query_last_sign().await;
        if now - last_sign < chrono::Duration::seconds(10) {
            return Err(ServerFnError::from(
                "Rate limit exceeded - wait 10 seconds".to_string(),
            ));
        }
        set_last_sign(now).await;

        let cid = cid.0;
        let key = secret_key().await;
        sign(
            key.key_info.r#type,
            &key.key_info.private_key,
            cid.to_bytes().as_slice(),
        )
        .map(|sig| LotusJson(sig))
        .map_err(|e| ServerFnError::from(e.to_string()))
    })
    .await
}

#[server]
pub async fn faucet_address() -> Result<LotusJson<Address>, ServerFnError> {
    let key = secret_key().await;
    Ok(LotusJson(key.address))
}

#[cfg(feature = "ssr")]
pub async fn query_last_sign() -> DateTime<Utc> {
    use axum::Extension;
    use chrono::{DateTime, Utc};
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await.expect("CloudFlare ENV must be set");
    let kv = env
        .kv("RATE_LIMIT")
        .expect("RATE_LIMIT kv store must be set");
    let timestamp_last_request = kv
        .get("GLOBAL_RATE_LIMIT")
        .json::<i64>()
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    DateTime::<Utc>::from_timestamp(timestamp_last_request, 0).unwrap()
}

#[cfg(feature = "ssr")]
pub async fn set_last_sign(at: DateTime<Utc>) {
    use axum::Extension;
    use chrono::{DateTime, Utc};
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await.expect("CloudFlare ENV must be set");
    let kv = env
        .kv("RATE_LIMIT")
        .expect("RATE_LIMIT kv store must be set");
    kv.put("GLOBAL_RATE_LIMIT", at.timestamp())
        .unwrap()
        .execute()
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
pub async fn secret_key() -> Key {
    use crate::key::KeyInfo;
    use axum::Extension;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await.expect("CloudFlare ENV must be set");
    let key_info = KeyInfo::from_str(
        &env.secret("SECRET_WALLET")
            .expect("SECRET_WALLET must be set")
            .to_string(),
    )
    .unwrap();
    Key::try_from(key_info).unwrap()
}
