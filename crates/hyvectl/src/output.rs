use std::fmt;
use std::io::{Result, Write, ErrorKind};
use serde::Serialize;
use hyveos_core::debug::{MeshTopologyEvent, MessageDebugEvent, MessageDebugEventType};
use hyveos_core::discovery::NeighbourEvent;
use hyveos_core::file_transfer::Cid;
use hyveos_core::gossipsub::ReceivedMessage;
use hyveos_core::req_resp::Response;
use hyveos_core::scripting::RunningScript;
use hyveos_sdk::services::req_resp::InboundRequest;

#[derive(Clone, Debug, Serialize)]
pub enum OutputField {
    String(String),
    GossipMessage(ReceivedMessage),
    MeshTopologyEvent(MeshTopologyEvent),
    ServiceDebugEvent(MessageDebugEvent),
    Request(InboundRequest<Vec<u8>>),
    Response(Response),
    RunningScripts(Vec<RunningScript>),
    Cid(Cid)
}

impl fmt::Display for OutputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputField::String(s) => write!(f, "{}", s),
            OutputField::GossipMessage(m) => {
                write!(
                    f,
                    "{{ propagation_source: {}, source: {:?}, message_id: {}, message: {:?} }}",
                    m.propagation_source, m.source, m.message_id, m.message
                )
            }
            OutputField::MeshTopologyEvent(m) => match &m.event {
                NeighbourEvent::Init(peers) => write!(f, "Init with peers: {:?}", peers),
                NeighbourEvent::Discovered(peer) => write!(f, "Discovered peer: {:?}", peer),
                NeighbourEvent::Lost(peers) => write!(f, "Lost peers: {:?}", peers),
            },
            OutputField::ServiceDebugEvent(m) => write!(f, "ServiceDebugEvent: {:?}", m),
            OutputField::Request(r) => write!(f, "InboundRequest: {:?}", r),
            OutputField::Response(r) => write!(f, "InboundResponse: {:?}", r),
            OutputField::RunningScripts(r) => todo!(),
            OutputField::Cid(r) => todo!()
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum CommandOutputType {
    Message(String),
    Result {
        fields: Vec<(&'static str, OutputField)>,
        #[serde(skip_serializing)]
        human_readable_template: Option<String>,
    },
    Progress(u64),
    Error(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandOutput {
    pub command: &'static str,
    pub success: bool,
    pub output: CommandOutputType
}

impl CommandOutput {

    pub fn message(command: &'static str, message: &str) -> Self {
        Self {
            command,
            success: true,
            output: CommandOutputType::Message(message.into())
        }
    }

    pub fn result(command: &'static str) -> Self {
        Self {
            command,
            success: true,
            output: CommandOutputType::Result {
                fields: vec![],
                human_readable_template: None,
            }
        }
    }

    pub fn progress(command: &'static str, progress: u64) -> Self {
        Self {
            command,
            success: true,
            output: CommandOutputType::Progress(progress)
        }
    }

    pub fn error(command: &'static str, message: &str) -> Self {
        Self {
            command,
            success: false,
            output: CommandOutputType::Error(message.into())
        }
    }


    pub fn with_field(mut self, key: &'static str, value: OutputField) -> Self {
        match &mut self.output {
            CommandOutputType::Result { fields, .. } => {fields.push((key, value));},
            _ => {}
        }
        self
    }

    pub fn with_human_readable_template(mut self, template: &'static str) -> Self {
        match &mut self.output {
            CommandOutputType::Result { human_readable_template, .. } => {
                *human_readable_template = Some(template.into());
            }
            _ => {}
        }
        self
    }

    fn safe_write_line(&self, output_stream: &mut dyn Write, line: &str) -> Result<()> {
        writeln!(output_stream, "{}", line)
    }

    pub fn write(&self, output_stream: &mut dyn Write) -> Result<()> {
        match &self.output {
            CommandOutputType::Message(message) => {
                self.safe_write_line(output_stream, message)
            }
            CommandOutputType::Result { fields, human_readable_template } => {
                if let Some(template) = human_readable_template {
                    let mut output = template.clone();
                    for (key, value) in fields.clone() {
                        let placeholder = format!("{{{}}}", key);
                        output = output.replace(&placeholder, &value.to_string());
                    }
                    Ok(self.safe_write_line(output_stream, &output)?)
                } else {
                    self.safe_write_line(output_stream, &format!("{} Result:", self.command))?;
                    self.safe_write_line(output_stream, &format!("  Success: {}", self.success))?;
                    Ok(for (key, value) in fields {
                        self.safe_write_line(output_stream, &format!("  {}: {}", key, value.to_string()))?;
                    })
                }
            },
            CommandOutputType::Progress(_) => {Ok(())}
            CommandOutputType::Error(message) => {
                let line = format!("ERROR ({}): {}", self.command, message);
                self.safe_write_line(output_stream, &line)
            }
        }
    }
}
