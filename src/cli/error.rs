use std::collections::hash_map::DefaultHasher;
use std::error;
use std::hash::Hasher;

use clap;

#[derive(Debug)]
pub struct Error {
  pub(crate) cause: Cause,
}

impl Error {
  pub fn exit(&self) -> ! {
    match &self.cause {
      Cause::Args(err) => err.exit(),
      Cause::Unexpected(err) => safe_exit(retcode(err)),
    }
  }
}

#[derive(Debug)]
pub enum Cause {
  Args(clap::Error),
  Unexpected(Box<dyn error::Error + Send + Sync>),
}

fn safe_exit(code: i32) -> ! {
  use std::io::Write;

  let _ = std::io::stdout().lock().flush();
  let _ = std::io::stderr().lock().flush();

  std::process::exit(code)
}

fn retcode(err: &Box<dyn error::Error + Send + Sync>) -> i32 {
  let mut hasher = DefaultHasher::new();
  hasher.write(err.to_string().as_bytes());
  // align to range: 30-255
  (hasher.finish() % 226 + 30) as i32
}
