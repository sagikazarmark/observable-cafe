//! Ruling a record off where the café day turns over.

use crate::state::{Observation, Sale};

/// Something written down at a moment, on a particular café day.
pub trait Written {
    /// Which line this is, counting from the café opening.
    ///
    /// Lines travel up the page as a record fills and the oldest falls off, so
    /// a day rule needs something that stays with the line it introduces
    /// rather than describing where that line currently sits.
    fn seq(&self) -> u64;

    /// The café day it was written on.
    fn day(&self) -> &str;
}

impl Written for Observation {
    fn seq(&self) -> u64 {
        self.seq
    }

    fn day(&self) -> &str {
        &self.day
    }
}

impl Written for Sale {
    fn seq(&self) -> u64 {
        self.seq
    }

    fn day(&self) -> &str {
        &self.day
    }
}

/// One line of a record, which is usually something written down but is
/// sometimes the owner heading a new day.
pub enum Ruled<'a, T> {
    /// A rule headed with the day, taking its identity from the line it
    /// introduces so that it travels alongside it.
    Day {
        day: &'a str,
        before: u64,
    },
    Line(&'a T),
}

impl<'a, T: Written> Ruled<'a, T> {
    /// Lays a record out as it was written, ruling off wherever the café day
    /// turns over.
    ///
    /// A page left open passes midnight after sixteen minutes, and without the
    /// rule two lines reading `09:00` would look like the same moment. Every
    /// record the café keeps needs this, which is why it is here rather than
    /// in whichever one needed it first.
    pub fn from(written: &'a [T]) -> Vec<Self> {
        let mut lines = Vec::with_capacity(written.len());
        let mut day: Option<&str> = None;

        for line in written {
            if day.is_some_and(|previous| previous != line.day()) {
                lines.push(Self::Day {
                    day: line.day(),
                    before: line.seq(),
                });
            }

            day = Some(line.day());
            lines.push(Self::Line(line));
        }

        lines
    }
}
