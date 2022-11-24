//! A command for terminating the REPL.

use std::borrow::Cow;
use crate::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use crate::looper::Looper;
use crate::terminal::Terminal;

/// The `quit` command. Once applied, will stop the [`RunFlag`](crate::looper::RunFlag). When control returns
/// to the [`Looper`], it will realise that the flag is in the [`RunFlag::Stopped`](crate::looper::RunFlag::Stopped)
/// state, and will immediately return from the loop.
pub struct Quit;

impl<C, E, T: Terminal> Command<C, E, T> for Quit {
    fn apply(&mut self, looper: &mut Looper<C, E, T>) -> Result<ApplyOutcome, ApplyCommandError<E>> {
        looper.run_flag().stop();
        looper.terminal().print_line("Exiting.")?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Quit`].
pub struct Parser;

impl<C, E, T: Terminal> NamedCommandParser<C, E, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, E, T>>, ParseCommandError> {
        self.parse_no_args(s, || Quit)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("q".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "quit".into()
    }
    
    fn description(&self) -> Description {
        Description {
            purpose: "Exits the program.".into(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[cfg(test)]
mod tests;
