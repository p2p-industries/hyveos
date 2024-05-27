use libp2p::kad::{
    AddProviderError, AddProviderOk, BootstrapError, BootstrapOk, GetProvidersError,
    GetProvidersOk, GetRecordError, GetRecordOk, PutRecordError, PutRecordOk, Quorum, Record,
    RecordKey,
};

use crate::{
    command::{SendMultipleResult, SendResult},
    impl_from_special_command,
};

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

impl_from_special_command!(Kad);
