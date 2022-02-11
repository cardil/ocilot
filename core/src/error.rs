use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
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
    }
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
    }
  }
}
