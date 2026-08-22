//! The calls the browser makes into the café.
//!
//! The bodies only exist in the server build; on the client the same functions
//! turn into an HTTP request to the matching endpoint.

use dioxus::prelude::*;

use crate::state::Snapshot;

/// Reports the café as it stands, together with what has been written down.
///
/// This is also what starts the clock: the café stands at opening time until
/// somebody is actually looking at it.
///
/// `observe_every` rides along on every poll rather than being sent once,
/// because it belongs to whoever is reading rather than to the café. It
/// changes how often the café is written down, not the café.
#[server]
pub async fn snapshot(observe_every: u64) -> ServerFnResult<Snapshot> {
    Ok(crate::server::snapshot_for(observe_every))
}

/// Rings up a single coffee, identified by its position on the menu.
#[server]
pub async fn buy(drink: usize) -> ServerFnResult<Snapshot> {
    Ok(crate::server::buy(drink))
}

/// Asks the owner to write an entry now instead of at the next interval.
#[server]
pub async fn note() -> ServerFnResult<Snapshot> {
    Ok(crate::server::note())
}

/// Puts the café back to how it opened.
#[server]
pub async fn reset() -> ServerFnResult<Snapshot> {
    Ok(crate::server::reset())
}
