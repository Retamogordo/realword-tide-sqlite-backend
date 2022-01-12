use tide::prelude::*;

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

