use dioxus::document::eval;
use dioxus::prelude::*;

use crate::state::Sale;

/// Keeps the roll scrolled to the newest receipt, on the same terms as the
/// notebook: it follows along until the reader scrolls away to look at
/// something older, and the scroll it performs itself must not be mistaken for
/// the reader scrolling.
const FOLLOW_NEWEST: &str = r#"
const panel = document.getElementById("receipt-roll");
if (panel && !panel.dataset.following) {
    panel.dataset.following = "1";

    const atBottom = () => panel.scrollHeight - panel.scrollTop - panel.clientHeight < 24;
    let following = true;
    let ours = false;

    const toBottom = () => {
        ours = true;
        panel.scrollTop = panel.scrollHeight;
        requestAnimationFrame(() => { ours = false; });
    };

    panel.addEventListener("scroll", () => { if (!ours) following = atBottom(); });
    new MutationObserver(() => { if (following) toBottom(); })
        .observe(panel, { childList: true, subtree: true });

    toBottom();
    document.fonts.ready.then(() => { if (following) toBottom(); });
}
"#;

/// Every sale, as it happened.
///
/// The other half of the sampling lesson, and the reason it is worth its own
/// tab rather than a line in the notebook: this is what the notebook is a
/// sample *of*. Three sales here and one entry there is the whole idea, and it
/// only lands if both can be counted.
#[component]
pub fn Receipts(receipts: Vec<Sale>, labelled: bool, today: String) -> Element {
    // Spawned rather than left to drop: the handle owns the running script.
    use_effect(move || {
        spawn(async move {
            let _ = eval(FOLLOW_NEWEST).await;
        });
    });

    rsx! {
        div { class: "receipts",
            div { class: "view-header",
                h2 { "Sales" }
                div { class: "date-stamp", "{today}" }
            }

            div { id: "receipt-roll", class: "roll",
                if receipts.is_empty() {
                    div { class: "empty-state",
                        strong { "Nothing sold yet" }
                        "Every sale is written up here the moment it happens."
                    }
                } else {
                    for sale in receipts.iter() {
                        div { key: "{sale.seq}", class: "receipt",
                            span { class: "receipt-time", "{sale.at}" }
                            // Without labels a sale is a sale: the café knows
                            // which drink it was, but this record does not keep
                            // the dimension, which is exactly what a café
                            // without labels looks like.
                            span { class: "receipt-drink",
                                if labelled {
                                    {sale.drink().map_or("Coffee", |drink| drink.name)}
                                } else {
                                    "Coffee"
                                }
                            }
                            span { class: "receipt-seq", "#{sale.seq}" }
                        }
                    }
                }
            }
        }
    }
}
