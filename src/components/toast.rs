use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

const VISIBLE_FOR: Duration = Duration::from_millis(1500);

/// How a toast is meant: confirming what was done, or refusing what was
/// asked. A refusal is inked differently, since "out of milk" read in the
/// same voice as "latte served" would take a moment too long to land.
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Confirms,
    Refuses,
}

/// Handle for the transient message shown at the bottom of the screen.
#[derive(Clone, Copy, PartialEq)]
pub struct Toaster {
    message: Signal<String>,
    tone: Signal<Tone>,
    visible: Signal<bool>,
    /// Bumped on every message so a stale timer cannot hide a newer toast.
    epoch: Signal<u64>,
}

pub fn use_toaster() -> Toaster {
    Toaster {
        message: use_signal(String::new),
        tone: use_signal(|| Tone::Confirms),
        visible: use_signal(|| false),
        epoch: use_signal(|| 0),
    }
}

impl Toaster {
    pub fn show(self, message: impl Into<String>) {
        self.serve(message, Tone::Confirms);
    }

    /// Shows `message` as a refusal rather than a confirmation.
    pub fn refuse(self, message: impl Into<String>) {
        self.serve(message, Tone::Refuses);
    }

    fn serve(mut self, message: impl Into<String>, tone: Tone) {
        let epoch = *self.epoch.read() + 1;

        self.epoch.set(epoch);
        self.message.set(message.into());
        self.tone.set(tone);
        self.visible.set(true);

        spawn(async move {
            Delay::new(VISIBLE_FOR).await;

            let superseded = *self.epoch.read() != epoch;
            if !superseded {
                self.visible.set(false);
            }
        });
    }
}

#[component]
pub fn Toast(toaster: Toaster) -> Element {
    let class = match (*toaster.visible.read(), *toaster.tone.read()) {
        (true, Tone::Refuses) => "toast show refused",
        (true, Tone::Confirms) => "toast show",
        (false, _) => "toast",
    };

    rsx! {
        div {
            class: "{class}",
            role: "status",
            aria_live: "polite",
            "{toaster.message}"
        }
    }
}
