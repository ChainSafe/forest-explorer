use leptos::prelude::{RwSignal, ServerFnError, Update};
use leptos::server_fn::codec::JsonEncoding;
use leptos::server_fn::error::{FromServerFnError, ServerFnErrorErr};
use serde::{Deserialize, Serialize};
use std::future::Future;
use thiserror::Error;
use uuid::Uuid;

/// This enum represents all possible errors that can occur in the faucet system,
/// including rate limiting and other server errors.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum FaucetError {
    /// Returned when a request is rate limited. Contains the number of seconds to wait before retrying.
    #[error("Rate limited. Try again in {retry_after_secs} seconds.")]
    RateLimited { retry_after_secs: i32 },
    /// Represents a server-side error with a message.
    #[error("Server error: {0}")]
    Server(String),
}

impl FromServerFnError for FaucetError {
    type Encoder = JsonEncoding;
    fn from_server_fn_error(err: ServerFnErrorErr) -> Self {
        FaucetError::Server(err.to_string())
    }
}

impl From<ServerFnError> for FaucetError {
    fn from(e: leptos::prelude::ServerFnError) -> Self {
        FaucetError::Server(e.to_string())
    }
}

pub async fn catch_all(
    errors: RwSignal<Vec<(Uuid, String)>>,
    cb: impl Future<Output = Result<(), anyhow::Error>>,
) {
    match cb.await {
        Ok(_) => (),
        Err(e) => errors.update(|errors| errors.push((Uuid::new_v4(), e.to_string()))),
    }
}
