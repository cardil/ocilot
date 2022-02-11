use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use clap;
use clap::Parser;
use tracing::Level;

use crate::cli::error;
use crate::cli::list;
use crate::cli::publish;
use crate::cli::verbosity::Verbosity;
use crate::cli::{build, logging};

#[derive(Parser, Debug)]
#[clap(name = "Ocilot", author, version, about)]
#[clap(about = "Create and publish OCI images", long_about = None)]
pub(crate) struct Args {
  #[clap(subcommand)]
  command: Commands,

  /// The format to output.
  #[clap(arg_enum, short = 'o', long, global = true, default_value = "human")]
  output: Format,

  #[clap(flatten)]
  verbose: Verbosity,

  /// A directory where the Ocilot will hold his cache files.
  ///
  /// Be default it will be located in OS default path for cache files. For
  /// example on Linux that's `$XDG_CACHE_HOME/ocilot` or $HOME/.cache/ocilot`.
  #[clap(short = 'c', long = "cache-dir", global = true, required = false)]
  cachedir: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, clap::ArgEnum)]
enum Format {
  Human,
  Json,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
  /// Builds a OCI image by stacking artifacts on top of base image.
  Build(build::Build),
  /// Publish a build image to remote registry.
  Publish(publish::Publish),
  /// List locally built images.
  List(list::List),
}

impl Args {
  pub fn ocilot_dir(&self) -> Option<PathBuf> {
    let default_dir = || dirs::cache_dir().map(|p| p.join("ocilot"));
    self.cachedir.clone().or_else(default_dir)
  }
}

pub(crate) trait Executable {
  fn execute(&self, args: &Args) -> Option<error::Error>;
}

#[derive(Debug)]
pub enum Console {
  Real(std::io::Stdout),
  Test(Vec<u8>),
}

#[derive(Debug)]
pub struct Output {
  pub logger: logging::WriterKind,
  pub console: Console,
}

#[derive(Debug)]
pub struct ExecutionContext {
  pub args: Vec<String>,
  pub output: Output,
}

impl ExecutionContext {
  fn default() -> ExecutionContext {
    ExecutionContext {
      args: env::args().collect(),
      output: Output {
        logger: logging::WriterKind::Regular,
        console: Console::Real(std::io::stdout()),
      },
    }
  }
}

pub fn execute(ox: Option<ExecutionContext>) {
  try_execute(ox.unwrap_or(ExecutionContext::default())).map(|err| err.exit());
}

fn try_execute(ctx: ExecutionContext) -> Option<error::Error> {
  let args = ctx.args.clone();
  let result_args = <Args as clap::Parser>::try_parse_from(args);

  match result_args {
    Ok(args) => try_execute_with_args(ctx, args),
    Err(err) => Some(error::Error {
      cause: error::Cause::Args(err),
    }),
  }
}

fn try_execute_with_args(ctx: ExecutionContext, args: Args) -> Option<error::Error> {
  let mut verbose = args.verbose.clone();
  verbose.set_default(Some(Level::INFO));
  let cache_dir = args.ocilot_dir().expect("Failed to get cache dir");
  let cfg = logging::Config {
    format: match args.output {
      Format::Human => logging::Format::Compact,
      Format::Json => logging::Format::Json,
    },
    kind: ctx.output.logger,
    level: verbose.log_level(),
    cachedir: cache_dir,
  };
  logging::configured(cfg, || {
    // the subscriber based on RUST_LOG envvar will only be set as the default
    // inside this closure...
    match &args.command {
      Commands::Build(build) => build.execute(&args),
      Commands::Publish(publish) => publish.execute(&args),
      Commands::List(list) => list.execute(&args),
    }
  })
}

#[cfg(test)]
mod tests {
  use crate::cli::error::Cause;
  use crate::cli::{args, logging};

  #[test]
  fn help() {
    let tec = TestExecutionContext::new(vec!["ocilot", "help"]);

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
    let rerr = maybe_err.unwrap();
    match &rerr.cause {
      Cause::Args(err) => {
        assert_eq!(err.kind, clap::ErrorKind::DisplayHelp);
        assert!(err.to_string().contains("Create and publish OCI images"));
      }
      cause => panic!("{:?}", cause),
    }
  }

  #[test]
  fn version() {
    let tec = TestExecutionContext::new(vec!["ocilot", "--version"]);

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
    let rerr = maybe_err.unwrap();
    match &rerr.cause {
      Cause::Args(err) => {
        assert_eq!(err.kind, clap::ErrorKind::DisplayVersion);
        assert!(err.to_string().contains("Ocilot"));
      }
      cause => panic!("{:?}", cause),
    }
  }

  #[test]
  fn list() {
    let tec = TestExecutionContext::new(vec!["ocilot", "list", "--output", "json"]);

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_none());
  }

  #[test]
  fn publish() {
    let tec = TestExecutionContext::new(vec!["ocilot", "publish"]);

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
  }

  #[test]
  fn build() {
    let tec = TestExecutionContext::new(vec![
      "ocilot",
      "build",
      "--artifact",
      "**/*.rs",
      "--artifact",
      "Cargo.toml",
      "--base",
      "busybox",
      "--image",
      "quay.io/cardil/ocilot-sources",
    ]);

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_none());
  }

  struct TestExecutionContext<'a> {
    args: Vec<&'a str>,
    output: args::Output,
  }

  impl TestExecutionContext<'_> {
    fn new(args: Vec<&str>) -> TestExecutionContext {
      TestExecutionContext {
        args,
        output: args::Output {
          logger: logging::WriterKind::Test,
          console: args::Console::Test(Vec::new()),
        },
      }
    }

    fn ctx(self) -> args::ExecutionContext {
      args::ExecutionContext {
        args: self.args.iter().map(|s| s.to_string()).collect(),
        output: self.output,
      }
    }
  }
}
