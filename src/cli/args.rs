use std::env;
use std::fmt::Debug;

use clap;
use clap::Parser;

use crate::cli::{build, logging};
use crate::cli::error;
use crate::cli::list;
use crate::cli::publish;

#[derive(Parser, Debug)]
#[clap(name = "Ocilot", author, version, about)]
#[clap(about = "Create and publish OCI images", long_about = None)]
pub(crate) struct Args {
  #[clap(subcommand)]
  command: Commands,

  /// The format to output.
  #[clap(arg_enum, short = 'o', long = "output", global = true, default_value = "human")]
  output: Format,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, clap::ArgEnum)]
enum Format {
  Human,
  Json,
  Yaml,
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

pub(crate) trait Executable {
  fn execute(&self, args: &Args) -> Option<error::Error>;
}

#[derive(Debug)]
pub struct LoggerConfig {
  pub writer: WriterKind,
}

#[derive(Debug, PartialEq)]
pub enum WriterKind {
  Regular,
  Test,
}

#[derive(Debug)]
pub struct ExecutionContext {
  pub args: Vec<String>,
  pub logger: LoggerConfig,
}

impl ExecutionContext {
  fn default() -> ExecutionContext {
    ExecutionContext {
      args: env::args().collect(),
      logger: LoggerConfig { writer: WriterKind::Regular },
    }
  }
}

pub fn execute(ox: Option<ExecutionContext>) {
  try_execute(ox.unwrap_or(ExecutionContext::default()))
    .map(|err| err.exit());
}

fn try_execute(ctx: ExecutionContext) -> Option<error::Error> {
  let args = ctx.args.clone();
  let result_args = <Args as clap::Parser>::try_parse_from(args);

  match result_args {
    Ok(args) => try_execute_with_args(ctx, args),
    Err(err) => Some(error::Error {
      cause: error::Cause::Args(err)
    })
  }
}

fn try_execute_with_args(ctx: ExecutionContext, args: Args) -> Option<error::Error> {
  logging::configured(ctx, || {
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
  use crate::cli::args;
  use crate::cli::args::{LoggerConfig, WriterKind};
  use crate::cli::error::Cause;

  #[test]
  fn help() {
    let tec = TestExecutionContext::new(
      vec!["ocilot", "help"],
    );

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
    let rerr = maybe_err.unwrap();
    match &rerr.cause {
      Cause::Args(err) => {
        assert_eq!(err.kind, clap::ErrorKind::DisplayHelp);
        assert!(err.to_string().contains("Create and publish OCI images"));
      }
      Cause::Unexpected(err) => {
        panic!("{}", err);
      }
    }
  }

  #[test]
  fn version() {
    let tec = TestExecutionContext::new(
      vec!["ocilot", "--version"],
    );

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
    let rerr = maybe_err.unwrap();
    match &rerr.cause {
      Cause::Args(err) => {
        assert_eq!(err.kind, clap::ErrorKind::DisplayVersion);
        assert!(err.to_string().contains("Ocilot"));
      }
      Cause::Unexpected(err) => {
        panic!("{}", err);
      }
    }
  }

  #[test]
  fn list() {
    let tec = TestExecutionContext::new(
      vec!["ocilot", "list", "--output", "yaml"]
    );

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_none());
  }

  struct TestExecutionContext<'a> {
    args: Vec<&'a str>,
    logger: LoggerConfig,
  }

  impl TestExecutionContext<'_> {
    fn new(args: Vec<&str>) -> TestExecutionContext {
      TestExecutionContext {
        args,
        logger: LoggerConfig { writer: WriterKind::Test },
      }
    }

    fn ctx(self) -> args::ExecutionContext {
      args::ExecutionContext {
        args: self.args.iter().map(|s| s.to_string()).collect(),
        logger: self.logger,
      }
    }
  }
}
