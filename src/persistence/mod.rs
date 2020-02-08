pub mod boards;
pub mod cards;
pub mod participants;
pub mod ranks;
pub mod votes;

use diesel::result::Error as DieselError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
  NotFound,
  Other,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Error::NotFound => "Not Found Error",
        Error::Other => "Other Error",
      }
    )
  }
}

impl From<DieselError> for Error {
  fn from(error: DieselError) -> Error {
    match error {
      DieselError::NotFound => Error::NotFound,
      _ => {
        error!(
          "Unexpected Error: {} - {}:{}",
          error.to_string(),
          file!(),
          line!()
        );
        Error::Other
      }
    }
  }
}
