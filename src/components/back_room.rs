//! The back room, where the shelf is kept stocked.
//!
//! Reached at `/admin` and linked from nowhere on the café page: the owner
//! knows the way, and a customer has no business behind the counter. It is
//! meant to be open in a second tab beside the café, so that a sale out
//! front is a dent back here a second later, and a delivery taken in here is
//! a latte that can be sold again out there.

use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::api;
use crate::components::{Header, Toast, use_toaster};
use crate::inventory::{Ingredient, Inventory, Roast};
use crate::menu::MENU;
use crate::state::Snapshot;

/// How often the back room looks at the café: the same second the café page
/// polls at, so the two tabs never disagree for longer than that.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// The back room: the shelf, and the board of what cannot be made.
#[component]
pub fn BackRoom() -> Element {
    let mut cafe = use_signal(Snapshot::new);
    let toaster = use_toaster();

    // The back room re-reads the café rather than keeping its own state, for
    // the same reason the café page does — with one difference: it asks as an
    // observer, so an unvisited café stays at opening time however long the
    // owner stands back here counting beans.
    use_future(move || async move {
        loop {
            if let Ok(observed) = api::stocktake().await {
                cafe.set(observed);
            }

            Delay::new(POLL_INTERVAL).await;
        }
    });

    let restock = move |ingredient: Ingredient| {
        spawn(async move {
            let Ok((taken, observed)) = api::restock(ingredient).await else {
                return;
            };

            cafe.set(observed);

            // The shelf takes what it has room for, so the toast reports what
            // actually landed rather than what the delivery held.
            if taken == 0 {
                toaster.refuse(format!("No room — the {} is full", shelved(ingredient)));
            } else {
                toaster.show(format!("Took in {taken} {}", of(ingredient)));
            }
        });
    };

    let snapshot = cafe();

    rsx! {
        div { class: "app back-room",
            header { class: "bar",
                div { class: "bar-lead",
                    Header {}

                    div { class: "clock-block",
                        span { class: "clock", aria_label: "Café clock", "{snapshot.clock}" }
                        span { class: "clock-note", "the café, out front" }
                    }
                }

                div { class: "bar-actions",
                    span { class: "back-room-title", "The back room" }
                }
            }

            main { class: "layout alone",
                section { class: "cafe-panel", aria_label: "The back room",
                    match snapshot.inventory {
                        Some(shelf) => rsx! {
                            div { class: "stock-list",
                                for ingredient in Ingredient::ALL {
                                    StockRow {
                                        key: "{ingredient.name()}",
                                        shelf,
                                        ingredient,
                                        on_restock: restock,
                                    }
                                }
                            }

                            EightySixBoard { shelf }
                        },
                        // The page draws before the first stocktake answers, so
                        // this is a moment of not knowing rather than a café
                        // without a shelf: a shelfless café never routes here.
                        None => rsx! {
                            div { class: "empty-state",
                                strong { "Counting the shelf…" }
                                "The back room is asking the café what is left."
                            }
                        },
                    }
                }
            }
        }

        Toast { toaster }
    }
}

/// One shelf of one ingredient: what is left, against what it holds, and the
/// delivery that tops it back up.
#[component]
fn StockRow(
    shelf: Inventory,
    ingredient: Ingredient,
    on_restock: EventHandler<Ingredient>,
) -> Element {
    let amount = shelf.amount(ingredient);
    let capacity = Inventory::capacity(ingredient);
    let percent = shelf.fullness(ingredient) * 100.0;

    let unit = match ingredient {
        Ingredient::Beans(_) => "g",
        Ingredient::Milk => "ml",
    };

    // The delivery the button takes in is the delivery the shelf defines, so
    // the label cannot promise a bag the shelf will not recognise.
    let action = match ingredient {
        Ingredient::Beans(_) => format!("Take in a {} g bag", Inventory::delivery(ingredient)),
        Ingredient::Milk => format!(
            "Take in a {} L bottle",
            Inventory::delivery(ingredient) / 1000
        ),
    };

    rsx! {
        article { class: "stock",
            div { class: "stock-head",
                span { class: "stock-name", "{title(ingredient)}" }
                // The same words `/metrics` uses for it, so the bar out here
                // and the series on the wire read as one thing.
                code { class: "stock-series", "{series(ingredient)}" }
            }

            div {
                class: "stock-bar",
                role: "meter",
                aria_label: "{title(ingredient)}",
                aria_valuemin: 0,
                aria_valuemax: "{capacity}",
                aria_valuenow: "{amount}",
                // Only the width travels inline, the way the thermometer
                // sends only its level: the colour is the class's to paint.
                // An inline style holding two declarations loses its second
                // when the first is updated, and a fill with its background
                // dropped is an empty-looking bar.
                div {
                    class: "stock-fill {tint(ingredient)}",
                    style: "width: {percent}%",
                }
            }

            div { class: "stock-foot",
                span { class: "stock-amount",
                    b { "{amount}" }
                    " {unit} of {capacity} {unit}"
                }
                button {
                    class: "ghost",
                    r#type: "button",
                    onclick: move |_| on_restock.call(ingredient),
                    "{action}"
                }
            }
        }
    }
}

/// The drinks the café cannot make right now, and what stands in the way.
///
/// Café slang eighty-sixes a dish the kitchen is out of; the board is where
/// the owner reads it before a customer finds out by ordering one. The café
/// page shows the shelf nowhere, deliberately — out front, the only signs of
/// an empty shelf are this board, the refusals, and the gauge on `/metrics`.
#[component]
fn EightySixBoard(shelf: Inventory) -> Element {
    let eighty_sixed: Vec<_> = MENU
        .iter()
        .filter_map(|drink| shelf.missing_for(drink.recipe).map(|short| (drink, short)))
        .collect();

    rsx! {
        div { class: "eighty-six", aria_label: "The 86 board",
            div { class: "eighty-six-head",
                span { class: "eighty-six-title", "86 board" }
                span { class: "eighty-six-note", "what the café cannot make right now" }
            }

            if eighty_sixed.is_empty() {
                div { class: "eighty-six-clear", "Nothing — every drink on the menu can be made." }
            } else {
                for (drink , short) in eighty_sixed {
                    div { key: "{drink.key}", class: "eighty-six-row",
                        span { class: "eighty-six-drink",
                            span { aria_hidden: "true", "{drink.icon} " }
                            "{drink.name}"
                        }
                        span { class: "eighty-six-reason", "out of {short.name()}" }
                    }
                }
            }
        }
    }
}

/// How the shelf card is headed.
fn title(ingredient: Ingredient) -> &'static str {
    match ingredient {
        Ingredient::Beans(Roast::Light) => "Light roast beans",
        Ingredient::Beans(Roast::Dark) => "Dark roast beans",
        Ingredient::Milk => "Milk",
    }
}

/// The words `/metrics` publishes this stock under.
fn series(ingredient: Ingredient) -> String {
    match ingredient {
        Ingredient::Beans(roast) => format!("cafe_beans_grams{{roast=\"{}\"}}", roast.key()),
        Ingredient::Milk => "cafe_milk_litres".to_owned(),
    }
}

/// The class that paints a fill in its ingredient's colour.
fn tint(ingredient: Ingredient) -> &'static str {
    match ingredient {
        Ingredient::Beans(Roast::Light) => "light-roast",
        Ingredient::Beans(Roast::Dark) => "dark-roast",
        Ingredient::Milk => "milk",
    }
}

/// The ingredient as a toast names what came in: an amount "of" this.
fn of(ingredient: Ingredient) -> String {
    match ingredient {
        Ingredient::Beans(roast) => format!("g of {}", roast.name()),
        Ingredient::Milk => "ml of milk".to_owned(),
    }
}

/// The ingredient as a full shelf is complained about: this is full.
fn shelved(ingredient: Ingredient) -> &'static str {
    match ingredient {
        Ingredient::Beans(_) => "shelf",
        Ingredient::Milk => "fridge",
    }
}
