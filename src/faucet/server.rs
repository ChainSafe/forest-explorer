#[cfg(feature = "ssr")]
use crate::utils::key::{sign, Key};
use crate::utils::lotus_json::{signed_message::SignedMessage, LotusJson};
use anyhow::Result;
#[cfg(feature = "ssr")]
use fvm_shared::address::Network;
use fvm_shared::{address::Address, message::Message};
use leptos::{prelude::ServerFnError, server};

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
    use crate::utils::lotus_json::signed_message::message_cid;
    use leptos::server_fn::error;
    use send_wrapper::SendWrapper;
    let LotusJson(msg) = msg;
    let cid = message_cid(&msg);
    let amount_limit = match is_mainnet {
        true => crate::faucet::constants::MAINNET_DRIP_AMOUNT.clone(),
        false => crate::faucet::constants::CALIBNET_DRIP_AMOUNT.clone(),
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
            crate::faucet::constants::MAINNET_RATE_LIMIT_SECONDS
        } else {
            crate::faucet::constants::CALIBNET_RATE_LIMIT_SECONDS
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
    use crate::utils::key::KeyInfo;
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
