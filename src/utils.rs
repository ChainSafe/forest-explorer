use std::future::Future;

use leptos::prelude::{RwSignal, Update};

pub async fn catch_all(
    errors: RwSignal<Vec<String>>,
    cb: impl Future<Output = Result<(), anyhow::Error>>,
) {
    match cb.await {
        Ok(_) => (),
        Err(e) => errors.update(|errors| errors.push(e.to_string())),
    }
}
