//! The café's own clock.
//!
//! The café runs faster than the world outside it: a second at the keyboard is
//! a minute behind the counter. A stage lasting a few minutes therefore covers
//! a whole working day, which gives the thermometers a morning to warm out of
//! and an evening to cool into.
//!
//! Everything in the demo takes its time from here, the temperatures included.
//! Reading the hour off the server's clock instead would put the calendar and
//! the notebook into open disagreement.

use std::ops::RangeInclusive;

#[cfg(feature = "server")]
use chrono::TimeDelta;
use chrono::{DateTime, Local};

/// The hour the café opens, and what its clock reads at tick zero.
const OPENS_AT: u32 = 8;

/// How much café time passes per tick. One tick is one second of real time.
#[cfg(feature = "server")]
const MINUTES_PER_TICK: i64 = 1;

/// How often the owner writes something down, in ticks, until told otherwise.
pub const DEFAULT_OBSERVE_EVERY: u64 = 7;

/// How far apart the writing down may be set, in ticks.
///
/// Widening it is what makes under-sampling visible: the café goes on moving
/// at the same rate whatever is chosen, so the record simply keeps less of it.
/// Below the floor there is nothing to miss; above the ceiling the wait stops
/// being worth sitting through.
pub const OBSERVE_RANGE: RangeInclusive<u64> = 5..=30;

/// Narrows an interval to something the café will accept.
///
/// Applied on both sides: the page keeps its own box honest, and the café does
/// not trust a number that arrived over the wire.
pub fn observe_every(requested: u64) -> u64 {
    requested.clamp(*OBSERVE_RANGE.start(), *OBSERVE_RANGE.end())
}

/// The moment the café opened, on today's date.
///
/// Both halves of the app work this out independently, so the browser can draw
/// a plausible thermometer before the server has told it anything.
pub fn opening() -> DateTime<Local> {
    let today = Local::now().date_naive();

    today
        .and_hms_opt(OPENS_AT, 0, 0)
        // The hour the café opens exists on every day of the year in every
        // timezone this demo is likely to meet, but a clock put forward over
        // it would leave nothing to return.
        .and_then(|opening| opening.and_local_timezone(Local).earliest())
        .unwrap_or_else(Local::now)
}

/// What the café clock reads `tick` ticks after opening.
///
/// Only the café itself counts ticks; the browser is told the time.
#[cfg(feature = "server")]
pub fn at(tick: u64) -> DateTime<Local> {
    let minutes = i64::try_from(tick).unwrap_or(i64::MAX) * MINUTES_PER_TICK;

    opening() + TimeDelta::minutes(minutes)
}

/// A reading as the owner would write it into the notebook.
pub fn written(at: DateTime<Local>) -> String {
    at.format("%H:%M").to_string()
}

/// The day an entry belongs to, as it would be headed on the page.
///
/// The café passes midnight after sixteen minutes of being watched, whatever
/// the interval, so entries have to say which day they are from.
pub fn dated(at: DateTime<Local>) -> String {
    at.format("%b %d, %Y").to_string().to_uppercase()
}
