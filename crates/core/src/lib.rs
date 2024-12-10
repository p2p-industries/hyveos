#![warn(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "serde")]
use std::str::FromStr;
use std::{env, path::PathBuf};

use dirs::runtime_dir;
use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use ulid::Ulid;

pub use crate::error::{Error, Result};

pub mod debug;
pub mod dht;
pub mod discovery;
pub mod error;
pub mod file_transfer;
pub mod gossipsub;
pub mod req_resp;
#[doc(hidden)]
#[cfg(feature = "scripting")]
pub mod scripting;

pub mod grpc {
    #![allow(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]

    tonic::include_proto!("script");
}

pub const BRIDGE_SHARED_DIR_ENV_VAR: &str = "HYVEOS_BRIDGE_SHARED_DIR";
pub const BRIDGE_SOCKET_ENV_VAR: &str = "HYVEOS_BRIDGE_SOCKET";
pub const DAEMON_NAME: &str = "hyved";

#[must_use]
pub fn get_runtime_base_path() -> PathBuf {
    runtime_dir()
        .unwrap_or_else(env::temp_dir)
        .join(DAEMON_NAME)
}

impl From<Vec<u8>> for grpc::Data {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl From<grpc::Data> for Vec<u8> {
    fn from(data: grpc::Data) -> Self {
        data.data
    }
}

impl From<Option<Vec<u8>>> for grpc::OptionalData {
    fn from(data: Option<Vec<u8>>) -> Self {
        Self {
            data: data.map(Into::into),
        }
    }
}

impl From<grpc::OptionalData> for Option<Vec<u8>> {
    fn from(data: grpc::OptionalData) -> Self {
        data.data.map(Into::into)
    }
}

impl From<PeerId> for grpc::Peer {
    fn from(peer_id: PeerId) -> Self {
        Self {
            peer_id: peer_id.to_string(),
        }
    }
}

impl TryFrom<grpc::Peer> for PeerId {
    type Error = Error;

    fn try_from(peer: grpc::Peer) -> Result<Self> {
        peer.peer_id.parse().map_err(Into::into)
    }
}

impl FromIterator<PeerId> for grpc::Peers {
    fn from_iter<I: IntoIterator<Item = PeerId>>(iter: I) -> Self {
        Self {
            peers: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<grpc::Peers> for Vec<PeerId> {
    type Error = Error;

    fn try_from(peers: grpc::Peers) -> Result<Self> {
        peers.peers.into_iter().map(TryInto::try_into).collect()
    }
}

impl From<Ulid> for grpc::Id {
    fn from(id: Ulid) -> Self {
        Self {
            ulid: id.to_string(),
        }
    }
}

impl TryFrom<grpc::Id> for Ulid {
    type Error = Error;

    fn try_from(id: grpc::Id) -> Result<Self> {
        id.ulid.parse().map_err(Into::into)
    }
}

#[cfg(feature = "serde")]
pub struct JsonResult<T, E>(Result<T, E>);

#[cfg(feature = "serde")]
impl<T, E> From<Result<T, E>> for JsonResult<T, E> {
    fn from(result: Result<T, E>) -> Self {
        Self(result)
    }
}

#[cfg(feature = "serde")]
impl<T, E> From<JsonResult<T, E>> for Result<T, E> {
    fn from(result: JsonResult<T, E>) -> Self {
        result.0
    }
}

#[cfg(feature = "serde")]
impl<T, E> Serialize for JsonResult<T, E>
where
    T: Serialize,
    E: ToString,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct as _;

        let mut state = serializer.serialize_struct("Result", 2)?;
        match &self.0 {
            Ok(value) => {
                state.serialize_field("success", &true)?;
                state.serialize_field("data", value)?;
            }
            Err(error) => {
                state.serialize_field("success", &false)?;
                state.serialize_field("error", &error.to_string())?;
            }
        }

        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T, E> Deserialize<'de> for JsonResult<T, E>
where
    T: Deserialize<'de>,
    E: FromStr,
    E::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        use serde::{
            de::{self, MapAccess, Visitor},
            Deserialize,
        };

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Success,
            Data,
            Error,
        }

        struct JsonResultVisitor<'de, T, E>
        where
            T: Deserialize<'de>,
            E: FromStr,
            E::Err: std::fmt::Display,
        {
            _marker: std::marker::PhantomData<JsonResult<T, E>>,
            _lifetime: std::marker::PhantomData<&'de ()>,
        }

        impl<'de, T, E> Visitor<'de> for JsonResultVisitor<'de, T, E>
        where
            T: Deserialize<'de>,
            E: FromStr,
            E::Err: std::fmt::Display,
        {
            type Value = JsonResult<T, E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct JsonResult")
            }

            fn visit_map<V>(self, mut map: V) -> Result<JsonResult<T, E>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut success = None;
                let mut data = None;
                let mut error = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Success => {
                            if success.is_some() {
                                return Err(de::Error::duplicate_field("success"));
                            }
                            success = Some(map.next_value()?);
                        }
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        Field::Error => {
                            if error.is_some() {
                                return Err(de::Error::duplicate_field("error"));
                            }
                            error = Some(
                                map.next_value::<&str>()?
                                    .parse()
                                    .map_err(de::Error::custom)?,
                            );
                        }
                    }
                }

                if success.ok_or_else(|| de::Error::missing_field("success"))? {
                    let data = data.ok_or_else(|| de::Error::missing_field("data"))?;

                    if error.is_some() {
                        return Err(de::Error::custom("both data and error fields are present"));
                    }

                    Ok(JsonResult(Ok(data)))
                } else {
                    let error = error.ok_or_else(|| de::Error::missing_field("error"))?;

                    if data.is_some() {
                        return Err(de::Error::custom("both data and error fields are present"));
                    }

                    Ok(JsonResult(Err(error)))
                }
            }
        }

        const FIELDS: &[&str] = &["success", "data", "error"];
        deserializer.deserialize_struct(
            "JsonResult",
            FIELDS,
            JsonResultVisitor {
                _marker: std::marker::PhantomData,
                _lifetime: std::marker::PhantomData,
            },
        )
    }
}
