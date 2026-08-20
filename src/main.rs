mod api;
mod app;
mod clock;
mod components;
mod menu;
mod route;
mod season;
mod stage;
mod state;

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
        let options =
            Options::parse(std::env::args_os().skip(1), std::env::var_os).unwrap_or_else(|error| {
                eprintln!("error: {error}");
                std::process::exit(2);
            });

        server::launch(
            options.automatic_observations,
            options.stage,
            options.navigation,
        );
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}

#[cfg(feature = "server")]
const STAGE_ENV: &str = "OBSERVABLE_CAFE_STAGE";

#[cfg(feature = "server")]
const DISABLE_AUTOMATIC_OBSERVATIONS_ENV: &str = "OBSERVABLE_CAFE_DISABLE_AUTOMATIC_OBSERVATIONS";

#[cfg(feature = "server")]
const ENABLE_NAVIGATION_ENV: &str = "OBSERVABLE_CAFE_ENABLE_NAVIGATION";

#[cfg(feature = "server")]
#[derive(Debug, PartialEq)]
struct Options {
    automatic_observations: bool,
    stage: Option<stage::Stage>,
    /// Whether a stage offers a way back to the index.
    ///
    /// Off by default, because a stage is usually embedded in a course page
    /// that does its own navigating, and a widget pointing away from the page
    /// around it is that page's business rather than the widget's.
    navigation: bool,
}

#[cfg(feature = "server")]
impl Options {
    fn parse(
        args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
        mut env: impl FnMut(&'static str) -> Option<std::ffi::OsString>,
    ) -> Result<Self, String> {
        let mut args = args
            .into_iter()
            .map(|arg| arg.as_ref().to_string_lossy().into_owned());
        let mut options = Self {
            automatic_observations: true,
            stage: None,
            navigation: false,
        };
        let mut automatic_observations_set_by_cli = false;
        let mut stage_set_by_cli = false;
        let mut navigation_set_by_cli = false;

        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--disable-automatic-observations" => {
                    options.automatic_observations = false;
                    automatic_observations_set_by_cli = true;
                }
                "--stage" => {
                    let stage = args
                        .next()
                        .ok_or_else(|| "--stage requires samples, labels, or types".to_owned())?;
                    options.stage = Some(parse_stage(&stage)?);
                    stage_set_by_cli = true;
                }
                "--enable-navigation" => {
                    options.navigation = true;
                    navigation_set_by_cli = true;
                }
                _ => {}
            }
        }

        if !stage_set_by_cli && let Some(stage) = env(STAGE_ENV) {
            options.stage = Some(parse_stage(&stage.to_string_lossy())?);
        }

        if !automatic_observations_set_by_cli
            && let Some(disabled) = env(DISABLE_AUTOMATIC_OBSERVATIONS_ENV)
        {
            options.automatic_observations = !parse_env_bool(
                DISABLE_AUTOMATIC_OBSERVATIONS_ENV,
                &disabled.to_string_lossy(),
            )?;
        }

        if !navigation_set_by_cli && let Some(enabled) = env(ENABLE_NAVIGATION_ENV) {
            options.navigation = parse_env_bool(ENABLE_NAVIGATION_ENV, &enabled.to_string_lossy())?;
        }

        // Refused rather than quietly ignored: a single stage is served
        // without the index, so the way back would lead nowhere.
        if options.navigation && options.stage.is_some() {
            return Err(
                "navigation cannot be enabled together with a single stage, which is served without the index"
                    .to_owned(),
            );
        }

        Ok(options)
    }
}

#[cfg(feature = "server")]
fn parse_stage(value: &str) -> Result<stage::Stage, String> {
    stage::Stage::named(value)
        .ok_or_else(|| format!("unknown stage {value:?}; expected samples, labels, or types"))
}

#[cfg(feature = "server")]
fn parse_env_bool(name: &str, value: &str) -> Result<bool, String> {
    value
        .parse()
        .map_err(|_| format!("{name} must be true or false, got {value:?}"))
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::{DISABLE_AUTOMATIC_OBSERVATIONS_ENV, ENABLE_NAVIGATION_ENV, Options, STAGE_ENV};
    use crate::stage::Stage;

    #[test]
    fn options_use_the_multi_stage_version_by_default() {
        assert_eq!(
            Options::parse(Vec::<&str>::new(), |_| None),
            Ok(Options {
                automatic_observations: true,
                stage: None,
                navigation: false,
            })
        );
    }

    #[test]
    fn options_can_disable_observations_and_select_a_stage() {
        assert_eq!(
            Options::parse(
                ["--disable-automatic-observations", "--stage", "labels"],
                |_| None,
            ),
            Ok(Options {
                automatic_observations: false,
                stage: Some(Stage::Labels),
                navigation: false,
            })
        );
    }

    #[test]
    fn options_can_turn_navigation_on() {
        assert_eq!(
            Options::parse(["--enable-navigation"], |_| None),
            Ok(Options {
                automatic_observations: true,
                stage: None,
                navigation: true,
            })
        );

        let env = |name| (name == ENABLE_NAVIGATION_ENV).then(|| "true".into());

        assert_eq!(
            Options::parse(Vec::<&str>::new(), env),
            Ok(Options {
                automatic_observations: true,
                stage: None,
                navigation: true,
            })
        );
    }

    #[test]
    fn navigation_cannot_be_combined_with_a_single_stage() {
        assert_eq!(
            Options::parse(["--enable-navigation", "--stage", "labels"], |_| None),
            Err(
                "navigation cannot be enabled together with a single stage, which is served without the index"
                    .to_owned()
            )
        );
    }

    #[test]
    fn options_reject_an_unknown_stage() {
        assert_eq!(
            Options::parse(["--stage", "unknown"], |_| None),
            Err("unknown stage \"unknown\"; expected samples, labels, or types".to_owned())
        );
    }

    #[test]
    fn options_ignore_arguments_the_application_does_not_own() {
        assert_eq!(
            Options::parse(["--host-wrapper-option"], |_| None),
            Options::parse(Vec::<&str>::new(), |_| None)
        );
    }

    #[test]
    fn options_can_be_set_with_environment_variables() {
        let env = |name| match name {
            STAGE_ENV => Some("types".into()),
            DISABLE_AUTOMATIC_OBSERVATIONS_ENV => Some("true".into()),
            _ => None,
        };

        assert_eq!(
            Options::parse(Vec::<&str>::new(), env),
            Ok(Options {
                automatic_observations: false,
                stage: Some(Stage::Types),
                navigation: false,
            })
        );
    }

    #[test]
    fn false_does_not_disable_automatic_observations() {
        let env = |name| (name == DISABLE_AUTOMATIC_OBSERVATIONS_ENV).then(|| "false".into());

        assert_eq!(
            Options::parse(Vec::<&str>::new(), env),
            Ok(Options {
                automatic_observations: true,
                stage: None,
                navigation: false,
            })
        );
    }

    #[test]
    fn options_reject_an_invalid_environment_boolean() {
        let env = |name| (name == DISABLE_AUTOMATIC_OBSERVATIONS_ENV).then(|| "sometimes".into());

        assert_eq!(
            Options::parse(Vec::<&str>::new(), env),
            Err(format!(
                "{DISABLE_AUTOMATIC_OBSERVATIONS_ENV} must be true or false, got \"sometimes\""
            ))
        );
    }

    #[test]
    fn command_line_options_take_precedence_over_environment_variables() {
        let env = |name| match name {
            STAGE_ENV => Some("types".into()),
            DISABLE_AUTOMATIC_OBSERVATIONS_ENV => Some("not-a-boolean".into()),
            _ => None,
        };

        assert_eq!(
            Options::parse(
                ["--stage", "samples", "--disable-automatic-observations"],
                env,
            ),
            Ok(Options {
                automatic_observations: false,
                stage: Some(Stage::Samples),
                navigation: false,
            })
        );
    }
}
