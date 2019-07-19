#[catch(500)]
pub fn internal_error() -> &'static str {
    ""
}

#[catch(422)]
pub fn unprocessible_entity() -> &'static str {
    ""
}

#[catch(404)]
pub fn not_found() -> &'static str {
    ""
}

#[catch(403)]
pub fn forbidden() -> &'static str {
    ""
}

#[catch(401)]
pub fn unauthorised() -> &'static str {
    ""
}

#[catch(400)]
pub fn bad_request() -> &'static str {
    ""
}
