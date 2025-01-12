use colored::{Color, ColoredString, Colorize};

#[derive(Clone, Debug)]
pub struct Theme {
    pub info: Color,
    pub result: Color,
    pub field: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            info: Color::BrightBlack,
            result: Color::BrightBlack,
            field: Color::BrightBlue,
        }
    }
}

impl Theme {
    pub fn info_msg(&self, msg: &str) -> ColoredString {
        msg.color(self.info).bold()
    }

    pub fn result_msg(&self, msg: &str) -> ColoredString {
        msg.color(self.result).bold()
    }

    pub fn field(&self, value: &str) -> ColoredString {
        value.color(self.field).bold()
    }
}
