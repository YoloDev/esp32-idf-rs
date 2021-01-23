use clap::{Clap, FromArgMatches, IntoApp};

#[derive(Clap)]
struct QuietVerbose {
  /// Increase the output's verbosity level
  /// Pass many times to increase verbosity level, up to 2.
  #[clap(
    name = "quietverbose",
    long = "verbose",
    short = 'v',
    parse(from_occurrences),
    conflicts_with = "quietquiet",
    global = true
  )]
  verbosity_level: u8,

  /// Decrease the output's verbosity level
  /// Used once, it will set warn log level.
  /// Used twice, will set the error log level.
  /// Used more times, it will silence the log completely.
  #[clap(
    name = "quietquiet",
    long = "quiet",
    short = 'q',
    parse(from_occurrences),
    conflicts_with = "quietverbose",
    global = true
  )]
  quiet_level: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
  /// Default level. Not set on the command line.
  Default,
  /// A level lower than all log levels.
  Off,
  /// Corresponds to the `Error` log level.
  Error,
  /// Corresponds to the `Warn` log level.
  Warn,
  /// Corresponds to the `Info` log level.
  Info,
  /// Corresponds to the `Debug` log level.
  Debug,
  /// Corresponds to the `Trace` log level.
  Trace,
}

impl IntoApp for Verbosity {
  fn into_app<'help>() -> clap::App<'help> {
    QuietVerbose::into_app()
  }

  fn augment_clap(app: clap::App<'_>) -> clap::App<'_> {
    QuietVerbose::augment_clap(app)
  }
}

impl FromArgMatches for Verbosity {
  fn from_arg_matches(matches: &clap::ArgMatches) -> Self {
    let qv = QuietVerbose::from_arg_matches(matches);
    match (qv.verbosity_level, qv.quiet_level) {
      (0, 0) => Verbosity::Default,
      (1, 0) => Verbosity::Debug,
      (_, 0) => Verbosity::Trace,
      (0, 1) => Verbosity::Warn,
      (0, 2) => Verbosity::Error,
      (0, _) => Verbosity::Off,
      (_, _) => unreachable!(),
    }
  }
}

impl Default for Verbosity {
  #[inline]
  fn default() -> Self {
    Self::Default
  }
}

#[derive(Clap)]
struct ColorOption {
  /// Force color output.
  #[clap(
    name = "colorforce",
    long = "force-color",
    short = 'c',
    conflicts_with = "colordisable",
    conflicts_with = "colorforceansi",
    global = true
  )]
  force_color: bool,

  /// Force ansi color output.
  #[clap(
    name = "colorforceansi",
    long = "force-ansi",
    short = 'C',
    conflicts_with = "colordisable",
    conflicts_with = "colorforce",
    global = true
  )]
  force_ansi: bool,

  /// Prevent color output
  #[clap(
    name = "colordisable",
    long = "no-color",
    conflicts_with = "colorforce",
    conflicts_with = "colorforceansi",
    global = true
  )]
  no_color: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorChoice {
  Default,
  Force,
  ForceAnsi,
  Disable,
}

impl IntoApp for ColorChoice {
  fn into_app<'help>() -> clap::App<'help> {
    ColorOption::into_app()
  }

  fn augment_clap(app: clap::App<'_>) -> clap::App<'_> {
    ColorOption::augment_clap(app)
  }
}

impl FromArgMatches for ColorChoice {
  fn from_arg_matches(matches: &clap::ArgMatches) -> Self {
    let o = ColorOption::from_arg_matches(matches);
    match (o.force_color, o.force_ansi, o.no_color) {
      (false, false, false) => ColorChoice::Default,
      (true, false, false) => ColorChoice::Force,
      (false, true, false) => ColorChoice::ForceAnsi,
      (false, false, true) => ColorChoice::Disable,
      (_, _, _) => unreachable!(),
    }
  }
}

impl Default for ColorChoice {
  #[inline]
  fn default() -> Self {
    Self::Default
  }
}
