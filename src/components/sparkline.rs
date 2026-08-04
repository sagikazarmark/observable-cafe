use std::ops::RangeInclusive;

use dioxus::prelude::*;

use crate::state::fraction;

const WIDTH: f64 = 420.0;
const HEIGHT: f64 = 66.0;
const X_PADDING: f64 = 4.0;
const Y_PADDING: f64 = 8.0;

/// The narrowest range the chart will draw against, in degrees.
///
/// Fitting exactly to the readings would turn a becalmed afternoon into a
/// mountain range, so a quiet stretch keeps this floor and simply looks quiet.
const NARROWEST: i32 = 6;

/// A line through what was written down, with a dot on the latest entry.
///
/// The values come from the notebook rather than from the thermometer, because
/// the notebook is all a chart can honestly be drawn from: whatever the
/// temperature did between two entries was never recorded.
#[component]
pub fn Sparkline(values: Vec<i32>, color: String, label: String) -> Element {
    let scale = window(&values);

    let points = {
        let usable_width = WIDTH - X_PADDING * 2.0;
        let usable_height = HEIGHT - Y_PADDING * 2.0;
        let denominator = values.len().saturating_sub(1).max(1) as f64;

        values
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let x = X_PADDING + (index as f64 / denominator) * usable_width;
                let y = HEIGHT - Y_PADDING - fraction(&scale, value) * usable_height;

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
            // Stretched to whatever width the card has rather than letterboxed
            // inside it. Strokes opt out of the scaling so they stay even.
            "preserveAspectRatio": "none",
            role: "img",
            "aria-label": "{label}",

            line { class: "grid", x1: 0, y1: 12, x2: WIDTH, y2: 12 }
            line { class: "grid", x1: 0, y1: 33, x2: WIDTH, y2: 33 }
            line { class: "grid", x1: 0, y1: 54, x2: WIDTH, y2: 54 }
            polyline { class: "line", points: "{polyline}", stroke: "{color}" }

            // A round cap on a line that goes nowhere draws a circle, which a
            // stretched `circle` element would not stay.
            line {
                class: "dot",
                x1: "{dot_x:.1}",
                y1: "{dot_y:.1}",
                x2: "{dot_x:.1}",
                y2: "{dot_y:.1}",
                stroke: "{color}",
            }
        }
    }
}

/// The range to draw `values` against.
///
/// Drawing them against everything the thermometer could possibly read leaves
/// an ordinary afternoon as a flat line: the readings occupy under a third of
/// the height, and a degree is worth two pixels. So the chart fits the window
/// it is actually showing, never narrower than [`NARROWEST`].
fn window(values: &[i32]) -> RangeInclusive<i32> {
    let lowest = values.iter().copied().min().unwrap_or(0);
    let highest = values.iter().copied().max().unwrap_or(0);

    let slack = (NARROWEST - (highest - lowest)).max(0);

    (lowest - slack / 2)..=(highest + slack - slack / 2)
}
