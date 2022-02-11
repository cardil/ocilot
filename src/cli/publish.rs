use std::io;

use clap::Args;
use ocilot_core::error::Error::Unexpected;
use tracing::info;
use Cause::Core;

use cli::{args, error};

use crate::cli;
use crate::cli::error::Cause;

#[derive(Debug, Args)]
pub struct Publish {}

impl args::Executable for Publish {
  fn execute(&self, args: &args::Args) -> Option<error::Error> {
    info!(args = ?args, "Publishing: {:?}", self);
    Some(error::Error {
      cause: Core(Unexpected(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        io::Error::new(io::ErrorKind::BrokenPipe, "Not yet implemented"),
      )))),
    })
  }
}
