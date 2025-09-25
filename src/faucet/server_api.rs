//! This file contains the server-side API for the faucet functionality. More fine grained,
//! server-side functions (that are not exposed to the client) are in the `server` module.

use crate::utils::{
    address::AnyAddress,
    lotus_json::{LotusJson, signed_message::SignedMessage},
};
use anyhow::Result;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use leptos::server_fn::codec::Json;
use leptos::{prelude::ServerFnError, server};

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

#[server(endpoint = "claim_token", input = Json, output = Json)]
pub async fn claim_token(
    faucet_info: FaucetInfo,
    address: String,
) -> Result<String, ServerFnError> {
    use crate::utils::address::{AddressAlloyExt, parse_address};
    use crate::utils::message::message_transfer;
    use crate::utils::rpc_context::Provider;
    use fvm_shared::address::Network;
    use fvm_shared::address::set_current_network;
    use send_wrapper::SendWrapper;

    SendWrapper::new(async move {
        match faucet_info {
            FaucetInfo::MainnetFIL => {
                return Err(ServerFnError::ServerError(
                    "Mainnet token claim is not supported.".to_string(),
                ));
            }
            FaucetInfo::CalibnetFIL => {
                let network = Network::Testnet;
                set_current_network(network);
                let recipient = parse_address(&address, network).map_err(ServerFnError::new)?;
                let rpc = Provider::from_network(network);
                let id_address = rpc.lookup_id(recipient).await.map_err(ServerFnError::new)?;
                let from = faucet_address(faucet_info)
                    .await
                    .map_err(ServerFnError::new)?
                    .to_filecoin_address(network)
                    .map_err(ServerFnError::new)?;
                let nonce = rpc
                    .mpool_get_nonce(from)
                    .await
                    .map_err(ServerFnError::new)?;
                let raw_msg = message_transfer(from, id_address, faucet_info.drip_amount().clone());
                let msg = rpc
                    .estimate_gas(raw_msg)
                    .await
                    .map_err(ServerFnError::new)?;
                let LotusJson(signed) = signed_fil_transfer(
                    LotusJson(id_address),
                    msg.gas_limit,
                    LotusJson(msg.gas_fee_cap),
                    LotusJson(msg.gas_premium),
                    nonce,
                    faucet_info,
                )
                .await?;
                let tx_id = rpc.mpool_push(signed).await.map_err(ServerFnError::new)?;
                Ok(tx_id.to_string())
            }
            FaucetInfo::CalibnetUSDFC => {
                let network = Network::Testnet;
                set_current_network(network);
                let recipient = parse_address(&address, network).map_err(ServerFnError::new)?;
                let rpc = Provider::from_network(network);
                let owner_fil_address = faucet_address(faucet_info)
                    .await
                    .map_err(ServerFnError::new)?
                    .to_filecoin_address(network)
                    .map_err(ServerFnError::new)?;
                let eth_to = recipient.into_eth_address().map_err(ServerFnError::new)?;
                let nonce = rpc
                    .mpool_get_nonce(owner_fil_address)
                    .await
                    .map_err(ServerFnError::new)?;
                let gas_price = rpc.gas_price().await.map_err(ServerFnError::new)?;
                let signed = signed_erc20_transfer(eth_to, nonce, gas_price, faucet_info).await?;
                let tx_id = rpc
                    .send_eth_transaction_signed(&signed)
                    .await
                    .map_err(ServerFnError::new)?;
                Ok(tx_id.to_string())
            }
        }
    })
    .await
}
