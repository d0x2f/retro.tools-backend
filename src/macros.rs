#[macro_export]
macro_rules! map_err {
  ($expression:expr) => {
    $expression.map_err(|error| {
      error!("{} - {}:{}", error.to_string(), file!(), line!());
      Status::InternalServerError
    });
  };
}
