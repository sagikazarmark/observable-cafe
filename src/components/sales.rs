use dioxus::prelude::*;

use crate::components::follow::use_follow_newest;
use crate::components::ruled::Ruled;
use crate::state::Sale;

/// Every sale, as it happened.
///
/// The other half of the sampling lesson, and the reason it is worth its own
/// tab rather than a line in the notebook: this is what the observations are a
/// sample *of*. Three sales here and one entry there is the whole idea, and it
/// only lands if both can be counted.
///
/// It is printed on a till roll rather than written in the notebook's hand, so
/// that the record of what happened and the record somebody kept are told
/// apart at a glance. What is on the roll is sales, one line each; nobody is
/// handed a receipt here.
#[component]
pub fn Sales(sales: Vec<Sale>, labelled: bool, today: String) -> Element {
    // The roll is headed by the day of its newest sale, as the entries are, so
    // that the two records agree about what day it is rather than one showing
    // the café day and the other the last thing that happened on it.
    let date_stamp = sales.last().map_or(today, |newest| newest.day.clone());

    use_follow_newest("sales-roll");

    let lines = Ruled::from(&sales);

    rsx! {
        div { class: "sales",
            div { class: "view-header",
                h2 { "Sales" }
                div { class: "date-stamp", "{date_stamp}" }
            }

            div { id: "sales-roll", class: "roll",
                if sales.is_empty() {
                    div { class: "empty-state",
                        strong { "Nothing sold yet" }
                        "Every sale is written up here the moment it happens."
                    }
                } else {
                    for line in lines.iter() {
                        match line {
                            Ruled::Day { day, before } => rsx! {
                                div { key: "day-{before}", class: "day-divider",
                                    span { "{day}" }
                                }
                            },
                            Ruled::Line(sale) => rsx! {
                                div { key: "{sale.seq}", class: "sale",
                                    span { class: "sale-time", "{sale.at}" }
                                    // Without labels a sale is a sale: the café
                                    // knows which drink it was, and this record
                                    // does not keep the dimension, which is
                                    // what a café without labels looks like.
                                    span { class: "sale-drink",
                                        if labelled {
                                            {sale.drink().map_or("Coffee", |drink| drink.name)}
                                        } else {
                                            "Coffee"
                                        }
                                    }
                                    span { class: "sale-seq", "#{sale.seq}" }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
