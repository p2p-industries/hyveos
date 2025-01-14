use futures::stream::BoxStream;
use hyveos_sdk::Connection;

use crate::{error::HyveCtlResult, out::CommandOutput};

pub trait CommandFamily {
    async fn run(self, _: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>>;
}

#[macro_export]
macro_rules! boxed_try_stream {
    ($($body:tt)*) => {
        ::futures::stream::StreamExt::boxed(
            ::async_stream::try_stream!{ $($body)* }
        )
    }
}
