// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use leptos::prelude::{RwSignal, Update};
use std::future::Future;
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
