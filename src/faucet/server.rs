#[cfg(feature = "ssr")]
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

#[server]
pub async fn faucet_address(faucet_info: FaucetInfo) -> Result<LotusJson<Address>, ServerFnError> {
    let key = secret_key(faucet_info).await?;
    Ok(LotusJson(key.address))
}

/// Returns the faucet address as a string, deriving it from the faucet information, and in turn,
/// from the secret key stored in the backend.
///
/// For native token faucets, it will return a Filecoin address, while for ERC-20 token faucets,
/// it will return an Ethereum address.
#[server]
pub async fn faucet_address_str(faucet_info: FaucetInfo) -> Result<String, ServerFnError> {
    use fvm_shared::address;
    match faucet_info.token_type() {
        TokenType::Native => {
            match faucet_info.network() {
                address::Network::Mainnet => {
                    address::set_current_network(address::Network::Mainnet);
                }
                address::Network::Testnet => {
                    address::set_current_network(address::Network::Testnet);
                }
            }
            let LotusJson(addr) = faucet_address(faucet_info).await?;
            Ok(addr.to_string())
        }
        TokenType::Erc20(_) => {
            let address = faucet_eth_address(faucet_info).await?;
            Ok(address.to_string())
        }
    }
}

#[server]
pub async fn faucet_eth_address(
    faucet_info: FaucetInfo,
) -> Result<alloy::primitives::Address, ServerFnError> {
    use alloy::signers::local::PrivateKeySigner;
    let key = read_faucet_secret(faucet_info).await?;
    let pk_signer: PrivateKeySigner = std::str::FromStr::from_str(&key)?;
    let pk_addr = pk_signer.address();
    Ok(pk_addr)
}

#[cfg(feature = "ssr")]
pub async fn read_faucet_secret(faucet_info: FaucetInfo) -> Result<String, ServerFnError> {
    use axum::Extension;
    use leptos::server_fn::error;
    use leptos_axum::extract;
    use std::{str::FromStr as _, sync::Arc};
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    #[allow(deprecated)]
    env.secret(faucet_info.secret_key_name())
        .map(|s| s.to_string())
        .map_err(|e| ServerFnError::<error::NoCustomError>::ServerError(e.to_string()))
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
pub async fn sign_with_eth_secret_key(
    tx_request: TransactionRequest,
    faucet_info: FaucetInfo,
) -> Result<Vec<u8>, ServerFnError> {
    use leptos::server_fn::error;
    use send_wrapper::SendWrapper;
    //if tx_request.value.is_some() {
    //    return Err(ServerFnError::ServerError(
    //        "Native token must not be sent in ERC-20 faucet".to_string(),
    //    ));
    //}
    console_log("Signing transaction request");

    // TODO check the value of the transaction request?
    SendWrapper::new(async move {
        use axum::Extension;
        use leptos_axum::extract;
        use std::sync::Arc;
        use worker::Env;
        let Extension(env): Extension<Arc<Env>> = extract().await?;
        console_log(&format!("Signing transaction request: {tx_request:?}"));
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

        use alloy::signers::local::PrivateKeySigner;
        console_log(&format!("Reading faucet secret key for {faucet_info}"));
        let key = read_faucet_secret(faucet_info).await?;
        console_log(&format!("Faucet secret key read successfully: {key}"));
        let pk_signer: PrivateKeySigner = std::str::FromStr::from_str(&key)?;
        console_log("private key signer created");
        let wallet = alloy::network::EthereumWallet::new(pk_signer);
        let tx_envolope = tx_request.build(&wallet).await;
        if tx_envolope.is_err() {
            return Err(ServerFnError::ServerError(format!(
                "Failed to build transaction envelope: {:?}",
                tx_envolope.err()
            )));
        }
        console_log(&format!("Transaction envelope: {tx_envolope:?}"));
        let tx_envolope = tx_envolope.unwrap();

        let rlp = alloy::eips::Encodable2718::encoded_2718(&tx_envolope);
        Ok(rlp)
    })
    .await
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

#[server]
pub async fn signed_erc20_transfer(
    recipient: alloy::primitives::Address,
    nonce: u64,
    gas_price: u64,
    faucet_info: FaucetInfo,
) -> Result<Vec<u8>, ServerFnError> {
    console_log("Signing ERC-20 transfer transaction");
    let gas_price = 100_000_000_000u64; // 100 Gwei, this is a placeholder, should be replaced with
                                        // actual gas price logic
    let contract_address = match faucet_info.token_type() {
        TokenType::Erc20(addr) => addr,
        _ => {
            return Err(ServerFnError::ServerError(
                "This function is only for ERC-20 token transfers".to_string(),
            ));
        }
    };
    let amount = faucet_info.drip_amount();
    sol! {
        #[sol(rpc)]
        contract ERC20 {
            function transfer(address to, uint256 amount) public returns (bool);
        }
    }

    let amount = Uint::from_be_slice(&amount.atto().to_signed_bytes_be());

    let gas_limit = 30_000_000; // the actual gas usage should be ~ 20M, but we add some buffer
    let calldata = ERC20::transferCall::new((recipient, amount)).abi_encode();

    let tx = alloy::rpc::types::TransactionRequest::default()
        .with_to(contract_address)
        .with_chain_id(faucet_info.chain_id())
        .with_nonce(nonce)
        .with_gas_limit(gas_limit)
        .with_gas_price(gas_price.into())
        .with_input(calldata);

    sign_with_eth_secret_key(tx.clone(), faucet_info).await
}
