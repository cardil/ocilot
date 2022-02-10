use tracing::Level;

#[derive(clap::Args, Debug, Clone)]
pub struct Verbosity {
  /// Pass many times for more log output
  ///
  /// By default, it'll report infos, warns, and errors. Passing `-v` one time
  /// also prints debug messages, `-vv` enables trace information.
  #[clap(long, short = 'v', parse(from_occurrences), global = true)]
  verbose: i8,

  /// Pass many times for less log output
  ///
  /// By default, it'll report infos, warns, and errors. Passing `-q` one time
  /// limits to warns and errors, `-qq` limits to errors, `-qqq` disable logs.
  #[clap(
    long,
    short = 'q',
    parse(from_occurrences),
    conflicts_with = "verbose",
    global = true
  )]
  quiet: i8,

  #[clap(skip)]
  default: i8,
}

impl Verbosity {
  /// Change the default level.
  ///
  /// When the level is lower than `log::Error` (the default), multiple `-q`s will be needed for
  /// complete silence
  ///
  /// `None` means all output is disabled.
  pub fn set_default(&mut self, level: Option<Level>) {
    self.default = level_value(level);
  }

  /// Get the log level.
  ///
  /// `None` means all output is disabled.
  pub fn log_level(&self) -> Option<Level> {
    level_enum(self.verbosity())
  }

  fn verbosity(&self) -> i8 {
    self.default - self.quiet + self.verbose
  }
}

fn level_value(level: Option<Level>) -> i8 {
  match level {
    None => -1,
    Some(Level::ERROR) => 0,
    Some(Level::WARN) => 1,
    Some(Level::INFO) => 2,
    Some(Level::DEBUG) => 3,
    Some(Level::TRACE) => 4,
  }
}

fn level_enum(verbosity: i8) -> Option<Level> {
  match verbosity {
    std::i8::MIN..=-1 => None,
    0 => Some(Level::ERROR),
    1 => Some(Level::WARN),
    2 => Some(Level::INFO),
    3 => Some(Level::DEBUG),
    4..=std::i8::MAX => Some(Level::TRACE),
  }
}

#[cfg(test)]
mod test {
  use clap::Parser;
  use tracing::Level;

  use crate::cli::verbosity::Verbosity;

  #[test]
  fn double_verbose() {
    #[derive(Debug, Parser)]
    struct Args {
      #[clap(flatten)]
      verbose: Verbosity,
    }

    let cases = [
      TestCase {
        input: "-vv",
        default: Some(Level::ERROR),
        want: Some(Level::INFO),
      },
      TestCase {
        input: "-q",
        default: Some(Level::INFO),
        want: Some(Level::WARN),
      },
      TestCase {
        input: "-qqq",
        default: Some(Level::INFO),
        want: None,
      },
    ];

    for case in cases {
      let result_args = <Args as Parser>::try_parse_from(["app", case.input]);
      match result_args {
        Err(err) => panic!("{}", err),
        Ok(args) => {
          let mut verbose = args.verbose;
          verbose.set_default(case.default);
          assert_eq!(verbose.log_level(), case.want);
        }
      };
    }
  }

  struct TestCase<'a> {
    input: &'a str,
    default: Option<Level>,
    want: Option<Level>,
  }
}
