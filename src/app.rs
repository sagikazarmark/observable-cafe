use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::components::{CoffeeMenu, Notebook, Thermometers, Toast, use_toaster};
use crate::rng::Rng;
use crate::state::Gauge;

const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn App() -> Element {
    let mut coffees_sold = use_signal(|| 0_u32);
    let mut inside = use_signal(Gauge::inside);
    let mut outside = use_signal(Gauge::outside);
    let toaster = use_toaster();

    // Both gauges take a new reading every 5–10 seconds, independently of
    // anything the visitor does.
    let mut readings = use_future(move || async move {
        let mut rng = Rng::from_clock();

        loop {
            let delay = 5_000 + rng.below(5_001);
            Delay::new(Duration::from_millis(delay)).await;

            let inside_step = signed_step(&mut rng);
            let outside_step = signed_step(&mut rng);

            inside.write().drift(inside_step);
            outside.write().drift(outside_step);
        }
    });

    let purchase = move |name: String| {
        coffees_sold += 1;
        toaster.show(format!("{name} purchased · coffees sold: {coffees_sold}"));
    };

    let reset = move |_| {
        coffees_sold.set(0);
        inside.set(Gauge::inside());
        outside.set(Gauge::outside());
        readings.restart();
        toaster.show("Demo reset");
    };

    rsx! {
        document::Stylesheet { href: MAIN_CSS }

        main { class: "app",
            header { class: "topbar",
                div {
                    p { class: "eyebrow", "Metrics playground" }
                    h1 { "The Observable Café" }
                    p { class: "subtitle",
                        "Buy a coffee to increase a counter. Watch the temperatures drift to see how gauges behave."
                    }
                }
                button { class: "reset", r#type: "button", onclick: reset, "Reset demo" }
            }

            div { class: "layout",
                section { class: "menu-panel", aria_labelledby: "coffeeMenuTitle",
                    div { class: "section-heading",
                        h2 { id: "coffeeMenuTitle", "Coffee menu" }
                        span { "Click to buy" }
                    }

                    CoffeeMenu { on_purchase: purchase }
                    Thermometers { inside, outside }
                }

                Notebook { coffees_sold, inside, outside }
            }
        }

        Toast { toaster }
    }
}

/// A step of one to three degrees, in either direction.
fn signed_step(rng: &mut Rng) -> i32 {
    let magnitude = rng.below(3) as i32 + 1;

    if rng.below(2) == 0 {
        -magnitude
    } else {
        magnitude
    }
}
