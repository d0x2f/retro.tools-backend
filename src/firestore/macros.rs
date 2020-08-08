#[macro_export]
macro_rules! to_participant_reference {
  ($project:expr, $participant_id:expr) => {
    format!(
      "projects/{}/databases/(default)/documents/participants/{}",
      $project, $participant_id
    )
  };
}

#[macro_export]
macro_rules! to_board_reference {
  ($project:expr, $board_id:expr) => {
    format!(
      "projects/{}/databases/(default)/documents/boards/{}",
      $project, $board_id
    )
  };
}

#[macro_export]
macro_rules! to_column_reference {
  ($project:expr, $board_id:expr, $column_id:expr) => {
    format!(
      "projects/{}/databases/(default)/documents/boards/{}/columns/{}",
      $project, $board_id, $column_id
    )
  };
}

#[macro_export]
macro_rules! to_card_reference {
  ($project:expr, $board_id:expr, $card_id:expr) => {
    format!(
      "projects/{}/databases/(default)/documents/boards/{}/cards/{}",
      $project, $board_id, $card_id
    )
  };
}

#[macro_export]
macro_rules! from_reference {
  ($reference:expr) => {
    $reference.rsplitn(2, '/').next().expect("document id")
  };
}

#[macro_export]
macro_rules! get_id {
  ($document:expr) => {
    from_reference!($document.name).into()
  };
}

#[macro_export]
macro_rules! get_create_time {
  ($document:expr) => {
    $document
      .create_time
      .ok_or_else(|| crate::error::Error::Other("field `create_time` not set in document.".into()))?
      .seconds
  };
}

#[macro_export]
macro_rules! get_array_field {
  ($document:expr, $field:literal) => {
    match $document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref())
    {
      Some(crate::firestore::v1::value::ValueType::ArrayValue(arr)) => Ok(arr),
      _ => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! extract_string {
  ($value:expr) => {
    match $value {
      Some(crate::firestore::v1::value::ValueType::ReferenceValue(s)) => Some(s.to_string()),
      Some(crate::firestore::v1::value::ValueType::StringValue(s)) => Some(s.clone()),
      _ => None,
    }
  };
}

#[macro_export]
macro_rules! get_reference_field {
  ($document:expr, $field:literal) => {
    match extract_string!($document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref()))
    {
      Some(s) => Ok(s),
      None => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! get_string_field {
  ($document:expr, $field:literal) => {
    match extract_string!($document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref()))
    {
      Some(s) => Ok(s),
      None => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! get_boolean_field {
  ($document:expr, $field:literal) => {
    match $document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref())
    {
      Some(crate::firestore::v1::value::ValueType::BooleanValue(b)) => Ok(*b),
      _ => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! get_integer_field {
  ($document:expr, $field:literal) => {
    match $document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref())
    {
      Some(crate::firestore::v1::value::ValueType::IntegerValue(i)) => Ok(*i),
      _ => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! reference_value {
  ($document_path:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::ReferenceValue(
        $document_path,
      )),
    }
  };
}

#[macro_export]
macro_rules! string_value {
  ($string:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::StringValue(
        $string.into(),
      )),
    }
  };
}

#[macro_export]
macro_rules! integer_value {
  ($int:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::IntegerValue($int)),
    }
  };
}

#[macro_export]
macro_rules! boolean_value {
  ($bool:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::BooleanValue($bool)),
    }
  };
}
