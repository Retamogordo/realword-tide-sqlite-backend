use tide::prelude::*;
use crate::endpoints::Request;

const TOKEN: &'static str = "Token ";

pub(crate) fn transform_string_to_vec<S>(tag_list: &Option<String>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer, {

    tag_list.as_ref().and_then(|tags| {
        let mut tag_vec = tags.split(",")
            .map(|tag| tag.trim())
            .filter(|tag| *tag != "")
            .collect::<Vec<&str>>();
            
        tag_vec.sort_by_key(|tag| tag.to_lowercase());
        Some(tag_vec)
    })
    .serialize(serializer)
}

pub(crate) fn transform_datetime<S>(dt: &chrono::DateTime<chrono::Utc>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer, {
        dt.checked_add_signed(chrono::Duration::milliseconds(42))
        .serialize(serializer)
}

pub(crate) fn token_from_request(req: &Request) -> Result<&str, tide::Error> {
    let hdr = req.header(http_types::headers::AUTHORIZATION)
        .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no authorization header in request"))?
        .get(0)
        .ok_or(tide::Error::from_str(tide::StatusCode::Unauthorized, "no token in request header"))?;

    let token = hdr.as_str().trim_start_matches(TOKEN).trim_start();
    Ok(token)
}
