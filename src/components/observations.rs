use dioxus::prelude::*;

use crate::components::follow::use_follow_newest;
use crate::components::ruled::Ruled;
use crate::inventory::{Ingredient, Roast};
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
                h2 { "Observations" }
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
                // Each line is written down where the café measures the thing:
                // an entry records what the owner had to look at, and a
                // reading that was never taken is not a blank in the notebook.
                if let Some(sold) = observation.sold {
                    div { class: "entry-line",
                        "Coffees sold: "
                        b { "{sold.total()}" }
                    }

                    // Only a café showing labels breaks the count down, and the
                    // total stays written above it: somebody keeping notes wants
                    // the headline, even though the machine-readable version
                    // publishes the parts alone.
                    if labelled {
                        div { class: "entry-breakdown",
                            for (drink , count) in sold.by_drink() {
                                div { key: "{drink.key}", class: "entry-line",
                                    "{drink.name}: "
                                    b { "{count}" }
                                }
                            }
                        }
                    }
                }

                if let Some(inside) = observation.inside {
                    div { class: "entry-line",
                        "Inside: "
                        b { "{inside}°C" }
                    }
                }
                if let Some(outside) = observation.outside {
                    div { class: "entry-line",
                        "Outside: "
                        b { "{outside}°C" }
                    }
                }

                // The shelf, in the café that keeps one. Two lines rather
                // than a table: this is somebody glancing into the back room
                // mid-shift, not a stocktake.
                if let Some(shelf) = observation.inventory {
                    div { class: "entry-line",
                        "Beans: "
                        b {
                            "{shelf.amount(Ingredient::Beans(Roast::Light))} g light, "
                            "{shelf.amount(Ingredient::Beans(Roast::Dark))} g dark"
                        }
                    }
                    div { class: "entry-line",
                        "Milk: "
                        b { "{shelf.amount(Ingredient::Milk)} ml" }
                    }
                }
            }
        }
    }
}
