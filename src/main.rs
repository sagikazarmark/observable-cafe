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
        let options = Options::parse(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("error: {error}");
            std::process::exit(2);
        });

        server::launch(options.automatic_observations, options.lesson);
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}

#[cfg(feature = "server")]
#[derive(Debug, PartialEq)]
struct Options {
    automatic_observations: bool,
    lesson: Option<lesson::Lesson>,
}

#[cfg(feature = "server")]
impl Options {
    fn parse(args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Result<Self, String> {
        let mut args = args
            .into_iter()
            .map(|arg| arg.as_ref().to_string_lossy().into_owned());
        let mut options = Self {
            automatic_observations: true,
            lesson: None,
        };

        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--disable-automatic-observations" => options.automatic_observations = false,
                "--lesson" => {
                    let lesson = args
                        .next()
                        .ok_or_else(|| "--lesson requires samples, labels, or types".to_owned())?;
                    options.lesson = Some(parse_lesson(&lesson)?);
                }
                _ => {}
            }
        }

        Ok(options)
    }
}

#[cfg(feature = "server")]
fn parse_lesson(value: &str) -> Result<lesson::Lesson, String> {
    lesson::Lesson::named(value)
        .ok_or_else(|| format!("unknown lesson {value:?}; expected samples, labels, or types"))
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::Options;
    use crate::lesson::Lesson;

    #[test]
    fn options_use_the_multi_lesson_version_by_default() {
        assert_eq!(
            Options::parse(Vec::<&str>::new()),
            Ok(Options {
                automatic_observations: true,
                lesson: None,
            })
        );
    }

    #[test]
    fn options_can_disable_observations_and_select_a_lesson() {
        assert_eq!(
            Options::parse(["--disable-automatic-observations", "--lesson", "labels"]),
            Ok(Options {
                automatic_observations: false,
                lesson: Some(Lesson::Labels),
            })
        );
    }

    #[test]
    fn options_reject_an_unknown_lesson() {
        assert_eq!(
            Options::parse(["--lesson", "unknown"]),
            Err("unknown lesson \"unknown\"; expected samples, labels, or types".to_owned())
        );
    }

    #[test]
    fn options_ignore_arguments_the_application_does_not_own() {
        assert_eq!(
            Options::parse(["--host-wrapper-option"]),
            Options::parse(Vec::<&str>::new())
        );
    }
}
