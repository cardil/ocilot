use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use std::{fmt, fs};

use tracing::{debug, error, trace, Level};
use tracing_subscriber;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::fmt::{Layer, TestWriter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::cli::error::{Cause, Result};
use ocilot_core as core;

#[derive(Debug, PartialEq)]
pub struct Config {
  pub format: Format,
  pub kind: WriterKind,
  pub level: Option<Level>,
  pub cachedir: PathBuf,
}

#[derive(Debug, PartialEq)]
pub enum Format {
  Compact,
  Json,
}

#[derive(Debug, PartialEq)]
pub enum WriterKind {
  Regular,
  Test,
}

pub fn configured(cfg: Config, f: impl FnOnce() -> Result<()>) -> Result<()> {
  fs::create_dir_all(&cfg.cachedir).expect(
    format!("Failed to create cache-dir: {:?}", &cfg.cachedir).as_str(),
  );
  let logfile_path = cfg.cachedir.join("last-log.jsonl");
  let logfile = File::options()
    .write(true)
    .create(true)
    .truncate(true)
    .open(logfile_path.as_path())
    .expect(format!("Failed to open log file: {:?}", logfile_path).as_str());

  let (non_blocking, _guard) = tracing_appender::non_blocking(logfile);

  let file_layer = Layer::new()
    .json()
    .with_file(true)
    .with_line_number(true)
    .with_target(true)
    .with_thread_names(true)
    .with_thread_ids(true)
    .with_writer(non_blocking);

  let json_layer = Layer::new()
    .json()
    .with_writer(writer(&cfg))
    .with_file(true)
    .with_line_number(true)
    .with_target(true);
  let stderr_layer = Layer::new()
    .pretty()
    .with_writer(writer(&cfg))
    .with_file(cfg.kind == WriterKind::Test)
    .with_line_number(cfg.kind == WriterKind::Test)
    .with_target(true)
    .with_timer(Uptime::default());

  let base_subscriber = Registry::default().with(file_layer);

  let handle_err = || {
    let maybe_err = f();
    let hint =
      "Consider checking the logfile for complete logs of last execution";
    match &maybe_err {
      Ok(_) => {
        debug!(hint, logfile = ?logfile_path);
      }
      Err(err) => match &err.cause {
        Cause::Args(_) => {}
        Cause::Core(cerr) => match cerr {
          core::error::Error::Unexpected(fatal) => {
            error!(
              hint,
              logfile = ?logfile_path,
              "Unexpected: '{}'", fatal
            );
            debug!(cause = ?fatal, "Caused by")
          }
          core::error::Error::InvalidInput { message, cause } => {
            error!("Invalid arguments: {}", message);
            trace!("Caused by: {:?}", cause);
          }
          core::error::Error::Bug(bug) => error!(
            hint,
            logfile = ?logfile_path,
            "Bug found: {:?}", bug
          ),
        },
      },
    }
    maybe_err
  };

  match cfg.format {
    Format::Compact => tracing::subscriber::with_default(
      base_subscriber.with(stderr_layer),
      handle_err,
    ),
    Format::Json => tracing::subscriber::with_default(
      base_subscriber.with(json_layer),
      handle_err,
    ),
  }
}

fn writer(cfg: &Config) -> BoxMakeWriter {
  let enable_level = cfg.level.unwrap_or(Level::ERROR);
  let enable = cfg.level.is_some();
  match cfg.kind {
    WriterKind::Regular => BoxMakeWriter::new(
      std::io::stderr
        .with_max_level(enable_level)
        .with_filter(move |_| enable),
    ),
    WriterKind::Test => BoxMakeWriter::new(TestWriter::default()),
  }
}

/// Retrieve and print the relative elapsed wall-clock time since an epoch.
///
/// The `Default` implementation for `Uptime` makes the epoch the current time.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Uptime {
  epoch: Instant,
}

impl Default for Uptime {
  fn default() -> Self {
    Uptime {
      epoch: Instant::now(),
    }
  }
}

impl From<Instant> for Uptime {
  fn from(epoch: Instant) -> Self {
    Uptime { epoch }
  }
}

impl FormatTime for Uptime {
  fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
    let e = self.epoch.elapsed();
    write!(w, "{:4}.{:06}s", e.as_secs(), e.subsec_micros())
  }
}
