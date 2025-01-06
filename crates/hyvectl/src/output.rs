use std::fmt;
use std::io::{Result, Write, ErrorKind};
use std::ptr::write;
use serde::Serialize;
use indicatif::ProgressBar;
use hyveos_core::debug::{MeshTopologyEvent, MessageDebugEvent, MessageDebugEventType};
use hyveos_core::discovery::NeighbourEvent;
use hyveos_core::file_transfer::Cid;
use hyveos_core::gossipsub::{Message, ReceivedMessage};
use hyveos_core::req_resp::{Request, Response};
use hyveos_core::scripting::RunningScript;
use hyveos_sdk::PeerId;
use hyveos_sdk::services::req_resp::InboundRequest;
use crate::color::Theme;

#[derive(Clone, Debug, Serialize)]
pub enum OutputField {
    String(String),
    PeerId(PeerId),
    ReceivedGossipMessage(ReceivedMessage),
    GossipMessage(Message),
    MeshTopologyEvent(MeshTopologyEvent),
    InboundRequest(InboundRequest<Vec<u8>>),
    Request(Request),
    Response(Response),
    RunningScripts(Vec<RunningScript>),
    Cid(Cid)
}

impl fmt::Display for OutputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputField::String(s) => write!(f, "{}", s),
            OutputField::PeerId(p) => write!(f, "{}", p),
            OutputField::ReceivedGossipMessage(m) => {
                write!(
                    f,
                    "{{ from: {}, message: {} }}",
                    m.source.ok_or(std::fmt::Error)?,
                    String::from_utf8(m.clone().message.data).map_err(|_| std::fmt::Error)?
                )
            }
            OutputField::MeshTopologyEvent(m) => match &m.event {
                NeighbourEvent::Init(peers) => {
                    write!(f, "Connected to {{")?;
                    for peer in peers {
                        write!(f, "{}", peer)?
                    }
                    Ok(write!(f, "}}")?)
                }
                NeighbourEvent::Discovered(peer) => write!(f, "Discovered {}", peer),
                NeighbourEvent::Lost(peer) => write!(f, "Lost {}", peer),
            },
            OutputField::GossipMessage(message) => {write!(f, "{{ topic: {}, message: {} }}", message.topic,
                                                           String::from_utf8(message.clone().data).map_err(|_| std::fmt::Error)?)},
            OutputField::InboundRequest(request) => {write!(f, "Inbound Request{:?}", request)},
            OutputField::Request(r) => write!(f, "{{ topic: {}, message: {} }}", r.clone().topic.unwrap_or_default(),  String::from_utf8(r.clone().data).map_err(|_| std::fmt::Error)?),
            OutputField::Response(r) => {
                match r {
                    Response::Data(data) => {
                        write!(f, "{{ message: {} }}", String::from_utf8(data.clone()).map_err(|_| std::fmt::Error)?)
                    }
                    Response::Error(err) => {
                        write!(f, "{{ error: {} }}", err)
                    }
                }
            },
            OutputField::RunningScripts(r) => {
                Ok(for script in r {
                    writeln!(f, "{}", script.id.to_string())?
                })
            },
            OutputField::Cid(r) => write!(f, "Cid: {:?}", r),
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
    Spinner {
        tick_strings: Vec<String>,
        message: String
    }
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

    pub fn spinner(message: &str, tick_strings: &[&str]) -> Self {
        Self {
            command: "",
            success: true,
            output: CommandOutputType::Spinner {
                tick_strings: tick_strings
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                message: message.to_string(),
            }
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

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| e.into())
    }

    fn safe_write_line(&self, output_stream: &mut dyn Write, line: &str) -> Result<()> {
        match writeln!(output_stream, "{}", line) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn write_json(&self,
                      output_stream: &mut dyn std::io::Write) -> Result<()> {
        let out = self.to_json()?;

        self.safe_write_line(output_stream, &out)
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
            CommandOutputType::Error(message) => {
                let line = format!("ERROR ({}): {}", self.command, message);
                let colored_line = if let Some(t) = theme {
                    t.error_msg(line).to_string()
                } else {
                    line
                };
                self.safe_write_line(output_stream, &colored_line)
            }
            _ => {
                Ok(())
            }
        }
    }

    pub fn write_to_spinner(
        &self,
        spinner: &ProgressBar,
        theme: &Option<Theme>,
    ) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        self.write(&mut buffer, theme)?;

        let text = String::from_utf8_lossy(&buffer).to_string();

        for line in text.lines() {
            spinner.println(line.to_string());
        }
        Ok(())
    }

}
