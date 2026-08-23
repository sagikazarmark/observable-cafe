//! What the café brews from.
//!
//! The shelf is the part of the café a customer never sees and a sale always
//! touches: making a drink takes its recipe off it, and a drink the shelf
//! cannot cover is refused rather than made. Only a café showing the
//! `inventory` feature keeps one; every other café brews from nothing, as
//! every café here did before it had a shelf.

use serde::{Deserialize, Serialize};

/// How a batch of beans was roasted.
///
/// The two roasts are two stocks, and on `/metrics` two series of one gauge:
/// the first label this café puts on anything other than a counter.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Roast {
    /// Bright and delicate: what the café brews for drinks drunk black,
    /// where there is no milk to hide behind.
    Light,
    /// Heavy and roasty: what still tastes of coffee through steamed milk.
    Dark,
}

impl Roast {
    /// Both roasts, in the order the shelf keeps them.
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    /// How this roast identifies itself in a label value.
    pub fn key(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// How the roast is spoken of on the page.
    pub fn name(self) -> &'static str {
        match self {
            Self::Light => "light roast",
            Self::Dark => "dark roast",
        }
    }
}

/// One thing the café can run out of.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Ingredient {
    Beans(Roast),
    Milk,
}

impl Ingredient {
    /// Everything on the shelf, in the order it is kept there.
    pub const ALL: [Self; 3] = [
        Self::Beans(Roast::Light),
        Self::Beans(Roast::Dark),
        Self::Milk,
    ];

    /// How the ingredient is spoken of on the page, and in a refusal.
    pub fn name(self) -> &'static str {
        match self {
            Self::Beans(roast) => match roast {
                Roast::Light => "light roast beans",
                Roast::Dark => "dark roast beans",
            },
            Self::Milk => "milk",
        }
    }
}

/// What goes into one drink.
///
/// Kept on the menu rather than behind the till, because the recipe is part
/// of what the drink is: the button a customer presses and the deduction it
/// causes must be reading the same numbers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Recipe {
    /// Which beans the drink is brewed from.
    pub roast: Roast,
    /// Grams of beans ground for one drink.
    pub beans: u32,
    /// Millilitres of milk steamed for one drink. An espresso steams none,
    /// which is exactly why it stays sellable after the milk runs out.
    pub milk: u32,
}

/// What is on the shelf right now.
///
/// The shelf holds at most what it was built to hold: a delivery that does
/// not fit is turned partly away rather than stacked on the floor, which is
/// also what gives the gauge a known ceiling to be read against.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Inventory {
    /// Grams of each roast, indexed as [`Roast::ALL`] orders them.
    beans: [u32; Roast::ALL.len()],
    /// Millilitres of milk.
    milk: u32,
}

/// Grams of one roast the shelf holds when full.
const BEANS_CAPACITY: u32 = 500;

/// Millilitres of milk the fridge holds when full.
const MILK_CAPACITY: u32 = 3000;

/// Grams of beans in one delivery: a whole bag, which is also a whole shelf.
const BEAN_BAG: u32 = 500;

/// Millilitres of milk in one delivery: one bottle, a third of the fridge.
const MILK_BOTTLE: u32 = 1000;

impl Inventory {
    /// A shelf as the café opens with it: full.
    ///
    /// Only the café stocks a shelf from nothing; the browser is handed one
    /// that already exists, so this is the server's alone.
    #[cfg(feature = "server")]
    pub fn full() -> Self {
        Self {
            beans: [BEANS_CAPACITY; Roast::ALL.len()],
            milk: MILK_CAPACITY,
        }
    }

    /// How much of `ingredient` is on the shelf, in its own unit.
    pub fn amount(&self, ingredient: Ingredient) -> u32 {
        match ingredient {
            Ingredient::Beans(roast) => self.beans[Self::index(roast)],
            Ingredient::Milk => self.milk,
        }
    }

    /// How much of `ingredient` the shelf holds when full.
    pub fn capacity(ingredient: Ingredient) -> u32 {
        match ingredient {
            Ingredient::Beans(_) => BEANS_CAPACITY,
            Ingredient::Milk => MILK_CAPACITY,
        }
    }

    /// How much of `ingredient` one delivery brings.
    pub fn delivery(ingredient: Ingredient) -> u32 {
        match ingredient {
            Ingredient::Beans(_) => BEAN_BAG,
            Ingredient::Milk => MILK_BOTTLE,
        }
    }

    /// How full the shelf is of `ingredient`, as a fraction between 0 and 1.
    pub fn fullness(&self, ingredient: Ingredient) -> f64 {
        f64::from(self.amount(ingredient)) / f64::from(Self::capacity(ingredient))
    }

    /// The first ingredient that stands between the shelf and this recipe,
    /// if anything does.
    ///
    /// All or nothing: ten grams of beans against an eighteen gram recipe is
    /// no espresso rather than a weak one.
    pub fn missing_for(&self, recipe: Recipe) -> Option<Ingredient> {
        if self.amount(Ingredient::Beans(recipe.roast)) < recipe.beans {
            return Some(Ingredient::Beans(recipe.roast));
        }

        if self.amount(Ingredient::Milk) < recipe.milk {
            return Some(Ingredient::Milk);
        }

        None
    }

    /// Makes one drink, taking its recipe off the shelf, or refuses it and
    /// takes nothing: a refusal names what was short and leaves the shelf
    /// exactly as it was.
    #[cfg(feature = "server")]
    pub fn brew(&mut self, recipe: Recipe) -> Result<(), Ingredient> {
        if let Some(short) = self.missing_for(recipe) {
            return Err(short);
        }

        self.beans[Self::index(recipe.roast)] -= recipe.beans;
        self.milk -= recipe.milk;

        Ok(())
    }

    /// Takes in one delivery of `ingredient`, and says how much of it fit:
    /// the shelf takes what it has room for and turns the rest away, so a
    /// full shelf takes nothing.
    #[cfg(feature = "server")]
    pub fn take_delivery(&mut self, ingredient: Ingredient) -> u32 {
        let room = Self::capacity(ingredient) - self.amount(ingredient);
        let taken = Self::delivery(ingredient).min(room);

        match ingredient {
            Ingredient::Beans(roast) => self.beans[Self::index(roast)] += taken,
            Ingredient::Milk => self.milk += taken,
        }

        taken
    }

    /// Where `roast` sits in the shelf's own table.
    fn index(roast: Roast) -> usize {
        Roast::ALL
            .iter()
            .position(|&kept| kept == roast)
            .expect("every roast is in ALL")
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::{Ingredient, Inventory, Recipe, Roast};

    /// A drink with milk in it, so both sides of the shelf are touched.
    const CAPPUCCINO: Recipe = Recipe {
        roast: Roast::Dark,
        beans: 18,
        milk: 100,
    };

    #[test]
    fn brewing_takes_the_recipe_off_the_shelf() {
        let mut shelf = Inventory::full();

        assert_eq!(shelf.brew(CAPPUCCINO), Ok(()));

        assert_eq!(
            shelf.amount(Ingredient::Beans(Roast::Dark)),
            Inventory::capacity(Ingredient::Beans(Roast::Dark)) - 18
        );
        assert_eq!(
            shelf.amount(Ingredient::Milk),
            Inventory::capacity(Ingredient::Milk) - 100
        );
        // The other roast was never touched: each drink draws from one.
        assert_eq!(
            shelf.amount(Ingredient::Beans(Roast::Light)),
            Inventory::capacity(Ingredient::Beans(Roast::Light))
        );
    }

    /// All or nothing: a refusal names what was short and takes nothing,
    /// not even the ingredients the shelf did have.
    #[test]
    fn a_drink_the_shelf_cannot_cover_is_refused_whole() {
        let mut shelf = Inventory::full();
        // Thirsty enough to empty the fridge while beans remain, so the
        // refusal is short of one ingredient and holding the other.
        let thirsty = Recipe {
            milk: Inventory::capacity(Ingredient::Milk) / 3,
            ..CAPPUCCINO
        };

        for _ in 0..3 {
            shelf.brew(thirsty).expect("still covered");
        }

        let beans_left = shelf.amount(Ingredient::Beans(Roast::Dark));

        assert_eq!(shelf.brew(thirsty), Err(Ingredient::Milk));
        assert_eq!(shelf.amount(Ingredient::Beans(Roast::Dark)), beans_left);
    }

    /// Ten grams against an eighteen gram recipe is no espresso rather than
    /// a weak one.
    #[test]
    fn a_partly_covered_recipe_counts_as_missing() {
        let mut shelf = Inventory::full();
        let short = Recipe {
            beans: Inventory::capacity(Ingredient::Beans(Roast::Dark)) - 10,
            ..CAPPUCCINO
        };

        shelf.brew(short).expect("the first one is covered");

        assert_eq!(
            shelf.missing_for(CAPPUCCINO),
            Some(Ingredient::Beans(Roast::Dark))
        );
    }

    #[test]
    fn a_delivery_tops_the_shelf_back_up() {
        let mut shelf = Inventory::full();
        shelf.brew(CAPPUCCINO).expect("covered");

        assert_eq!(shelf.take_delivery(Ingredient::Milk), 100);
        assert_eq!(
            shelf.amount(Ingredient::Milk),
            Inventory::capacity(Ingredient::Milk)
        );
    }

    /// The shelf takes what it has room for and turns the rest away.
    #[test]
    fn a_full_shelf_takes_nothing_in() {
        let mut shelf = Inventory::full();

        assert_eq!(shelf.take_delivery(Ingredient::Beans(Roast::Light)), 0);
        assert_eq!(
            shelf.amount(Ingredient::Beans(Roast::Light)),
            Inventory::capacity(Ingredient::Beans(Roast::Light))
        );
    }
}
