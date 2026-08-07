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
    server::launch(automatic_observations_enabled(std::env::args_os().skip(1)));

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}

#[cfg(feature = "server")]
fn automatic_observations_enabled(
    args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) -> bool {
    !args
        .into_iter()
        .any(|arg| arg.as_ref() == std::ffi::OsStr::new("--disable-automatic-observations"))
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::automatic_observations_enabled;

    #[test]
    fn automatic_observations_are_enabled_by_default() {
        assert!(automatic_observations_enabled(Vec::<&str>::new()));
    }

    #[test]
    fn automatic_observations_can_be_disabled() {
        assert!(!automatic_observations_enabled([
            "--disable-automatic-observations"
        ]));
    }
}
