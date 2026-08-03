//! Readings that move on their own, so the gauges have something to show.

use std::ops::RangeInclusive;
use std::sync::Once;
use std::time::Duration;

use super::rng::Rng;
use crate::state::Gauge;

/// How long the café waits between readings.
const PAUSE_MS: RangeInclusive<u64> = 5_000..=10_000;

/// A reading together with the range it is allowed to wander between.
pub struct Thermometer {
    reading: Gauge,
    drift: RangeInclusive<i32>,
}

impl Thermometer {
    pub fn inside() -> Self {
        Self {
            reading: Gauge::inside(),
            drift: 16..=28,
        }
    }

    pub fn outside() -> Self {
        Self {
            reading: Gauge::outside(),
            drift: 6..=30,
        }
    }

    pub fn reading(&self) -> &Gauge {
        &self.reading
    }

    /// Takes a reading one to three degrees away from the last one.
    fn tick(&mut self, rng: &mut Rng) {
        let magnitude = rng.below(3) as i32 + 1;
        let step = if rng.below(2) == 0 {
            -magnitude
        } else {
            magnitude
        };

        // Keep readings realistic while still demonstrating that gauges move
        // both ways: bounce off an edge instead of clamping to it.
        let value = self.reading.value();
        let next = if self.drift.contains(&(value + step)) {
            value + step
        } else {
            value - step
        };

        self.reading.record(next);
    }
}

static STARTED: Once = Once::new();

/// Starts the task that takes a reading every few seconds.
///
/// The dev server rebuilds the router on every hot-patch, so this guards
/// against ending up with one task per rebuild.
pub fn start() {
    STARTED.call_once(|| {
        tokio::spawn(async {
            let mut rng = Rng::from_clock();

            loop {
                let span = PAUSE_MS.end() - PAUSE_MS.start() + 1;
                let pause = PAUSE_MS.start() + rng.below(span);
                tokio::time::sleep(Duration::from_millis(pause)).await;

                {
                    let mut cafe = super::cafe();
                    cafe.inside.tick(&mut rng);
                    cafe.outside.tick(&mut rng);
                }
            }
        });
    });
}
