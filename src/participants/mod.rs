pub mod db;
pub mod models;
pub mod routes;

use ::firestore::FirestoreDb;
use actix_identity::Identity;
use actix_web::error::Error as ActixError;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use core::future::Future;

use crate::error::Error;
use models::Participant;

pub async fn new(
  firestore: &FirestoreDb,
  identity: impl Future<Output = Result<Identity, ActixError>>,
  req: HttpRequest,
) -> Result<Participant, Error> {
  let identity = identity.await;
  Ok(match identity {
    Ok(s) => Participant {
      id: s.id().unwrap(),
    },
    _ => {
      let participant = db::new(firestore).await?;
      Identity::login(&req.extensions(), participant.id.clone())?;
      participant
    }
  })
}
