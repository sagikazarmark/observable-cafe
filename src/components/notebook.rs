use dioxus::document::eval;
use dioxus::prelude::*;

use crate::state::Observation;

/// Keeps the notebook scrolled to the newest entry, unless the reader has
/// scrolled away from the bottom to look at something older.
///
/// Yanking the page back every five seconds would make the history unreadable,
/// and reading the history is what the notebook is for.
///
/// The scroll this performs itself must not be mistaken for the reader
/// scrolling away, so the listener is muted while it runs. That mute is what
/// stopped this following at all once before, when the panel still scrolled
/// smoothly and the half-finished animation read as somebody scrolling up. It
/// therefore stays as the guard against anyone reintroducing smooth scrolling.
const FOLLOW_NEWEST: &str = r#"
const panel = document.getElementById("notebook-entries");
if (panel && !panel.dataset.following) {
    panel.dataset.following = "1";

    const atBottom = () => panel.scrollHeight - panel.scrollTop - panel.clientHeight < 24;
    let following = true;
    let ours = false;

    const toBottom = () => {
        ours = true;
        panel.scrollTop = panel.scrollHeight;
        // Scroll events are delivered before the next frame, so by the time
        // this runs the mute has done its job.
        requestAnimationFrame(() => { ours = false; });
    };

    panel.addEventListener("scroll", () => { if (!ours) following = atBottom(); });
    new MutationObserver(() => { if (following) toBottom(); })
        .observe(panel, { childList: true, subtree: true });

    toBottom();

    // An entry is not its final height until the handwriting has been applied
    // to it, which would otherwise leave the first scroll short.
    document.fonts.ready.then(() => { if (following) toBottom(); });
}
"#;

/// The notebook the owner writes the café down in.
///
/// Every entry is one moment, written out in full. Nothing here fades or gets
/// struck out as it ages: an old reading is not a wrong one, it is the record.
#[component]
pub fn Notebook(observations: Vec<Observation>, labelled: bool, today: String) -> Element {
    // The café runs past midnight after sixteen minutes, so the heading
    // follows the newest entry rather than the day the café opened.
    let date_stamp = observations
        .last()
        .map_or(today, |newest| newest.day.clone());

    // Spawned rather than left to drop: the handle owns the running script.
    use_effect(move || {
        spawn(async move {
            let _ = eval(FOLLOW_NEWEST).await;
        });
    });

    let lines = Line::from(&observations);
    let newest = observations.last().map(|entry| entry.seq);

    rsx! {
        div { class: "notebook",
            div { class: "notebook-header",
                h2 { id: "notebookTitle", "Café observations" }
                div { class: "date-stamp", "{date_stamp}" }
            }

            div { id: "notebook-entries", class: "entries",
                if observations.is_empty() {
                    div { class: "empty-state",
                        strong { "Nothing written down yet" }
                        "The owner looks up every few minutes."
                    }
                } else {
                    for line in lines.iter() {
                        match line {
                            Line::Day { day, before } => rsx! {
                                div { key: "day-{before}", class: "day-divider",
                                    span { "{day}" }
                                }
                            },
                            Line::Entry(observation) => rsx! {
                                Entry {
                                    key: "{observation.seq}",
                                    observation: observation.clone(),
                                    labelled,
                                    fresh: Some(observation.seq) == newest,
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// One line of the notebook, which is usually an observation but is sometimes
/// the owner heading a new day.
enum Line {
    /// A rule headed with the day, taking its identity from the entry it
    /// introduces so that it travels down the page alongside it.
    Day {
        day: String,
        before: u64,
    },
    Entry(Observation),
}

impl Line {
    /// Lays observations out as they are written, ruling off wherever the café
    /// day turns over; otherwise two entries reading `09:00` would look like
    /// the same moment.
    fn from(observations: &[Observation]) -> Vec<Self> {
        let mut lines = Vec::with_capacity(observations.len());
        let mut day: Option<&str> = None;

        for observation in observations {
            if day.is_some_and(|previous| previous != observation.day) {
                lines.push(Self::Day {
                    day: observation.day.clone(),
                    before: observation.seq,
                });
            }

            day = Some(&observation.day);
            lines.push(Self::Entry(observation.clone()));
        }

        lines
    }
}

#[component]
fn Entry(observation: Observation, labelled: bool, fresh: bool) -> Element {
    let class = if fresh { "entry fresh" } else { "entry" };

    rsx! {
        article { class: "{class}",
            span { class: "entry-time", "{observation.at}" }

            div { class: "entry-body",
                div { class: "entry-line",
                    "Coffees sold: "
                    b { "{observation.sold.total()}" }
                }

                // Only the stage about labels breaks the count down, and the
                // total stays written above it: somebody keeping notes wants
                // the headline, even though the machine-readable version
                // publishes the parts alone.
                if labelled {
                    div { class: "entry-breakdown",
                        for (drink , count) in observation.sold.by_drink() {
                            div { key: "{drink.key}", class: "entry-line",
                                "{drink.name}: "
                                b { "{count}" }
                            }
                        }
                    }
                }

                div { class: "entry-line",
                    "Inside: "
                    b { "{observation.inside}°C" }
                }
                div { class: "entry-line",
                    "Outside: "
                    b { "{observation.outside}°C" }
                }
            }
        }
    }
}
