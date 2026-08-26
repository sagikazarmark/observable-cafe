//! What the café measures.
//!
//! Features decide what the page shows; this axis decides what the café
//! records at all. A number the café does not measure exists nowhere — not on
//! the page, not in the notebook, and not at `/metrics` — because the
//! exposition reports the café, and this is part of what the café is made of.
//!
//! The shelf is deliberately absent. Whether the café keeps one is
//! [`Feature::Inventory`]'s to answer, and a café without a shelf has
//! unlimited beans and milk rather than unmeasured ones; the roast rides with
//! the shelf for the same reason, since the two roasts running out separately
//! is the very lesson the shelf is there to teach.
//!
//! [`Feature::Inventory`]: crate::feature::Feature::Inventory

use serde::{Deserialize, Serialize};

use crate::feature::Preset;

/// One thing the café can be told to measure.
///
/// The kebab-case spellings are the ones the command line, the environment and
/// the settings file all use, so a metric is named the same way wherever it is
/// asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum Metric {
    /// The till's running count of coffees sold.
    CoffeesSold,
    /// The till recording *which* drink each sale was: a dimension of the
    /// count rather than a number of its own, so it is part of the count and
    /// nothing without it.
    Drink,
    /// The two thermometers. One instrument rather than two: the lesson needs
    /// a gauge, and a café reading only one of its thermometers teaches
    /// nothing a café reading both does not.
    Temperatures,
}

impl Metric {
    /// Every metric, in the order the café would introduce them.
    pub const ALL: [Self; 3] = [Self::CoffeesSold, Self::Drink, Self::Temperatures];

    /// What this metric is spelled as, wherever it is asked for.
    ///
    /// Only the half of the build that reads settings has anything to say a
    /// metric name to; the browser is handed the answers, not the questions.
    #[cfg(feature = "server")]
    pub fn name(self) -> &'static str {
        match self {
            Self::CoffeesSold => "coffees-sold",
            Self::Drink => "drink",
            Self::Temperatures => "temperatures",
        }
    }

    /// The metric this one is a dimension of, if it is a dimension.
    ///
    /// A dimension is only measured when what it divides up is measured too,
    /// so that turning off the count turns off the label on it without having
    /// to name each one.
    fn part_of(self) -> Option<Self> {
        match self {
            Self::CoffeesSold | Self::Temperatures => None,
            Self::Drink => Some(Self::CoffeesSold),
        }
    }

    /// Where this metric sits in the table [`Metrics::resolve`] works in.
    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&metric| metric == self)
            .expect("every metric is in ALL")
    }
}

impl Preset {
    /// Exactly what this preset measures.
    ///
    /// The same rule as [`Preset::features`], for the same reason: a preset
    /// starts from nothing, so an example built against one goes on serving
    /// the exposition it was built against.
    pub fn metrics(self) -> &'static [Metric] {
        match self {
            Self::Samples => &[Metric::CoffeesSold],
            Self::Labels => &[Metric::CoffeesSold, Metric::Drink],
            Self::Types => &[Metric::CoffeesSold, Metric::Drink, Metric::Temperatures],
        }
    }
}

/// What the café measures, once everything has been asked and answered.
///
/// Every field is the final answer: a dimension of a count that is not kept is
/// not kept either, so nothing reading this has to remember what is part of
/// what.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Metrics {
    /// Whether the till counts the coffees at all.
    pub coffees_sold: bool,
    /// Whether the till records which drink each sale was.
    ///
    /// The record itself changes shape with this, rather than being summed on
    /// the way out: a café that does not measure the dimension never knew it,
    /// which is what a café without labels actually is.
    pub drink: bool,
    /// Whether the café reads its thermometers.
    pub temperatures: bool,
}

impl Metrics {
    /// The whole café, which is what it measures when nobody has said
    /// otherwise.
    pub fn all() -> Self {
        Self::resolve(None, &[], &[]).expect("nothing to contradict")
    }

    /// Works out what to measure from a preset and the metrics asked for or
    /// refused on top of it.
    ///
    /// Without a preset every metric is measured; with one, only what it
    /// names. `collect` and `no_collect` then have the last word, so an
    /// example can start from a preset and still differ from it in one place.
    ///
    /// A metric named in both is returned as an error rather than resolved
    /// one way or the other: nobody means both, so it is a mistake to report
    /// rather than a preference to honour.
    pub fn resolve(
        preset: Option<Preset>,
        collect: &[Metric],
        no_collect: &[Metric],
    ) -> Result<Self, Metric> {
        if let Some(&contradicted) = collect.iter().find(|metric| no_collect.contains(metric)) {
            return Err(contradicted);
        }

        // A preset is a way of starting from nothing, so that what it measures
        // cannot be changed by a metric written afterwards.
        let mut asked_for = [preset.is_none(); Metric::ALL.len()];

        for metric in preset.iter().flat_map(|preset| preset.metrics()) {
            asked_for[metric.index()] = true;
        }

        for metric in no_collect {
            asked_for[metric.index()] = false;
        }

        for metric in collect {
            asked_for[metric.index()] = true;
        }

        let measured = |metric: Metric| {
            std::iter::successors(Some(metric), |metric| metric.part_of())
                .all(|metric| asked_for[metric.index()])
        };

        Ok(Self {
            coffees_sold: measured(Metric::CoffeesSold),
            drink: measured(Metric::Drink),
            temperatures: measured(Metric::Temperatures),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Metric, Metrics};
    use crate::feature::Preset;

    /// Somebody who has said nothing about metrics has not asked for a
    /// smaller café; they have not asked for anything.
    #[test]
    fn everything_is_measured_by_default() {
        assert_eq!(
            Metrics::resolve(None, &[], &[]),
            Ok(Metrics {
                coffees_sold: true,
                drink: true,
                temperatures: true,
            })
        );
    }

    /// The whole point of the axis: a `samples` course serves an exposition
    /// with one counter in it and nothing else.
    #[test]
    fn a_preset_measures_only_what_it_names() {
        assert_eq!(
            Metrics::resolve(Some(Preset::Samples), &[], &[]),
            Ok(Metrics {
                coffees_sold: true,
                drink: false,
                temperatures: false,
            })
        );
    }

    #[test]
    fn the_presets_are_the_ladder_the_stages_were() {
        let labels = Metrics::resolve(Some(Preset::Labels), &[], &[]).unwrap();
        let types = Metrics::resolve(Some(Preset::Types), &[], &[]).unwrap();

        assert!(labels.drink && !labels.temperatures);
        assert!(types.drink && types.temperatures);
    }

    #[test]
    fn a_preset_can_be_added_to_and_taken_from() {
        let metrics =
            Metrics::resolve(Some(Preset::Samples), &[Metric::Temperatures], &[]).unwrap();
        assert!(metrics.temperatures && metrics.coffees_sold);

        let metrics = Metrics::resolve(Some(Preset::Types), &[], &[Metric::Drink]).unwrap();
        assert!(metrics.coffees_sold && !metrics.drink);
    }

    /// A dimension of a count that is not kept is not kept either, however it
    /// was asked for.
    #[test]
    fn the_drink_is_not_measured_without_the_count() {
        let metrics = Metrics::resolve(None, &[], &[Metric::CoffeesSold]).unwrap();

        assert!(!metrics.coffees_sold && !metrics.drink);
        assert!(metrics.temperatures);

        let metrics = Metrics::resolve(Some(Preset::Samples), &[Metric::Drink], &[]).unwrap();
        assert!(metrics.drink, "the count is named by the preset");

        let metrics = Metrics::resolve(None, &[Metric::Drink], &[Metric::CoffeesSold]).unwrap();
        assert!(!metrics.drink, "a dimension cannot outlive its count");
    }

    /// Nobody means both, so it is a mistake rather than a preference.
    #[test]
    fn a_metric_cannot_be_both_collected_and_not() {
        assert_eq!(
            Metrics::resolve(None, &[Metric::Drink], &[Metric::Drink]),
            Err(Metric::Drink)
        );
    }
}
