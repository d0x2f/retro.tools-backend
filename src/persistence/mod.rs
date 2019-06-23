use firestore1_beta1::Document;
use firestore1_beta1::Error;
use firestore1_beta1::Firestore as OrigFirestore;
use firestore1_beta1::ListDocumentsResponse;
use oauth2::ServiceAccountKey;
use std::env;
use std::fs::File;
use std::io::prelude::*;

type Firestore = OrigFirestore<hyper::Client, oauth2::ServiceAccountAccess<hyper::Client>>;

fn get_service_account_credentials() -> Result<ServiceAccountKey, String> {
    let secret_base64 = match env::var("GOOGLE_APPLICATION_CREDENTIALS_B64") {
        Err(_) => {
            return Err(
                "Environment variable 'GOOGLE_APPLICATION_CREDENTIALS_B64' not set.".to_string(),
            )
        }
        Ok(value) => value,
    };

    let secret_json = match base64::decode(&secret_base64) {
        Err(_) => {
            return Err(
                "Unable to decode 'GOOGLE_APPLICATION_CREDENTIALS_B64' as base64.".to_string(),
            )
        }
        Ok(value) => match String::from_utf8(value) {
            Err(_) => return Err("Error converting credentials base64 to string.".to_string()),
            Ok(value) => value,
        },
    };

    let mut file = match File::create("/tmp/service_account") {
        Err(_) => return Err("Unable to create temporary file.".to_string()),
        Ok(value) => value,
    };
    match file.write_all(secret_json.into_bytes().as_slice()) {
        Err(_) => return Err("Unable to write to temporary file.".to_string()),
        Ok(value) => value,
    };
    match oauth2::service_account_key_from_file(&"/tmp/service_account".to_string()) {
        Err(_) => return Err("Unable to read temporary file.".to_string()),
        Ok(value) => Ok(value),
    }
}

fn create_firestore_client(service_account_key: ServiceAccountKey) -> Firestore {
    let auth = oauth2::ServiceAccountAccess::new(
        service_account_key,
        hyper::Client::with_connector(hyper::net::HttpsConnector::new(
            hyper_rustls::TlsClient::new(),
        )),
    );
    Firestore::new(
        hyper::Client::with_connector(hyper::net::HttpsConnector::new(
            hyper_rustls::TlsClient::new(),
        )),
        auth,
    )
}

fn create_client() -> Result<Firestore, String> {
    let service_account_key = get_service_account_credentials()?;
    Ok(create_firestore_client(service_account_key))
}

pub fn put(parent: &str, collection: &str, document: Document) -> Result<(), String> {
    let result = create_client()?
        .projects()
        .databases_documents_create_document(document, parent, collection)
        .doit();

    match result {
        Err(error) => Err(format!("{}", error)),
        Ok(_) => Ok(()),
    }
}

pub fn patch(
    parent: &str,
    collection: &str,
    id: String,
    mut document: Document,
) -> Result<(), String> {
    document.name = Some(format!("{}/{}/{}", parent, collection, id));
    let result = create_client()?
        .projects()
        .databases_documents_patch(
            document,
            format!("{}/{}/{}", parent, collection, id).as_str(),
        )
        .add_update_mask_field_paths("name")
        .doit();

    match result {
        Err(error) => Err(format!("{}", error)),
        Ok(_) => Ok(()),
    }
}

pub fn get(parent: &str, collection: &str, id: String) -> Result<Document, String> {
    let result = create_client()?
        .projects()
        .databases_documents_get(format!("{}/{}/{}", parent, collection, id).as_str())
        .doit();

    match result {
        Err(error) => Err(format!("{}", error)),
        Ok(r) => Ok(r.1),
    }
}

fn get_list_page(
    firestore: &Firestore,
    parent: &str,
    collection: &str,
    page_token: Option<String>,
) -> Result<ListDocumentsResponse, String> {
    let mut query = firestore
        .projects()
        .databases_documents_list(parent, collection)
        .page_size(10);

    if let Some(t) = page_token {
        query = query.page_token(t.as_str());
    }

    let result = query.doit();

    match result {
        Err(error) => Err(format!("{}", error)),
        Ok(r) => Ok(r.1),
    }
}

pub fn get_list(parent: &str, collection: &str) -> Result<Vec<Document>, String> {
    let firestore = create_client()?;
    let mut documents = Vec::<Document>::new();
    let mut page_token: Option<String> = None;
    let mut pages_remain = true;

    while pages_remain {
        let page = get_list_page(&firestore, parent, collection, page_token)?;
        if let Some(mut d) = page.documents {
            documents.append(&mut d);
        }
        page_token = page.next_page_token;
        match page_token {
            Some(_) => pages_remain = true,
            None => pages_remain = false,
        };
    }
    Ok(documents)
}
