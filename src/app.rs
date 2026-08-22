use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::api;
use crate::clock;
use crate::components::{CoffeeMenu, Header, Notebook, Thermometers, Toast, use_toaster};
use crate::feature::Features;
use crate::menu::MENU;
use crate::state::Snapshot;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const HAND: Asset = asset!("/assets/patrick-hand.woff2");

/// How often the page asks the server what the café looks like.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[component]
pub fn App() -> Element {
    // Decided by the server and handed down: the café is told what to show
    // once, when it starts, so the page has nothing to ask and nothing to
    // choose. Anything wanting a different café runs a different café.
    let features = use_server_cached(|| {
        #[cfg(feature = "server")]
        {
            crate::server::features()
        }

        #[cfg(not(feature = "server"))]
        {
            Features::all()
        }
    });

    use_context_provider(|| features);

    rsx! {
        // The same string `/version` reports, so a page already open can be
        // told which build drew it without going and asking.
        document::Meta { name: "version", content: crate::VERSION }

        document::Stylesheet { href: MAIN_CSS }

        // Declared here rather than in the stylesheet so the font resolves to
        // whatever filename the asset pipeline settles on.
        document::Style {
            "@font-face {{ font-family: 'Patrick Hand'; src: url('{HAND}') format('woff2'); font-weight: 400; font-display: swap; }}"
        }

        Cafe {}
    }
}

/// The café, showing as much of itself as it was told to.
#[component]
pub fn Cafe() -> Element {
    let features: Features = use_context();
    let mut cafe = use_signal(Snapshot::new);
    let mut observe_every = use_signal(|| clock::DEFAULT_OBSERVE_EVERY);
    let toaster = use_toaster();

    // The café keeps counting and measuring whether or not anybody is looking,
    // so the page holds no state of its own; it re-reads the server's. How
    // often it is written down is the one thing the page does decide, and it
    // is asserted on every poll rather than sent once.
    use_future(move || async move {
        loop {
            if let Ok(observed) = api::snapshot(observe_every()).await {
                cafe.set(observed);
            }

            Delay::new(POLL_INTERVAL).await;
        }
    });

    let purchase = move |drink: usize| {
        spawn(async move {
            let Ok(observed) = api::buy(drink).await else {
                return;
            };

            // Deliberately no number. The sale has happened, but how many have
            // been sold is not something anybody knows until it is written
            // down. Saying it here would give the game away.
            if let Some(bought) = MENU.get(drink) {
                toaster.show(format!("{} served", bought.name));
            }

            cafe.set(observed);
        });
    };

    let note = move |_| {
        spawn(async move {
            if let Ok(observed) = api::note().await {
                cafe.set(observed);
            }
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

    let snapshot = cafe();
    let (lowest, highest) = (*clock::OBSERVE_RANGE.start(), *clock::OBSERVE_RANGE.end());
    // Without the record there is one panel rather than two, and it should
    // have the page rather than half of it.
    let layout = if features.notebook {
        "layout"
    } else {
        "layout alone"
    };

    rsx! {
        div { class: "app",
            header { class: "bar",
                div { class: "bar-lead",
                    // Left out where the café is embedded in a page that has
                    // already named it: saying so twice costs height the
                    // record is worth more of.
                    if features.header {
                        Header {}
                    }

                    div { class: "clock-block",
                        span { class: "clock", aria_label: "Café clock", "{snapshot.clock}" }
                        // Without this the box and the notebook appear to
                        // disagree: seven seconds produces a seven minute gap.
                        span { class: "clock-note", "1 second = 1 minute" }
                    }
                }

                div { class: "bar-actions",
                    // Only while the café writes itself down on a timer. With
                    // the timer off there is no cadence to pace: the notebook
                    // fills when it is asked to and at no other time.
                    if features.automatic_observations {
                        label { class: "interval",
                            "Observe every"
                            input {
                                r#type: "number",
                                min: "{lowest}",
                                max: "{highest}",
                                step: "1",
                                value: "{observe_every}",
                                // Committed on blur or Enter rather than per
                                // keystroke: typing 15 passes through 1, and the
                                // café should not be re-paced by a half-typed
                                // number. Anything outside the range snaps back.
                                onchange: move |event| {
                                    let chosen = event
                                        .value()
                                        .parse::<u64>()
                                        .unwrap_or(clock::DEFAULT_OBSERVE_EVERY);

                                    observe_every.set(clock::observe_every(chosen));
                                },
                            }
                            "s"
                        }
                    }

                    // Offered whether or not the timer is running: turning the
                    // timer off hands the notebook over to whoever is reading
                    // rather than closing it, so that a sale and the entry
                    // recording it can be put side by side deliberately. A
                    // café keeping no notebook at all offers neither.
                    if features.observations {
                        button {
                            class: "ghost",
                            r#type: "button",
                            onclick: note,
                            "Observe now"
                        }
                    }

                    button {
                        class: "ghost",
                        r#type: "button",
                        onclick: reset,
                        "Reset demo"
                    }
                }
            }

            main { class: "{layout}",
                section { class: "cafe-panel", aria_label: "The café",
                    CoffeeMenu { on_purchase: purchase }
                    Thermometers {
                        inside: snapshot.inside.clone(),
                        outside: snapshot.outside.clone(),
                    }
                }

                // Left out entirely when there is nothing to keep in it: an
                // empty panel beside the café teaches nothing and takes half
                // the width to do it.
                if features.notebook {
                    Notebook { snapshot }
                }
            }
        }

        Toast { toaster }
    }
}
