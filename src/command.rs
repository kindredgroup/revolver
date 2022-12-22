//! Specification of an executable command and a parser for building command instances from user input.
//! This module fulfils the 'execute' part of a REPL application.

pub mod help;
mod lint;
pub mod quit;

pub use lint::*;

use crate::looper::Looper;
use crate::terminal::{AccessTerminalError, Terminal};
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use thiserror::Error;

/// Produced when a command could not executed.
#[derive(Debug, Clone, Error)]
pub enum ApplyCommandError<E> {
    #[error("application: {0}")]
    Application(E),

    #[error("access terminal: {0}")]
    AccessTerminal(#[from] AccessTerminalError),
}

/// Conversions for error variants.
impl<E> ApplyCommandError<E> {
    /// Converts the error variant into an [`Option`] containing the underlying application error.
    pub fn application(self) -> Option<E> {
        match self {
            ApplyCommandError::Application(err) => Some(err),
            ApplyCommandError::AccessTerminal(_) => None,
        }
    }

    /// Converts the error variant into an [`Option<AccessTerminalError>`].
    pub fn access_terminal(self) -> Option<AccessTerminalError> {
        match self {
            ApplyCommandError::Application(_) => None,
            ApplyCommandError::AccessTerminal(err) => Some(err),
        }
    }
}

/// The definition of an executable command.
///
/// `T` is the terminal type.
pub trait Command<T: Terminal> {
    /// The application context type. (The part of the application this is not the REPL library.)
    type Context;

    /// The type of error that can be produced by the execution of the command. It is shuttled inside
    type Error;

    /// Applies the command for the given [`Looper`]. References to the underlying application
    /// context and the terminal interface are supplied by the [`Looper`].
    ///
    /// # Errors
    /// [`ApplyCommandError`] if the command could not be executed.
    fn apply(&mut self, looper: &mut Looper<Self::Context, Self::Error, T>)
        -> Result<ApplyOutcome, ApplyCommandError<Self::Error>>;
}

/// The outcome of applying a [`Command`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The command was successfully executed, with all of its side-effects (if any) applied
    /// to the application state.
    Applied,

    /// The execution of the command was aborted without an accompanying error. (For example, it may have
    /// been aborted at the user's request.)
    Skipped,
}

/// A parser for constructing [`Command`] implementations from a text string (a line read from the
/// terminal interface).
pub trait NamedCommandParser<T> {
    /// The application context type. (The part of the application this is not the REPL library.)
    type Context;

    /// The type of error that can be produced by the execution of the command. It is shuttled inside
    type Error;

    /// Parses the given string slice, returning [`Command`] object.
    ///
    /// # Errors
    /// [`ParseCommandError`] if the command couldn't be parsed.
    #[allow(clippy::type_complexity)]
    fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = Self::Context, Error = Self::Error>>, ParseCommandError>;

    /// Optional shorthand moniker for the command. The user may type in this string instead of the
    /// full command name.
    fn shorthand(&self) -> Option<Cow<'static, str>>;

    /// The (mandatory) complete name of the command. The user will type in the name of the command,
    /// followed by some (depending on the command) arguments.
    fn name(&self) -> Cow<'static, str>;

    /// Describes the command. The description is displayed when invoking the `help` command.
    fn description(&self) -> Description;

    /// A convenience method for creating a [`Command`] object by invoking the given `ctor` closure,
    /// assuming that this command does not require any arguments.
    ///
    /// # Errors
    /// [`ParseCommandError`] if the command couldn't be parsed.
    #[allow(clippy::type_complexity)]
    fn parse_no_args<M>(
        &self,
        s: &str,
        ctor: impl FnOnce() -> M,
    ) -> Result<Box<dyn Command<T, Context = Self::Context, Error = Self::Error>>, ParseCommandError>
    where
        M: Command<T, Context = Self::Context, Error = Self::Error> + 'static,
        T: Terminal,
        Self: Sized,
    {
        if s.is_empty() {
            Ok(Box::new(ctor()))
        } else {
            Err(ParseCommandError(
                format!("invalid arguments to '{}': '{s}'", self.name()).into(),
            ))
        }
    }
}

/// A comprehensive description of a command. May include examples.
#[derive(Debug, Clone)]
pub struct Description {
    /// Why the command exists. One (or more) fully punctuated sentences.
    pub purpose: Cow<'static, str>,

    /// Syntax for arguments, if any. Leave blank if the command does not accept arguments.
    /// (Do not include the name of the command.)
    pub usage: Cow<'static, str>,

    /// Zero or more examples. Should be empty if the command does not take arguments, in which
    /// case the example is implied.
    pub examples: Vec<Example>,
}

/// An example of using a command.
#[derive(Debug, Clone)]
pub struct Example {
    /// What the example fulfils. Part-sentence (starts with a lowercase letter, no trailing period).
    pub scenario: Cow<'static, str>,

    /// Sample arguments.
    /// (Do not include the name of the command.)
    pub command: Cow<'static, str>,
}

impl Example {
    /// Verifies that the example's command is parsable.
    ///
    /// # Errors
    /// If parsing fails.
    fn assert_parsable<C, E, T, P: NamedCommandParser<T, Context = C, Error = E> + ?Sized>(
        &self,
        parser: &P,
    ) -> Result<(), ParseCommandError> {
        parser.parse(&self.command)?;
        Ok(())
    }
}

/// Decodes user input (typically a line read from a terminal interface) into a dynamic [`Command`] object, using
/// a preconfigured map of parsers.
pub struct Commander<C, E, T> {
    parsers: Vec<Box<dyn NamedCommandParser<T, Context = C, Error = E>>>,
    by_shorthand: BTreeMap<String, usize>,
    by_name: BTreeMap<String, usize>,
}

impl<C, E, T> Commander<C, E, T> {
    /// Creates a new [`Commander`] from the given vector of parsers.
    ///
    /// # Panics
    /// If there was an error building a [`Commander`] from the given parsers.
    pub fn new(parsers: Vec<Box<dyn NamedCommandParser<T, Context = C, Error = E>>>) -> Self {
        parsers.try_into().unwrap()
    }

    /// An iterator over the underlying parsers.
    pub fn parsers(&self) -> impl Iterator<Item = &Box<dyn NamedCommandParser<T, Context = C, Error = E>>> {
        self.by_name.values().map(|&idx| &self.parsers[idx])
    }
}

/// Raised by [`Commander`] if there was something wrong with the parsers given to it. Perhaps
/// the parsers were incorrectly specified or conflicted amongst themselves.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct InvalidCommandParserSpec(String);

/// Raised by either [`Commander`] or a [`NamedCommandParser`] if the supplied string slice could
/// not be parsed into a valid [`Command`] object.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct ParseCommandError(pub Cow<'static, str>);

impl ParseCommandError {
    /// Converts anything representable as a [`String`] into a [`ParseCommandError`], consuming
    /// the original. This is mostly used in error conversion; e.g., in [`Result::map_err()`].
    #[allow(clippy::needless_pass_by_value)]
    pub fn convert<E: ToString>(err: E) -> Self {
        Self(err.to_string().into())
    }
}

impl<C, E, T> TryFrom<Vec<Box<dyn NamedCommandParser<T, Context = C , Error = E>>>> for Commander<C, E, T> {
    type Error = InvalidCommandParserSpec;

    fn try_from(parsers: Vec<Box<dyn NamedCommandParser<T, Context = C , Error = E>>>) -> Result<Self, Self::Error> {
        // helper function for inserting a parser reference into some tree map and returning an error if a duplicate
        // mapping is detected
        fn insert<N: Ord + Display>(
            key: N,
            value: usize,
            map: &mut BTreeMap<N, usize>,
        ) -> Result<(), InvalidCommandParserSpec> {
            match map.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                    Ok(())
                }
                Entry::Occupied(entry) => duplicate_error(entry.key()),
            }
        }

        // helper function for checking if a given entry exists in a map and returning an error if found
        fn check<N: Ord + Display>(
            key: &N,
            map: &BTreeMap<N, usize>,
        ) -> Result<(), InvalidCommandParserSpec> {
            if map.contains_key(key) {
                duplicate_error(key)
            } else {
                Ok(())
            }
        }

        // helper function for generating a 'duplicate command parser' error for a given key
        fn duplicate_error<N: Display>(key: &N) -> Result<(), InvalidCommandParserSpec> {
            Err(InvalidCommandParserSpec(format!(
                "duplicate command parser for '{key}'"
            )))
        }

        let mut by_shorthand = BTreeMap::default();
        let mut by_name = BTreeMap::default();

        for (index, parser) in parsers.iter().enumerate() {
            // check that all example commands are parsable
            for example in &parser.description().examples {
                example.assert_parsable(&**parser).map_err(|err| {
                    InvalidCommandParserSpec(format!(
                        "unparsable example command '{}': {err}",
                        example.command
                    ))
                })?;
            }

            if let Some(shorthand) = parser.shorthand() {
                let shorthand = shorthand.into_owned();
                check(&shorthand, &by_name)?;
                insert(shorthand, index, &mut by_shorthand)?;
            }

            if parser.name().len() < 2 {
                return Err(InvalidCommandParserSpec(format!(
                    "invalid command name '{}': must contain at least 2 characters",
                    parser.name()
                )));
            }

            let name = parser.name().into_owned();
            check(&name, &by_shorthand)?;
            insert(name, index, &mut by_name)?;
        }

        Ok(Self {
            parsers,
            by_shorthand,
            by_name,
        })
    }
}

impl<C, E, T> Commander<C, E, T> {
    /// Parses the given string slice into a [`Command`] object.
    ///
    /// The input should be in the form `<command_identifier> [<command_args>]` where
    /// `<command_identifier>` ∈ {`<command_name>`, `<command_shorthand>}`.
    ///
    /// # Errors
    /// [`ParseCommandError`] if a [`Command`] object could not be constructed.
    pub fn parse(&self, s: &str) -> Result<Box<dyn Command<T, Context = C , Error = E>>, ParseCommandError> {
        if s.is_empty() {
            return Err(ParseCommandError("empty command string".into()));
        }

        let index = s.find(' ').unwrap_or(s.len());
        let name = &s[..index];

        let &parser_idx = self
            .by_shorthand
            .get(name)
            .or_else(|| self.by_name.get(name))
            .ok_or_else(|| ParseCommandError(format!("no command parser for '{name}'").into()))?;

        let command_frag = if index == s.len() {
            ""
        } else {
            &s[index + 1..]
        };
        self.parsers[parser_idx].parse(command_frag)
    }
}

pub(crate) fn read_command<C, E, T: Terminal>(
    looper: &mut Looper<C, E, T>,
    prompt: &str,
) -> Result<Box<dyn Command<T, Context = C , Error = E>>, AccessTerminalError> {
    let (terminal, commander, _) = looper.split();
    terminal.read_value(prompt, |str| commander.parse(str))
}

#[cfg(test)]
mod tests;
