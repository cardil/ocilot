use std::env;

use clap;
use clap::Parser;

use crate::cli::build;
use crate::cli::list;
use crate::cli::publish;

#[derive(Parser)]
#[clap(name = "Ocilot", author, version, about)]
#[clap(about = "Create and publish OCI images", long_about = None)]
struct Args {
  #[clap(subcommand)]
  command: Commands,

  /// The format to output.
  #[clap(arg_enum, short = 'o', long = "output", global = true, default_value = "human")]
  output: Format,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ArgEnum)]
enum Format {
  Human,
  Json,
  Yaml,
}

#[derive(clap::Subcommand)]
enum Commands {
  /// Builds a OCI image by stacking artifacts on top of base image.
  Build(build::Build),
  /// Publish a build image to remote registry.
  Publish(publish::Publish),
  /// List locally built images.
  List(list::List),
}

pub struct ExecutionContext {
  pub args: Vec<String>,
}

impl ExecutionContext {
  fn default() -> ExecutionContext {
    ExecutionContext {
      args: env::args().collect(),
    }
  }
}

pub fn execute(ox: Option<ExecutionContext>) {
  try_execute(ox.unwrap_or(ExecutionContext::default()))
    .map(|err| err.exit());
}

fn try_execute(ctx: ExecutionContext) -> Option<clap::Error> {
  let result_args = <Args as clap::Parser>::try_parse_from(ctx.args);

  match result_args {
    Ok(args) => {
      match &args.command {
        Commands::Build(build) => {
          println!("Building: {:?}", build.to_core());
        }
        Commands::Publish(publish) => {
          println!("Publishing: {:?}", publish);
        }
        Commands::List(list) => {
          println!("Listing: {:?}", list);
        }
      }
      None
    }
    Err(err) => Some(err)
  }
}

#[cfg(test)]
mod tests {
  use crate::cli::args;

  #[test]
  fn help() {
    let tec = TestExecutionContext {
      args: vec!["ocilot", "help"],
    };

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
    let err = maybe_err.unwrap();
    assert_eq!(err.kind, clap::ErrorKind::DisplayHelp);
    assert!(err.to_string().contains("Create and publish OCI images"));
  }

  #[test]
  fn version() {
    let tec = TestExecutionContext {
      args: vec!["ocilot", "--version"],
    };

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_some());
    let err = maybe_err.unwrap();
    assert_eq!(err.kind, clap::ErrorKind::DisplayVersion);
    assert!(err.to_string().contains("Ocilot"));
  }

  #[test]
  fn list() {
    let tec = TestExecutionContext {
      args: vec!["ocilot", "list", "--output", "yaml"],
    };

    let maybe_err = args::try_execute(tec.ctx());

    assert!(maybe_err.is_none());
  }

  struct TestExecutionContext<'a> {
    args: Vec<&'a str>,
  }

  impl TestExecutionContext<'_> {
    fn ctx(&self) -> args::ExecutionContext {
      args::ExecutionContext {
        args: self.args.iter().map(|s| s.to_string()).collect(),
      }
    }
  }
}
