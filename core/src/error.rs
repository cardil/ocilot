use std::fmt::{Display, Formatter};
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  Bug(String),
  Unexpected(Box<dyn std::error::Error>),
  InvalidInput {
    message: String,
    cause: Option<Box<dyn std::error::Error>>,
  },
}

impl Error {
  pub fn invalid_input(message: &str) -> Error {
    Error::InvalidInput {
      message: message.to_string(),
      cause: None,
    }
  }

  pub fn invalid_input_from(cause: Box<dyn std::error::Error>) -> Error {
    Error::InvalidInput {
      message: format!("{}", &cause),
      cause: Some(cause),
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Unexpected(err) => write!(f, "unexpected: {}", &err),
      Error::InvalidInput { message, cause: _ } => {
        write!(f, "invalid input: {}", &message)
      }
      Error::Bug(msg) => write!(f, "bug: {}", msg),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(ioerr: std::io::Error) -> Self {
    Error::Unexpected(Box::new(ioerr))
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Unexpected(err) => err.source(),
      Error::InvalidInput { message: _, cause } => match cause {
        None => None,
        Some(err) => Some(&***Box::from(err)),
      },
      Error::Bug(_) => None,
    }
  }
}
