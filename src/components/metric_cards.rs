use dioxus::prelude::*;

use crate::components::Sparkline;
use crate::inventory::{Ingredient, Roast};
use crate::state::{Gauge, Snapshot};

/// The same café, sorted by metric rather than by moment.
///
/// This is where a number is named as a counter or a gauge, so it is shown
/// only once the café has the words for it.
#[component]
pub fn MetricCards(snapshot: Snapshot) -> Element {
    let inside = snapshot.recorded(|observation| observation.inside);
    let outside = snapshot.recorded(|observation| observation.outside);

    // Each card belongs to an instrument, so a café without the instrument is
    // missing the card rather than showing an empty one: a metric the café
    // does not measure is not a metric at zero.
    rsx! {
        div { class: "cards",
            if let Some(sold) = snapshot.sold {
                article { class: "metric",
                    div { class: "metric-topline",
                        span { class: "metric-name", "Coffees sold" }
                        span { class: "metric-type counter-badge", "Counter" }
                    }
                    div { class: "metric-value-row",
                        span { class: "metric-value", "{sold.total()}" }
                        span { class: "metric-unit", "since the café opened" }
                    }

                    // One row per drink that has been sold, and none for the
                    // rest: a series begins when something is first observed
                    // under it. A till keeping one number has no rows at all,
                    // which is what the value above being the only series
                    // looks like.
                    div { class: "metric-series",
                        for (drink , count) in sold.by_drink() {
                            div { key: "{drink.key}", class: "series",
                                code { "drink=\"{drink.key}\"" }
                                b { "{count}" }
                            }
                        }
                    }
                }
            }

            if let Some(gauge) = snapshot.inside.clone() {
                TemperatureCard {
                    name: "Inside temperature",
                    gauge,
                    recorded: inside,
                    color: "var(--inside)",
                }
            }

            if let Some(gauge) = snapshot.outside.clone() {
                TemperatureCard {
                    name: "Outside temperature",
                    gauge,
                    recorded: outside,
                    color: "var(--outside)",
                }
            }

            // Only a café keeping a shelf has these to show, and they are the
            // other thing a gauge does: the thermometers wander, but stock
            // falls with every sale and jumps when a delivery lands.
            if snapshot.inventory.is_some() {
                BeansCard { snapshot: snapshot.clone() }
                MilkCard { snapshot: snapshot.clone() }
            }
        }
    }
}

/// The beans, published as one gauge in two series: the first label this café
/// puts on anything other than a counter.
///
/// Unlike the drinks above, both roasts are there from the start: a series
/// begins when something is first observed under it, and a shelf is there to
/// be read from the moment the café opens.
#[component]
fn BeansCard(snapshot: Snapshot) -> Element {
    let Some(shelf) = snapshot.inventory else {
        return rsx! {};
    };

    let total: u32 = Roast::ALL
        .iter()
        .map(|&roast| shelf.amount(Ingredient::Beans(roast)))
        .sum();

    rsx! {
        article { class: "metric",
            div { class: "metric-topline",
                span { class: "metric-name", "Coffee beans" }
                span { class: "metric-type gauge-badge", "Gauge" }
            }
            div { class: "metric-value-row",
                span { class: "metric-value", "{total}" }
                span { class: "metric-unit", "g on the shelf · right now" }
            }

            div { class: "metric-series",
                for roast in Roast::ALL {
                    StockSeries {
                        key: "{roast.key()}",
                        row: rsx! {
                            code { "roast=\"{roast.key()}\"" }
                            b { "{shelf.amount(Ingredient::Beans(roast))} g" }
                        },
                        name: "{roast.name()} beans",
                        unit: "g",
                        recorded: snapshot.recorded_stock(Ingredient::Beans(roast)),
                        color: match roast {
                            Roast::Light => "var(--light-roast)",
                            Roast::Dark => "var(--dark-roast)",
                        },
                    }
                }
            }
        }
    }
}

/// The milk, kept in millilitres and published in litres: the exposition
/// speaks base units, and the two spellings of one fridge are worth seeing
/// side by side.
#[component]
fn MilkCard(snapshot: Snapshot) -> Element {
    let Some(shelf) = snapshot.inventory else {
        return rsx! {};
    };

    let millilitres = shelf.amount(Ingredient::Milk);
    let litres = f64::from(millilitres) / 1000.0;

    rsx! {
        article { class: "metric",
            div { class: "metric-topline",
                span { class: "metric-name", "Milk" }
                span { class: "metric-type gauge-badge", "Gauge" }
            }
            div { class: "metric-value-row",
                span { class: "metric-value", "{millilitres}" }
                span { class: "metric-unit", "ml in the fridge · right now" }
            }

            div { class: "metric-series",
                // The exposition's own spelling of the number above: litres
                // on the wire, millilitres in the fridge.
                StockSeries {
                    row: rsx! {
                        code { "cafe_milk_litres" }
                        b { "{litres}" }
                    },
                    name: "milk",
                    unit: "ml",
                    recorded: snapshot.recorded_stock(Ingredient::Milk),
                    color: "var(--milk)",
                }
            }
        }
    }
}

/// One series of a stock gauge: how `/metrics` spells it, and the readings
/// the notebook holds of it, on the same terms as the temperature charts —
/// what was sold or delivered between two entries was never recorded.
#[component]
fn StockSeries(
    row: Element,
    name: String,
    unit: &'static str,
    recorded: Vec<i32>,
    color: &'static str,
) -> Element {
    let lowest = recorded.iter().copied().min();
    let highest = recorded.iter().copied().max();

    rsx! {
        div { class: "series", {row} }
        // A café that keeps no notebook, or has not filled it yet, has
        // nothing to draw: the amount above is the whole of what it knows.
        if !recorded.is_empty() {
            Sparkline {
                values: recorded.clone(),
                color,
                label: "{name}, as written down",
            }
        }
        if let (Some(lowest), Some(highest)) = (lowest, highest) {
            div { class: "chart-range",
                span { "written down: {lowest} {unit} to {highest} {unit}" }
                span { "{recorded.len()} entries" }
            }
        }
    }
}

#[component]
fn TemperatureCard(name: String, gauge: Gauge, recorded: Vec<i32>, color: String) -> Element {
    let value = gauge.value();
    // The chart fits the readings rather than the thermometer's whole scale,
    // so it has to say what it is fitted to; otherwise a wiggle could be a
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
            // A café that keeps no notebook has nothing to draw a chart from:
            // the reading above is the whole of what it knows. An empty frame
            // would suggest a history that was never kept.
            if !recorded.is_empty() {
                Sparkline {
                    values: recorded.clone(),
                    color,
                    label: "{name}, as written down",
                }
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
