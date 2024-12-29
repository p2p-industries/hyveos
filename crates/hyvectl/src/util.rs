use hyveos_sdk::Connection;
use std::error::Error;
use futures::{stream, Stream, StreamExt, TryStreamExt};
use futures::stream::BoxStream;
use crate::output::CommandOutput;

pub trait CommandFamily {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>>;
}

pub async fn resolve_stream<S, T, E>(
    stream_future: Result<S, E>,
) -> BoxStream<'static, Result<T, Box<dyn Error>>>
where
    S: Stream<Item = Result<T, E>> + Send + 'static,
    E: Into<Box<dyn Error>> + Send + 'static,
{
    match stream_future {
        Ok(stream) => stream.map_err(|e| e.into()).boxed(),
        Err(e) => { stream::once(async move { Err(e.into()) }).boxed() }
    }
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