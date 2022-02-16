use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use clap;
use ocilot_core as core;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
  pub(crate) cause: Cause,
}

impl Error {
  pub fn exit(&self) -> ! {
    match &self.cause {
      Cause::Args(err) => err.exit(),
      Cause::Core(err) => safe_exit(retcode(err)),
    }
  }
}

impl From<core::error::Error> for Error {
  fn from(err: ocilot_core::error::Error) -> Self {
    Error {
      cause: Cause::Core(err),
    }
  }
}

#[derive(Debug)]
pub enum Cause {
  Args(clap::Error),
  Core(core::error::Error),
}

fn safe_exit(code: i32) -> ! {
  use std::io::Write;

  let _ = std::io::stdout().lock().flush();
  let _ = std::io::stderr().lock().flush();

  std::process::exit(code)
}

fn retcode(err: &core::error::Error) -> i32 {
  let mut hasher = DefaultHasher::new();
  hasher.write(err.to_string().as_bytes());
  // align to range: 30-255
  (hasher.finish() % 226 + 30) as i32
}
