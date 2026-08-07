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
    {
        let options =
            Options::parse(std::env::args_os().skip(1), std::env::var_os).unwrap_or_else(|error| {
                eprintln!("error: {error}");
                std::process::exit(2);
            });

        server::launch(options.automatic_observations, options.lesson);
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}

#[cfg(feature = "server")]
const LESSON_ENV: &str = "OBSERVABLE_CAFE_LESSON";

#[cfg(feature = "server")]
const DISABLE_AUTOMATIC_OBSERVATIONS_ENV: &str = "OBSERVABLE_CAFE_DISABLE_AUTOMATIC_OBSERVATIONS";

#[cfg(feature = "server")]
#[derive(Debug, PartialEq)]
struct Options {
    automatic_observations: bool,
    lesson: Option<lesson::Lesson>,
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
            lesson: None,
        };
        let mut automatic_observations_set_by_cli = false;
        let mut lesson_set_by_cli = false;

        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--disable-automatic-observations" => {
                    options.automatic_observations = false;
                    automatic_observations_set_by_cli = true;
                }
                "--lesson" => {
                    let lesson = args
                        .next()
                        .ok_or_else(|| "--lesson requires samples, labels, or types".to_owned())?;
                    options.lesson = Some(parse_lesson(&lesson)?);
                    lesson_set_by_cli = true;
                }
                _ => {}
            }
        }

        if !lesson_set_by_cli && let Some(lesson) = env(LESSON_ENV) {
            options.lesson = Some(parse_lesson(&lesson.to_string_lossy())?);
        }

        if !automatic_observations_set_by_cli
            && let Some(disabled) = env(DISABLE_AUTOMATIC_OBSERVATIONS_ENV)
        {
            options.automatic_observations = !parse_env_bool(
                DISABLE_AUTOMATIC_OBSERVATIONS_ENV,
                &disabled.to_string_lossy(),
            )?;
        }

        Ok(options)
    }
}

#[cfg(feature = "server")]
fn parse_lesson(value: &str) -> Result<lesson::Lesson, String> {
    lesson::Lesson::named(value)
        .ok_or_else(|| format!("unknown lesson {value:?}; expected samples, labels, or types"))
}

#[cfg(feature = "server")]
fn parse_env_bool(name: &str, value: &str) -> Result<bool, String> {
    value
        .parse()
        .map_err(|_| format!("{name} must be true or false, got {value:?}"))
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::{DISABLE_AUTOMATIC_OBSERVATIONS_ENV, LESSON_ENV, Options};
    use crate::lesson::Lesson;

    #[test]
    fn options_use_the_multi_lesson_version_by_default() {
        assert_eq!(
            Options::parse(Vec::<&str>::new(), |_| None),
            Ok(Options {
                automatic_observations: true,
                lesson: None,
            })
        );
    }

    #[test]
    fn options_can_disable_observations_and_select_a_lesson() {
        assert_eq!(
            Options::parse(
                ["--disable-automatic-observations", "--lesson", "labels"],
                |_| None,
            ),
            Ok(Options {
                automatic_observations: false,
                lesson: Some(Lesson::Labels),
            })
        );
    }

    #[test]
    fn options_reject_an_unknown_lesson() {
        assert_eq!(
            Options::parse(["--lesson", "unknown"], |_| None),
            Err("unknown lesson \"unknown\"; expected samples, labels, or types".to_owned())
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
            LESSON_ENV => Some("types".into()),
            DISABLE_AUTOMATIC_OBSERVATIONS_ENV => Some("true".into()),
            _ => None,
        };

        assert_eq!(
            Options::parse(Vec::<&str>::new(), env),
            Ok(Options {
                automatic_observations: false,
                lesson: Some(Lesson::Types),
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
                lesson: None,
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
            LESSON_ENV => Some("types".into()),
            DISABLE_AUTOMATIC_OBSERVATIONS_ENV => Some("not-a-boolean".into()),
            _ => None,
        };

        assert_eq!(
            Options::parse(
                ["--lesson", "samples", "--disable-automatic-observations"],
                env,
            ),
            Ok(Options {
                automatic_observations: false,
                lesson: Some(Lesson::Samples),
            })
        );
    }
}
