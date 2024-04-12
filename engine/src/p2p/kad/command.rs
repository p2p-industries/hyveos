use libp2p::kad::{
    AddProviderError, AddProviderOk, BootstrapError, BootstrapOk, GetProvidersError,
    GetProvidersOk, GetRecordError, GetRecordOk, PutRecordError, PutRecordOk, Quorum, Record,
    RecordKey,
};

use crate::p2p::command::{SendMultipleResult, SendResult};

#[non_exhaustive]
pub enum Command {
    Bootstrap {
        sender: SendMultipleResult<BootstrapOk, BootstrapError>,
    },
    PutRecord {
        record: Record,
        quorum: Quorum,
        sender: SendResult<PutRecordOk, PutRecordError>,
    },
    GetRecord {
        key: RecordKey,
        sender: SendMultipleResult<GetRecordOk, GetRecordError>,
    },
    GetProviders {
        key: RecordKey,
        sender: SendMultipleResult<GetProvidersOk, GetProvidersError>,
    },
    StartProviding {
        key: RecordKey,
        sender: SendResult<AddProviderOk, AddProviderError>,
    },
}

impl From<Command> for crate::p2p::command::Command {
    fn from(value: Command) -> Self {
        crate::p2p::command::Command::Kad(value)
    }
}
