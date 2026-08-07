use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::api;
use crate::clock;
use crate::components::{CoffeeMenu, Record, Thermometers, Toast, use_toaster};
use crate::lesson::Lesson;
use crate::menu::MENU;
use crate::route::Route;
use crate::state::Snapshot;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const HAND: Asset = asset!("/assets/patrick-hand.woff2");

/// How often the page asks the server what the café looks like.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[component]
pub fn App() -> Element {
    let single_lesson = use_server_cached(|| {
        #[cfg(feature = "server")]
        {
            crate::server::single_lesson()
        }

        #[cfg(not(feature = "server"))]
        {
            None::<Lesson>
        }
    });

    rsx! {
        document::Stylesheet { href: MAIN_CSS }

        // Declared here rather than in the stylesheet so the font resolves to
        // whatever filename the asset pipeline settles on.
        document::Style {
            "@font-face {{ font-family: 'Patrick Hand'; src: url('{HAND}') format('woff2'); font-weight: 400; font-display: swap; }}"
        }

        if let Some(lesson) = single_lesson {
            Cafe { lesson }
        } else {
            Router::<Route> {}
        }
    }
}

/// The café, showing as much of itself as `lesson` calls for.
#[component]
pub fn Cafe(lesson: Lesson) -> Element {
    let mut cafe = use_signal(Snapshot::new);
    let mut observe_every = use_signal(|| clock::DEFAULT_OBSERVE_EVERY);
    let toaster = use_toaster();

    // The café keeps counting and measuring whether or not anybody is looking,
    // so the page holds no state of its own — it re-reads the server's. How
    // often it is written down is the one thing the page does decide, and it
    // is asserted on every poll rather than sent once.
    use_future(move || async move {
        loop {
            if let Ok(observed) = api::snapshot(lesson, observe_every()).await {
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
            // down — saying it here would give the game away.
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

    rsx! {
        div { class: "app",
            // No title: this is embedded in a page that already has one, and
            // the height is worth more to the notebook.
            header { class: "bar",
                div { class: "clock-block",
                    span { class: "clock", aria_label: "Café clock", "{snapshot.clock}" }
                    // Without this the box and the notebook appear to
                    // disagree: seven seconds produces a seven minute gap.
                    span { class: "clock-note", "1 second = 1 minute" }
                }

                div { class: "bar-actions",
                    if snapshot.automatic_observations {
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

                        // No link to /metrics here. A lesson is embedded in a
                        // course page, and pointing away from it is that page's
                        // business; the index is where the endpoint is offered.
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

            main { class: "layout",
                section { class: "cafe-panel", aria_label: "The café",
                    CoffeeMenu { on_purchase: purchase }
                    Thermometers {
                        inside: snapshot.inside.clone(),
                        outside: snapshot.outside.clone(),
                    }
                }

                Record { lesson, snapshot }
            }
        }

        Toast { toaster }
    }
}
