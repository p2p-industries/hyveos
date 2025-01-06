use hyveos_sdk::Connection;

use futures::stream::BoxStream;
use crate::output::CommandOutput;

pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait CommandFamily {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>>;
}

#[macro_export]
macro_rules! single_output_stream {
    ($body:expr) => {{

        stream::once(async move {
            Ok($body)
        })
        .boxed()
    }};
}
#[macro_export]
macro_rules! boxed_try_stream {
    ($($body:tt)*) => {
        async_stream::try_stream!{ $($body)* }.boxed()
    }
}