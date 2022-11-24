//! Mocking of a terminal device.

use crate::terminal::{Terminal, AccessTerminalError, streaming};

/// A single invocation of one of the mock's methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    ReadLine(Result<String, String>),
    Print(String, Result<(), String>),
}

impl Invocation {
    /// Returns a [`Some`] with a reference to the arguments if this is a [`Invocation::ReadLine`] variant, or [`None`] otherwise.
    pub fn read_line(&self) -> Option<&Result<String, String>> {
        match self {
            Invocation::ReadLine(v) => Some(v),
            Invocation::Print(_, _) => None
        }
    }

    /// Returns a [`Some`] with a reference to the arguments if this is a [`Invocation::Print`] variant, or [`None`] otherwise.
    pub fn print(&self) -> Option<(&str, &Result<(), String>)> {
        match self {
            Invocation::ReadLine(_) => None,
            Invocation::Print(out, res) => Some((out, res))
        }
    }
}

/// Convenience trait for converting an [`Option<&Result<String, String>>`] from the
/// [`Invocation::ReadLine`] variant to the input string slice.
pub trait ReadLineInput {
    /// The input string slice.
    fn unwrap_input(&self) -> &str;
}

impl ReadLineInput for Option<&Result<String, String>> {
    fn unwrap_input(&self) -> &str {
        self.unwrap().as_ref().unwrap()
    }
}

/// Convenience trait for converting an [`Option<(&str, &Result<(), String>)>`] from the
/// [`Invocation::Print`] variant to the output string slice.
pub trait PrintOutput {
    fn unwrap_output(&self) -> &str;
}

impl PrintOutput for Option<(&str, &Result<(), String>)> {
    fn unwrap_output(&self) -> &str {
        self.unwrap().0
    }
}

/// Mock [`Terminal`] implementation, containing delegates for the [`Terminal::read_line`] and
/// [`Terminal::print`] operations, as well as an invocation tracker.
pub struct Mock<'d> {
    on_read_line: streaming::InputReader<'d>,
    on_print: streaming::OutputWriter<'d>,
    invocations: Vec<Invocation>,
}

impl<'d> Default for Mock<'d> {
    fn default() -> Self {
        Self {
            on_read_line: Box::new(|| Ok(String::default())),
            on_print: Box::new(|_| Ok(())),
            invocations: vec![],
        }
    }
}

impl<'d> Mock<'d> {
    /// Specifies a delegate closure to be invoked on the [`Terminal::read_line`] call.
    #[must_use]
    pub fn on_read_line(
        mut self,
        delegate: impl FnMut() -> Result<String, AccessTerminalError> + 'd,
    ) -> Self {
        self.on_read_line = Box::new(delegate);
        self
    }

    /// Specifies a delegate closure to be invoked on the [`Terminal::print`] call.
    #[must_use]
    pub fn on_print(mut self, delegate: impl FnMut(&str) -> Result<(), AccessTerminalError> + 'd) -> Self {
        self.on_print = Box::new(delegate);
        self
    }

    /// Lists the invocations that have been recorded against this mock.
    pub fn invocations(&self) -> &[Invocation] {
        &self.invocations[..]
    }
}

impl<'d> Terminal for Mock<'d> {
    fn print(&mut self, s: &str) -> Result<(), AccessTerminalError> {
        let result = (*self.on_print)(s);
        self.invocations.push(Invocation::Print(
            s.into(),
            result
                .as_ref()
                .map(Clone::clone)
                .map_err(ToString::to_string),
        ));
        result
    }

    fn read_line(&mut self) -> Result<String, AccessTerminalError> {
        let result = (*self.on_read_line)();
        self.invocations.push(Invocation::ReadLine(
            result
                .as_ref()
                .map(Clone::clone)
                .map_err(ToString::to_string),
        ));
        result
    }
}

/// Generates a `read_line` closure that returns one item at a time from a pre-canned slice of lines. If the closure
/// is invoked after the slice is exhausted, it will return an [`AccessTerminalError`].
pub fn lines<S: ToString + 'static>(lines: &[S]) -> impl FnMut() -> Result<String, AccessTerminalError> + '_ {
    let mut lines = lines;
    move || {
        if lines.is_empty() {
            return Err(AccessTerminalError("no more lines".into()))
        }
        let s = &lines[0];
        lines = &lines[1..];
        Ok(s.to_string())
    }
}

#[cfg(test)]
mod tests;
