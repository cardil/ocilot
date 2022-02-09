use std::fmt;
use std::fs::File;
use std::time::Instant;

use tracing::{error, Level};
use tracing_subscriber;
use tracing_subscriber::fmt::{Layer, TestWriter};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::cli::error;
use crate::cli::error::Cause;

#[derive(Debug, PartialEq)]
pub struct Config {
  pub format: Format,
  pub kind: WriterKind,
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

pub fn configured(cfg: Config, f: impl FnOnce() -> Option<error::Error>) -> Option<error::Error> {
  let config_dir = dirs::config_dir().expect("Failed to get config dir");
  let ocilot_dir = config_dir.join("ocilot");
  let logfile_path = ocilot_dir.join("last-log.jsonl");
  let logfile = File::options()
    .write(true)
    .create(true)
    .truncate(true)
    .open(logfile_path.as_path())
    .expect(format!("Failed to open log file: {:?}", logfile_path).as_str());

  let (non_blocking, _guard) =
    tracing_appender::non_blocking(logfile);

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
    .compact()
    .with_writer(writer(&cfg))
    .with_file(false)
    .with_line_number(false)
    .with_target(true)
    .with_timer(Uptime::default());

  let base_subscriber = Registry::default()
    .with(file_layer);

  let handle_err = || {
    let maybe_err = f();
    match &maybe_err {
      None => {}
      Some(err) => match &err.cause {
        Cause::Args(_) => {}
        Cause::Unexpected(fatal) => {
          let details = format!("{:?}", fatal);
          let logfile = logfile_path.as_path().to_str();
          let hint = "Consider checking the logfile for complete logs of last execution";
          error!(
            hint,
            logfile,
            details = details.as_str(),
            "Unexpected: '{}'",
            fatal.to_string()
          )
        }
      }
    }
    maybe_err
  };

  match cfg.format {
    Format::Compact =>
      tracing::subscriber::with_default(
        base_subscriber.with(stderr_layer), handle_err),
    Format::Json =>
      tracing::subscriber::with_default(
        base_subscriber.with(json_layer), handle_err),
  }
}

fn writer(cfg: &Config) -> BoxMakeWriter {
  let enable_level = Level::INFO;
  match cfg.kind {
    WriterKind::Regular => BoxMakeWriter::new(std::io::stderr
      .with_max_level(enable_level)),
    WriterKind::Test => BoxMakeWriter::new(TestWriter::default())
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
