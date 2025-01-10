use hyveos_sdk::Connection;

use futures::stream::BoxStream;
use crate::error::HyveCtlResult;
use crate::out::CommandOutput;

pub trait CommandFamily {
    async fn run(self, connection: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>>;
}

#[macro_export]
macro_rules! boxed_try_stream {
    ($($body:tt)*) => {
        async_stream::try_stream!{ $($body)* }.boxed()
    }
}