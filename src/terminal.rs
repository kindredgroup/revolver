//! An abstract, text-based interface with the user. This module fulfils the 'read' and
//! 'print' part of a REPL application.

mod mock;
mod streaming;

pub use mock::*;
pub use streaming::*;

use std::fmt::{Display};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct AccessTerminalError(pub String);

/// Specification of a text-based I/O device for interfacing with the user. Ordinarily, this is a
/// terminal utilising `stdin` and `stdout` devices; however, the separation of a concrete terminal
/// device from its specification allows for fine-grained mocking/testing of user interactions.
pub trait Terminal {
    /// Prints a string slice to the output device.
    ///
    /// # Errors
    /// If the terminal device could not be accessed for writing.
    fn print(&mut self, s: &str) -> Result<(), AccessTerminalError>;

    /// Prints a string slice with an added trailing newline separator.
    ///
    /// # Errors
    /// If the terminal device could not be accessed for writing.
    fn print_line(&mut self, s: &str) -> Result<(), AccessTerminalError> {
        let mut buf = s.to_owned();
        buf.push('\n');
        self.print(&buf)
    }

    /// Reads a complete line from the input device, blocking until the input becomes
    /// available for consumption.
    ///
    /// # Errors
    /// If the terminal device could not be accessed for reading.
    fn read_line(&mut self) -> Result<String, AccessTerminalError>;

    /// A variation of [`Self::read_from_str`] that operates on any type `V` that also implements the
    /// [`Default`] trait. The default value is returned when an empty (comprising only whitespace
    /// characters) input line is submitted by the user.
    ///
    /// # Errors
    /// If the terminal device could not be accessed for reading or writing.
    fn read_from_str_default<V: FromStr + Default>(&mut self, prompt: &str,
    ) -> Result<V, AccessTerminalError>
        where
            <V as FromStr>::Err: Display
    {
        self.read_value(prompt, |str| {
            if str.is_empty() {
                Ok(V::default())
            } else {
                str.parse()
            }
        })
    }

    /// Reads a value from a terminal device for any type `V` that implements the `FromStr<V>` trait.
    /// The user is prompted repeatedly until the parser yields a non-error value.
    /// (The parser error is never propagated to the caller.)
    ///
    /// # Errors
    /// If the terminal device could not be accessed for reading or writing.
    fn read_from_str<V: FromStr>(&mut self, prompt: &str,
    ) -> Result<V, AccessTerminalError>
        where
            <V as FromStr>::Err: Display
    {
        self.read_value(prompt, FromStr::from_str)
    }

    /// Reads a value from a terminal device using the supplied `parser` function. The user is prompted
    /// repeatedly until the parser yields a non-error value. (The parser error is never propagated to
    /// the caller.)
    ///
    /// # Errors
    /// If the terminal device could not be accessed for reading or writing.
    fn read_value<V, E: Display>(
        &mut self,
        prompt: &str,
        parser: impl Fn(&str) -> Result<V, E>,
    ) -> Result<V, AccessTerminalError> {
        loop {
            self.print(prompt)?;
            let read = self.read_line()?;
            let parsed = parser(read.trim());
            match parsed {
                Ok(val) => return Ok(val),
                Err(err) => {
                    self.print_line(&format!("Invalid input: {err}."))?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
