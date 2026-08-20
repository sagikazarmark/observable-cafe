//! Which build of the café is answering.
//!
//! One line of plain text, and nothing about the café's state: this says what
//! is deployed, not how it is getting on. Kept separate from `/metrics` for
//! that reason, and because a deployment is usually checked before anything
//! is scraped.

use dioxus::server::axum::http::header::CONTENT_TYPE;
use dioxus::server::axum::response::IntoResponse;

const PLAIN_TEXT: &str = "text/plain; charset=utf-8";

/// The version, with the newline a terminal expects. Shell substitution strips
/// it again, so `curl` reads well either way.
pub async fn report() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, PLAIN_TEXT)],
        format!("{}\n", crate::VERSION),
    )
}
