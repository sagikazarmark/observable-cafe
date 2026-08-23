//! The calls the browser makes into the café.
//!
//! The bodies only exist in the server build; on the client the same functions
//! turn into an HTTP request to the matching endpoint.

use dioxus::prelude::*;

use crate::inventory::Ingredient;
use crate::state::{Order, Snapshot};

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
///
/// The answer says what came of asking as well as how the café now stands: a
/// café keeping a shelf can refuse a drink it cannot cover, and the page has
/// to be able to say so.
#[server]
pub async fn buy(drink: usize) -> ServerFnResult<(Order, Snapshot)> {
    Ok(crate::server::buy(drink))
}

/// Reports the café to the back room, without disturbing it.
///
/// The back room is an observer rather than a visitor: looking at the shelf
/// does not start the clock, and it has no say in how often the café is
/// written down — the interval belongs to whoever is reading the café page.
#[server]
pub async fn stocktake() -> ServerFnResult<Snapshot> {
    Ok(crate::server::snapshot())
}

/// Takes in one delivery of an ingredient, and says how much of it fit.
///
/// A café keeping no shelf takes nothing in, however the endpoint is called:
/// a delivery needs somewhere to land.
#[server]
pub async fn restock(ingredient: Ingredient) -> ServerFnResult<(u32, Snapshot)> {
    Ok(crate::server::restock(ingredient))
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
