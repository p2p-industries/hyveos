use std::fmt;

use rustyline::ExternalPrinter;

#[derive(Debug)]
pub struct Printer<T> {
    string: String,
    printer: T,
}

impl<T> From<T> for Printer<T>
where
    T: ExternalPrinter,
{
    fn from(printer: T) -> Self {
        Self {
            string: String::new(),
            printer,
        }
    }
}

impl<T> fmt::Write for Printer<T>
where
    T: ExternalPrinter,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.string.push_str(s);

        if let Some(newline) = self.string.rfind('\n') {
            let mut string = self.string.split_off(newline + 1);
            std::mem::swap(&mut string, &mut self.string);
            self.printer.print(string).map_err(|_| fmt::Error)?;
        }

        Ok(())
    }
}
