use dioxus::prelude::*;

use crate::components::Sparkline;
use crate::state::{Gauge, Snapshot};

/// The same café, sorted by metric rather than by moment.
///
/// This is where a number is named as a counter or a gauge, so it only appears
/// in the lesson that has the words for it.
#[component]
pub fn MetricCards(snapshot: Snapshot) -> Element {
    let inside = snapshot.recorded(|observation| observation.inside);
    let outside = snapshot.recorded(|observation| observation.outside);

    rsx! {
        div { class: "cards",
            article { class: "metric",
                div { class: "metric-topline",
                    span { class: "metric-name", "Coffees sold" }
                    span { class: "metric-type counter-badge", "Counter" }
                }
                div { class: "metric-value-row",
                    span { class: "metric-value", "{snapshot.sold.total()}" }
                    span { class: "metric-unit", "since the café opened" }
                }

                // One row per drink that has been sold, and none for the rest:
                // a series begins when something is first observed under it.
                div { class: "metric-series",
                    for (drink , count) in snapshot.sold.by_drink() {
                        div { key: "{drink.key}", class: "series",
                            code { "drink=\"{drink.key}\"" }
                            b { "{count}" }
                        }
                    }
                }
            }

            TemperatureCard {
                name: "Inside temperature",
                gauge: snapshot.inside.clone(),
                recorded: inside,
                color: "var(--inside)",
            }

            TemperatureCard {
                name: "Outside temperature",
                gauge: snapshot.outside.clone(),
                recorded: outside,
                color: "var(--outside)",
            }
        }
    }
}

#[component]
fn TemperatureCard(name: String, gauge: Gauge, recorded: Vec<i32>, color: String) -> Element {
    let value = gauge.value();
    // The chart fits the readings rather than the thermometer's whole scale,
    // so it has to say what it is fitted to — otherwise a wiggle could be a
    // degree or ten and there is no way to tell them apart.
    let lowest = recorded.iter().copied().min();
    let highest = recorded.iter().copied().max();

    rsx! {
        article { class: "metric",
            div { class: "metric-topline",
                span { class: "metric-name", "{name}" }
                span { class: "metric-type gauge-badge", "Gauge" }
            }
            div { class: "metric-value-row",
                span { class: "metric-value", "{value}" }
                span { class: "metric-unit", "°C · right now" }
            }
            Sparkline {
                values: recorded.clone(),
                color,
                label: "{name}, as written down",
            }
            if let (Some(lowest), Some(highest)) = (lowest, highest) {
                div { class: "chart-range",
                    span { "written down: {lowest}°C to {highest}°C" }
                    span { "{recorded.len()} entries" }
                }
            }
        }
    }
}
