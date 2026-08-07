//! Where each stage lives.
//!
//! A course embeds whichever URL it is currently explaining. `/metrics` is
//! deliberately not among these: there is one scrape endpoint, it lives at the
//! path every real target uses, and it is the same for every stage.

use dioxus::prelude::*;

use crate::app::Cafe;
use crate::components::StageIndex;
use crate::stage::Stage;

#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    Index {},

    #[route("/samples")]
    Samples {},

    #[route("/labels")]
    Labels {},

    #[route("/types")]
    Types {},

    #[route("/:..segments")]
    Missing { segments: Vec<String> },
}

impl Stage {
    /// The page this stage is served from.
    pub fn route(self) -> Route {
        match self {
            Self::Samples => Route::Samples {},
            Self::Labels => Route::Labels {},
            Self::Types => Route::Types {},
        }
    }
}

/// The way in: what there is to look at, and where the scrape endpoint is.
#[component]
fn Index() -> Element {
    rsx! {
        StageIndex {}
    }
}

#[component]
fn Samples() -> Element {
    rsx! {
        Cafe { stage: Stage::Samples }
    }
}

#[component]
fn Labels() -> Element {
    rsx! {
        Cafe { stage: Stage::Labels }
    }
}

#[component]
fn Types() -> Element {
    rsx! {
        Cafe { stage: Stage::Types }
    }
}

/// A mistyped URL is shown the way in rather than an error page. The café is
/// still open, and the index says what there is to look at.
#[component]
fn Missing(segments: Vec<String>) -> Element {
    rsx! {
        StageIndex {}
    }
}
