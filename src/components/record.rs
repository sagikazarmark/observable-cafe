use dioxus::prelude::*;

use crate::components::{MetricCards, Notebook};
use crate::lesson::Lesson;
use crate::state::Snapshot;

/// Which view of the record is showing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Notebook,
    Metrics,
}

/// What the café has written down.
///
/// The notebook is always here. The lesson about metric types adds a second
/// way of reading the same entries, and only then does a tab bar appear —
/// there is nothing to switch between before that.
#[component]
pub fn Record(lesson: Lesson, snapshot: Snapshot) -> Element {
    let view = use_signal(|| View::Notebook);
    let showing = if lesson.typed() {
        view()
    } else {
        View::Notebook
    };

    rsx! {
        section { class: "record", aria_labelledby: "notebookTitle",
            if lesson.typed() {
                div { class: "tabs", role: "tablist",
                    Tab { view, showing, tab: View::Notebook, label: "Notebook" }
                    Tab { view, showing, tab: View::Metrics, label: "Metrics" }
                }
            }

            match showing {
                View::Notebook => rsx! {
                    Notebook {
                        observations: snapshot.observations.clone(),
                        labelled: lesson.labelled(),
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
