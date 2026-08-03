use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

const VISIBLE_FOR: Duration = Duration::from_millis(1500);

/// Handle for the transient message shown at the bottom of the screen.
#[derive(Clone, Copy, PartialEq)]
pub struct Toaster {
    message: Signal<String>,
    visible: Signal<bool>,
    /// Bumped on every message so a stale timer cannot hide a newer toast.
    epoch: Signal<u64>,
}

pub fn use_toaster() -> Toaster {
    Toaster {
        message: use_signal(String::new),
        visible: use_signal(|| false),
        epoch: use_signal(|| 0),
    }
}

impl Toaster {
    pub fn show(mut self, message: impl Into<String>) {
        let epoch = *self.epoch.read() + 1;

        self.epoch.set(epoch);
        self.message.set(message.into());
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
    let class = if *toaster.visible.read() {
        "toast show"
    } else {
        "toast"
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
