use hyveos_sdk::Connection;
use std::error::Error;
use futures::stream::BoxStream;
use crate::output::CommandOutput;

pub trait CommandFamily {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>>;
}
