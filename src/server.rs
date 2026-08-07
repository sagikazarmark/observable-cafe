//! The café itself.
//!
//! State lives here rather than in the browser so that every visitor sees the
//! same counter and the same readings — and so that `/metrics` has something
//! to report.

mod metrics;
mod rng;
mod simulation;

use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};

use dioxus::server::axum::{Router, http::StatusCode, routing::get};

use crate::clock;
use crate::stage::Stage;
use crate::state::{Observation, Sales, Snapshot};
use simulation::Thermometer;

/// How many observations the notebook keeps: an hour of café time, five
/// minutes at a time, which is a few minutes of anybody's attention.
const NOTEBOOK_LIMIT: usize = 60;

/// The one and only café. It lasts as long as the process does; restarting the
/// server is the only thing besides the reset button that clears it.
static CAFE: LazyLock<Mutex<Cafe>> = LazyLock::new(|| Mutex::new(Cafe::new(None, true)));
static SINGLE_STAGE: OnceLock<Option<Stage>> = OnceLock::new();

struct Cafe {
    /// Whether the clock writes observations as their interval comes due.
    automatic_observations: bool,
    /// The stage the café is currently set up for.
    ///
    /// `None` until somebody opens one, which is also why the clock has not
    /// started: an unvisited café should still read 08:00 when it is first
    /// looked at, not whatever time the server happened to boot.
    stage: Option<Stage>,
    /// Seconds of real time since opening, and so minutes of café time.
    tick: u64,
    /// Ticks between entries, as whoever is reading has asked for.
    ///
    /// Changing this does not disturb the café: the notebook simply carries on
    /// at the new spacing, which is what makes the two resolutions comparable
    /// in one record.
    observe_every: u64,
    /// How many entries have been written since opening, so each can be told
    /// apart from the one that took its place on the page.
    written: u64,
    /// The tick the last observation was written at, whether by the clock or
    /// by hand. Asking for one by hand therefore restarts the wait for the
    /// next, rather than producing two entries moments apart.
    last_observed: u64,
    sold: Sales,
    inside: Thermometer,
    outside: Thermometer,
    notebook: VecDeque<Observation>,
}

impl Cafe {
    fn new(stage: Option<Stage>, automatic_observations: bool) -> Self {
        let opening = clock::opening();

        Self {
            automatic_observations,
            stage,
            tick: 0,
            observe_every: clock::DEFAULT_OBSERVE_EVERY,
            written: 0,
            last_observed: 0,
            sold: Sales::default(),
            inside: Thermometer::inside(opening),
            outside: Thermometer::outside(opening),
            notebook: VecDeque::new(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        let now = clock::at(self.tick);

        Snapshot {
            automatic_observations: self.automatic_observations,
            clock: clock::written(now),
            day: clock::dated(now),
            sold: self.sold,
            inside: self.inside.reading().clone(),
            outside: self.outside.reading().clone(),
            observations: self.notebook.iter().cloned().collect(),
        }
    }

    /// Writes down everything as it stands, and forgets the oldest entry once
    /// the notebook is full.
    fn observe(&mut self) {
        self.last_observed = self.tick;
        self.written += 1;

        let now = clock::at(self.tick);
        self.notebook.push_back(Observation {
            seq: self.written,
            at: clock::written(now),
            day: clock::dated(now),
            sold: self.sold,
            inside: self.inside.reading().value(),
            outside: self.outside.reading().value(),
        });

        if self.notebook.len() > NOTEBOOK_LIMIT {
            self.notebook.pop_front();
        }
    }

    fn observation_due(&self) -> bool {
        self.automatic_observations && self.tick - self.last_observed >= self.observe_every
    }
}

/// Nothing done under this lock can panic, so a poisoned lock is not worth
/// taking the café down for.
fn cafe() -> MutexGuard<'static, Cafe> {
    CAFE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Reports the café without disturbing it, for `/metrics`.
///
/// A scrape is an observer, not a visitor: it neither starts the clock nor
/// decides which stage the café is set up for.
pub fn snapshot() -> Snapshot {
    cafe().snapshot()
}

/// The only stage this process serves, or `None` for the stage index.
pub fn single_stage() -> Option<Stage> {
    SINGLE_STAGE.get().copied().flatten()
}

/// Reports the café to a page showing `stage`, setting it up for that stage
/// first if it is not already.
///
/// Opening a different stage puts the café back to opening time, because the
/// stages are meant to be arrived at fresh — but reloading the one already
/// showing leaves everything alone. Nothing else starts the clock, so it does
/// not begin until somebody is actually looking.
pub fn snapshot_for(stage: Stage, observe_every: u64) -> Snapshot {
    // Idempotent, so the poll that arrives every second only starts it once.
    simulation::start();

    let mut cafe = cafe();
    if cafe.stage != Some(stage) {
        *cafe = Cafe::new(Some(stage), cafe.automatic_observations);
    }

    // Applied after any rebuild, and on its own account: the interval belongs
    // to whoever is reading rather than to the café, so setting it leaves the
    // counter, the thermometers and the notebook exactly where they were.
    cafe.observe_every = clock::observe_every(observe_every);

    cafe.snapshot()
}

/// Rings up one drink, identified by its position on the menu.
///
/// Nothing is written down here. The sale is real the instant it happens and
/// `/metrics` will say so, but the notebook will not hear about it until the
/// owner next looks up.
pub fn buy(drink: usize) -> Snapshot {
    let mut cafe = cafe();
    cafe.sold.ring_up(drink);

    cafe.snapshot()
}

/// Writes an entry now, rather than waiting for the next one to come round.
pub fn note() -> Snapshot {
    let mut cafe = cafe();
    cafe.observe();

    cafe.snapshot()
}

pub fn reset() -> Snapshot {
    let mut cafe = cafe();
    *cafe = Cafe::new(cafe.stage, cafe.automatic_observations);

    cafe.snapshot()
}

/// Serves the app, its server functions and the scrape endpoint.
///
/// The simulation is deliberately not started here — the café stands at
/// opening time until somebody opens a stage.
pub fn launch(automatic_observations: bool, stage: Option<Stage>) -> ! {
    SINGLE_STAGE
        .set(stage)
        .expect("server configuration must only be set once");
    cafe().automatic_observations = automatic_observations;

    dioxus::serve(move || async move {
        let router = if stage.is_some() {
            single_stage_router()
        } else {
            dioxus::server::router(crate::app::App).route("/metrics", get(metrics::scrape))
        };

        Ok(router)
    })
}

/// Serves one stage at the root without installing the multi-stage fallback.
fn single_stage_router() -> Router {
    dioxus::server::router(crate::app::App)
        .route("/metrics", get(metrics::scrape))
        .route("/{*path}", get(|| async { StatusCode::NOT_FOUND }))
}

#[cfg(test)]
mod tests {
    use super::Cafe;

    #[test]
    fn disabled_automatic_observations_never_become_due() {
        let mut cafe = Cafe::new(None, false);
        cafe.tick = cafe.observe_every;

        assert!(!cafe.observation_due());
        assert!(!cafe.snapshot().automatic_observations);
    }

    #[test]
    fn enabled_automatic_observations_become_due_at_the_interval() {
        let mut cafe = Cafe::new(None, true);
        cafe.tick = cafe.observe_every;

        assert!(cafe.observation_due());
    }
}
