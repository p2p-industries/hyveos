use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonResult<T, E> {
    Ok {
        success: bool,
        data: T,
    },
    Err {
        success: bool,
        #[serde(
            bound(serialize = "E: Display", deserialize = "E: FromStr, E::Err: Display"),
            with = "to_and_from_string"
        )]
        error: E,
    },
}

impl<T, E> From<Result<T, E>> for JsonResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(data) => JsonResult::Ok {
                success: true,
                data,
            },
            Err(error) => JsonResult::Err {
                success: false,
                error,
            },
        }
    }
}

impl<T, E> From<JsonResult<T, E>> for Result<T, E> {
    fn from(result: JsonResult<T, E>) -> Self {
        match result {
            JsonResult::Ok { data, .. } => Ok(data),
            JsonResult::Err { error, .. } => Err(error),
        }
    }
}

mod to_and_from_string {
    use std::{fmt::Display, str::FromStr};

    use serde::{Deserialize as _, Deserializer, Serializer};

    pub fn serialize<S, V>(v: &V, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: std::fmt::Display,
    {
        serializer.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D, V>(deserializer: D) -> Result<V, D::Error>
    where
        D: Deserializer<'de>,
        V: FromStr,
        V::Err: Display,
    {
        let s = String::deserialize(deserializer)?;
        V::from_str(&s).map_err(serde::de::Error::custom)
    }
}
