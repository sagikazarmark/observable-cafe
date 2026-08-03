mod app;
mod components;
mod rng;
mod state;

fn main() {
    dioxus::launch(app::App);
}
