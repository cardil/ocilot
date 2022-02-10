use clap::Args;
use tracing::{info, trace};

use crate::cli;

#[derive(Debug, Args)]
pub struct List {}

impl cli::args::Executable for List {
  fn execute(&self, args: &cli::args::Args) -> Option<cli::error::Error> {
    trace!(args = ?args);
    info!("Listing: {:?}", self);
    None
  }
}
