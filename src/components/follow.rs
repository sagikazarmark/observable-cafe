use dioxus::document::eval;
use dioxus::prelude::*;

/// Where the panel id is written into the script below.
const PANEL: &str = "PANEL_ID";

/// Keeps a record scrolled to its newest line, unless the reader has scrolled
/// away from the bottom to look at something older.
///
/// Yanking the page back every five seconds would make the history unreadable,
/// and reading the history is what a record is for.
///
/// The scroll this performs itself must not be mistaken for the reader
/// scrolling away, so the listener is muted while it runs. That mute is what
/// stopped this following at all once before, when the panel still scrolled
/// smoothly and the half-finished animation read as somebody scrolling up. It
/// therefore stays as the guard against anyone reintroducing smooth scrolling.
const FOLLOW_NEWEST: &str = r#"
const panel = document.getElementById("PANEL_ID");
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

    // A line is not its final height until the handwriting has been applied to
    // it, which would otherwise leave the first scroll short.
    document.fonts.ready.then(() => { if (following) toBottom(); });
}
"#;

/// Has the element with this id follow its newest line.
///
/// One script rather than one per record: the mute guard above is subtle
/// enough that a second copy would eventually be edited on its own and quietly
/// lose it.
pub fn use_follow_newest(panel: &'static str) {
    // Spawned rather than left to drop: the handle owns the running script.
    use_effect(move || {
        let script = FOLLOW_NEWEST.replace(PANEL, panel);

        spawn(async move {
            let _ = eval(&script).await;
        });
    });
}
