use dioxus::prelude::*;

use crate::components::follow::use_follow_newest;
use crate::components::ruled::Ruled;
use crate::state::Observation;

/// The entries the owner writes the café down in.
///
/// Every entry is one moment, written out in full. Nothing here fades or gets
/// struck out as it ages: an old reading is not a wrong one, it is the record.
#[component]
pub fn Observations(observations: Vec<Observation>, labelled: bool, today: String) -> Element {
    // The café runs past midnight after sixteen minutes, so the heading
    // follows the newest entry rather than the day the café opened.
    let date_stamp = observations
        .last()
        .map_or(today, |newest| newest.day.clone());

    use_follow_newest("observation-entries");

    let lines = Ruled::from(&observations);
    let newest = observations.last().map(|entry| entry.seq);

    rsx! {
        div { class: "observations",
            div { class: "view-header",
                h2 { "Café observations" }
                div { class: "date-stamp", "{date_stamp}" }
            }

            div { id: "observation-entries", class: "entries",
                if observations.is_empty() {
                    div { class: "empty-state",
                        strong { "Nothing written down yet" }
                        "The owner looks up every few minutes."
                    }
                } else {
                    for line in lines.iter() {
                        match line {
                            Ruled::Day { day, before } => rsx! {
                                div { key: "day-{before}", class: "day-divider",
                                    span { "{day}" }
                                }
                            },
                            Ruled::Line(observation) => rsx! {
                                Entry {
                                    key: "{observation.seq}",
                                    observation: (*observation).clone(),
                                    labelled,
                                    fresh: Some(observation.seq) == newest,
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Entry(observation: Observation, labelled: bool, fresh: bool) -> Element {
    let class = if fresh { "entry fresh" } else { "entry" };

    rsx! {
        article { class: "{class}",
            span { class: "entry-time", "{observation.at}" }

            div { class: "entry-body",
                div { class: "entry-line",
                    "Coffees sold: "
                    b { "{observation.sold.total()}" }
                }

                // Only a café showing labels breaks the count down, and the
                // total stays written above it: somebody keeping notes wants
                // the headline, even though the machine-readable version
                // publishes the parts alone.
                if labelled {
                    div { class: "entry-breakdown",
                        for (drink , count) in observation.sold.by_drink() {
                            div { key: "{drink.key}", class: "entry-line",
                                "{drink.name}: "
                                b { "{count}" }
                            }
                        }
                    }
                }

                div { class: "entry-line",
                    "Inside: "
                    b { "{observation.inside}°C" }
                }
                div { class: "entry-line",
                    "Outside: "
                    b { "{observation.outside}°C" }
                }
            }
        }
    }
}
