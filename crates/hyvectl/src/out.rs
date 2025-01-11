use crate::color::Theme;
use indicatif::ProgressBar;
use serde::Serialize;
use std::io::{Result, Write};

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
        message: String,
    },
}

impl CommandOutput {
    pub fn message(message: &str) -> Self {
        Self::Message(message.into())
    }

    pub fn result() -> Self {
        Self::Result {
            fields: vec![],
            tty_template: String::default(),
            non_tty_template: String::default(),
        }
    }

    pub fn progress(progress: u64) -> Self {
        Self::Progress(progress)
    }

    pub fn spinner(message: &str, tick_strings: &[&str]) -> Self {
        Self::Spinner {
            tick_strings: tick_strings.iter().map(|s| (*s).to_string()).collect(),
            message: message.to_string(),
        }
    }

    pub fn with_field(mut self, key: &'static str, value: String) -> Self {
        if let Self::Result { fields, .. } = &mut self {
            fields.push((key, value));
        }
        self
    }

    pub fn with_tty_template(mut self, template: &'static str) -> Self {
        if let Self::Result { tty_template, .. } = &mut self {
            *tty_template = template.into();
        }
        self
    }

    pub fn with_non_tty_template(mut self, template: &'static str) -> Self {
        if let Self::Result {
            non_tty_template, ..
        } = &mut self
        {
            *non_tty_template = template.into();
        }
        self
    }

    fn to_json(&self, is_tty: bool) -> Result<String> {
        use serde_json::{json, Map, Value};

        match &self {
            Self::Result { fields, .. } => {
                let mut obj = Map::new();

                for (key, field) in fields {
                    let val: Value = json!(field);
                    obj.insert((*key).to_string(), val);
                }

                let s = if is_tty {
                    let mut buffer = Vec::new();

                    colored_json::write_colored_json(&Value::Object(obj), &mut buffer)?;

                    String::from_utf8_lossy(&buffer).to_string()
                } else {
                    serde_json::to_string_pretty(&Value::Object(obj))?
                };

                Ok(s)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Non-Result JSON not supported",
            )),
        }
    }

    fn safe_write_line(output_stream: &mut dyn Write, line: &str) -> Result<()> {
        match writeln!(output_stream, "{line}") {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn write_json(&self, output_stream: &mut dyn Write, is_tty: bool) -> Result<()> {
        let out = self.to_json(is_tty)?;

        Self::safe_write_line(output_stream, &out)
    }

    pub fn write(
        &self,
        output_stream: &mut dyn Write,
        theme: Option<&Theme>,
        is_tty: bool,
    ) -> Result<()> {
        match &self {
            Self::Message(message) => {
                let styled_msg = if let Some(t) = theme {
                    t.info_msg(message).to_string()
                } else {
                    message.to_string()
                };
                Self::safe_write_line(output_stream, &styled_msg)
            }
            Self::Result {
                fields,
                tty_template,
                non_tty_template,
            } => {
                let mut output = if is_tty {
                    tty_template.clone()
                } else {
                    non_tty_template.clone()
                };

                for (key, value) in fields {
                    let placeholder = format!("{{{key}}}");
                    let formatted_value = if let Some(t) = theme {
                        t.field(value).to_string()
                    } else {
                        value.clone()
                    };
                    output = output.replace(&placeholder, &formatted_value);
                }

                let final_line = if let Some(t) = theme {
                    t.result_msg(&output).to_string()
                } else {
                    output
                };
                Self::safe_write_line(output_stream, &final_line)?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn write_to_spinner(
        &self,
        spinner: &ProgressBar,
        theme: Option<&Theme>,
        is_tty: bool,
    ) -> Result<()> {
        let mut buffer = Vec::new();
        self.write(&mut buffer, theme, is_tty)?;

        let text = String::from_utf8_lossy(&buffer).to_string();

        for line in text.lines() {
            spinner.println(line);
        }
        Ok(())
    }
}
