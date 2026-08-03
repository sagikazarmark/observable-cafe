use dioxus::prelude::*;

use crate::state::Gauge;

const WIDTH: f64 = 420.0;
const HEIGHT: f64 = 66.0;
const X_PADDING: f64 = 4.0;
const Y_PADDING: f64 = 8.0;

/// The history of a gauge, drawn as a line with a dot on the latest reading.
#[component]
pub fn Sparkline(gauge: ReadSignal<Gauge>, color: String, label: String) -> Element {
    let points = {
        let gauge = gauge.read();
        let usable_width = WIDTH - X_PADDING * 2.0;
        let usable_height = HEIGHT - Y_PADDING * 2.0;
        let denominator = gauge.history().len().saturating_sub(1).max(1) as f64;

        gauge
            .history()
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let x = X_PADDING + (index as f64 / denominator) * usable_width;
                let y = HEIGHT - Y_PADDING - gauge.fraction(value) * usable_height;

                (x, y)
            })
            .collect::<Vec<_>>()
    };

    let polyline = points
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");
    let (dot_x, dot_y) = points.last().copied().unwrap_or((X_PADDING, HEIGHT));

    rsx! {
        svg {
            class: "sparkline",
            view_box: "0 0 {WIDTH} {HEIGHT}",
            role: "img",
            "aria-label": "{label}",

            line { class: "grid", x1: 0, y1: 12, x2: WIDTH, y2: 12 }
            line { class: "grid", x1: 0, y1: 33, x2: WIDTH, y2: 33 }
            line { class: "grid", x1: 0, y1: 54, x2: WIDTH, y2: 54 }
            polyline { class: "line", points: "{polyline}", stroke: "{color}" }
            circle {
                class: "dot",
                cx: "{dot_x:.1}",
                cy: "{dot_y:.1}",
                r: 5,
                fill: "{color}",
            }
        }
    }
}
