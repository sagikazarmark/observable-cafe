mod api;
mod app;
mod clock;
mod components;
mod lesson;
mod menu;
mod route;
mod season;
mod state;

#[cfg(feature = "server")]
mod server;

fn main() {
    #[cfg(feature = "server")]
    server::launch();

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}
