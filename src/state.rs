//! The state the server keeps and hands to the browser.
//!
//! Everything here crosses the wire, so it only describes what has been
//! observed. How the observations come about lives in [`crate::server`].

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::season;

/// How many readings a gauge keeps around for its sparkline.
#[cfg(feature = "server")]
const HISTORY_LIMIT: usize = 14;

/// Everything the café has observed so far.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub coffees_sold: u32,
    pub inside: Gauge,
    pub outside: Gauge,
}

impl Snapshot {
    /// A café that has not sold anything and has taken a single reading.
    pub fn new() -> Self {
        Self {
            coffees_sold: 0,
            inside: Gauge::inside(),
            outside: Gauge::outside(),
        }
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// A temperature reading and the readings that came before it.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Gauge {
    value: i32,
    /// Difference between the current value and the one before it.
    delta: i32,
    history: Vec<i32>,
    /// Values the thermometer and the sparkline are drawn against.
    scale: RangeInclusive<i32>,
}

impl Gauge {
    /// A thermometer indoors, reading what the season suggests it should.
    pub fn inside() -> Self {
        Self::new(season::inside(), season::inside_range())
    }

    /// A thermometer outdoors, reading what the season suggests it should.
    pub fn outside() -> Self {
        Self::new(season::outside(), season::outside_range())
    }

    fn new(initial: i32, scale: RangeInclusive<i32>) -> Self {
        Self {
            value: initial,
            delta: 0,
            history: vec![initial],
            scale,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn history(&self) -> &[i32] {
        &self.history
    }

    /// Takes `value` as the latest reading, remembering the previous ones.
    ///
    /// Only the café moves a gauge — the browser is handed the result.
    #[cfg(feature = "server")]
    pub fn record(&mut self, value: i32) {
        self.delta = value - self.value;
        self.value = value;

        self.history.push(value);
        if self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
    }

    /// Where `value` sits on the gauge's scale, as a fraction between 0 and 1.
    pub fn fraction(&self, value: i32) -> f64 {
        let clamped = value.clamp(*self.scale.start(), *self.scale.end());
        let span = self.scale.end() - self.scale.start();

        f64::from(clamped - self.scale.start()) / f64::from(span)
    }

    /// Height of the liquid in a thermometer, as a percentage of the tube.
    pub fn level(&self) -> f64 {
        10.0 + self.fraction(self.value) * 82.0
    }

    pub fn change_label(&self) -> String {
        match self.delta {
            0 => "No change in this reading".to_owned(),
            delta if delta > 0 => format!("▲ {delta}°C from previous reading"),
            delta => format!("▼ {}°C from previous reading", delta.abs()),
        }
    }
}
