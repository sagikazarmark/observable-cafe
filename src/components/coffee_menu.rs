use dioxus::prelude::*;

use crate::menu::MENU;

/// The clickable menu the coffee counter is driven from.
///
/// A purchase reports the drink's position on the menu rather than its name:
/// that index is what the café counts against, and what becomes a label value.
#[component]
pub fn CoffeeMenu(on_purchase: EventHandler<usize>) -> Element {
    rsx! {
        div { class: "coffee-grid",
            for (index , coffee) in MENU.iter().enumerate() {
                button {
                    key: "{coffee.key}",
                    class: "coffee-card",
                    r#type: "button",
                    onclick: move |_| on_purchase.call(index),

                    span { class: "coffee-icon", aria_hidden: "true", "{coffee.icon}" }
                    span { class: "coffee-name", "{coffee.name}" }
                    span { class: "coffee-detail", "{coffee.detail}" }
                }
            }
        }
    }
}
