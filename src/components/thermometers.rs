use dioxus::prelude::*;

use crate::state::Gauge;

/// Thermometers mirroring the two gauges tracked in the notebook.
#[component]
pub fn Thermometers(inside: ReadSignal<Gauge>, outside: ReadSignal<Gauge>) -> Element {
    rsx! {
        div { class: "thermometers", aria_label: "Current temperatures",
            Thermometer { variant: "inside", label: "Inside temperature", gauge: inside }
            Thermometer { variant: "outside", label: "Outside temperature", gauge: outside }
        }
    }
}

#[component]
fn Thermometer(variant: String, label: String, gauge: ReadSignal<Gauge>) -> Element {
    let (level, value, change) = {
        let gauge = gauge.read();
        (gauge.level(), gauge.value(), gauge.change_label())
    };

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
