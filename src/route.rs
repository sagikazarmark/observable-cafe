//! Where each lesson lives.
//!
//! A course embeds whichever URL it is currently explaining. `/metrics` is
//! deliberately not among these: there is one scrape endpoint, it lives at the
//! path every real target uses, and it is the same for every lesson.

use dioxus::prelude::*;

use crate::app::Cafe;
use crate::components::LessonIndex;
use crate::lesson::Lesson;

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

impl Lesson {
    /// The page this lesson is served from.
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
        LessonIndex {}
    }
}

#[component]
fn Samples() -> Element {
    rsx! {
        Cafe { lesson: Lesson::Samples }
    }
}

#[component]
fn Labels() -> Element {
    rsx! {
        Cafe { lesson: Lesson::Labels }
    }
}

#[component]
fn Types() -> Element {
    rsx! {
        Cafe { lesson: Lesson::Types }
    }
}

/// A mistyped URL is shown the way in rather than an error page — the café is
/// still open, and the index says what there is to look at.
#[component]
fn Missing(segments: Vec<String>) -> Element {
    rsx! {
        LessonIndex {}
    }
}
