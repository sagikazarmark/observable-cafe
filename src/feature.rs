//! What the café shows.
//!
//! The café used to be a ladder of stages, each the same café with one more
//! idea in it. That ladder only ever decided two things about the page, so it
//! is gone: what to show is now a set of features, asked for by name.
//!
//! Nothing here is decided by the browser. The café is told what to show once,
//! when it starts, and every page it serves shows the same thing.

use serde::{Deserialize, Serialize};

/// One thing the café can be told to show.
///
/// The kebab-case spellings are the ones the command line, the environment and
/// the settings file all use, so a feature is named the same way wherever it
/// is asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum Feature {
    /// The notebook itself: the book on the counter, and with it everything
    /// kept in it.
    ///
    /// Turning this off turns off the entries, the receipts and the metrics
    /// view together, which is the short way of asking for a café that writes
    /// nothing down where anybody can see it. `/metrics` is unaffected, as it
    /// is by every feature here: it reports the café rather than the record.
    Notebook,
    /// The entries: the café written down every so often, and the button that
    /// asks for one now.
    Observations,
    /// The clock writing entries as their interval comes due, and the box that
    /// paces it. Without it the notebook is filled by hand or not at all.
    AutomaticObservations,
    /// Every sale, written out as it happens.
    ///
    /// The other half of the sampling lesson: what the notebook is a sample
    /// *of*, so the two can be read side by side.
    Receipts,
    /// Breaking the count down by which drink it was.
    Labels,
    /// Reading the same record sorted by metric, where a number is named as a
    /// counter or a gauge.
    Types,
}

impl Feature {
    /// Every feature, in the order the café would introduce them.
    pub const ALL: [Self; 6] = [
        Self::Notebook,
        Self::Observations,
        Self::AutomaticObservations,
        Self::Receipts,
        Self::Labels,
        Self::Types,
    ];

    /// What this feature is spelled as, wherever it is asked for.
    ///
    /// Only the half of the build that reads settings has anything to say a
    /// feature name to; the browser is handed the answers, not the questions.
    #[cfg(feature = "server")]
    pub fn name(self) -> &'static str {
        match self {
            Self::Notebook => "notebook",
            Self::Observations => "observations",
            Self::AutomaticObservations => "automatic-observations",
            Self::Receipts => "receipts",
            Self::Labels => "labels",
            Self::Types => "types",
        }
    }

    /// The feature this one is part of, if it is part of one.
    ///
    /// A feature is only shown when everything it is part of is shown too, so
    /// that turning off the notebook turns off what is kept in it without having
    /// to name each one, and goes on doing so when another is added later.
    fn part_of(self) -> Option<Self> {
        match self {
            Self::Notebook => None,
            Self::AutomaticObservations => Some(Self::Observations),
            Self::Observations | Self::Receipts | Self::Labels | Self::Types => {
                Some(Self::Notebook)
            }
        }
    }

    /// Where this feature sits in the table [`Features::resolve`] works in.
    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&feature| feature == self)
            .expect("every feature is in ALL")
    }

    /// Whether this feature puts anything in the notebook on its own.
    ///
    /// [`Feature::Labels`] does not: it changes how the others are written out
    /// rather than adding anything of its own, so a notebook holding nothing but
    /// labels is an empty notebook.
    fn fills_the_notebook(self) -> bool {
        matches!(self, Self::Observations | Self::Receipts | Self::Types)
    }
}

/// A named set of features to start from.
///
/// These are the stages the café used to be a ladder of. Each starts from
/// nothing and names what it wants, so a feature added later joins the café
/// that was asked for by default and none of the presets, and an example built
/// against one of them goes on showing what it showed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    /// A record is made of samples. Whatever happens between them is not kept.
    Samples,
    /// A measurement can be broken down by a dimension: here, which drink.
    Labels,
    /// A counter and a gauge are different kinds of number.
    Types,
}

impl Preset {
    /// Exactly what this preset shows.
    fn features(self) -> &'static [Feature] {
        match self {
            Self::Samples => &[
                Feature::Notebook,
                Feature::Observations,
                Feature::AutomaticObservations,
            ],
            Self::Labels => &[
                Feature::Notebook,
                Feature::Observations,
                Feature::AutomaticObservations,
                Feature::Labels,
            ],
            Self::Types => &[
                Feature::Notebook,
                Feature::Observations,
                Feature::AutomaticObservations,
                Feature::Labels,
                Feature::Types,
            ],
        }
    }
}

/// What the café shows, once everything has been asked and answered.
///
/// Every field is the final answer: a feature that is part of one that is off
/// is off here too, so nothing reading this has to remember what is part of
/// what.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Features {
    /// Whether the notebook is drawn at all.
    ///
    /// False when it was turned off, and false when nothing is left to put in
    /// it: an empty panel beside the café teaches nothing and takes half the
    /// width to do it.
    pub notebook: bool,
    pub observations: bool,
    pub automatic_observations: bool,
    pub receipts: bool,
    /// Whether counts are broken down by drink. A modifier rather than a view:
    /// on its own it puts nothing in the notebook.
    pub labels: bool,
    pub types: bool,
}

impl Features {
    /// The whole café, which is what it shows when nobody has said otherwise.
    pub fn all() -> Self {
        Self::resolve(None, &[], &[]).expect("nothing to contradict")
    }

    /// Works out what to show from a preset and the features asked for or
    /// refused on top of it.
    ///
    /// Without a preset every feature is shown; with one, only what it names.
    /// `enable` and `disable` then have the last word, so an example can start
    /// from a preset and still differ from it in one place.
    ///
    /// A feature named in both is returned as an error rather than resolved
    /// one way or the other: nobody means both, so it is a mistake to report
    /// rather than a preference to honour.
    pub fn resolve(
        preset: Option<Preset>,
        enable: &[Feature],
        disable: &[Feature],
    ) -> Result<Self, Feature> {
        if let Some(&contradicted) = enable.iter().find(|feature| disable.contains(feature)) {
            return Err(contradicted);
        }

        // A preset is a way of starting from nothing, so that what it shows
        // cannot be changed by a feature written afterwards.
        let mut asked_for = [preset.is_none(); Feature::ALL.len()];

        for feature in preset.iter().flat_map(|preset| preset.features()) {
            asked_for[feature.index()] = true;
        }

        for feature in disable {
            asked_for[feature.index()] = false;
        }

        for feature in enable {
            asked_for[feature.index()] = true;
        }

        let shown = |feature: Feature| {
            std::iter::successors(Some(feature), |feature| feature.part_of())
                .all(|feature| asked_for[feature.index()])
        };

        Ok(Self {
            notebook: shown(Feature::Notebook)
                && Feature::ALL
                    .into_iter()
                    .any(|feature| feature.fills_the_notebook() && shown(feature)),
            observations: shown(Feature::Observations),
            automatic_observations: shown(Feature::AutomaticObservations),
            receipts: shown(Feature::Receipts),
            labels: shown(Feature::Labels),
            types: shown(Feature::Types),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Feature, Features, Preset};

    /// Somebody who has said nothing about features has not asked for a
    /// smaller café; they have not asked for anything.
    #[test]
    fn everything_is_shown_by_default() {
        assert_eq!(
            Features::resolve(None, &[], &[]),
            Ok(Features {
                notebook: true,
                observations: true,
                automatic_observations: true,
                receipts: true,
                labels: true,
                types: true,
            })
        );
    }

    /// The whole reason presets start from nothing: `samples` predates
    /// receipts, and adding receipts must not put them in it.
    #[test]
    fn a_preset_shows_only_what_it_names() {
        assert_eq!(
            Features::resolve(Some(Preset::Samples), &[], &[]),
            Ok(Features {
                notebook: true,
                observations: true,
                automatic_observations: true,
                receipts: false,
                labels: false,
                types: false,
            })
        );
    }

    #[test]
    fn the_presets_are_the_ladder_the_stages_were() {
        let labels = Features::resolve(Some(Preset::Labels), &[], &[]).unwrap();
        let types = Features::resolve(Some(Preset::Types), &[], &[]).unwrap();

        assert!(labels.labels && !labels.types);
        assert!(types.labels && types.types);
    }

    #[test]
    fn a_preset_can_be_added_to_and_taken_from() {
        let features = Features::resolve(Some(Preset::Samples), &[Feature::Receipts], &[]).unwrap();
        assert!(features.receipts && features.observations);

        let features =
            Features::resolve(Some(Preset::Types), &[], &[Feature::AutomaticObservations]).unwrap();
        assert!(features.observations && !features.automatic_observations);
    }

    #[test]
    fn a_feature_can_be_taken_from_the_whole_cafe() {
        let features = Features::resolve(None, &[], &[Feature::Labels]).unwrap();

        assert!(!features.labels);
        assert!(features.observations && features.receipts && features.types);
    }

    /// Nobody means both, so it is a mistake rather than a preference.
    #[test]
    fn a_feature_cannot_be_both_enabled_and_disabled() {
        assert_eq!(
            Features::resolve(None, &[Feature::Types], &[Feature::Types]),
            Err(Feature::Types)
        );
    }

    /// The short way of asking for a café with no notebook at all, which goes on
    /// working when another feature is kept in the notebook later.
    #[test]
    fn turning_off_the_notebook_turns_off_what_is_kept_in_it() {
        let features = Features::resolve(None, &[], &[Feature::Notebook]).unwrap();

        assert_eq!(
            features,
            Features {
                notebook: false,
                observations: false,
                automatic_observations: false,
                receipts: false,
                labels: false,
                types: false,
            }
        );
    }

    /// Being part of something that is off beats being asked for, so that
    /// "no notebook" means no notebook however the rest of it is worded.
    #[test]
    fn a_feature_kept_in_the_notebook_cannot_outlive_it() {
        let features = Features::resolve(None, &[Feature::Types], &[Feature::Notebook]).unwrap();

        assert!(!features.types && !features.notebook);
    }

    /// Observations off means off altogether, by hand as well as on a timer.
    #[test]
    fn the_timer_cannot_outlive_the_notebook() {
        let features = Features::resolve(
            None,
            &[Feature::AutomaticObservations],
            &[Feature::Observations],
        )
        .unwrap();

        assert!(!features.observations && !features.automatic_observations);
    }

    /// An empty panel beside the café teaches nothing and takes half the width
    /// to do it, so it is not drawn.
    #[test]
    fn a_notebook_with_nothing_in_it_is_not_shown() {
        let features = Features::resolve(
            None,
            &[],
            &[Feature::Observations, Feature::Receipts, Feature::Types],
        )
        .unwrap();

        assert!(!features.notebook);
    }

    /// Labels change how the notebook is written out rather than putting
    /// anything in it, so they cannot hold an empty one open.
    #[test]
    fn labels_alone_do_not_fill_the_notebook() {
        let features = Features::resolve(
            None,
            &[Feature::Labels],
            &[Feature::Observations, Feature::Receipts, Feature::Types],
        )
        .unwrap();

        assert!(features.labels);
        assert!(!features.notebook);
    }

    /// Anything still keeping something holds the panel open.
    #[test]
    fn one_remaining_view_is_enough_to_show_the_notebook() {
        for kept in [Feature::Observations, Feature::Receipts, Feature::Types] {
            let features = Features::resolve(Some(Preset::Samples), &[kept], &[]).unwrap();

            assert!(
                features.notebook,
                "{} should hold the notebook open",
                kept.name()
            );
        }

        let receipts_only =
            Features::resolve(None, &[], &[Feature::Observations, Feature::Types]).unwrap();
        assert!(receipts_only.notebook && receipts_only.receipts);
    }
}
