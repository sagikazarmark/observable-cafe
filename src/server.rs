//! The café itself.
//!
//! State lives here rather than in the browser so that every visitor sees the
//! same counter and the same notebook, and so that `/metrics` has something
//! to report.

mod metrics;
mod rng;
mod simulation;
mod version;

use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, MutexGuard};

use dioxus::server::axum::routing::get;

use crate::clock;
use crate::feature::Features;
use crate::state::{Observation, Sale, Sales, Snapshot};
use simulation::Thermometer;

/// How many observations the notebook keeps: an hour of café time, five
/// minutes at a time, which is a few minutes of anybody's attention.
const NOTEBOOK_LIMIT: usize = 60;

/// How many sales the roll keeps. The same bound for the same reason: a
/// café left running all afternoon should not grow without limit.
const SALES_LIMIT: usize = 60;

/// The one and only café. It lasts as long as the process does; restarting the
/// server is the only thing besides the reset button that clears it.
static CAFE: LazyLock<Mutex<Cafe>> = LazyLock::new(|| Mutex::new(Cafe::new(Features::all())));

struct Cafe {
    /// What this café shows, as it was told at startup.
    ///
    /// Kept here rather than in a static of its own so that the parts of the
    /// café that depend on it can be built and tested a café at a time.
    features: Features,
    /// Seconds of real time since opening, and so minutes of café time.
    ///
    /// Stays at zero until somebody opens the page: an unvisited café should
    /// read 08:00 when it is first looked at, rather than whatever time the
    /// process happened to drift to.
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
    /// How many coffees have been rung up since opening, which numbers the
    /// sales the same way `written` numbers the notebook.
    rung_up: u64,
    sold: Sales,
    inside: Thermometer,
    outside: Thermometer,
    notebook: VecDeque<Observation>,
    sales: VecDeque<Sale>,
}

impl Cafe {
    fn new(features: Features) -> Self {
        let opening = clock::opening();

        Self {
            features,
            tick: 0,
            observe_every: clock::DEFAULT_OBSERVE_EVERY,
            written: 0,
            last_observed: 0,
            rung_up: 0,
            sold: Sales::default(),
            inside: Thermometer::inside(opening),
            outside: Thermometer::outside(opening),
            notebook: VecDeque::new(),
            sales: VecDeque::new(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        let now = clock::at(self.tick);

        Snapshot {
            clock: clock::written(now),
            day: clock::dated(now),
            sold: self.sold,
            inside: self.inside.reading().clone(),
            outside: self.outside.reading().clone(),
            observations: self.notebook.iter().cloned().collect(),
            sales: self.sales.iter().cloned().collect(),
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

    /// Rings up one drink and writes it up on the roll.
    ///
    /// The sale is kept whether or not this café shows its sales. It happened;
    /// what a page does with it is the page's business, and a café that kept
    /// its records according to who was watching would be a poor example of
    /// anything.
    fn ring_up(&mut self, drink: usize) {
        // A café cannot sell what is not on its menu. Nothing is counted and
        // nothing is written up, so the sales, the notebook and `/metrics`
        // cannot come to disagree about how many coffees there have been.
        if !self.sold.ring_up(drink) {
            return;
        }

        self.rung_up += 1;

        let now = clock::at(self.tick);
        self.sales.push_back(Sale::rung_up(
            self.rung_up,
            clock::written(now),
            clock::dated(now),
            drink,
        ));

        if self.sales.len() > SALES_LIMIT {
            self.sales.pop_front();
        }
    }

    /// Whether the clock owes the notebook an entry.
    ///
    /// A café that keeps no notebook is never owed one, however the timer was
    /// asked for: turning observations off turns them off altogether.
    fn observation_due(&self) -> bool {
        self.features.automatic_observations && self.tick - self.last_observed >= self.observe_every
    }
}

/// Nothing done under this lock can panic, so a poisoned lock is not worth
/// taking the café down for.
fn cafe() -> MutexGuard<'static, Cafe> {
    CAFE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Reports the café without disturbing it, for `/metrics`.
///
/// A scrape is an observer, not a visitor: it does not start the clock.
pub fn snapshot() -> Snapshot {
    cafe().snapshot()
}

/// What this café shows.
///
/// Handed to the page as it is rendered rather than asked for afterwards: it
/// is settled before the process starts serving and never changes.
pub fn features() -> Features {
    cafe().features
}

/// Reports the café to somebody who is looking at it, and starts the clock if
/// this is the first such person.
///
/// Nothing else starts it, so an unvisited café still opens at 08:00.
///
/// `observe_every` rides along because the interval belongs to whoever is
/// reading rather than to the café: setting it leaves the counter, the
/// thermometers and the notebook exactly where they were.
pub fn snapshot_for(observe_every: u64) -> Snapshot {
    // Idempotent, so the poll that arrives every second only starts it once.
    simulation::start();

    let mut cafe = cafe();
    cafe.observe_every = clock::observe_every(observe_every);

    cafe.snapshot()
}

/// Rings up one drink, identified by its position on the menu.
///
/// Nothing is written in the notebook here. The sale is real the instant it
/// happens and `/metrics` will say so, but the notebook will not hear about it
/// until the owner next looks up.
pub fn buy(drink: usize) -> Snapshot {
    let mut cafe = cafe();
    cafe.ring_up(drink);

    cafe.snapshot()
}

/// Writes an entry now, rather than waiting for the next one to come round.
///
/// A café keeping no notebook writes nothing: the endpoint is still reachable
/// by anything that cares to call it, and answering it with an entry would
/// undo the one thing that café was asked to demonstrate.
pub fn note() -> Snapshot {
    let mut cafe = cafe();
    if cafe.features.observations {
        cafe.observe();
    }

    cafe.snapshot()
}

pub fn reset() -> Snapshot {
    let mut cafe = cafe();
    *cafe = Cafe::new(cafe.features);

    cafe.snapshot()
}

/// Serves the café, its server functions and the endpoints beside them.
///
/// The simulation is deliberately not started here; the café stands at opening
/// time until somebody looks at it.
pub fn launch(features: Features) -> ! {
    cafe().features = features;

    dioxus::serve(move || async move {
        Ok(dioxus::server::router(crate::app::App)
            // Asked for by machines that do not know or care what the page is
            // showing, so they are the same whatever it was configured to show.
            .route("/metrics", get(metrics::scrape))
            .route("/version", get(version::report)))
    })
}

#[cfg(test)]
mod tests {
    use super::Cafe;
    use crate::feature::{Feature, Features};

    fn cafe_showing(disabled: &[Feature]) -> Cafe {
        Cafe::new(Features::resolve(None, &[], disabled).expect("nothing contradicts"))
    }

    #[test]
    fn disabled_automatic_observations_never_become_due() {
        let mut cafe = cafe_showing(&[Feature::AutomaticObservations]);
        cafe.tick = cafe.observe_every;

        assert!(!cafe.observation_due());
    }

    /// Turning the timer off leaves the notebook to whoever is reading, rather
    /// than closing it.
    #[test]
    fn an_entry_can_still_be_asked_for_by_hand() {
        let mut cafe = cafe_showing(&[Feature::AutomaticObservations]);
        cafe.observe();

        assert_eq!(cafe.snapshot().observations.len(), 1);
    }

    /// Turning observations off closes the notebook, so the timer has nothing
    /// to write in even though nobody said anything about the timer.
    #[test]
    fn disabled_observations_stop_the_timer_too() {
        let mut cafe = cafe_showing(&[Feature::Observations]);
        cafe.tick = cafe.observe_every;

        assert!(!cafe.observation_due());
    }

    #[test]
    fn enabled_automatic_observations_become_due_at_the_interval() {
        let mut cafe = cafe_showing(&[]);
        cafe.tick = cafe.observe_every;

        assert!(cafe.observation_due());
    }

    /// The sale happened; a café that kept its records according to who was
    /// watching would be a poor example of anything.
    #[test]
    fn sales_are_written_up_whether_or_not_they_are_shown() {
        let mut cafe = cafe_showing(&[Feature::Sales]);
        cafe.ring_up(0);
        cafe.ring_up(2);

        let sales = cafe.snapshot().sales;

        assert_eq!(sales.len(), 2);
        assert_eq!(sales[0].seq, 1);
        assert_eq!(sales[1].drink().map(|drink| drink.key), Some("latte"));
    }

    /// A café left running all afternoon should not grow without limit.
    #[test]
    fn the_roll_keeps_only_the_most_recent_sales() {
        let mut cafe = cafe_showing(&[]);
        for _ in 0..super::SALES_LIMIT + 5 {
            cafe.ring_up(0);
        }

        let sales = cafe.snapshot().sales;

        assert_eq!(sales.len(), super::SALES_LIMIT);
        assert_eq!(sales[0].seq, 6);
    }

    /// The sales, the notebook and `/metrics` are one sale seen three ways.
    /// A sale the total behind it does not have would have the café lie
    /// about the very thing it is demonstrating, so an order for something the
    /// café does not sell rings up nothing at all.
    #[test]
    fn a_drink_that_is_not_on_the_menu_is_not_sold() {
        let mut cafe = cafe_showing(&[]);
        cafe.ring_up(0);
        cafe.ring_up(crate::menu::MENU.len());

        let snapshot = cafe.snapshot();

        assert_eq!(snapshot.sold.total(), 1);
        assert_eq!(snapshot.sales.len(), 1);
        assert_eq!(snapshot.sales[0].seq, 1);
    }

    /// A page left open passes midnight, and a sale reading `00:03` beside
    /// one reading `23:58` is two days rather than five minutes.
    #[test]
    fn a_sale_records_the_day_it_was_rung_up_on() {
        let mut cafe = cafe_showing(&[]);
        cafe.ring_up(0);

        let opening = cafe.snapshot();

        // The café opens at 08:00 and a tick is a minute, so midnight is
        // sixteen hours of café time after opening.
        cafe.tick = 16 * 60;
        cafe.ring_up(0);

        let sales = cafe.snapshot().sales;

        assert_eq!(sales[0].day, opening.day);
        assert_ne!(sales[1].day, sales[0].day);
    }
}
