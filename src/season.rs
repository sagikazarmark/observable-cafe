//! What a thermometer ought to read right now.
//!
//! Temperatures follow the calendar: cold in January, warm in July, and a few
//! degrees cooler before dawn than mid-afternoon. Working the readings out from
//! the current date keeps them plausible whenever the demo happens to be run.
//!
//! The numbers describe a temperate climate — mild summers, freezing winters.

use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use chrono::{DateTime, Datelike, Local, Timelike};

/// The average outside temperature across the whole year.
const ANNUAL_MEAN: f64 = 11.0;
/// How far above that average midsummer gets, and midwinter below it.
const SEASONAL_SWING: f64 = 11.0;
/// The day of the year the outside temperature peaks: late July.
const WARMEST_DAY: f64 = 205.0;

/// How far the afternoon rises above the day's average, and dawn falls below.
const DAILY_SWING: f64 = 5.0;
/// The hour the outside temperature peaks.
const WARMEST_HOUR: f64 = 15.0;

/// The temperature the café heats itself to.
const ROOM: f64 = 22.0;
/// How much of a hot afternoon makes it past the door.
const DRAUGHT: f64 = 0.25;

/// The temperature outside the café.
pub fn outside() -> i32 {
    outside_at(Local::now()).round() as i32
}

/// The temperature in the café.
pub fn inside() -> i32 {
    inside_at(Local::now()).round() as i32
}

/// The temperatures outside plausible for the rest of the day.
///
/// Wide enough that a thermometer drawn against it has somewhere to go.
pub fn outside_range() -> RangeInclusive<i32> {
    let middle = daily_mean(Local::now()).round() as i32;

    (middle - 10)..=(middle + 10)
}

/// The temperatures inside plausible for the rest of the day.
pub fn inside_range() -> RangeInclusive<i32> {
    let middle = indoors(daily_mean(Local::now()) + DAILY_SWING).round() as i32;

    (middle - 6)..=(middle + 6)
}

/// The outside temperature at a given moment, before rounding.
fn outside_at(at: DateTime<Local>) -> f64 {
    let hour = f64::from(at.hour()) + f64::from(at.minute()) / 60.0;
    let time_of_day = DAILY_SWING * (TAU * (hour - WARMEST_HOUR) / 24.0).cos();

    daily_mean(at) + time_of_day
}

fn inside_at(at: DateTime<Local>) -> f64 {
    indoors(outside_at(at))
}

/// What the day averages out to outside, ignoring the hour.
fn daily_mean(at: DateTime<Local>) -> f64 {
    let day = f64::from(at.ordinal());

    ANNUAL_MEAN + SEASONAL_SWING * (TAU * (day - WARMEST_DAY) / 365.0).cos()
}

/// The café heats itself in the cold, but nothing stops it warming up when the
/// weather is hotter than the room.
fn indoors(outside: f64) -> f64 {
    ROOM + DRAUGHT * (outside - ROOM).max(0.0)
}
