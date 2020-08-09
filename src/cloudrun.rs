use crate::error::Error;
use bytes::Bytes;

pub fn get_metadata(entry: &'static str) -> std::result::Result<Bytes, Error> {
  let response = reqwest::blocking::Client::new()
    .get(
      format!(
        "http://metadata.google.internal/computeMetadata/v1/{}",
        entry
      )
      .as_str(),
    )
    .header("Metadata-Flavor", "Google")
    .send()?;

  Ok(response.bytes()?)
}

pub fn get_project_id() -> std::result::Result<String, Error> {
  let bytes = get_metadata("project/project-id")?;
  let project_id = String::from_utf8(bytes.to_vec())?;
  Ok(project_id)
}
