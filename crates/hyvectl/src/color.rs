use colored::{Color, ColoredString, Colorize};

pub trait Coloring {
    fn apply(&self, input: String) -> ColoredString;
}

#[derive(Clone, Debug)]
pub struct Style {
    color: Color,
    bold: bool,
    underline: bool,
}

impl Coloring for Style {
    fn apply(&self, input: String) -> ColoredString {
        let mut styled = input.color(self.color);
        if self.bold {
            styled = styled.bold();
        }
        if self.underline {
            styled = styled.underline();
        }

        styled
    }
}

impl Style {
    pub fn new(color: Color, bold: bool, underline: bool) -> Self {
        Self { color, bold, underline }
    }
}