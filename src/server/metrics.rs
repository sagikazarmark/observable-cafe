//! The scrape endpoint the whole demo exists for.
//!
//! There is one of these, and it always looks like a real target: the same
//! bytes whatever the page is showing, and the same bytes to a browser as to
//! `curl`. It reports the café as it stands the instant it is asked and keeps
//! nothing, so reloading it is a scrape, and the numbers here run ahead of the
//! ones in the notebook.

use std::fmt::Write as _;

use dioxus::server::axum::http::header::CONTENT_TYPE;
use dioxus::server::axum::response::IntoResponse;

use crate::inventory::{Ingredient, Roast};
use crate::state::{Sales, Snapshot};

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

    // Reported when the café keeps the count and left out when it does not,
    // which still reads no feature: what the café measures decides what the
    // café is made of, and this page reports the café.
    if let Some(sold) = &snapshot.sold {
        out.push_str(
            "# HELP cafe_coffees_sold_total Coffees sold since the café opened.\n\
             # TYPE cafe_coffees_sold_total counter\n",
        );

        match sold {
            // One series per drink actually sold, and no unlabelled series
            // alongside them: a total published next to its own breakdown
            // would be counted twice by any query that summed the lot. Adding
            // them up is the reader's job.
            Sales::ByDrink(_) => {
                for (drink, count) in sold.by_drink() {
                    let _ = writeln!(
                        out,
                        "cafe_coffees_sold_total{{drink=\"{key}\"}} {count}",
                        key = drink.key,
                    );
                }
            }
            // A till keeping one number is one series, and it is there from
            // the first scrape: a plain counter exists from the moment the
            // till does. Beginning only when something is first observed
            // under it is the labelled form's lesson, not this one's.
            Sales::Total(count) => {
                let _ = writeln!(out, "cafe_coffees_sold_total {count}");
            }
        }
    }

    if let (Some(inside), Some(outside)) = (&snapshot.inside, &snapshot.outside) {
        let _ = write!(
            out,
            "# HELP cafe_inside_temperature_celsius Most recent reading from the thermometer in the café.\n\
             # TYPE cafe_inside_temperature_celsius gauge\n\
             cafe_inside_temperature_celsius {inside}\n\
             # HELP cafe_outside_temperature_celsius Most recent reading from the thermometer outside.\n\
             # TYPE cafe_outside_temperature_celsius gauge\n\
             cafe_outside_temperature_celsius {outside}\n",
            inside = inside.value(),
            outside = outside.value(),
        );
    }

    // Reported when the café has a shelf and left out when it has not, which
    // still reads no feature: `inventory` decides what the café is made of,
    // and this page reports the café.
    if let Some(shelf) = &snapshot.inventory {
        out.push_str(
            "# HELP cafe_beans_grams Grams of coffee beans on the shelf, by roast.\n\
             # TYPE cafe_beans_grams gauge\n",
        );

        // Both roasts from the first scrape, unlike the drinks above: a
        // series begins when something is first observed under it, and a
        // shelf is there to be read from the moment the café opens.
        for roast in Roast::ALL {
            let _ = writeln!(
                out,
                "cafe_beans_grams{{roast=\"{key}\"}} {grams}",
                key = roast.key(),
                grams = shelf.amount(Ingredient::Beans(roast)),
            );
        }

        // Millilitres behind the counter, litres on the wire: the exposition
        // speaks base units, as the conventions ask, and leaves the page free
        // to speak the fridge's own.
        let _ = write!(
            out,
            "# HELP cafe_milk_litres Litres of milk in the fridge.\n\
             # TYPE cafe_milk_litres gauge\n\
             cafe_milk_litres {litres}\n",
            litres = f64::from(shelf.amount(Ingredient::Milk)) / 1000.0,
        );
    }

    out
}

#[cfg(test)]
mod tests {
    use super::exposition;
    use crate::clock;
    use crate::state::{Gauge, Sales, Snapshot};

    /// A café measuring only what the test says it does. [`Snapshot::new`]
    /// starts with no instruments at all, which is exactly the point here.
    fn measuring(sold: Option<Sales>) -> Snapshot {
        Snapshot {
            sold,
            ..Snapshot::new()
        }
    }

    #[test]
    fn a_labelled_count_is_published_as_its_breakdown() {
        let mut sold = Sales::new(true);
        sold.ring_up(0);

        let out = exposition(&measuring(Some(sold)));

        assert!(out.contains("cafe_coffees_sold_total{drink=\"espresso\"} 1"));
        assert!(!out.contains("cafe_coffees_sold_total 1"));
    }

    /// A till keeping one number is one series, and it is there from the
    /// first scrape rather than from the first sale.
    #[test]
    fn an_unlabelled_count_is_published_from_zero() {
        let out = exposition(&measuring(Some(Sales::new(false))));

        assert!(out.contains("cafe_coffees_sold_total 0"));
        assert!(!out.contains("drink="));
    }

    /// Not a series with no data but no metric at all, `HELP` and `TYPE`
    /// included: a café that does not measure its sales has nothing to say
    /// about them, not a blank to leave.
    #[test]
    fn an_unmeasured_count_is_not_published_at_all() {
        let out = exposition(&measuring(None));

        assert!(!out.contains("cafe_coffees_sold_total"));
    }

    #[test]
    fn unread_thermometers_are_not_published() {
        let out = exposition(&measuring(None));

        assert!(!out.contains("temperature"));
    }

    #[test]
    fn read_thermometers_are_published() {
        let opening = clock::opening();
        let mut snapshot = measuring(None);
        snapshot.inside = Some(Gauge::inside(opening));
        snapshot.outside = Some(Gauge::outside(opening));

        let out = exposition(&snapshot);

        assert!(out.contains("cafe_inside_temperature_celsius"));
        assert!(out.contains("cafe_outside_temperature_celsius"));
    }
}
