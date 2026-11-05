//! Parse TV timestamps whose extended years may omit the leading `+` sign.

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::de::{Deserialize, Deserializer, Error};

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error> {
    let value = Cow::<'de, str>::deserialize(deserializer)?;
    let normalized = if value
        .split_once('-')
        .is_some_and(|(year, _)| year.len() > 4 && year.bytes().all(|byte| byte.is_ascii_digit()))
    {
        Cow::Owned(format!("+{value}"))
    } else {
        value
    };
    normalized.parse().map_err(D::Error::custom)
}
