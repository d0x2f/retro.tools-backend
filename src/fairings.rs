use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::hyper::header::{CacheControl, CacheDirective};
use rocket::{Request, Response};

pub struct GlobalHeaders;

impl Fairing for GlobalHeaders {
  fn info(&self) -> Info {
    Info {
      name: "Global Headers",
      kind: Kind::Response,
    }
  }

  fn on_response(&self, _request: &Request, response: &mut Response) {
    response.set_header(CacheControl(vec![CacheDirective::Private]));
  }
}
