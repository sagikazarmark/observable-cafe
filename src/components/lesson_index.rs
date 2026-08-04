use dioxus::prelude::*;

use crate::lesson::Lesson;

/// The way in.
///
/// Unlike the lessons, this page is not meant to be embedded, so it keeps the
/// title the lessons give up to the course page around them.
#[component]
pub fn LessonIndex() -> Element {
    rsx! {
        main { class: "index",
            header { class: "index-head",
                h1 { "The Observable Café" }
                p { class: "index-lede",
                    "An interactive example for teaching metrics. Each lesson is the same "
                    "café with one more idea in it, so take them in order."
                }
            }

            nav { class: "lesson-list", aria_label: "Lessons",
                for (number , lesson) in Lesson::ALL.iter().enumerate() {
                    Link {
                        key: "{lesson.title()}",
                        class: "lesson-link",
                        to: lesson.route(),

                        span { class: "lesson-number", "{number + 1}" }
                        span { class: "lesson-copy",
                            span { class: "lesson-title", "{lesson.title()}" }
                            span { class: "lesson-teaches", "{lesson.teaches()}" }
                        }
                    }
                }
            }

            // Opened in its own tab: it is a scrape endpoint rather than a
            // page, and leaving the app to look at it is the wrong shape.
            a {
                class: "endpoint-link",
                href: "/metrics",
                target: "_blank",
                rel: "noopener",

                code { "/metrics" }
                span {
                    "The scrape endpoint, in the format every scraper reads. The same for "
                    "every lesson, and it never updates on its own — reloading it is a scrape."
                }
            }
        }
    }
}
