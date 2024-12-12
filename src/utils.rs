use std::future::Future;

use leptos::{RwSignal, SignalUpdate as _};

pub async fn catch_all(
    errors: RwSignal<Vec<String>>,
    cb: impl Future<Output = Result<(), anyhow::Error>>,
) {
    match cb.await {
        Ok(_) => (),
        Err(e) => errors.update(|errors| errors.push(e.to_string())),
    }
}
