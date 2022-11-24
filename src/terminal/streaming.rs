//! Terminal device that composes over I/O streams using [`Input`] and [`Output`]
//! traits. Out-of-the-box adapters exist for `stdin` and `stdout` streams. Adapters may be written
//! to interface with nonstandard streams by supplying a custom closure.

use crate::terminal::{AccessTerminalError, Terminal};
use std::io;
use std::io::{stdin, stdout, Write};

/// Terminal implementation over stream-like input/output abstractions.
pub struct Streaming<I: Input, O: Output> {
    pub input: I,
    pub output: O,
}

impl<'a> Default for Streaming<InputAdapter<'a>, OutputAdapter<'a>> {
    fn default() -> Self {
        Self {
            input: InputAdapter::default(),
            output: OutputAdapter::default(),
        }
    }
}

impl<I: Input, O: Output> Terminal for Streaming<I, O> {
    fn print(&mut self, s: &str) -> Result<(), AccessTerminalError> {
        self.output.print(s)
    }

    fn read_line(&mut self) -> Result<String, AccessTerminalError> {
        self.input.read_line()
    }
}

/// Piecewise abstraction over an input device.
pub trait Input {
    /// Reads a complete line from the underlying stream, blocking until the input becomes
    /// available for consumption.
    ///
    /// # Errors
    /// If the stream could not be accessed for reading.
    fn read_line(&mut self) -> Result<String, AccessTerminalError>;
}

/// Signature of a closure that implements the input side of the terminal device.
pub type InputReader<'a> = Box<dyn FnMut() -> Result<String, AccessTerminalError> + 'a>;

/// Adapts an [`InputReader`] closure to the [`Input`] trait. The default adapter implementation
/// delegates to `stdin`.
pub struct InputAdapter<'a>(pub InputReader<'a>);

impl<'a> InputAdapter<'a> {
    /// Creates an [`InputAdapter`] over the given closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() -> Result<String, AccessTerminalError> + 'a,
    {
        Self(Box::new(f))
    }
}

impl Default for InputAdapter<'_> {
    fn default() -> Self {
        Self(Box::new(|| {
            let mut buf = String::default();
            stdin().read_line(&mut buf)?;
            Ok(buf)
        })) // $coverage:ignore -- the closing brace eludes coverage
    }
}

impl Input for InputAdapter<'_> {
    fn read_line(&mut self) -> Result<String, AccessTerminalError> {
        self.0()
    }
}

/// Piecewise abstraction over an output device.
pub trait Output {
    /// Prints a string slice to the output stream.
    ///
    /// # Errors
    /// If the stream could not be accessed for writing.
    fn print(&mut self, s: &str) -> Result<(), AccessTerminalError>;
}

/// Signature of a closure that implements the output side of the terminal device.
pub type OutputWriter<'a> = Box<dyn FnMut(&str) -> Result<(), AccessTerminalError> + 'a>;

/// Adapts an [`OutputWriter`] closure to the [`Output`] trait. The default adapter implementation
/// delegates to `stdout`.
pub struct OutputAdapter<'a>(pub OutputWriter<'a>);

impl<'a> OutputAdapter<'a> {
    /// Creates an [`OutputAdapter`] over the given closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&str) -> Result<(), AccessTerminalError> + 'a,
    {
        Self(Box::new(f))
    }
}

impl Default for OutputAdapter<'_> {
    fn default() -> Self {
        Self(Box::new(|str| {
            print!("{str}");
            stdout().flush()?;
            Ok(())
        }))
    }
}

impl Output for OutputAdapter<'_> {
    fn print(&mut self, s: &str) -> Result<(), AccessTerminalError> {
        self.0(s)
    }
}

impl From<io::Error> for AccessTerminalError {
    fn from(err: io::Error) -> Self {
        Self(err.to_string())
    }
}

#[cfg(test)]
mod tests;
