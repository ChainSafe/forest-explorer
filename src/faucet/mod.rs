mod controller;
mod model;
#[cfg(feature = "ssr")]
mod rate_limiter;

pub mod constants;

#[cfg(feature = "ssr")]
mod server;

// This needs to be public so that the client-side can seamlessly call the server functions.
pub mod server_api;

pub mod views;
