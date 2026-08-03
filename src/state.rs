use std::ops::RangeInclusive;

/// How many readings a gauge keeps around for its sparkline.
const HISTORY_LIMIT: usize = 14;

/// A temperature reading that drifts up and down over time.
#[derive(Clone, PartialEq)]
pub struct Gauge {
    value: i32,
    /// Difference between the current value and the one before it.
    delta: i32,
    history: Vec<i32>,
    /// Values the reading is allowed to drift between.
    drift: RangeInclusive<i32>,
    /// Values the thermometer and the sparkline are drawn against.
    scale: RangeInclusive<i32>,
}

impl Gauge {
    pub fn inside() -> Self {
        Self::new(22, 16..=28, 14..=30)
    }

    pub fn outside() -> Self {
        Self::new(17, 6..=30, 4..=32)
    }

    fn new(initial: i32, drift: RangeInclusive<i32>, scale: RangeInclusive<i32>) -> Self {
        Self {
            value: initial,
            delta: 0,
            history: vec![initial],
            drift,
            scale,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn history(&self) -> &[i32] {
        &self.history
    }

    /// Moves the reading by `step`, recording the result in the history.
    pub fn drift(&mut self, step: i32) {
        // Keep readings realistic while still demonstrating that gauges move
        // both ways: bounce off an edge instead of clamping to it.
        let step = if self.drift.contains(&(self.value + step)) {
            step
        } else {
            -step
        };

        self.value += step;
        self.delta = step;

        self.history.push(self.value);
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
