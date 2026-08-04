//! The scrape endpoint the whole demo exists for.
//!
//! There is one of these, and it always looks like a real target: the same
//! bytes whichever lesson is open, and the same bytes to a browser as to
//! `curl`. It reports the café as it stands the instant it is asked and keeps
//! nothing, so reloading it is a scrape, and the numbers here run ahead of the
//! ones in the notebook.

use std::fmt::Write as _;

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
    let mut out = String::new();

    out.push_str(
        "# HELP cafe_coffees_sold_total Coffees sold since the café opened.\n\
         # TYPE cafe_coffees_sold_total counter\n",
    );

    // One series per drink actually sold, and no unlabelled series alongside
    // them: a total published next to its own breakdown would be counted twice
    // by any query that summed the lot. Adding them up is the reader's job.
    for (drink, count) in snapshot.sold.by_drink() {
        let _ = writeln!(
            out,
            "cafe_coffees_sold_total{{drink=\"{key}\"}} {count}",
            key = drink.key,
        );
    }

    let _ = write!(
        out,
        "# HELP cafe_inside_temperature_celsius Most recent reading from the thermometer in the café.\n\
         # TYPE cafe_inside_temperature_celsius gauge\n\
         cafe_inside_temperature_celsius {inside}\n\
         # HELP cafe_outside_temperature_celsius Most recent reading from the thermometer outside.\n\
         # TYPE cafe_outside_temperature_celsius gauge\n\
         cafe_outside_temperature_celsius {outside}\n",
        inside = snapshot.inside.value(),
        outside = snapshot.outside.value(),
    );

    out
}
