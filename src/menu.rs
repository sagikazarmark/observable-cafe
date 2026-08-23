//! What the café sells.
//!
//! The menu is shared rather than owned by the browser: `key` doubles as the
//! label value under which a drink is published, so the buttons on screen and
//! the series on `/metrics` cannot drift apart. The recipe lives here for the
//! same reason: it is part of what the drink is, and the button a customer
//! presses and the deduction it causes must be reading the same numbers.

use crate::inventory::{Recipe, Roast};

/// One item on the menu.
#[derive(Clone, Copy)]
pub struct Drink {
    /// How the drink identifies itself in a label value.
    pub key: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub detail: &'static str,
    /// What making one takes off the shelf, in a café that keeps one.
    ///
    /// The roasts follow the cup rather than the other way round: light roast
    /// for drinks drunk black, where there is no milk to hide behind, and
    /// dark roast for the milk drinks, where anything subtler would drown.
    pub recipe: Recipe,
}

pub const MENU: [Drink; 4] = [
    Drink {
        key: "espresso",
        icon: "☕",
        name: "Espresso",
        detail: "Small, intense, immediate",
        recipe: Recipe {
            roast: Roast::Light,
            beans: 18,
            milk: 0,
        },
    },
    Drink {
        key: "cappuccino",
        icon: "🥛",
        name: "Cappuccino",
        detail: "Espresso with velvety foam",
        recipe: Recipe {
            roast: Roast::Dark,
            beans: 18,
            milk: 100,
        },
    },
    Drink {
        key: "latte",
        icon: "🫗",
        name: "Latte",
        detail: "Smooth and milk-forward",
        recipe: Recipe {
            roast: Roast::Dark,
            beans: 18,
            milk: 200,
        },
    },
    Drink {
        key: "americano",
        icon: "♨️",
        name: "Americano",
        detail: "Espresso lengthened with water",
        recipe: Recipe {
            roast: Roast::Light,
            beans: 18,
            milk: 0,
        },
    },
];
