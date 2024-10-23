use crate::{
    key::{sign, Key},
    lotus_json::LotusJson,
};
use cid::Cid;
use fvm_shared::{address::Address, crypto::signature::Signature};
use leptos::{server, ServerFnError};

#[server]
pub async fn sign_with_secret_key(
    cid: LotusJson<Cid>,
) -> Result<LotusJson<Signature>, ServerFnError<String>> {
    let cid = cid.0;
    let key = secret_key().await;
    sign(
        key.key_info.r#type,
        &key.key_info.private_key,
        cid.to_bytes().as_slice(),
    )
    .map(|sig| LotusJson(sig))
    .map_err(|e| ServerFnError::from(e.to_string()))
}

#[server]
pub async fn faucet_address() -> Result<LotusJson<Address>, ServerFnError> {
    let key = secret_key().await;
    Ok(LotusJson(key.address))
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
