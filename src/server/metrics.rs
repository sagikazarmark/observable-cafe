//! The scrape endpoint the whole demo exists for.

use dioxus::server::axum::http::header::CONTENT_TYPE;
use dioxus::server::axum::response::IntoResponse;

use crate::state::Snapshot;

/// Prometheus text exposition format, the dialect every scraper understands.
const EXPOSITION_FORMAT: &str = "text/plain; version=0.0.4; charset=utf-8";

pub async fn scrape() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, EXPOSITION_FORMAT)],
        exposition(&super::snapshot()),
    )
}

fn exposition(snapshot: &Snapshot) -> String {
    format!(
        "# HELP cafe_coffees_sold_total Coffees sold since the demo was last reset.\n\
         # TYPE cafe_coffees_sold_total counter\n\
         cafe_coffees_sold_total {sold}\n\
         # HELP cafe_temperature_celsius Most recent temperature reading.\n\
         # TYPE cafe_temperature_celsius gauge\n\
         cafe_temperature_celsius{{location=\"inside\"}} {inside}\n\
         cafe_temperature_celsius{{location=\"outside\"}} {outside}\n",
        sold = snapshot.coffees_sold,
        inside = snapshot.inside.value(),
        outside = snapshot.outside.value(),
    )
}
