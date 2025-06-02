//! This file contains the server-side API for the faucet functionality. More fine grained,
//! server-side functions (that are not exposed to the client) are in the `[`super::server`] module.

use crate::{
    faucet::constants::TokenType,
    utils::{
        address::AddressAlloyExt as _,
        lotus_json::{signed_message::SignedMessage, LotusJson},
    },
};
use alloy::{network::TransactionBuilder as _, primitives::Uint, rpc::types::TransactionRequest};
use alloy::{sol, sol_types::SolCall};
use anyhow::Result;
use fvm_shared::{address::Address, message::Message};
use leptos::{leptos_dom::logging::console_log, prelude::ServerFnError, server};

#[cfg(feature = "ssr")]
use super::server::{check_rate_limit, read_faucet_secret, secret_key, sign_with_eth_secret_key};

use super::constants::FaucetInfo;
/// Returns the faucet address. This assumes the faucet in place is a native token faucet.
#[server]
pub async fn faucet_address(faucet_info: FaucetInfo) -> Result<LotusJson<Address>, ServerFnError> {
    if matches!(faucet_info.token_type(), TokenType::Erc20(_)) {
        return Err(ServerFnError::ServerError(
            "This function is only for native token faucets".to_string(),
        ));
    }
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

/// Returns the faucet address as an Ethereum address, which is used for ERC-20 token faucets.
/// This assumes that the faucet is configured to use an ERC-20 token.
#[server]
pub async fn faucet_eth_address(
    faucet_info: FaucetInfo,
) -> Result<alloy::primitives::Address, ServerFnError> {
    if matches!(faucet_info.token_type(), TokenType::Native) {
        return Err(ServerFnError::ServerError(
            "This function is only for ERC-20 token faucets".to_string(),
        ));
    }
    use alloy::signers::local::PrivateKeySigner;
    let key = read_faucet_secret(faucet_info).await?;
    let pk_signer: PrivateKeySigner = std::str::FromStr::from_str(&key)?;
    let pk_addr = pk_signer.address();
    Ok(pk_addr)
}

#[server]
pub async fn sign_with_secret_key(
    msg: LotusJson<Message>,
    faucet_info: FaucetInfo,
) -> Result<LotusJson<SignedMessage>, ServerFnError> {
    use crate::utils::key::sign;
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
        check_rate_limit(faucet_info).await?;
        let key = secret_key(faucet_info).await?;
        let sig = sign(
            key.key_info.r#type,
            &key.key_info.private_key,
            cid.to_bytes().as_slice(),
        )
        .map_err(|e| ServerFnError::new(e))?;
        Ok(LotusJson(SignedMessage {
            message: msg,
            signature: sig,
        }))
    })
    .await
}

/// Signs an ERC-20 transfer transaction to the specified recipient with the given nonce and gas
/// price. The required params are needed so that the server doesn't have to call the provider.
///
/// Note: it's important that the transaction is constructed server-side to avoid exposing the
/// `calldata` to the client, which could lead to security issues if the client were to
/// manipulate the transaction data.
#[server]
pub async fn signed_erc20_transfer(
    recipient: alloy::primitives::Address,
    nonce: u64,
    gas_price: u64,
    faucet_info: FaucetInfo,
) -> Result<Vec<u8>, ServerFnError> {
    console_log(&format!("Signing ERC-20 transfer transaction for {faucet_info} to {recipient} with nonce {nonce} and gas price {gas_price}"));
    sol! {
        #[sol(rpc)]
        contract ERC20 {
            function transfer(address to, uint256 amount) public returns (bool);
        }
    }

    let contract_address = match faucet_info.token_type() {
        TokenType::Erc20(addr) => addr,
        _ => {
            return Err(ServerFnError::ServerError(
                "This function is only for ERC-20 token transfers".to_string(),
            ));
        }
    };
    let amount = faucet_info.drip_amount();
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
