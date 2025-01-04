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
use crate::color::Theme;

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
        human_readable_template: String,
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
                human_readable_template: String::default(),
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
                *human_readable_template = template.into();
            }
            _ => {}
        }
        self
    }

    fn safe_write_line(&self, output_stream: &mut dyn Write, line: &str) -> Result<()> {
        writeln!(output_stream, "{}", line)
    }

        pub fn write(
            &self,
            output_stream: &mut dyn std::io::Write,
            theme: &Option<Theme>
        ) -> std::io::Result<()> {
            match &self.output {
                CommandOutputType::Message(message) => {
                    let styled_msg = if let Some(t) = theme {
                        t.info_msg(message.clone()).to_string()
                    } else {
                        message.to_string()
                    };
                    self.safe_write_line(output_stream, &styled_msg)
                },
                CommandOutputType::Result { fields, human_readable_template } => {
                    let mut output = human_readable_template.clone();
                    for (key, value) in fields {
                        let placeholder = format!("{{{}}}", key);
                        let formatted_value = match value {
                            OutputField::String(s) => {
                                if let Some(t) = theme.clone() {
                                    t.field(s.clone()).to_string()
                                } else {
                                    s.clone()
                                }
                            },
                            _ => value.to_string(),
                        };
                        output = output.replace(&placeholder, &formatted_value);
                    }

                    let final_line = if let Some(t) = theme {
                        t.result_msg(output).to_string()
                    } else {
                        output
                    };
                    self.safe_write_line(output_stream, &final_line)?;

                    Ok(())
                },
                CommandOutputType::Progress(_) => {
                    Ok(())
                },
                CommandOutputType::Error(message) => {
                    let line = format!("ERROR ({}): {}", self.command, message);
                    let colored_line = if let Some(t) = theme {
                        t.error_msg(line).to_string()
                    } else {
                        line
                    };
                    self.safe_write_line(output_stream, &colored_line)
                }
            }
        }
}
