use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::api;
use crate::components::{CoffeeMenu, Notebook, Thermometers, Toast, use_toaster};
use crate::state::Snapshot;

const MAIN_CSS: Asset = asset!("/assets/main.css");

/// How often the page asks the server what it has observed.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[component]
pub fn App() -> Element {
    let mut cafe = use_signal(Snapshot::new);
    let toaster = use_toaster();

    // The café keeps counting and measuring whether or not anybody is looking,
    // so the page holds no state of its own — it re-reads the server's.
    use_future(move || async move {
        loop {
            if let Ok(observed) = api::observe().await {
                cafe.set(observed);
            }

            Delay::new(POLL_INTERVAL).await;
        }
    });

    let purchase = move |name: String| {
        spawn(async move {
            let Ok(observed) = api::buy_coffee().await else {
                return;
            };

            toaster.show(format!(
                "{name} purchased · coffees sold: {}",
                observed.coffees_sold
            ));
            cafe.set(observed);
        });
    };

    let reset = move |_| {
        spawn(async move {
            if let Ok(observed) = api::reset().await {
                cafe.set(observed);
                toaster.show("Demo reset");
            }
        });
    };

    let Snapshot {
        coffees_sold,
        inside,
        outside,
    } = cafe();

    rsx! {
        document::Stylesheet { href: MAIN_CSS }

        main { class: "app",
            header { class: "topbar",
                h1 { "The Observable Café" }
                button { class: "reset", r#type: "button", onclick: reset, "Reset demo" }
            }

            div { class: "layout",
                section { class: "menu-panel", aria_labelledby: "coffeeMenuTitle",
                    div { class: "section-heading",
                        h2 { id: "coffeeMenuTitle", "Coffee menu" }
                        span { "Click to buy" }
                    }

                    CoffeeMenu { on_purchase: purchase }
                    Thermometers { inside: inside.clone(), outside: outside.clone() }
                }

                Notebook { coffees_sold, inside, outside }
            }
        }

        Toast { toaster }
    }
}
