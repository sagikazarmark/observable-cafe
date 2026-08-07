use chrono::Local;

/// A small xorshift generator.
///
/// The simulated readings only have to look plausible, so seeding from the
/// clock avoids platform specific entropy sources because `getrandom` needs
/// extra build configuration on `wasm32-unknown-unknown`.
pub struct Rng(u64);

impl Rng {
    pub fn from_clock() -> Self {
        let now = Local::now();
        let seed = (now.timestamp_millis() as u64) ^ ((now.timestamp_subsec_nanos() as u64) << 21);

        // A xorshift state of zero only ever produces zero.
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.0 = state;

        state
    }

    /// Returns a value in `0..bound`.
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}
