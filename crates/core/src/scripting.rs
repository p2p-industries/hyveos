use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    error::{Error, Result},
    grpc,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RunningScript {
    pub id: Ulid,
    pub image: Arc<str>,
    pub name: Option<Arc<str>>,
}

impl From<RunningScript> for grpc::RunningScript {
    fn from(script: RunningScript) -> Self {
        Self {
            id: script.id.into(),
            image: grpc::DockerImage {
                name: script.image.to_string(),
            },
            name: script.name.map(|name| name.to_string()),
        }
    }
}

impl TryFrom<grpc::RunningScript> for RunningScript {
    type Error = Error;

    fn try_from(script: grpc::RunningScript) -> Result<Self> {
        Ok(Self {
            id: script.id.try_into()?,
            image: script.image.name.into(),
            name: script.name.map(Into::into),
        })
    }
}
