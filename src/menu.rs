//! What the café sells.
//!
//! The menu is shared rather than owned by the browser: `key` doubles as the
//! label value under which a drink is published, so the buttons on screen and
//! the series on `/metrics` cannot drift apart.

/// One item on the menu.
#[derive(Clone, Copy)]
pub struct Drink {
    /// How the drink identifies itself in a label value.
    pub key: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub detail: &'static str,
}

pub const MENU: [Drink; 4] = [
    Drink {
        key: "espresso",
        icon: "☕",
        name: "Espresso",
        detail: "Small, intense, immediate",
    },
    Drink {
        key: "cappuccino",
        icon: "🥛",
        name: "Cappuccino",
        detail: "Espresso with velvety foam",
    },
    Drink {
        key: "latte",
        icon: "🫗",
        name: "Latte",
        detail: "Smooth and milk-forward",
    },
    Drink {
        key: "americano",
        icon: "♨️",
        name: "Americano",
        detail: "Espresso lengthened with water",
    },
];
