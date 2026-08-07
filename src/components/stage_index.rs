use dioxus::prelude::*;

use crate::stage::Stage;

/// The way in.
///
/// Unlike the stages, this page is not meant to be embedded, so it keeps the
/// title the stages give up to the page around them.
#[component]
pub fn StageIndex() -> Element {
    rsx! {
        main { class: "index",
            header { class: "index-head",
                h1 { "The Observable Café" }
                p { class: "index-lede",
                    "An interactive example for exploring metrics. Each stage is the same "
                    "café with one more idea in it, so explore them in order."
                }
            }

            nav { class: "stage-list", aria_label: "Stages",
                for (number , stage) in Stage::ALL.iter().enumerate() {
                    Link {
                        key: "{stage.title()}",
                        class: "stage-link",
                        to: stage.route(),

                        span { class: "stage-number", "{number + 1}" }
                        span { class: "stage-copy",
                            span { class: "stage-title", "{stage.title()}" }
                            span { class: "stage-premise", "{stage.premise()}" }
                            span { class: "stage-explanation", "{stage.explanation()}" }
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
                    "every stage, and it never updates on its own. Reloading it is a scrape."
                }
            }
        }
    }
}
