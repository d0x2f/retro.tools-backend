#[macro_export]
macro_rules! map_diesel_err {
  ($expression:expr) => {
    $expression.map_err(|error| match error {
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
    });
  };
}
