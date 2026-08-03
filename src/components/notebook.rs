use chrono::Local;
use dioxus::prelude::*;

use crate::components::Sparkline;
use crate::state::Gauge;

/// The notebook where every observed metric is written down.
#[component]
pub fn Notebook(
    coffees_sold: ReadSignal<u32>,
    inside: ReadSignal<Gauge>,
    outside: ReadSignal<Gauge>,
) -> Element {
    let date_stamp = use_signal(|| Local::now().format("%b %d, %Y").to_string().to_uppercase());

    rsx! {
        section { class: "notebook-wrap", aria_labelledby: "notebookTitle",
            div { class: "notebook",
                div { class: "notebook-header",
                    div {
                        p { class: "eyebrow", "Live measurements" }
                        h2 { id: "notebookTitle", "Café metrics notebook" }
                    }
                    div { class: "date-stamp", "{date_stamp}" }
                }

                div { class: "metric-list",
                    div {
                        if coffees_sold() == 0 {
                            div { class: "empty-state",
                                strong { "No coffee sales observed yet" }
                                "Click any coffee to create the counter metric."
                            }
                        } else {
                            article { class: "metric entering",
                                div { class: "metric-topline",
                                    span { class: "metric-name", "Coffees sold" }
                                    span { class: "metric-type counter-badge", "Counter" }
                                }
                                div { class: "metric-value-row",
                                    span { class: "metric-value", "{coffees_sold}" }
                                    span { class: "metric-unit", "total purchases" }
                                }
                                p { class: "metric-note",
                                    "A counter only increases. Every purchase adds exactly one."
                                }
                            }
                        }
                    }

                    TemperatureMetric {
                        name: "Inside temperature",
                        gauge: inside,
                        color: "var(--inside)",
                        note: "A gauge can rise or fall. It represents the latest observed value.",
                    }

                    TemperatureMetric {
                        name: "Outside temperature",
                        gauge: outside,
                        color: "var(--outside)",
                        note: "This gauge updates independently every 5–10 seconds.",
                    }
                }
            }
        }
    }
}

#[component]
fn TemperatureMetric(
    name: String,
    gauge: ReadSignal<Gauge>,
    color: String,
    note: String,
) -> Element {
    let value = gauge.read().value();

    rsx! {
        article { class: "metric",
            div { class: "metric-topline",
                span { class: "metric-name", "{name}" }
                span { class: "metric-type gauge-badge", "Gauge" }
            }
            div { class: "metric-value-row",
                span { class: "metric-value", "{value}" }
                span { class: "metric-unit", "°C · current value" }
            }
            Sparkline { gauge, color, label: "{name} history" }
            p { class: "metric-note", "{note}" }
        }
    }
}
