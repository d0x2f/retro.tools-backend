use firestore1_beta1::Document;
use firestore1_beta1::Value;
use std::collections::HashMap;
use std::convert::From;
use std::default::Default;

fn get_string_value(value: Option<String>) -> Value {
    Value {
        string_value: value,
        ..Default::default()
    }
}

fn get_integer_value(value: Option<u8>) -> Value {
    Value {
        integer_value: match value {
            Some(s) => Some(s.to_string()),
            None => None
        },
        ..Default::default()
    }
}

fn get_boolean_value(value: Option<bool>) -> Value {
    Value {
        boolean_value: value,
        ..Default::default()
    }
}

pub trait IntoDocument {
    fn into_document(&self) -> Document;
}

pub trait FromDocument {
    fn into_hashmap(document: Document) -> Self;
}

#[derive(Serialize, Deserialize)]
pub struct Board {
    id: Option<String>,
    name: Option<String>,
    owner_token: Option<String>,
    max_votes: Option<u8>,
    voting_open: Option<bool>,
    cards_open: Option<bool>,
}

impl IntoDocument for Board {
    fn into_document(&self) -> Document {
        let mut fields: HashMap<String, Value> = HashMap::new();
        fields.insert(String::from("name"), get_string_value(self.name.clone()));
        fields.insert(
            String::from("owner_token"),
            get_string_value(self.owner_token.clone()),
        );
        fields.insert(String::from("max_votes"), get_integer_value(self.max_votes));
        fields.insert(
            String::from("voting_open"),
            get_boolean_value(self.voting_open),
        );
        fields.insert(
            String::from("cards_open"),
            get_boolean_value(self.cards_open),
        );

        Document {
            fields: Some(fields),
            ..Default::default()
        }
    }
}

impl Board {
    pub fn from_document(document: Document) -> Option<Self> {
        let mut map = document.fields?;
        let name = document.name?;
        let mut split_name: Vec<&str> = name.split("/").collect();
        let id = split_name.pop()?;
        Some(Board {
            id: Some(id.to_string()),
            name: map.remove("name")?.string_value,
            owner_token: map.remove("owner_token")?.string_value,
            max_votes: map.remove("max_votes")?.integer_value?.parse().ok(),
            voting_open: map.remove("voting_open")?.boolean_value,
            cards_open: map.remove("cards_open")?.boolean_value,
        })
    }
}
