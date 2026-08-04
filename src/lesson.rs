//! The lessons the café can teach.
//!
//! Each one is the same café with one more idea in it, and each has its own
//! URL so that a course can embed whichever it is currently explaining. They
//! are named after what they teach rather than numbered, so a fourth can be
//! slotted in anywhere without renaming the others.

use serde::{Deserialize, Serialize};

/// Which lesson the page is showing.
///
/// This travels to the café as well as around the browser: the café is set up
/// for one lesson at a time, and needs to be told which.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Lesson {
    /// A record is made of samples: the owner writes things down every so
    /// often, and whatever happens in between goes unrecorded.
    Samples,
    /// A measurement can be broken down by a dimension — here, which drink.
    Labels,
    /// A counter and a gauge are different kinds of number.
    Types,
}

impl Lesson {
    /// Every lesson, in the order they are meant to be taken.
    pub const ALL: [Self; 3] = [Self::Samples, Self::Labels, Self::Types];

    /// What the lesson is called on the index.
    pub fn title(self) -> &'static str {
        match self {
            Self::Samples => "Samples",
            Self::Labels => "Labels",
            Self::Types => "Types",
        }
    }

    /// The one idea the lesson adds, in a sentence.
    pub fn teaches(self) -> &'static str {
        match self {
            Self::Samples => {
                "A record is made of samples. Whatever happens between them is not kept."
            }
            Self::Labels => "A measurement can be broken down by a dimension — here, which drink.",
            Self::Types => "A counter and a gauge are different kinds of number.",
        }
    }

    /// Whether coffees are broken down by drink rather than counted together.
    pub fn labelled(self) -> bool {
        matches!(self, Self::Labels | Self::Types)
    }

    /// Whether the record can also be read as per-metric cards.
    ///
    /// That view is where a metric is named as a counter or a gauge, so it
    /// only appears once there is vocabulary for it.
    pub fn typed(self) -> bool {
        matches!(self, Self::Types)
    }
}
