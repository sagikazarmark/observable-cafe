//! Where the café's settings come from.
//!
//! The same three settings can be given in a file, in the environment, or on
//! the command line, because a café run by hand and a café run by a container
//! platform are told things in different ways. All three spell them the same,
//! and each in that order overrules the one before it when they disagree.

use std::path::{Path, PathBuf};

use clap::Parser;
use figment::{
    Figment,
    providers::{Data, Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::stage::Stage;

/// The prefix every one of the café's environment variables carries.
const ENV_PREFIX: &str = "OBSERVABLE_CAFE_";

/// The variable that says where the settings file is.
///
/// Read by `clap` rather than by the layering below, since it decides what
/// that layering is made of and so cannot be one of its layers.
const CONFIG_ENV: &str = "OBSERVABLE_CAFE_CONFIG";

/// The settings file read when nothing points anywhere else.
///
/// Looked for in the working directory and nowhere else: one place to put it
/// is one place to look when a setting turns out not to be what was expected.
const DEFAULT_CONFIG_FILE: &str = "observable-cafe.toml";

/// Anything that stops the café being told what to do.
///
/// Boxed because a `figment::Error` carries enough context to be worth the
/// allocation only on the path that is about to end the process anyway.
pub type Error = Box<figment::Error>;

/// The settings as the command line spells them.
///
/// The fields that were not asked for are left out of the serialised form
/// rather than sent on as `false` or `null`, which is what lets a setting
/// given in the environment survive the layer above it.
#[derive(Debug, Parser, Serialize)]
#[command(
    version,
    about = "A café that can be observed.",
    // The note above is for whoever is reading this file, not for whoever
    // typed `--help`.
    long_about = None,
)]
struct Cli {
    /// Read settings from this file rather than `observable-cafe.toml`
    // Skipped rather than serialised: this says where a layer comes from
    // instead of being something that layer can set.
    #[arg(long, value_name = "PATH", env = CONFIG_ENV)]
    #[serde(skip)]
    config: Option<PathBuf>,

    /// Stop adding notebook entries on a timer
    #[arg(long)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    disable_automatic_observations: bool,

    /// Serve only this stage, at `/`
    #[arg(long, value_name = "STAGE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    stage: Option<Stage>,

    /// Give each stage a way back to the index
    #[arg(long)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    enable_navigation: bool,
}

/// The settings once every source has had its say, still worded the way they
/// were asked for.
///
/// Kept apart from [`Options`] so that the negatives the two front doors are
/// spelled with stay at the front door: nothing further in has to remember
/// that disabling observations means the timer is off.
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Settings {
    disable_automatic_observations: bool,
    stage: Option<Stage>,
    enable_navigation: bool,
}

/// What the café is run with.
#[derive(Debug, PartialEq)]
pub struct Options {
    /// Whether the clock writes observations as their interval comes due.
    pub automatic_observations: bool,
    /// The only stage this process serves, or `None` for the stage index.
    pub stage: Option<Stage>,
    /// Whether a stage offers a way back to the index.
    ///
    /// Off by default, because a stage is usually embedded in a course page
    /// that does its own navigating, and a widget pointing away from the page
    /// around it is that page's business rather than the widget's.
    pub navigation: bool,
}

impl Options {
    /// Reads the settings file, the environment and the command line, in
    /// increasing order of authority.
    ///
    /// Anything the command line cannot make sense of is reported by `clap`,
    /// which ends the process rather than returning.
    pub fn parse() -> Result<Self, Error> {
        Self::resolve(Cli::parse())
    }

    fn resolve(cli: Cli) -> Result<Self, Error> {
        let mut figment = Figment::from(Serialized::defaults(Settings::default()));

        if let Some(file) = config_file(cli.config.as_deref())? {
            figment = figment.merge(file);
        }

        let settings: Settings = figment
            // `config` says where the file layer comes from rather than what
            // is in it, so it is read by `clap` and left out here.
            .merge(Env::prefixed(ENV_PREFIX).ignore(&["config"]))
            .merge(Serialized::defaults(cli))
            .extract()
            .map_err(Box::new)?;

        settings.try_into()
    }
}

/// The settings file to read, if there is one to read.
///
/// A file that was asked for and is not there is an error, because the café
/// would otherwise run on defaults that look nothing like what was meant. The
/// default file is not asked for, so its absence is simply the ordinary case
/// of a café configured some other way, or not at all.
fn config_file(asked_for: Option<&Path>) -> Result<Option<Data<Toml>>, Error> {
    let Some(path) = asked_for else {
        let default = Path::new(DEFAULT_CONFIG_FILE);

        return Ok(default.is_file().then(|| Toml::file_exact(default)));
    };

    if !path.is_file() {
        return Err(Box::new(
            format!("no configuration file at {}", path.display()).into(),
        ));
    }

    Ok(Some(Toml::file_exact(path)))
}

impl TryFrom<Settings> for Options {
    type Error = Error;

    fn try_from(settings: Settings) -> Result<Self, Self::Error> {
        // Refused rather than quietly ignored: a single stage is served
        // without the index, so the way back would lead nowhere.
        if settings.enable_navigation && settings.stage.is_some() {
            return Err(Box::new("navigation cannot be enabled together with a single stage, which is served without the index".into()));
        }

        Ok(Self {
            automatic_observations: !settings.disable_automatic_observations,
            stage: settings.stage,
            navigation: settings.enable_navigation,
        })
    }
}

// `Jail` hands its closure a `figment::error::Result`, so the size of that
// error is not this module's to do anything about.
#[cfg(test)]
#[allow(clippy::result_large_err)]
mod tests {
    use super::{CONFIG_ENV, Cli, DEFAULT_CONFIG_FILE, ENV_PREFIX, Error, Options};
    use crate::stage::Stage;
    use clap::Parser;
    use figment::Jail;

    /// Reads `args` as the command line, with the environment as `jail` left
    /// it. The name the binary was invoked as is added back, since `clap`
    /// expects to be handed the whole of `argv`.
    fn parse(args: &[&str]) -> Result<Options, Error> {
        let cli =
            Cli::try_parse_from(std::iter::once("observable-cafe").chain(args.iter().copied()))
                .expect("arguments should parse");

        Options::resolve(cli)
    }

    #[test]
    fn the_multi_stage_version_is_served_by_default() {
        Jail::expect_with(|_| {
            assert_eq!(
                parse(&[]),
                Ok(Options {
                    automatic_observations: true,
                    stage: None,
                    navigation: false,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn observations_can_be_disabled_and_a_stage_selected() {
        Jail::expect_with(|_| {
            assert_eq!(
                parse(&["--disable-automatic-observations", "--stage", "labels"]),
                Ok(Options {
                    automatic_observations: false,
                    stage: Some(Stage::Labels),
                    navigation: false,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn navigation_can_be_turned_on() {
        Jail::expect_with(|jail| {
            assert_eq!(
                parse(&["--enable-navigation"]),
                Ok(Options {
                    automatic_observations: true,
                    stage: None,
                    navigation: true,
                })
            );

            jail.set_env(format!("{ENV_PREFIX}ENABLE_NAVIGATION"), "true");

            assert_eq!(
                parse(&[]),
                Ok(Options {
                    automatic_observations: true,
                    stage: None,
                    navigation: true,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn navigation_cannot_be_combined_with_a_single_stage() {
        Jail::expect_with(|_| {
            assert_eq!(
                parse(&["--enable-navigation", "--stage", "labels"])
                    .unwrap_err()
                    .to_string(),
                "navigation cannot be enabled together with a single stage, which is served without the index"
            );

            Ok(())
        });
    }

    #[test]
    fn an_unknown_stage_is_rejected() {
        assert!(Cli::try_parse_from(["observable-cafe", "--stage", "unknown"]).is_err());
    }

    #[test]
    fn an_unknown_argument_is_rejected() {
        assert!(Cli::try_parse_from(["observable-cafe", "--host-wrapper-option"]).is_err());
    }

    #[test]
    fn settings_can_be_given_in_a_file() {
        Jail::expect_with(|jail| {
            jail.create_file(
                DEFAULT_CONFIG_FILE,
                r#"
                stage = "types"
                disable_automatic_observations = true
            "#,
            )?;

            assert_eq!(
                parse(&[]),
                Ok(Options {
                    automatic_observations: false,
                    stage: Some(Stage::Types),
                    navigation: false,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn a_file_can_be_named_instead_of_the_default_one() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"stage = "types""#)?;
            jail.create_file("elsewhere.toml", r#"stage = "labels""#)?;

            assert_eq!(
                parse(&["--config", "elsewhere.toml"]).unwrap().stage,
                Some(Stage::Labels)
            );

            jail.set_env(CONFIG_ENV, "elsewhere.toml");

            assert_eq!(parse(&[]).unwrap().stage, Some(Stage::Labels));

            Ok(())
        });
    }

    /// Nothing has to be configured, so nothing having been is not an error.
    #[test]
    fn a_missing_default_file_is_not_an_error() {
        Jail::expect_with(|_| {
            assert!(parse(&[]).is_ok());

            Ok(())
        });
    }

    /// A file that was asked for and is not there would otherwise leave the
    /// café running on defaults that look nothing like what was meant.
    #[test]
    fn a_missing_named_file_is_an_error() {
        Jail::expect_with(|_| {
            assert_eq!(
                parse(&["--config", "nowhere.toml"])
                    .unwrap_err()
                    .to_string(),
                "no configuration file at nowhere.toml"
            );

            Ok(())
        });
    }

    #[test]
    fn an_unknown_key_in_a_file_is_rejected() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"stgae = "types""#)?;

            assert!(parse(&[]).is_err());

            Ok(())
        });
    }

    #[test]
    fn the_environment_and_the_command_line_both_beat_a_file() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"stage = "types""#)?;

            jail.set_env(format!("{ENV_PREFIX}STAGE"), "labels");
            assert_eq!(parse(&[]).unwrap().stage, Some(Stage::Labels));

            assert_eq!(
                parse(&["--stage", "samples"]).unwrap().stage,
                Some(Stage::Samples)
            );

            Ok(())
        });
    }

    #[test]
    fn settings_can_be_given_in_the_environment() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}STAGE"), "types");
            jail.set_env(
                format!("{ENV_PREFIX}DISABLE_AUTOMATIC_OBSERVATIONS"),
                "true",
            );

            assert_eq!(
                parse(&[]),
                Ok(Options {
                    automatic_observations: false,
                    stage: Some(Stage::Types),
                    navigation: false,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn false_does_not_disable_automatic_observations() {
        Jail::expect_with(|jail| {
            jail.set_env(
                format!("{ENV_PREFIX}DISABLE_AUTOMATIC_OBSERVATIONS"),
                "false",
            );

            assert_eq!(
                parse(&[]),
                Ok(Options {
                    automatic_observations: true,
                    stage: None,
                    navigation: false,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn an_environment_variable_that_is_not_a_boolean_is_rejected() {
        Jail::expect_with(|jail| {
            jail.set_env(
                format!("{ENV_PREFIX}DISABLE_AUTOMATIC_OBSERVATIONS"),
                "sometimes",
            );

            assert!(parse(&[]).is_err());

            Ok(())
        });
    }

    #[test]
    fn an_unknown_environment_variable_is_rejected() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}STGAE"), "types");

            assert!(parse(&[]).is_err());

            Ok(())
        });
    }

    #[test]
    fn the_command_line_takes_precedence_over_the_environment() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}STAGE"), "types");
            jail.set_env(
                format!("{ENV_PREFIX}DISABLE_AUTOMATIC_OBSERVATIONS"),
                "false",
            );

            assert_eq!(
                parse(&["--stage", "samples", "--disable-automatic-observations"]),
                Ok(Options {
                    automatic_observations: false,
                    stage: Some(Stage::Samples),
                    navigation: false,
                })
            );

            Ok(())
        });
    }
}
