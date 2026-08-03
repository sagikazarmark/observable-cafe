//! The café itself.
//!
//! State lives here rather than in the browser so that every visitor sees the
//! same counter and the same readings — and so that `/metrics` has something
//! to report.

mod metrics;
mod rng;
mod simulation;

use std::sync::{LazyLock, Mutex, MutexGuard};

use dioxus::server::axum::routing::get;

use crate::state::Snapshot;
use simulation::Thermometer;

/// The one and only café. It lasts as long as the process does; restarting the
/// server is the only thing besides the reset button that clears it.
static CAFE: LazyLock<Mutex<Cafe>> = LazyLock::new(|| Mutex::new(Cafe::new()));

struct Cafe {
    coffees_sold: u32,
    inside: Thermometer,
    outside: Thermometer,
}

impl Cafe {
    fn new() -> Self {
        Self {
            coffees_sold: 0,
            inside: Thermometer::inside(),
            outside: Thermometer::outside(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            coffees_sold: self.coffees_sold,
            inside: self.inside.reading().clone(),
            outside: self.outside.reading().clone(),
        }
    }
}

/// Nothing done under this lock can panic, so a poisoned lock is not worth
/// taking the café down for.
fn cafe() -> MutexGuard<'static, Cafe> {
    CAFE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn snapshot() -> Snapshot {
    cafe().snapshot()
}

pub fn buy_coffee() -> Snapshot {
    let mut cafe = cafe();
    cafe.coffees_sold += 1;

    cafe.snapshot()
}

pub fn reset() -> Snapshot {
    let mut cafe = cafe();
    *cafe = Cafe::new();

    cafe.snapshot()
}

/// Serves the app, its server functions and the scrape endpoint.
pub fn launch() -> ! {
    dioxus::serve(|| async move {
        simulation::start();

        let router =
            dioxus::server::router(crate::app::App).route("/metrics", get(metrics::scrape));

        Ok(router)
    })
}
