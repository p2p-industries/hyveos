use std::io::{Result, Write};
use serde::Serialize;
use indicatif::ProgressBar;
use crate::color::Theme;


#[derive(Clone, Debug, Serialize)]
pub enum CommandOutput {
    Message(String),
    Result {
        fields: Vec<(&'static str, String)>,
        #[serde(skip_serializing)]
        tty_template: String,
        #[serde(skip_serializing)]
        non_tty_template: String,
    },
    Progress(u64),
    Spinner {
        tick_strings: Vec<String>,
        message: String
    }
}

impl CommandOutput {

    pub fn message(message: &str) -> Self {
        Self::Message {
            0: message.into()
        }
    }

    pub fn result() -> Self {
        Self::Result {
            fields: vec![],
            tty_template: String::default(),
            non_tty_template: String::default(),
        }
    }

    pub fn progress(progress: u64) -> Self {
        Self::Progress {
            0: progress
        }
    }

    pub fn spinner(message: &str, tick_strings: &[&str]) -> Self {
        Self::Spinner {
            tick_strings: tick_strings
                .iter()
                .map(|s| s.to_string())
                .collect(),
            message: message.to_string(),
        }
    }

    pub fn with_field(mut self, key: &'static str, value: String) -> Self {
        match &mut self {
            Self::Result { fields, .. } => {fields.push((key, value));},
            _ => {}
        }
        self
    }

    pub fn with_tty_template(mut self, template: &'static str) -> Self {
        match &mut self {
            Self::Result { tty_template, .. } => {
                *tty_template = template.into();
            }
            _ => {}
        }
        self
    }

    pub fn with_non_tty_template(mut self, template: &'static str) -> Self {
        match &mut self {
            Self::Result { non_tty_template, .. } => {
                *non_tty_template = template.into();
            }
            _ => {}
        }
        self
    }

    fn to_json(&self) -> Result<String> {
        use serde_json::{Value, Map, json};

        match &self {
            Self::Result { fields, .. } => {
                let mut obj = Map::new();

                for (key, field) in fields {
                    let val: Value = json!(field);
                    obj.insert(key.to_string(), val);
                }

                let s = serde_json::to_string_pretty(&Value::Object(obj))?;
                Ok(s)
            },
            _ => {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Non-Result JSON not supported"))
            }
        }
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
                      output_stream: &mut dyn Write) -> Result<()> {
        let out = self.to_json()?;

        self.safe_write_line(output_stream, &out)
    }

    pub fn write(
        &self,
        output_stream: &mut dyn Write,
        theme: &Option<Theme>,
        is_tty: bool,
    ) -> Result<()> {
        match &self {
            Self::Message(message) => {
                let styled_msg = if let Some(t) = theme {
                    t.info_msg(message.clone()).to_string()
                } else {
                    message.to_string()
                };
                self.safe_write_line(output_stream, &styled_msg)
            },
            Self::Result { fields, tty_template, non_tty_template } => {
                let mut output = if is_tty {
                    tty_template.clone()
                } else {
                    non_tty_template.clone()
                };

                for (key, value) in fields {
                    let placeholder = format!("{{{}}}", key);
                    let formatted_value =  if let Some(t) = theme.clone() {
                        t.field(value.clone()).to_string()
                    } else {
                        value.clone()
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
            _ => {
                Ok(())
            }
        }
    }

    pub fn write_to_spinner(
        &self,
        spinner: &ProgressBar,
        theme: &Option<Theme>,
        is_tty: bool,
    ) -> Result<()> {
        let mut buffer = Vec::new();
        self.write(&mut buffer, theme, is_tty)?;

        let text = String::from_utf8_lossy(&buffer).to_string();

        for line in text.lines() {
            spinner.println(line.to_string());
        }
        Ok(())
    }
}
