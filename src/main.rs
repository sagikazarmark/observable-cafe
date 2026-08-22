mod api;
mod app;
mod clock;
mod components;
mod feature;
mod menu;
mod season;
mod state;

#[cfg(feature = "server")]
mod config;
#[cfg(feature = "server")]
mod server;

/// Which build of the café this is, as `cargo` was told it.
///
/// Reported at `/version` and written into the head of every page, so that a
/// café running somewhere can be matched to a tag without guessing, whether
/// what is to hand is `curl` or a browser tab that is already open.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    #[cfg(feature = "server")]
    {
        let options = config::Options::parse().unwrap_or_else(|error| {
            eprintln!("error: {error}");
            std::process::exit(2);
        });

        server::launch(options.features);
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}
