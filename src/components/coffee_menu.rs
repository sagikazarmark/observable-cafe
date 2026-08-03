use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct Coffee {
    icon: &'static str,
    name: &'static str,
    detail: &'static str,
}

const MENU: [Coffee; 4] = [
    Coffee {
        icon: "☕",
        name: "Espresso",
        detail: "Small, intense, immediate",
    },
    Coffee {
        icon: "🥛",
        name: "Cappuccino",
        detail: "Espresso with velvety foam",
    },
    Coffee {
        icon: "🫗",
        name: "Latte",
        detail: "Smooth and milk-forward",
    },
    Coffee {
        icon: "♨️",
        name: "Americano",
        detail: "Espresso lengthened with water",
    },
];

/// The clickable menu that drives the coffees sold counter.
#[component]
pub fn CoffeeMenu(on_purchase: EventHandler<String>) -> Element {
    rsx! {
        div { class: "coffee-grid",
            for coffee in MENU {
                button {
                    key: "{coffee.name}",
                    class: "coffee-card",
                    r#type: "button",
                    onclick: move |_| on_purchase.call(coffee.name.to_owned()),

                    span { class: "coffee-icon", aria_hidden: "true", "{coffee.icon}" }
                    span { class: "coffee-name", "{coffee.name}" }
                    span { class: "coffee-detail", "{coffee.detail}" }
                }
            }
        }
    }
}
