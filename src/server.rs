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
use crate::inventory::{Ingredient, Inventory};
use crate::menu::MENU;
use crate::metric::Metrics;
use crate::state::{Observation, Order, Sale, Sales, Snapshot};
use simulation::Thermometer;

/// How many observations the notebook keeps: an hour of café time, five
/// minutes at a time, which is a few minutes of anybody's attention.
const NOTEBOOK_LIMIT: usize = 60;

/// How many sales the roll keeps. The same bound for the same reason: a
/// café left running all afternoon should not grow without limit.
const SALES_LIMIT: usize = 60;

/// The one and only café. It lasts as long as the process does; restarting the
/// server is the only thing besides the reset button that clears it.
static CAFE: LazyLock<Mutex<Cafe>> =
    LazyLock::new(|| Mutex::new(Cafe::new(Features::all(), Metrics::all())));

struct Cafe {
    /// What this café shows, as it was told at startup.
    ///
    /// Kept here rather than in a static of its own so that the parts of the
    /// café that depend on it can be built and tested a café at a time.
    features: Features,
    /// What this café measures, told at startup the same way.
    ///
    /// Kept so the café outlives a reset with the same instruments: the till
    /// and the thermometers below are built from it and speak for themselves
    /// afterwards.
    metrics: Metrics,
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
    /// The till's running count, in a café that keeps one.
    ///
    /// `None` is no counter rather than nothing sold yet: a café not
    /// measuring its sales has no number for anything to report, however many
    /// coffees it makes.
    sold: Option<Sales>,
    /// The shelf, in a café that keeps one.
    ///
    /// `None` is no shelf rather than an empty one: a café without the
    /// feature brews from nothing and can never run out.
    inventory: Option<Inventory>,
    /// The thermometers, in a café that reads them. One instrument, so the
    /// two are present or absent together.
    inside: Option<Thermometer>,
    outside: Option<Thermometer>,
    notebook: VecDeque<Observation>,
    sales: VecDeque<Sale>,
}

impl Cafe {
    fn new(features: Features, metrics: Metrics) -> Self {
        let opening = clock::opening();

        Self {
            features,
            metrics,
            tick: 0,
            observe_every: clock::DEFAULT_OBSERVE_EVERY,
            written: 0,
            last_observed: 0,
            rung_up: 0,
            sold: metrics.coffees_sold.then(|| Sales::new(metrics.drink)),
            inventory: features.inventory.then(Inventory::full),
            inside: metrics.temperatures.then(|| Thermometer::inside(opening)),
            outside: metrics.temperatures.then(|| Thermometer::outside(opening)),
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
            inside: self.inside.as_ref().map(|inside| inside.reading().clone()),
            outside: self
                .outside
                .as_ref()
                .map(|outside| outside.reading().clone()),
            observations: self.notebook.iter().cloned().collect(),
            sales: self.sales.iter().cloned().collect(),
            inventory: self.inventory,
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
            inside: self.inside.as_ref().map(|inside| inside.reading().value()),
            outside: self
                .outside
                .as_ref()
                .map(|outside| outside.reading().value()),
            inventory: self.inventory,
        });

        if self.notebook.len() > NOTEBOOK_LIMIT {
            self.notebook.pop_front();
        }
    }

    /// Rings up one drink and writes it up on the roll, or refuses it and
    /// says why.
    ///
    /// The sale is kept whether or not this café shows its sales. It happened;
    /// what a page does with it is the page's business, and a café that kept
    /// its records according to who was watching would be a poor example of
    /// anything.
    fn ring_up(&mut self, drink: usize) -> Order {
        // A café cannot sell what is not on its menu. Nothing is counted and
        // nothing is written up, so the sales, the notebook and `/metrics`
        // cannot come to disagree about how many coffees there have been.
        let Some(ordered) = MENU.get(drink) else {
            return Order::OffMenu;
        };

        // The shelf has the first word, and refusing is all-or-nothing: a
        // drink the shelf cannot cover is not counted, not written up, and
        // not reported, so every record agrees no coffee was made.
        if let Some(shelf) = &mut self.inventory
            && let Err(short) = shelf.brew(ordered.recipe)
        {
            return Order::OutOf(short);
        }

        // Counted where there is a counter to count it. The sale happens
        // either way: a café that does not measure its sales still makes the
        // coffee, it just has no number that knows about it.
        if let Some(sold) = &mut self.sold
            && !sold.ring_up(drink)
        {
            return Order::OffMenu;
        }

        self.rung_up += 1;

        let now = clock::at(self.tick);
        self.sales.push_back(Sale::rung_up(
            self.rung_up,
            clock::written(now),
            clock::dated(now),
            // Noted only where the café measures the dimension: a till that
            // does not record which drink writes up "a coffee", and there is
            // nothing behind it to remember.
            self.metrics.drink.then_some(drink),
        ));

        if self.sales.len() > SALES_LIMIT {
            self.sales.pop_front();
        }

        Order::Served
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

/// Rings up one drink, identified by its position on the menu, and says what
/// came of asking.
///
/// Nothing is written in the notebook here. The sale is real the instant it
/// happens and `/metrics` will say so, but the notebook will not hear about it
/// until the owner next looks up.
pub fn buy(drink: usize) -> (Order, Snapshot) {
    let mut cafe = cafe();
    let order = cafe.ring_up(drink);

    (order, cafe.snapshot())
}

/// Takes in one delivery of `ingredient`, and says how much of it fit.
///
/// A café keeping no shelf takes nothing in: the endpoint is reachable by
/// anything that cares to call it, and conjuring a shelf for a delivery to
/// land on would give the café stock it has nowhere to keep.
pub fn restock(ingredient: Ingredient) -> (u32, Snapshot) {
    let mut cafe = cafe();
    let taken = match &mut cafe.inventory {
        Some(shelf) => shelf.take_delivery(ingredient),
        None => 0,
    };

    (taken, cafe.snapshot())
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
    *cafe = Cafe::new(cafe.features, cafe.metrics);

    cafe.snapshot()
}

/// Serves the café, its server functions and the endpoints beside them.
///
/// The simulation is deliberately not started here; the café stands at opening
/// time until somebody looks at it.
pub fn launch(features: Features, metrics: Metrics) -> ! {
    // The whole café rather than just the fields: what instruments it has and
    // whether there is a shelf behind the counter follow from what was asked
    // for.
    *cafe() = Cafe::new(features, metrics);

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
    use crate::inventory::{Ingredient, Inventory, Roast};
    use crate::menu::MENU;
    use crate::metric::{Metric, Metrics};
    use crate::state::Order;

    fn cafe_showing(disabled: &[Feature]) -> Cafe {
        cafe_measuring(disabled, &[])
    }

    /// A café told what to show and what to measure, resolved the way the
    /// configuration resolves them: the metrics first, the features against
    /// them.
    fn cafe_measuring(disabled: &[Feature], not_measured: &[Metric]) -> Cafe {
        let metrics = Metrics::resolve(None, &[], not_measured).expect("nothing contradicts");
        let features =
            Features::resolve(None, &[], disabled, &metrics).expect("nothing contradicts");

        Cafe::new(features, metrics)
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
        // Shelfless, so the roll fills before anything runs out: this test is
        // about the roll's own limit, not the shelf's.
        let mut cafe = cafe_showing(&[Feature::Inventory]);
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

        assert_eq!(cafe.ring_up(MENU.len()), Order::OffMenu);

        let snapshot = cafe.snapshot();

        assert_eq!(snapshot.sold.expect("this café keeps a count").total(), 1);
        assert_eq!(snapshot.sales.len(), 1);
        assert_eq!(snapshot.sales[0].seq, 1);
    }

    /// The latte's position on the menu, for the shelf tests: the drink that
    /// spends milk fastest, so the fridge is the first thing it empties.
    const LATTE: usize = 2;

    #[test]
    fn selling_takes_the_recipe_off_the_shelf() {
        let mut cafe = cafe_showing(&[]);
        let recipe = MENU[LATTE].recipe;

        assert_eq!(cafe.ring_up(LATTE), Order::Served);

        let shelf = cafe.snapshot().inventory.expect("this café keeps a shelf");

        assert_eq!(
            shelf.amount(Ingredient::Milk),
            Inventory::capacity(Ingredient::Milk) - recipe.milk
        );
        assert_eq!(
            shelf.amount(Ingredient::Beans(recipe.roast)),
            Inventory::capacity(Ingredient::Beans(recipe.roast)) - recipe.beans
        );
    }

    /// The refusal is all-or-nothing: nothing is counted, nothing goes on the
    /// roll, and the shelf keeps what it had, so every record agrees that no
    /// coffee was made.
    #[test]
    fn a_drink_the_shelf_cannot_cover_is_refused_and_counted_nowhere() {
        let mut cafe = cafe_showing(&[]);

        let fridge = Inventory::capacity(Ingredient::Milk) / MENU[LATTE].recipe.milk;
        for _ in 0..fridge {
            assert_eq!(cafe.ring_up(LATTE), Order::Served);
        }

        assert_eq!(cafe.ring_up(LATTE), Order::OutOf(Ingredient::Milk));

        let snapshot = cafe.snapshot();

        assert_eq!(
            snapshot.sold.expect("this café keeps a count").total(),
            fridge
        );
        assert_eq!(snapshot.sales.len(), fridge as usize);
    }

    /// An espresso steams no milk, so an empty fridge does not touch it: what
    /// runs out is per drink, which is the reason the beans carry a label.
    #[test]
    fn an_empty_fridge_does_not_stop_the_espresso() {
        let mut cafe = cafe_showing(&[]);

        for _ in 0..Inventory::capacity(Ingredient::Milk) / MENU[LATTE].recipe.milk {
            cafe.ring_up(LATTE);
        }

        assert_eq!(cafe.ring_up(LATTE), Order::OutOf(Ingredient::Milk));
        assert_eq!(cafe.ring_up(0), Order::Served);
    }

    /// A café without the feature has no shelf rather than an empty one, so
    /// it brews from nothing and can never run out.
    #[test]
    fn a_cafe_without_a_shelf_never_runs_out() {
        let mut cafe = cafe_showing(&[Feature::Inventory]);

        for _ in 0..200 {
            assert_eq!(cafe.ring_up(LATTE), Order::Served);
        }

        assert_eq!(cafe.snapshot().inventory, None);
    }

    /// The shelf is written into the notebook with everything else, so the
    /// gauge cards have a record to draw from — and only some of what the
    /// shelf did between entries, which is the point.
    #[test]
    fn observations_write_down_the_shelf() {
        let mut cafe = cafe_showing(&[]);
        cafe.ring_up(LATTE);
        cafe.observe();

        let observed = cafe.snapshot().observations[0]
            .inventory
            .expect("this café keeps a shelf");

        assert_eq!(
            observed.amount(Ingredient::Milk),
            Inventory::capacity(Ingredient::Milk) - MENU[LATTE].recipe.milk
        );
    }

    #[test]
    fn a_cafe_without_a_shelf_writes_none_down() {
        let mut cafe = cafe_showing(&[Feature::Inventory]);
        cafe.observe();

        assert_eq!(cafe.snapshot().observations[0].inventory, None);
    }

    /// A delivery lands on the shelf and nowhere else: restocking is not a
    /// sale, so the counter and the roll have nothing to say about it.
    #[test]
    fn a_delivery_restocks_the_shelf_without_selling_anything() {
        let mut cafe = cafe_showing(&[]);
        cafe.ring_up(LATTE);

        let taken = match &mut cafe.inventory {
            Some(shelf) => shelf.take_delivery(Ingredient::Milk),
            None => unreachable!("this café keeps a shelf"),
        };

        assert_eq!(taken, MENU[LATTE].recipe.milk);

        let snapshot = cafe.snapshot();
        let shelf = snapshot.inventory.expect("this café keeps a shelf");

        assert_eq!(
            shelf.amount(Ingredient::Milk),
            Inventory::capacity(Ingredient::Milk)
        );
        assert_eq!(snapshot.sold.expect("this café keeps a count").total(), 1);
    }

    /// Emptied by lattes, refilled by a bottle: the gauge goes down and comes
    /// back up, which is what makes it a gauge rather than a counter.
    #[test]
    fn a_restocked_ingredient_sells_again() {
        let mut cafe = cafe_showing(&[]);

        for _ in 0..Inventory::capacity(Ingredient::Milk) / MENU[LATTE].recipe.milk {
            cafe.ring_up(LATTE);
        }
        assert_eq!(cafe.ring_up(LATTE), Order::OutOf(Ingredient::Milk));

        if let Some(shelf) = &mut cafe.inventory {
            shelf.take_delivery(Ingredient::Milk);
        }

        assert_eq!(cafe.ring_up(LATTE), Order::Served);
    }

    /// The roasts are two stocks: emptying one leaves the drinks brewed from
    /// the other on sale, which is what the label on the gauge is for.
    #[test]
    fn the_roasts_run_out_separately() {
        let mut cafe = cafe_showing(&[]);
        let roast = MENU[0].recipe.roast;
        assert_eq!(roast, Roast::Light);

        let shelf = Inventory::capacity(Ingredient::Beans(roast)) / MENU[0].recipe.beans;
        for _ in 0..shelf {
            assert_eq!(cafe.ring_up(0), Order::Served);
        }

        assert_eq!(cafe.ring_up(0), Order::OutOf(Ingredient::Beans(roast)));
        assert_eq!(cafe.ring_up(1), Order::Served);
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

    /// The coffee is made either way: a café that does not measure its sales
    /// has no number that knows about them, and that is all it is missing.
    /// The roll still fills and the shelf is still spent, because neither of
    /// those is the count.
    #[test]
    fn a_cafe_without_a_till_counter_still_sells() {
        let mut cafe = cafe_measuring(&[], &[Metric::CoffeesSold]);

        assert_eq!(cafe.ring_up(LATTE), Order::Served);
        cafe.observe();

        let snapshot = cafe.snapshot();

        assert!(snapshot.sold.is_none());
        assert_eq!(snapshot.sales.len(), 1);
        assert!(snapshot.observations[0].sold.is_none());

        let shelf = snapshot.inventory.expect("this café keeps a shelf");
        assert_eq!(
            shelf.amount(Ingredient::Milk),
            Inventory::capacity(Ingredient::Milk) - MENU[LATTE].recipe.milk
        );
    }

    /// A till that does not record the dimension keeps one number, and its
    /// sales say only that a coffee was sold: there is nothing behind them to
    /// remember, rather than something withheld.
    #[test]
    fn a_cafe_without_the_drink_dimension_counts_one_number() {
        let mut cafe = cafe_measuring(&[], &[Metric::Drink]);
        cafe.ring_up(0);
        cafe.ring_up(LATTE);

        let snapshot = cafe.snapshot();
        let sold = snapshot.sold.expect("this café keeps a count");

        assert_eq!(sold.total(), 2);
        assert_eq!(sold.by_drink().count(), 0);
        assert!(snapshot.sales[0].drink().is_none());
    }

    /// No thermometer is no temperature anywhere: not in the café's own
    /// reading, and not in an entry either. What the exposition makes of the
    /// same absence is tested beside the exposition.
    #[test]
    fn a_cafe_without_thermometers_reads_no_temperature() {
        let mut cafe = cafe_measuring(&[], &[Metric::Temperatures]);
        cafe.observe();

        let snapshot = cafe.snapshot();

        assert!(snapshot.inside.is_none() && snapshot.outside.is_none());
        assert!(snapshot.observations[0].inside.is_none());
        assert!(snapshot.observations[0].outside.is_none());
    }
}
