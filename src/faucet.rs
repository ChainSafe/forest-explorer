#[cfg(feature = "ssr")]
use crate::key::{sign, Key};
use crate::lotus_json::LotusJson;
use cid::Cid;
use fvm_shared::{address::Address, crypto::signature::Signature};
use leptos::{server, ServerFnError};

#[server]
pub async fn sign_with_secret_key(
    cid: LotusJson<Cid>,
) -> Result<LotusJson<Signature>, ServerFnError> {
    use send_wrapper::SendWrapper;
    SendWrapper::new(async move {
        let may_sign = query_rate_limiter().await?;
        if !may_sign {
            return Err(ServerFnError::ServerError(
                "Rate limit exceeded - wait 30 seconds".to_string(),
            ));
        }

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
pub async fn query_rate_limiter() -> Result<bool, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use std::sync::Arc;
    use worker::{Env, Method, Request};

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let rate_limiter = env
        .durable_object("RATE_LIMITER")?
        .id_from_name("RATE_LIMITER")?
        .get_stub()?;
    Ok(rate_limiter
        .fetch_with_request(Request::new("http://do/rate_limiter", Method::Get)?)
        .await?
        .json::<bool>()
        .await?)
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
