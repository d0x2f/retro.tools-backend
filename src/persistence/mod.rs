#[macro_use]
mod macros;

pub mod boards;
pub mod cards;
pub mod participants;
pub mod ranks;
pub mod votes;

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
