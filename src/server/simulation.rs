//! Readings that move on their own, so the gauges have something to show.
//!
//! Temperature is restless: it moves every tick, while the notebook catches
//! only every fifth one. A rise and fall that happens between two entries
//! leaves no trace anywhere, which is the whole reason sampling is worth
//! teaching.

use std::ops::RangeInclusive;
use std::sync::Once;
use std::time::Duration;

use chrono::{DateTime, Local};

use super::rng::Rng;
use crate::clock;
use crate::season;
use crate::state::Gauge;

/// One tick of real time. A tick is also one minute of café time.
const TICK: Duration = Duration::from_secs(1);

/// How long a thermometer holds a reading before taking another, in ticks.
///
/// Deliberately not a fixed number. Weather keeps no schedule, and a fixed
/// gap here against a fixed gap between entries would lock the two together,
/// so the same movements would fall into the same blind spot every time.
const MOVES_EVERY: RangeInclusive<u64> = 5..=10;

/// A reading together with what it is measuring.
pub struct Thermometer {
    reading: Gauge,
    /// The temperature the season and the hour call for.
    base: fn(DateTime<Local>) -> i32,
    /// How far a reading may sit either side of that.
    swing: i32,
    /// Ticks left before this thermometer moves again. Each keeps its own, so
    /// the two do not step together.
    holds_for: u64,
}

impl Thermometer {
    pub fn inside(at: DateTime<Local>) -> Self {
        Self {
            reading: Gauge::inside(at),
            base: season::inside,
            swing: 2,
            holds_for: 0,
        }
    }

    pub fn outside(at: DateTime<Local>) -> Self {
        Self {
            reading: Gauge::outside(at),
            base: season::outside,
            swing: 3,
            holds_for: 0,
        }
    }

    pub fn reading(&self) -> &Gauge {
        &self.reading
    }

    /// Counts down to the next reading, and takes one when it comes due.
    ///
    /// A reading sits still for several seconds at a time. Moving every tick
    /// reads as a broken sensor rather than as weather, and the notebook is
    /// where the attention is meant to be.
    ///
    /// The base moves as the café day goes on, so readings follow a morning up
    /// and an evening down rather than wandering about a fixed point.
    fn tick(&mut self, rng: &mut Rng, at: DateTime<Local>) {
        if self.holds_for > 0 {
            self.holds_for -= 1;
            return;
        }

        let span = MOVES_EVERY.end() - MOVES_EVERY.start() + 1;
        self.holds_for = MOVES_EVERY.start() + rng.below(span);

        let base = (self.base)(at);
        let drift = (base - self.swing)..=(base + self.swing);
        let value = self.reading.value();

        let next = if value < *drift.start() {
            // The day has moved on without the reading — follow it.
            value + 1
        } else if value > *drift.end() {
            value - 1
        } else {
            // A degree either way, sometimes two. Now that readings only come
            // every few seconds there is no need to stand still as well.
            let step = match rng.below(6) {
                0 => 2,
                1 => -2,
                2 | 3 => 1,
                _ => -1,
            };

            if drift.contains(&(value + step)) {
                value + step
            } else {
                // Keep readings realistic while still demonstrating that
                // gauges move both ways: bounce off an edge rather than
                // clamping to it.
                value - step
            }
        };

        self.reading.record(next);
    }
}

static STARTED: Once = Once::new();

/// Starts the task that runs the café clock.
///
/// The dev server rebuilds the router on every hot-patch, so this guards
/// against ending up with one task per rebuild.
pub fn start() {
    STARTED.call_once(|| {
        tokio::spawn(async {
            let mut rng = Rng::from_clock();

            loop {
                tokio::time::sleep(TICK).await;

                // The lock is taken and released inside the loop body so that
                // it is never held across the sleep above.
                {
                    let mut cafe = super::cafe();
                    cafe.tick += 1;

                    let now = clock::at(cafe.tick);
                    cafe.inside.tick(&mut rng, now);
                    cafe.outside.tick(&mut rng, now);

                    if cafe.observation_due() {
                        cafe.observe();
                    }
                }
            }
        });
    });
}
