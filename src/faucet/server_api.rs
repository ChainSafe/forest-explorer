//! This file contains the server-side API for the faucet functionality. More fine grained,
//! server-side functions (that are not exposed to the client) are in the `server` module.

use crate::utils::{
    address::AnyAddress,
    lotus_json::{LotusJson, signed_message::SignedMessage},
};
use alloy::primitives::TxHash;
use anyhow::Result;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use leptos::{prelude::ServerFnError, server, server_fn::codec::GetUrl};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use axum::http::StatusCode;

#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

#[cfg(feature = "ssr")]
use alloy::{sol, sol_types::SolCall};

#[cfg(feature = "ssr")]
use super::server::{
    check_rate_limit, read_faucet_secret, secret_key, sign_with_eth_secret_key,
    sign_with_secret_key,
};

#[cfg(feature = "ssr")]
use crate::faucet::constants::TokenType;

use super::constants::FaucetInfo;
use crate::utils::error::FaucetError;

/// Returns the faucet address. This assumes the faucet in place is a native token faucet.
#[server]
async fn faucet_fil_address(faucet_info: FaucetInfo) -> Result<LotusJson<Address>, ServerFnError> {
    if matches!(faucet_info.token_type(), TokenType::Erc20(_)) {
        return Err(ServerFnError::ServerError(
            "This function is only for native token faucets".to_string(),
        ));
    }
    let key = secret_key(faucet_info).await?;
    Ok(LotusJson(key.address))
}

/// Returns the faucet address, deriving it from the faucet information, and in turn,
/// from the secret key stored in the backend.
///
/// For native token faucets, it will return a Filecoin address, while for ERC-20 token faucets,
/// it will return an Ethereum address.
#[server]
pub async fn faucet_address(faucet_info: FaucetInfo) -> Result<AnyAddress, ServerFnError> {
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
            let addr = faucet_fil_address(faucet_info).await?;
            Ok(AnyAddress::Filecoin(addr))
        }
        TokenType::Erc20(_) => {
            let address = faucet_eth_address(faucet_info).await?;
            Ok(AnyAddress::Ethereum(address))
        }
    }
}

/// Returns the faucet address as an Ethereum address, which is used for ERC-20 token faucets.
/// This assumes that the faucet is configured to use an ERC-20 token.
#[server]
async fn faucet_eth_address(
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

/// Signs a Filecoin transfer message to the specified recipient with the given parameters.
/// The required params are needed so that the server doesn't have to call the provider.
/// Note: it's important that the message is constructed server-side to avoid exposing the
/// `message` to the client, which could lead to security issues if the client were to
/// manipulate the message data.
/// This function is used for native Filecoin token transfers.
#[server]
pub async fn signed_fil_transfer(
    to: LotusJson<Address>,
    gas_limit: u64,
    gas_fee_cap: LotusJson<TokenAmount>,
    gas_premium: LotusJson<TokenAmount>,
    sequence: u64,
    faucet_info: FaucetInfo,
) -> Result<LotusJson<SignedMessage>, FaucetError> {
    use crate::utils::message::message_transfer_native;
    let LotusJson(to) = to;
    let LotusJson(gas_fee_cap) = gas_fee_cap;
    let LotusJson(gas_premium) = gas_premium;

    let rate_limit_seconds =
        check_rate_limit(faucet_info, AnyAddress::Filecoin(LotusJson(to))).await?;
    // Make sure gas values aren't too high
    let gas_limit = gas_limit.min(faucet_info.max_gas_limit());
    let gas_fee_cap = gas_fee_cap.min(faucet_info.max_gas_fee_cap());
    let gas_premium = gas_premium.min(faucet_info.max_gas_premium());
    if let Some(secs) = rate_limit_seconds {
        return Err(FaucetError::RateLimited {
            retry_after_secs: secs,
        });
    }

    let from = faucet_address(faucet_info)
        .await?
        .to_filecoin_address(faucet_info.network())
        .map_err(|e| FaucetError::Server(e.to_string()))?;

    let unsigned_msg = message_transfer_native(
        from,
        to,
        faucet_info.drip_amount().clone(),
        gas_limit,
        gas_fee_cap,
        gas_premium,
        sequence,
    );
    let signed = sign_with_secret_key(unsigned_msg, faucet_info).await?;
    Ok(signed)
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
) -> Result<Vec<u8>, FaucetError> {
    use crate::utils::conversions::TokenAmountAlloyExt as _;
    use alloy::network::TransactionBuilder as _;

    let rate_limit_seconds = check_rate_limit(faucet_info, AnyAddress::Ethereum(recipient)).await?;
    if let Some(secs) = rate_limit_seconds {
        return Err(FaucetError::RateLimited {
            retry_after_secs: secs,
        });
    }
    log::info!(
        "Signing ERC-20 transfer transaction for {faucet_info} to {recipient} with nonce {nonce} and gas price {gas_price}"
    );
    sol! {
        #[sol(rpc)]
        contract ERC20 {
            function transfer(address to, uint256 amount) public returns (bool);
        }
    }

    let contract_address = match faucet_info.token_type() {
        TokenType::Erc20(addr) => addr,
        _ => {
            return Err(FaucetError::Server(
                "This function is only for ERC-20 token transfers".to_string(),
            ));
        }
    };
    let amount = faucet_info.drip_amount().to_alloy_amount();

    let gas_limit = faucet_info.max_gas_limit();
    let calldata = ERC20::transferCall::new((recipient, amount)).abi_encode();

    let tx = alloy::rpc::types::TransactionRequest::default()
        .with_to(contract_address)
        .with_chain_id(faucet_info.chain_id())
        .with_nonce(nonce)
        .with_gas_limit(gas_limit)
        .with_gas_price(gas_price.into())
        .with_input(calldata);

    let signed = sign_with_eth_secret_key(tx.clone(), faucet_info).await?;
    Ok(signed)
}

#[derive(Serialize, Deserialize)]
pub struct ClaimResponse {
    pub faucet_info: FaucetInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<TxHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ServerFnError>,
}

/// Server API endpoint for claiming calibnet tokens from the faucet.
/// Returns a transaction ID on successful token claim.
/// Supports distribution of `CalibnetFIL` and `CalibnetUSDFC` tokens.
/// Subject to rate limiting to prevent abuse.
#[server(endpoint = "claim_token", input = GetUrl)]
pub async fn claim_token(
    faucet_info: FaucetInfo,
    address: String,
) -> Result<TxHash, ServerFnError> {
    use crate::utils::rpc_context::Provider;
    use fvm_shared::address::set_current_network;
    use send_wrapper::SendWrapper;

    let network = faucet_info.network();
    set_current_network(network);
    let recipient = parse_and_validate_address(&address, faucet_info)?;
    let rpc = Provider::from_network(network);
    let from = faucet_address(faucet_info)
        .await?
        .to_filecoin_address(network)
        .map_err(ServerFnError::new)?;

    SendWrapper::new(async move {
        ensure_faucet_has_funds(&rpc, &from, &faucet_info).await?;
        match faucet_info {
            FaucetInfo::MainnetFIL => {
                set_response_status(StatusCode::IM_A_TEAPOT);
                Err(ServerFnError::ServerError(
                    "I'm a teapot - mainnet tokens are not available.".to_string(),
                ))
            }
            FaucetInfo::CalibnetFIL => handle_native_claim(faucet_info, recipient, from, rpc).await,
            FaucetInfo::CalibnetUSDFC => {
                handle_erc20_claim(faucet_info, recipient, from, rpc).await
            }
        }
    })
    .await
}

#[server(endpoint = "claim_token_all", input = GetUrl)]
pub async fn claim_token_all(address: String) -> Result<Vec<ClaimResponse>, ServerFnError> {
    let faucets = [FaucetInfo::CalibnetUSDFC, FaucetInfo::CalibnetFIL];
    let mut results = Vec::with_capacity(faucets.len());

    for faucet in faucets {
        let response = match claim_token(faucet, address.clone()).await {
            Ok(tx_hash) => ClaimResponse {
                faucet_info: faucet,
                tx_hash: Some(tx_hash),
                error: None,
            },
            Err(e) => ClaimResponse {
                faucet_info: faucet,
                tx_hash: None,
                error: Some(e),
            },
        };
        results.push(response);
    }

    Ok(results)
}

/// Checks if the provided address is valid for the faucet, ensuring invalid addresses are rejected.
#[cfg(feature = "ssr")]
fn check_valid_address(address: Address, faucet_info: FaucetInfo) -> Result<(), ServerFnError> {
    use crate::utils::address::AddressAlloyExt;
    use fvm_shared::address::Protocol;

    if matches!(faucet_info, FaucetInfo::CalibnetUSDFC)
        && (address.protocol() == Protocol::ID || address.into_eth_address().is_err())
    {
        log::error!("Invalid address: {:?}", address);
        set_response_status(StatusCode::BAD_REQUEST);
        return Err(ServerFnError::ServerError("Invalid address: Only Ethereum-compatible addresses (delegated t4 addresses or native Ethereum 0x addresses) are allowed for Calibnet USDFC token claims.".to_string()));
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn parse_and_validate_address(
    address: &str,
    faucet_info: FaucetInfo,
) -> Result<Address, ServerFnError> {
    match crate::utils::address::parse_address(address, faucet_info.network()) {
        Ok(addr) => {
            check_valid_address(addr, faucet_info)?;
            Ok(addr)
        }
        Err(e) => {
            log::error!("Invalid address - failed to parse: {}", e);
            set_response_status(StatusCode::BAD_REQUEST);
            Err(ServerFnError::ServerError(format!(
                "Invalid address - failed to parse: {}",
                e
            )))
        }
    }
}

#[cfg(feature = "ssr")]
async fn ensure_faucet_has_funds(
    rpc: &crate::utils::rpc_context::Provider,
    from: &Address,
    faucet_info: &FaucetInfo,
) -> Result<(), ServerFnError> {
    let faucet_balance = rpc
        .wallet_balance(*from, &faucet_info.token_type())
        .await
        .map_err(ServerFnError::new)?;
    let max_gas_estimate = faucet_info.max_gas_limit() * faucet_info.max_gas_fee_cap();
    if faucet_balance < (faucet_info.drip_amount() + max_gas_estimate) {
        return Err(ServerFnError::ServerError(
            "Faucet is empty, Request top-up".to_string(),
        ));
    }
    Ok(())
}

#[cfg(feature = "ssr")]
async fn handle_native_claim(
    faucet_info: FaucetInfo,
    recipient: Address,
    from: Address,
    rpc: crate::utils::rpc_context::Provider,
) -> Result<TxHash, ServerFnError> {
    use crate::utils::message::message_transfer;

    let id_address = rpc.lookup_id(recipient).await.unwrap_or_else(|_| {
        log::debug!("ID lookup failed, using recipient address: {:?}", recipient);
        recipient
    });
    let nonce = rpc
        .mpool_get_nonce(from)
        .await
        .map_err(ServerFnError::new)?;
    let drip_amount = faucet_info.drip_amount();
    let raw_msg = message_transfer(from, id_address, drip_amount.clone());
    let msg = rpc
        .estimate_gas(raw_msg)
        .await
        .map_err(ServerFnError::new)?;

    match signed_fil_transfer(
        LotusJson(id_address),
        msg.gas_limit,
        LotusJson(msg.gas_fee_cap),
        LotusJson(msg.gas_premium),
        nonce,
        faucet_info,
    )
    .await
    {
        Ok(LotusJson(smsg)) => {
            let cid = rpc.mpool_push(smsg).await.map_err(ServerFnError::new)?;
            let tx_hash = rpc
                .eth_get_transaction_hash_by_cid(cid)
                .await
                .map_err(ServerFnError::new)?;
            Ok(tx_hash)
        }
        Err(err) => Err(handle_faucet_error(err)),
    }
}

#[cfg(feature = "ssr")]
async fn handle_erc20_claim(
    faucet_info: FaucetInfo,
    recipient: Address,
    from: Address,
    rpc: crate::utils::rpc_context::Provider,
) -> Result<TxHash, ServerFnError> {
    use crate::utils::address::AddressAlloyExt;

    let eth_to = recipient.into_eth_address().map_err(ServerFnError::new)?;
    let nonce = rpc
        .mpool_get_nonce(from)
        .await
        .map_err(ServerFnError::new)?;
    let gas_price = rpc.gas_price().await.map_err(ServerFnError::new)?;

    match signed_erc20_transfer(eth_to, nonce, gas_price, faucet_info).await {
        Ok(signed) => {
            let tx_hash = rpc
                .send_eth_transaction_signed(&signed)
                .await
                .map_err(ServerFnError::new)?;
            Ok(tx_hash)
        }
        Err(err) => Err(handle_faucet_error(err)),
    }
}

#[cfg(feature = "ssr")]
fn handle_faucet_error(err: FaucetError) -> ServerFnError {
    match err {
        FaucetError::RateLimited { retry_after_secs } => {
            log::warn!("Rate limit exceeded: retry_after_secs={}", retry_after_secs);
            set_response_status(StatusCode::TOO_MANY_REQUESTS);
            ServerFnError::ServerError(format!(
                "Too many requests: Rate limited. Try again in {} seconds.",
                retry_after_secs
            ))
        }
        FaucetError::Server(msg) => {
            log::error!("Failed to drip tokens: {}", msg);
            set_response_status(StatusCode::INTERNAL_SERVER_ERROR);
            ServerFnError::ServerError(format!("Server error: {}", msg))
        }
    }
}

#[cfg(feature = "ssr")]
fn set_response_status(status: StatusCode) {
    leptos::context::use_context::<ResponseOptions>().map(|res| res.set_status(status));
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    #[allow(unused_macros)]
    macro_rules! assert_address {
        ($address:expr, $faucet:expr) => {{
            let network = $faucet.network();
            let addr = crate::utils::address::parse_address($address, network).unwrap();
            assert!(crate::faucet::server_api::check_valid_address(addr, $faucet).is_ok());
            assert!(
                crate::faucet::server_api::parse_and_validate_address($address, $faucet).is_ok()
            );
        }};
        ($address:expr, $faucet:expr, false) => {{
            let network = $faucet.network();
            let addr = crate::utils::address::parse_address($address, network).unwrap();
            assert!(crate::faucet::server_api::check_valid_address(addr, $faucet).is_err());
            assert!(
                crate::faucet::server_api::parse_and_validate_address($address, $faucet).is_err()
            );
        }};
    }

    #[test]
    fn test_check_valid_address_mainnet() {
        let addresses = [
            "f03603846",
            "f1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy",
            "f2yjb6dq3jggychgnuhevcwe7ehv3ot2rkhkbk4qy",
            "f3s5kg6rehbbmgvngpec6b7m4uxmwbscdafn2pvtrrp65wbgjuymrr2z6qbkqiunkyjul6b62buqk76q47cjeq",
            "f410fv2oexfiizeuzm3xtoie3gnxfpfwwglg4q3dgxki",
            "0xff0000000000000000000000000000000036fd86",
            "0xAe9C4b9508c929966ef37209b336E5796D632CDc",
        ];
        for addr in addresses.iter() {
            assert_address!(*addr, crate::faucet::server_api::FaucetInfo::MainnetFIL);
        }
    }

    #[test]
    fn test_check_valid_address_calibnet() {
        let addresses = [
            "t03603846",
            "t1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy",
            "t2yjb6dq3jggychgnuhevcwe7ehv3ot2rkhkbk4qy",
            "t3s5kg6rehbbmgvngpec6b7m4uxmwbscdafn2pvtrrp65wbgjuymrr2z6qbkqiunkyjul6b62buqk76q47cjeq",
            "t410fv2oexfiizeuzm3xtoie3gnxfpfwwglg4q3dgxki",
            "0xff0000000000000000000000000000000036f672",
            "0xAe9C4b9508c929966ef37209b336E5796D632CDc",
        ];
        for addr in addresses.iter() {
            assert_address!(*addr, crate::faucet::server_api::FaucetInfo::CalibnetFIL);
        }
    }

    #[test]
    fn test_check_valid_address_calibnet_usdfc() {
        let valid_addresses = [
            "0xAe9C4b9508c929966ef37209b336E5796D632CDc",
            "t410fv2oexfiizeuzm3xtoie3gnxfpfwwglg4q3dgxki",
        ];
        let invalid_addresses = [
            "t03603846",
            "t1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy",
            "t2yjb6dq3jggychgnuhevcwe7ehv3ot2rkhkbk4qy",
            "t3s5kg6rehbbmgvngpec6b7m4uxmwbscdafn2pvtrrp65wbgjuymrr2z6qbkqiunkyjul6b62buqk76q47cjeq",
            "0xff0000000000000000000000000000000036f672",
        ];

        for addr in valid_addresses.iter() {
            assert_address!(*addr, crate::faucet::server_api::FaucetInfo::CalibnetUSDFC);
        }

        for addr in invalid_addresses.iter() {
            assert_address!(
                *addr,
                crate::faucet::server_api::FaucetInfo::CalibnetUSDFC,
                false
            );
        }
    }
}
