use std::fmt;
use std::fs::File;
use std::time::Instant;

use tracing::error;
use tracing_subscriber;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::TestWriter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::layer::SubscriberExt;

use crate::cli::{args, error};
use crate::cli::args::WriterKind;
use crate::cli::error::Cause;

pub fn configured(ctx: args::ExecutionContext, f: impl FnOnce() -> Option<error::Error>) -> Option<error::Error> {
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

  let file_layer = tracing_subscriber::fmt::Layer::new()
    .json()
    .with_file(true)
    .with_line_number(true)
    .with_target(true)
    .with_thread_names(true)
    .with_thread_ids(true)
    .with_writer(non_blocking);
  let enable_level = tracing::Level::INFO;
  let writer = match ctx.logger.writer {
    WriterKind::Regular => BoxMakeWriter::new(std::io::stderr.with_max_level(enable_level)),
    WriterKind::Test => BoxMakeWriter::new(TestWriter::default())
  };

  let stderr_layer = tracing_subscriber::fmt::Layer::new()
    .with_writer(writer)
    .pretty()
    .with_file(false)
    .with_line_number(false)
    .with_target(true)
    .with_timer(Uptime::default());
  let subscriber = tracing_subscriber::registry()
    .with(stderr_layer)
    .with(file_layer);

  tracing::subscriber::with_default(subscriber, || {
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
  })
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
