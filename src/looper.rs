//! The mechanism for iteratively running commands based on successive user input. This module fulfils the
//! 'loop' part of a REPL application.

use std::fmt::Display;
use crate::command::{ApplyCommandError, ApplyOutcome, Commander, read_command};
use crate::terminal::{AccessTerminalError, Terminal};

/// Whether or not the looper is running. By setting the flag to [`RunFlag::Stopped`], a command
/// can signal the termination of the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunFlag {
    Running,
    Stopped
}

impl Default for RunFlag {
    fn default() -> Self {
        Self::Stopped
    }
}

impl RunFlag {
    /// Signals a start.
    pub fn start(&mut self) {
        *self = Self::Running;
    }

    /// Signals a stop.
    pub fn stop(&mut self) {
        *self = Self::Stopped;
    }

    /// Is the flag in a running state?
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Controls the main application loop. Encapsulates a [`Terminal`] device for interfacing with the user,
/// a [`Commander`] for parsing commands, a [`RunFlag`] that tracks the state of the application, and
/// a caller-specified context that represents the rest of the application state.
pub struct Looper<'a, C, E, T: Terminal> {
    terminal: &'a mut T,
    commander: &'a Commander<C, E, T>,
    run_flag: RunFlag,
    context: &'a mut C
}

impl<'a, C, E, T: Terminal> Looper<'a, C, E, T> {
    /// Creates a new [`Looper`].
    pub fn new(terminal: &'a mut T, commander: &'a Commander<C, E, T>, context: &'a mut C) -> Self {
        Self {
            terminal,
            commander,
            run_flag: RunFlag::default(),
            context
        }
    }

    /// A mutable reference to the underlying [`Terminal`] interface.
    pub fn terminal(&mut self) -> &mut T {
        self.terminal
    }

    /// A reference to the [`Commander`].
    pub fn commander(&self) -> &Commander<C, E, T> {
        self.commander
    }

    /// A mutable reference to the [`RunFlag`]. This is exposed so that any command can terminate
    /// the loop. (The [`Looper`] will return after the command executes.)
    pub fn run_flag(&mut self) -> &mut RunFlag {
        &mut self.run_flag
    }

    /// Split-borrow of the underlying components. Used when you need to reference two or more
    /// of these simultaneously, which wouldn't otherwise pass the borrow checker.
    pub fn split(&mut self) -> (&mut T, &Commander<C, E, T>, &mut C) {
        (self.terminal, self.commander, self.context)
    }

    /// A mutable reference to the application context.
    pub fn context(&mut self) -> &mut C {
        self.context
    }
}

/// The outcome of the last executed command. Used to present a slightly different prompt.
enum LastCommandOutcome {
    Applied,
    Skipped,
    Erred
}

impl LastCommandOutcome {
    fn prompt(&self) -> &'static str {
        match self {
            LastCommandOutcome::Applied => "+>> ",
            LastCommandOutcome::Skipped => "->> ",
            LastCommandOutcome::Erred => "!>> "
        }
    }
}

impl From<ApplyOutcome> for LastCommandOutcome {
    fn from(outcome: ApplyOutcome) -> Self {
        match outcome {
            ApplyOutcome::Applied => Self::Applied,
            ApplyOutcome::Skipped => Self::Skipped
        }
    }
}

impl<'a, C, E: Display, T: Terminal> Looper<'a, C, E, T> {
    /// Starts the loop, blocking until one of the commands internally terminates the loop.
    ///
    /// If any of the commands yields some other error, it will be printed to the user and the next
    /// command will be executed (as per the user's terminal input). Only an [`AccessTerminalError`] will
    /// percolate up the call stack.
    ///
    /// This method may be called repeatedly. Calling it after the looper has returned will
    /// start a new loop, resetting the [`RunFlag`] before running the first command. It is up
    /// to the caller to reset the application context.
    ///
    /// # Errors
    /// [`AccessTerminalError`] if the terminal device could not be accessed for reading or writing.
    pub fn run(&mut self) -> Result<(), AccessTerminalError> {
        self.run_flag.start();
        let mut last_command_outcome = LastCommandOutcome::Applied;
        while self.run_flag.is_running() {
            let mut command = read_command(self, last_command_outcome.prompt())?;
            let result = command.apply(self);
            match result {
                Ok(apply_outcome) => {
                    last_command_outcome = apply_outcome.into();
                },
                Err(ApplyCommandError::Application(err)) => {
                    self.terminal.print_line(&format!("Command error: {err}."))?;
                    last_command_outcome = LastCommandOutcome::Erred;
                },
                Err(ApplyCommandError::AccessTerminal(err)) => {
                    return Err(err)
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;