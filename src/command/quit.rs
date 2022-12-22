//! A command for terminating the REPL.

use std::borrow::Cow;
use std::marker::PhantomData;
use crate::command::{ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser, ParseCommandError};
use crate::looper::Looper;
use crate::terminal::Terminal;

/// The `quit` command. Once applied, will stop the [`RunFlag`](crate::looper::RunFlag). When control returns
/// to the [`Looper`], it will realise that the flag is in the [`RunFlag::Stopped`](crate::looper::RunFlag::Stopped)
/// state, and will immediately return from the loop.
pub struct Quit<C, E> {
    __phantom_data: PhantomData<(C, E)>
}

impl<C, E> Default for Quit<C, E> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default(),
        }
    }
}

impl<C, E, T: Terminal> Command<T> for Quit<C, E> {
    type Context = C;
    type Error = E;

    fn apply(&mut self, looper: &mut Looper<C, E, T>) -> Result<ApplyOutcome, ApplyCommandError<E>> {
        looper.run_flag().stop();
        looper.terminal().print_line("Exiting.")?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Quit`].
pub struct Parser<C, E> {
    __phantom_data: PhantomData<(C, E)>
}

impl<C, E> Default for Parser<C, E> {
    fn default() -> Self {
        Self {
            __phantom_data: PhantomData::default(),
        }
    }
}

impl<C: 'static, E: 'static, T: Terminal> NamedCommandParser<T> for Parser<C, E> {
    type Context = C;
    type Error = E;

    fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = C , Error = E>>, ParseCommandError> {
        self.parse_no_args(s, Quit::default)
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
