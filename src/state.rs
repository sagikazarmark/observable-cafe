//! The state the server keeps and hands to the browser.
//!
//! Everything here crosses the wire, so it only describes what has been
//! observed. How the observations come about lives in [`crate::server`].

use std::ops::RangeInclusive;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::clock;
use crate::menu::{Drink, MENU};
use crate::season;

/// Everything the café can say about itself right now.
///
/// The counter and the thermometers are the café as it stands this instant;
/// `observations` is only the part somebody happened to write down.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// What the café clock reads, so the page can explain why the notebook
    /// says what it says.
    pub clock: String,
    /// The day the café clock is on, for a notebook with nothing in it yet.
    pub day: String,
    pub sold: Sales,
    pub inside: Gauge,
    pub outside: Gauge,
    pub observations: Vec<Observation>,
    /// Every sale, in the order they were rung up.
    ///
    /// What `observations` is a sample of: the two are meant to be read
    /// against each other, so both travel whether or not the page shows them.
    pub sales: Vec<Sale>,
}

impl Snapshot {
    /// A café at opening time, before anything has been sold or written down.
    ///
    /// The browser draws one of these until the server answers.
    pub fn new() -> Self {
        let opening = clock::opening();

        Self {
            clock: clock::written(opening),
            day: clock::dated(opening),
            sold: Sales::default(),
            inside: Gauge::inside(opening),
            outside: Gauge::outside(opening),
            observations: Vec::new(),
            sales: Vec::new(),
        }
    }

    /// The readings one thermometer contributed to the notebook, oldest first.
    ///
    /// This is everything a chart of a gauge can honestly be drawn from: the
    /// values between observations were never recorded anywhere.
    pub fn recorded(&self, reading: fn(&Observation) -> i32) -> Vec<i32> {
        self.observations.iter().map(reading).collect()
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// One entry in the owner's notebook: everything, as it stood at one moment.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// Which entry this is, counting from the café opening.
    ///
    /// Entries shuffle down the page as the notebook fills and the oldest
    /// falls off, so the page needs something that stays with an observation
    /// rather than describing where it currently sits. Two entries can share a
    /// clock reading; no two share this.
    pub seq: u64,
    /// The café clock when this was written, as `09:05`.
    pub at: String,
    /// The day it was written on. A page left open runs past midnight after
    /// sixteen minutes, and `09:05` alone would then be two different moments.
    pub day: String,
    pub sold: Sales,
    pub inside: i32,
    pub outside: i32,
}

/// One sale, written out as it happened.
///
/// A sale is an event: it says what was sold and when, and nothing about
/// how many have been sold altogether. Counting is the notebook's job, and the
/// difference between the two is why both are worth showing.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Sale {
    /// Which sale this is, counting from the café opening, so that one entry
    /// can be told from the one that takes its place as the roll fills.
    pub seq: u64,
    /// The café clock when it was rung up, as `09:05`.
    pub at: String,
    /// The café day it was rung up on. A page left open runs past midnight
    /// after sixteen minutes, and `09:05` alone would then be two moments.
    pub day: String,
    /// Which drink, as a position in [`MENU`], which is fixed. The name is not
    /// sent: the browser has the same menu.
    drink: usize,
}

impl Sale {
    /// What was sold, if the menu still has it.
    pub fn drink(&self) -> Option<&'static Drink> {
        MENU.get(self.drink)
    }

    /// Writes up one sale, identified by the drink’s position on the menu.
    #[cfg(feature = "server")]
    pub fn rung_up(seq: u64, at: String, day: String, drink: usize) -> Self {
        Self {
            seq,
            at,
            day,
            drink,
        }
    }
}

/// How many of each drink has been sold.
///
/// Indexed by position in [`MENU`], which is fixed, so counts travel as four
/// numbers rather than as names repeated on every observation.
#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Sales([u32; MENU.len()]);

impl Sales {
    pub fn total(&self) -> u32 {
        self.0.iter().sum()
    }

    /// The drinks that have actually been sold, in menu order.
    ///
    /// A drink nobody has ordered is absent rather than zero: a series does
    /// not exist until something has been observed under it, which is why
    /// graphs come out with gaps in them.
    pub fn by_drink(&self) -> impl Iterator<Item = (&'static Drink, u32)> {
        MENU.iter().zip(self.0).filter(|&(_, count)| count > 0)
    }

    /// Rings up one drink, identified by its position on the menu, and says
    /// whether the café sells it.
    ///
    /// Answered rather than ignored so that nothing else can record a sale
    /// this total does not have. The notebook, the sales and `/metrics`
    /// are one sale seen three ways, and a sale with no total behind it
    /// would have the café lie about the very thing it is demonstrating.
    #[cfg(feature = "server")]
    pub fn ring_up(&mut self, drink: usize) -> bool {
        let Some(count) = self.0.get_mut(drink) else {
            return false;
        };

        *count += 1;
        true
    }
}

/// A temperature reading, and what it is being read against.
///
/// A gauge keeps no history of its own. What was written down lives in the
/// notebook; what was not is gone, which is rather the point.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Gauge {
    value: i32,
    /// Difference between the current value and the one before it.
    delta: i32,
    /// The thermometer's printed scale, which the liquid is drawn against.
    ///
    /// Charts do not use this: a fixed instrument range leaves an ordinary
    /// afternoon looking like a flat line, so they fit their own readings.
    scale: RangeInclusive<i32>,
}

impl Gauge {
    /// A thermometer indoors, reading what the season suggests it should.
    pub fn inside(at: DateTime<Local>) -> Self {
        Self::new(season::inside(at), season::inside_range(at))
    }

    /// A thermometer outdoors, reading what the season suggests it should.
    pub fn outside(at: DateTime<Local>) -> Self {
        Self::new(season::outside(at), season::outside_range(at))
    }

    fn new(initial: i32, scale: RangeInclusive<i32>) -> Self {
        Self {
            value: initial,
            delta: 0,
            scale,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    /// Takes `value` as the latest reading.
    ///
    /// Only the café moves a gauge; the browser is handed the result.
    #[cfg(feature = "server")]
    pub fn record(&mut self, value: i32) {
        self.delta = value - self.value;
        self.value = value;
    }

    /// Height of the liquid in a thermometer, as a percentage of the tube.
    pub fn level(&self) -> f64 {
        10.0 + fraction(&self.scale, self.value) * 82.0
    }

    pub fn change_label(&self) -> String {
        match self.delta {
            0 => "No change in this reading".to_owned(),
            delta if delta > 0 => format!("▲ {delta}°C from previous reading"),
            delta => format!("▼ {}°C from previous reading", delta.abs()),
        }
    }
}

/// Where `value` sits on `scale`, as a fraction between 0 and 1.
pub fn fraction(scale: &RangeInclusive<i32>, value: i32) -> f64 {
    let clamped = value.clamp(*scale.start(), *scale.end());
    let span = (scale.end() - scale.start()).max(1);

    f64::from(clamped - scale.start()) / f64::from(span)
}
