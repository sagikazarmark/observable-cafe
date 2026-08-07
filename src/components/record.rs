use dioxus::prelude::*;

use crate::components::{MetricCards, Notebook};
use crate::stage::Stage;
use crate::state::Snapshot;

/// Which view of the record is showing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Notebook,
    Metrics,
}

/// What the café has written down.
///
/// The notebook is always here. The stage about metric types adds a second
/// way of reading the same entries, and only then does a tab bar appear —
/// there is nothing to switch between before that.
#[component]
pub fn Record(stage: Stage, snapshot: Snapshot) -> Element {
    let view = use_signal(|| View::Notebook);
    let showing = if stage.typed() {
        view()
    } else {
        View::Notebook
    };

    rsx! {
        section { class: "record", aria_labelledby: "notebookTitle",
            if stage.typed() {
                div { class: "tabs", role: "tablist",
                    Tab { view, showing, tab: View::Notebook, label: "Notebook" }
                    Tab { view, showing, tab: View::Metrics, label: "Metrics" }
                }
            }

            match showing {
                View::Notebook => rsx! {
                    Notebook {
                        observations: snapshot.observations.clone(),
                        labelled: stage.labelled(),
                        today: snapshot.day.clone(),
                    }
                },
                View::Metrics => rsx! {
                    MetricCards { snapshot: snapshot.clone() }
                },
            }
        }
    }
}

#[component]
fn Tab(view: Signal<View>, showing: View, tab: View, label: String) -> Element {
    let selected = showing == tab;
    let class = if selected { "tab on" } else { "tab" };

    rsx! {
        button {
            class: "{class}",
            r#type: "button",
            role: "tab",
            aria_selected: selected,
            onclick: move |_| view.set(tab),
            "{label}"
        }
    }
}
