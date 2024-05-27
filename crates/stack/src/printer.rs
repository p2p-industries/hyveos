use std::{
    fmt,
    future::Future as _,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use rustyline::ExternalPrinter;
use tokio::{
    io::AsyncWrite,
    sync::{Mutex, MutexGuard},
};
use tracing_subscriber::fmt::MakeWriter;

pub struct Printer {
    string: String,
    printer: Box<dyn ExternalPrinter + Send>,
}

impl<T: 'static + ExternalPrinter + Send> From<T> for Printer {
    fn from(printer: T) -> Self {
        Self {
            string: String::new(),
            printer: Box::new(printer),
        }
    }
}

impl fmt::Write for Printer {
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

impl io::Write for Printer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let string =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fmt::Write::write_str(self, string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.printer
            .print(std::mem::take(&mut self.string))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }
}

pub struct MutexGuardWriter<'a>(MutexGuard<'a, Printer>);

impl<'a> io::Write for MutexGuardWriter<'a> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub struct SharedPrinter {
    printer: Arc<Mutex<Printer>>,
}

impl<T: 'static + ExternalPrinter + Send> From<T> for SharedPrinter {
    fn from(printer: T) -> Self {
        Self {
            printer: Arc::new(Mutex::new(printer.into())),
        }
    }
}

impl fmt::Write for SharedPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        tokio::task::block_in_place(|| self.printer.blocking_lock().write_str(s))
    }
}

impl io::Write for SharedPrinter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        tokio::task::block_in_place(|| self.printer.blocking_lock().write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        tokio::task::block_in_place(|| self.printer.blocking_lock().flush())
    }
}

impl AsyncWrite for SharedPrinter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let lock_future = self.printer.lock();
        tokio::pin!(lock_future);
        let mut printer = futures::ready!(lock_future.poll(cx));

        Poll::Ready(io::Write::write(&mut *printer, buf))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let lock_future = self.printer.lock();
        tokio::pin!(lock_future);
        let mut printer = futures::ready!(lock_future.poll(cx));

        Poll::Ready(io::Write::flush(&mut *printer))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let lock_future = self.printer.lock();
        tokio::pin!(lock_future);
        let mut printer = futures::ready!(lock_future.poll(cx));

        Poll::Ready(io::Write::flush(&mut *printer))
    }
}

impl<'a> MakeWriter<'a> for SharedPrinter {
    type Writer = MutexGuardWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        MutexGuardWriter(tokio::task::block_in_place(|| self.printer.blocking_lock()))
    }
}

impl Clone for SharedPrinter {
    fn clone(&self) -> Self {
        Self {
            printer: self.printer.clone(),
        }
    }
}
