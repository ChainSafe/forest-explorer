use std::future::Future;

use leptos::prelude::{RwSignal, Update};
use uuid::Uuid;

pub async fn catch_all(
    errors: RwSignal<Vec<(Uuid, String)>>,
    cb: impl Future<Output = Result<(), anyhow::Error>>,
) {
    match cb.await {
        Ok(_) => (),
        Err(e) => errors.update(|errors| errors.push((Uuid::new_v4(), e.to_string()))),
    }
}
