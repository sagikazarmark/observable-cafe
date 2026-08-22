use dioxus::prelude::*;

/// The sign the café opens with: its mark, and then its name.
///
/// It heads the bar, to the left of the clock, so that the page says where you
/// are before it says what time it is there.
///
/// The mark is drawn here rather than fetched, which costs no request and
/// leaves it taking its colours from the same palette as the rest of the café.
/// Its steam is a plotted line, since what this café gives off is a series.
#[component]
pub fn Header() -> Element {
    rsx! {
        div { class: "masthead",
            svg {
                class: "masthead-mark",
                view_box: "0 0 32 32",
                fill: "none",
                // Decoration: the name is spelled out immediately beside it,
                // and a reader told "coffee cup, The Observable Café" has been
                // told the same thing twice.
                "aria-hidden": "true",

                // The steam, drawn as a reading rather than as a curl. Narrower
                // than the cup and well clear of the rim, so that it rises off
                // it rather than resting on it.
                path {
                    d: "M9.5 10 L13 5.5 L16.5 8.5 L20.5 3.5",
                    stroke: "var(--accent)",
                    stroke_width: "1.7",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }

                // The cup.
                path {
                    d: "M7 13.5h15v4.5a7.5 7.5 0 0 1-15 0z",
                    fill: "var(--paper)",
                    stroke: "var(--coffee)",
                    stroke_width: "1.8",
                    stroke_linejoin: "round",
                }

                // Its handle.
                path {
                    d: "M22 15h2.2a3 3 0 0 1 0 6H22",
                    stroke: "var(--coffee)",
                    stroke_width: "1.8",
                    stroke_linecap: "round",
                }

                // The saucer under it.
                path {
                    d: "M5 27.5h22",
                    stroke: "var(--coffee)",
                    stroke_width: "1.8",
                    stroke_linecap: "round",
                }
            }

            span { class: "masthead-name", "The Observable Café" }
        }
    }
}
