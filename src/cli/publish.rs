use std::io;

use clap::Args;
use ocilot_core as core;
use tracing::info;
use Cause::Core;

use cli::{args, error};

use crate::cli;
use crate::cli::error::Cause;

#[derive(Debug, Args)]
pub struct Publish {}

impl args::Executable for Publish {
  fn execute(&self, args: &args::Args) -> error::Result<()> {
    info!(args = ?args, "Publishing: {:?}", self);
    Err(error::Error {
      cause: Core(core::error::Error::Unexpected(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        io::Error::new(io::ErrorKind::BrokenPipe, "Not yet implemented"),
      )))),
    })
  }
}
