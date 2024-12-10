use std::io::{Result, Write, ErrorKind};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum OutputField {
    String(String),
    Int(u64),
    List(Vec<OutputField>),
}

impl OutputField {
    pub fn to_string(&self) -> String {
        match self {
            OutputField::String(s) => s.clone(),
            OutputField::Int(i) => i.to_string(),
            OutputField::List(list) => {
                let items: Vec<String> = list.iter().map(|item| item.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandOutput {
    pub command: &'static str,
    pub success: bool,
    pub fields: Vec<(&'static str, OutputField)>,
    #[serde(skip_serializing)]
    pub human_readable_template: Option<String>,
}

impl CommandOutput {
    pub fn new(command: &'static str) -> Self {
        Self {
            command,
            success: false,
            fields: Vec::new(),
            human_readable_template: None,
        }
    }

    pub fn add_field(mut self, key: &'static str, value: OutputField) -> Self {
        self.fields.push((key, value));
        self
    }

    pub fn with_human_readable_template(mut self, template: &'static str) -> Self {
        self.human_readable_template = Some(template.to_string());
        self
    }

    pub fn with_success(mut self) -> Self {
        self.success = true;
        self
    }

    fn safe_write_line(&self, output_stream: &mut dyn Write, line: &str) -> Result<()> {
        if let Err(e) = writeln!(output_stream, "{}", line) {
            if e.kind() == ErrorKind::BrokenPipe {
                return Ok(());
            } else {
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn write_impl(&self, output_stream: &mut dyn Write) -> Result<()> {
        if let Some(template) = &self.human_readable_template {
            let mut output = template.clone();
            for (key, value) in &self.fields {
                let placeholder = format!("{{{}}}", key);
                output = output.replace(&placeholder, &value.to_string());
            }
            self.safe_write_line(output_stream, &output)?;
        } else {
            self.safe_write_line(output_stream, &format!("{} Result:", self.command))?;
            self.safe_write_line(output_stream, &format!("  Success: {}", self.success))?;
            for (key, value) in &self.fields {
                self.safe_write_line(output_stream, &format!("  {}: {}", key, value.to_string()))?;
            }
        }

        Ok(())
    }

    pub fn write(&self, output_stream: &mut dyn Write) -> Result<()> {
        self.write_impl(output_stream)?;
        Ok(())
    }
}
