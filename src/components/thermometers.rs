use dioxus::prelude::*;

use crate::state::Gauge;

/// Thermometers mirroring the two gauges tracked in the notebook.
#[component]
pub fn Thermometers(inside: Gauge, outside: Gauge) -> Element {
    rsx! {
        div { class: "thermometers", aria_label: "Current temperatures",
            Thermometer { variant: "inside", label: "Inside temperature", gauge: inside }
            Thermometer { variant: "outside", label: "Outside temperature", gauge: outside }
        }
    }
}

#[component]
fn Thermometer(variant: String, label: String, gauge: Gauge) -> Element {
    let level = gauge.level();
    let value = gauge.value();
    let change = gauge.change_label();

    rsx! {
        article { class: "thermo-card {variant}",
            div { class: "thermo", style: "--level: {level}%", aria_hidden: "true" }
            div {
                div { class: "temp-label", "{label}" }
                span { class: "temp-value", "{value}°C" }
                div { class: "temp-change", "{change}" }
            }
        }
    }
}
