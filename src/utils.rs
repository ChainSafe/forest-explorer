use std::future::Future;

use fvm_shared::address::{Address, Network};
use leptos::{RwSignal, SignalUpdate as _};

pub fn parse_address(s: &str) -> anyhow::Result<(Address, Network)> {
    Ok(Network::Testnet
        .parse_address(s)
        .map(|addr| (addr, Network::Testnet))
        .or_else(|_| {
            Network::Mainnet
                .parse_address(s)
                .map(|addr| (addr, Network::Mainnet))
        })?)
}

pub async fn catch_all(
    errors: RwSignal<Vec<String>>,
    cb: impl Future<Output = Result<(), anyhow::Error>>,
) {
    match cb.await {
        Ok(_) => (),
        Err(e) => errors.update(|errors| errors.push(e.to_string())),
    }
}
