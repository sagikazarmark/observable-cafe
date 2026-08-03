//! The calls the browser makes into the café.
//!
//! The bodies only exist in the server build; on the client the same functions
//! turn into an HTTP request to the matching endpoint.

use dioxus::prelude::*;

use crate::state::Snapshot;

/// Reports everything the café has observed so far.
#[server]
pub async fn observe() -> ServerFnResult<Snapshot> {
    Ok(crate::server::snapshot())
}

/// Rings up a single coffee.
#[server]
pub async fn buy_coffee() -> ServerFnResult<Snapshot> {
    Ok(crate::server::buy_coffee())
}

/// Puts the café back to how it opened.
#[server]
pub async fn reset() -> ServerFnResult<Snapshot> {
    Ok(crate::server::reset())
}
