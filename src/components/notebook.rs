use dioxus::prelude::*;

use crate::components::{MetricCards, Observations, Receipts};
use crate::feature::Features;
use crate::state::Snapshot;

/// One way of reading the notebook.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    /// The entries the owner writes every so often.
    Observations,
    /// Every sale, as it happened.
    Receipts,
    /// The same entries sorted by metric rather than by moment.
    Metrics,
}

impl View {
    /// What the tab for this view says.
    fn label(self) -> &'static str {
        match self {
            Self::Observations => "Observations",
            Self::Receipts => "Receipts",
            Self::Metrics => "Metrics",
        }
    }

    /// The views this café keeps, in the order they are offered.
    fn shown(features: Features) -> Vec<Self> {
        [
            (features.observations, Self::Observations),
            (features.receipts, Self::Receipts),
            (features.types, Self::Metrics),
        ]
        .into_iter()
        .filter_map(|(shown, view)| shown.then_some(view))
        .collect()
    }
}

/// The notebook: the book on the counter, and everything kept in it.
///
/// The tab bar appears only where there is a choice to make. One view is shown
/// as itself, and no view at all does not reach here: a café with nothing to
/// keep leaves the notebook out rather than drawing an empty one.
#[component]
pub fn Notebook(snapshot: Snapshot) -> Element {
    let features: Features = use_context();
    let views = View::shown(features);
    let chosen = use_signal(|| views.first().copied());

    // The first view is the fallback rather than a fixed one: whichever views
    // this café keeps, it opens on the first of them.
    let showing = chosen()
        .filter(|view| views.contains(view))
        .or_else(|| views.first().copied());

    rsx! {
        section { class: "notebook-panel", aria_label: "The café’s notebook",
            if views.len() > 1 {
                div { class: "tabs", role: "tablist",
                    for view in views.iter().copied() {
                        Tab {
                            key: "{view.label()}",
                            chosen,
                            showing,
                            tab: view,
                            label: view.label(),
                        }
                    }
                }
            }

            match showing {
                Some(View::Observations) => rsx! {
                    Observations {
                        observations: snapshot.observations.clone(),
                        labelled: features.labels,
                        today: snapshot.day.clone(),
                    }
                },
                Some(View::Receipts) => rsx! {
                    Receipts {
                        receipts: snapshot.receipts.clone(),
                        labelled: features.labels,
                        today: snapshot.day.clone(),
                    }
                },
                Some(View::Metrics) => rsx! {
                    MetricCards { snapshot: snapshot.clone() }
                },
                None => rsx! {},
            }
        }
    }
}

#[component]
fn Tab(chosen: Signal<Option<View>>, showing: Option<View>, tab: View, label: String) -> Element {
    let selected = showing == Some(tab);
    let class = if selected { "tab on" } else { "tab" };

    rsx! {
        button {
            class: "{class}",
            r#type: "button",
            role: "tab",
            aria_selected: selected,
            onclick: move |_| chosen.set(Some(tab)),
            "{label}"
        }
    }
}
