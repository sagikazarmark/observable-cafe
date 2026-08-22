//! Where the café's settings come from.
//!
//! The same settings can be given in a file, in the environment, or on the
//! command line, because a café run by hand and a café run by a container
//! platform are told things in different ways. All three spell them the same,
//! and each in that order overrules the one before it when they disagree.

use std::path::{Path, PathBuf};

use clap::Parser;
use figment::{
    Figment,
    providers::{Data, Env, Format, Serialized, Toml},
};
use serde::de::value::StrDeserializer;
use serde::{Deserialize, Deserializer, Serialize};

use crate::feature::{Feature, Features, Preset};

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

    /// Show this set of features rather than all of them
    #[arg(long, value_name = "PRESET")]
    #[serde(skip_serializing_if = "Option::is_none")]
    preset: Option<Preset>,

    /// Show this feature as well, whatever the preset says
    #[arg(long, value_name = "FEATURE", value_delimiter = ',')]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    enable: Vec<Feature>,

    /// Do not show this feature, whatever the preset says
    #[arg(long, value_name = "FEATURE", value_delimiter = ',')]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    disable: Vec<Feature>,
}

/// The settings once every source has had its say, still worded the way they
/// were asked for.
///
/// Kept apart from [`Options`] so that working out what a preset and two lists
/// of exceptions add up to happens in one place, and nothing further in has to
/// know there was a preset at all.
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Settings {
    preset: Option<Preset>,
    /// Each list is taken from one layer whole rather than added to across
    /// layers: overruling a list means giving the list that replaces it, which
    /// is the same rule the other settings follow.
    #[serde(deserialize_with = "listed_features")]
    enable: Vec<Feature>,
    #[serde(deserialize_with = "listed_features")]
    disable: Vec<Feature>,
}

/// What the café is run with.
#[derive(Debug, PartialEq)]
pub struct Options {
    /// What the café shows, with the preset already worked out.
    pub features: Features,
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

/// Reads a list of features written either as a list or as one string with
/// commas in it.
///
/// The command line takes `--enable receipts,types`, and a file has real lists
/// to write them as, but an environment variable holds one string and nobody
/// spells a list `[receipts,types]` in a shell. Accepting both is what lets one
/// spelling serve all three.
fn listed_features<'de, D>(deserializer: D) -> Result<Vec<Feature>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Listed {
        Together(String),
        Apart(Vec<Feature>),
    }

    match Listed::deserialize(deserializer)? {
        Listed::Apart(features) => Ok(features),
        Listed::Together(listed) => listed
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(|name| Feature::deserialize(StrDeserializer::new(name)))
            .collect(),
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
        let features = Features::resolve(settings.preset, &settings.enable, &settings.disable)
            // Refused rather than resolved one way or the other: nobody means
            // both, so it is a mistake to report rather than a preference.
            .map_err(|contradicted| -> Error {
                Box::new(format!("{} is both enabled and disabled", contradicted.name()).into())
            })?;

        Ok(Self { features })
    }
}

// `Jail` hands its closure a `figment::error::Result`, so the size of that
// error is not this module's to do anything about.
#[cfg(test)]
#[allow(clippy::result_large_err)]
mod tests {
    use super::{CONFIG_ENV, Cli, DEFAULT_CONFIG_FILE, ENV_PREFIX, Error, Options};
    use crate::feature::Features;
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

    /// What a café nobody has configured shows.
    #[test]
    fn the_whole_cafe_is_served_by_default() {
        Jail::expect_with(|_| {
            assert_eq!(
                parse(&[]),
                Ok(Options {
                    features: Features::all()
                })
            );

            Ok(())
        });
    }

    #[test]
    fn a_preset_can_be_asked_for_on_the_command_line() {
        Jail::expect_with(|_| {
            let features = parse(&["--preset", "samples"]).unwrap().features;

            assert!(features.observations);
            assert!(!features.labels && !features.types && !features.receipts);

            Ok(())
        });
    }

    #[test]
    fn features_can_be_added_to_and_taken_from_a_preset() {
        Jail::expect_with(|_| {
            let features = parse(&[
                "--preset",
                "types",
                "--enable",
                "receipts",
                "--disable",
                "automatic-observations",
            ])
            .unwrap()
            .features;

            assert!(features.receipts && features.types);
            assert!(features.observations && !features.automatic_observations);

            Ok(())
        });
    }

    /// One flag per feature would be a long command line for a café that wants
    /// two of them.
    #[test]
    fn several_features_can_be_named_at_once() {
        Jail::expect_with(|_| {
            let listed = parse(&["--disable", "labels,types"]).unwrap().features;
            let repeated = parse(&["--disable", "labels", "--disable", "types"])
                .unwrap()
                .features;

            assert_eq!(listed, repeated);
            assert!(!listed.labels && !listed.types && listed.observations);

            Ok(())
        });
    }

    #[test]
    fn a_feature_that_is_both_enabled_and_disabled_is_refused() {
        Jail::expect_with(|_| {
            assert_eq!(
                parse(&["--enable", "types", "--disable", "types"])
                    .unwrap_err()
                    .to_string(),
                "types is both enabled and disabled"
            );

            Ok(())
        });
    }

    #[test]
    fn settings_can_be_given_in_a_file() {
        Jail::expect_with(|jail| {
            jail.create_file(
                DEFAULT_CONFIG_FILE,
                r#"
                preset = "labels"
                enable = ["receipts"]
            "#,
            )?;

            let features = parse(&[]).unwrap().features;

            assert!(features.labels && features.receipts);
            assert!(!features.types);

            Ok(())
        });
    }

    #[test]
    fn a_file_can_be_named_instead_of_the_default_one() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"preset = "types""#)?;
            jail.create_file("elsewhere.toml", r#"preset = "samples""#)?;

            assert!(
                !parse(&["--config", "elsewhere.toml"])
                    .unwrap()
                    .features
                    .labels
            );

            jail.set_env(CONFIG_ENV, "elsewhere.toml");

            assert!(!parse(&[]).unwrap().features.labels);

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
            jail.create_file(DEFAULT_CONFIG_FILE, r#"prest = "types""#)?;

            assert!(parse(&[]).is_err());

            Ok(())
        });
    }

    #[test]
    fn an_unknown_feature_in_a_file_is_rejected() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"disable = ["lables"]"#)?;

            assert!(parse(&[]).is_err());

            Ok(())
        });
    }

    #[test]
    fn settings_can_be_given_in_the_environment() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}PRESET"), "samples");
            jail.set_env(format!("{ENV_PREFIX}ENABLE"), "receipts");

            let features = parse(&[]).unwrap().features;

            assert!(features.observations && features.receipts);
            assert!(!features.labels && !features.types);

            Ok(())
        });
    }

    /// A variable holds one string, and nobody spells a list `[a,b]` in a
    /// shell, so the environment spells a list the way the command line does.
    #[test]
    fn several_features_can_be_named_in_one_variable() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}PRESET"), "samples");
            jail.set_env(format!("{ENV_PREFIX}ENABLE"), "receipts, types");

            let features = parse(&[]).unwrap().features;

            assert!(features.receipts && features.types);

            Ok(())
        });
    }

    #[test]
    fn an_unknown_environment_variable_is_rejected() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}PRSET"), "types");

            assert!(parse(&[]).is_err());

            Ok(())
        });
    }

    #[test]
    fn the_environment_and_the_command_line_both_beat_a_file() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"preset = "types""#)?;

            jail.set_env(format!("{ENV_PREFIX}PRESET"), "samples");
            assert!(!parse(&[]).unwrap().features.labels);

            assert!(parse(&["--preset", "labels"]).unwrap().features.labels);

            Ok(())
        });
    }

    #[test]
    fn the_command_line_takes_precedence_over_the_environment() {
        Jail::expect_with(|jail| {
            jail.set_env(format!("{ENV_PREFIX}PRESET"), "types");

            let features = parse(&["--preset", "samples", "--enable", "receipts"])
                .unwrap()
                .features;

            assert!(features.receipts && !features.types);

            Ok(())
        });
    }

    /// A list is taken from one layer whole, so overruling one means giving
    /// the list that replaces it rather than adding to what was there.
    #[test]
    fn a_list_replaces_the_one_below_it_rather_than_joining_it() {
        Jail::expect_with(|jail| {
            jail.create_file(DEFAULT_CONFIG_FILE, r#"disable = ["types"]"#)?;

            let features = parse(&["--disable", "labels"]).unwrap().features;

            assert!(features.types && !features.labels);

            Ok(())
        });
    }
}
