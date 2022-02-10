use std::io;

use clap::Args;
use tracing::info;

use cli::{args, error};

use crate::cli;

#[derive(Debug, Args)]
pub struct Publish {}

impl args::Executable for Publish {
  fn execute(&self, args: &args::Args) -> Option<error::Error> {
    info!(args = ?args, "Publishing: {:?}", self);
    Some(error::Error {
      cause: error::Cause::Unexpected(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        io::Error::new(io::ErrorKind::BrokenPipe, "Not yet implemented"),
      ))),
    })
  }
}
