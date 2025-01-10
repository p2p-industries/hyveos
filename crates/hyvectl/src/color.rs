use colored::{Color, ColoredString, Colorize};

#[derive(Clone, Debug)]
pub struct Theme {
    pub info_color: Color,
    pub result_color: Color,
    pub field_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            info_color: Color::BrightBlack,
            result_color: Color::BrightBlack,
            field_color: Color::BrightBlue,
        }
    }
}

impl Theme {
    pub fn info_msg(&self, msg: String) -> ColoredString {
        msg.color(self.info_color).bold()
    }

    pub fn result_msg(&self, msg: String) -> ColoredString {
        msg.color(self.result_color).bold()
    }

    pub fn field(&self, value: String) -> ColoredString {
        value.color(self.field_color).bold()
    }
}
