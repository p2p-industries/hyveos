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
pub struct RunningApp {
    pub id: Ulid,
    pub image: Arc<str>,
    pub name: Option<Arc<str>>,
}

impl From<RunningApp> for grpc::RunningApp {
    fn from(app: RunningApp) -> Self {
        Self {
            id: app.id.into(),
            image: grpc::DockerImage {
                name: app.image.to_string(),
            },
            name: app.name.map(|name| name.to_string()),
        }
    }
}

impl TryFrom<grpc::RunningApp> for RunningApp {
    type Error = Error;

    fn try_from(app: grpc::RunningApp) -> Result<Self> {
        Ok(Self {
            id: app.id.try_into()?,
            image: app.image.name.into(),
            name: app.name.map(Into::into),
        })
    }
}
