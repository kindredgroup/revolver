// $coverage:ignore-start

use std::borrow::Cow;
use super::*;
use crate::looper::{Looper};
use std::convert::Infallible;
use std::str::FromStr;
use crate::terminal::{AccessTerminalError, Terminal};

struct TestContext;

#[derive(Debug)]
struct SampleCommand;

struct TestTerminal;

impl Terminal for TestTerminal {
    fn print(&mut self, _: &str) -> Result<(), AccessTerminalError> {
        unimplemented!()
    }

    fn read_line(&mut self) -> Result<String, AccessTerminalError> {
        unimplemented!()
    }
}

impl<T: Terminal> Command<TestContext, Infallible, T> for SampleCommand {
    fn apply(&mut self, _: &mut Looper<TestContext, Infallible, T>) -> Result<ApplyOutcome, ApplyCommandError<Infallible>> {
        Ok(ApplyOutcome::Applied)
    }
}

impl FromStr for SampleCommand {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_empty() {
            return Err(ParseCommandError(
                format!("invalid arguments to 'sample': '{s}'").into(),
            ));
        }
        Ok(Self)
    }
}

struct Parser;

impl<T: Terminal> NamedCommandParser<TestContext, Infallible, T> for Parser {
    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<TestContext, Infallible, T>>, ParseCommandError> {
        SampleCommand::from_str(s)
            .map(|cmd| Box::new(cmd) as Box<dyn Command<TestContext, Infallible, T>>)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("s".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "sample".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "some purpose".into(),
            usage: "some usage".into(),
            examples: Vec::default()
        }
    }
}

#[test]
fn invalid_command_parser_spec_implements_display() {
    assert_eq!("foo", InvalidCommandParserSpec("foo".into()).to_string());
}

#[test]
fn parse_command_error_implements_display() {
    assert_eq!("foo", ParseCommandError("foo".into()).to_string());
}

#[test]
fn description_implements_debug() {
    let parser = Parser;
    let description = <Parser as NamedCommandParser<_, _, TestTerminal>>::description(&parser);
    let s = format!("{:?}", description);
    assert!(s.contains("Description"));
}

#[test]
fn commander() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> = vec![Box::new(Parser)];
    let commander = Commander::new(parsers);
    assert_eq!(1, commander.parsers().count());
    assert_eq!(None, commander.parse("s").err());
    assert_eq!(None, commander.parse("sample").err());
    assert_eq!(
        Some(ParseCommandError("empty command string".into())),
        commander.parse("").err()
    );
    assert_eq!(
        Some(ParseCommandError("no command parser for ''".into())),
        commander.parse(" ").err()
    );
    assert_eq!(
        Some(ParseCommandError("no command parser for 'z'".into())),
        commander.parse("z").err()
    );
    assert_eq!(
        Some(ParseCommandError("no command parser for 'zzz'".into())),
        commander.parse("zzz").err()
    );
    assert_eq!(
        Some(ParseCommandError("no command parser for 'zzz'".into())),
        commander.parse("zzz ").err()
    );
    assert_eq!(None, commander.parse("s ").err());
    assert_eq!(
        Some(ParseCommandError(
            "invalid arguments to 'sample': ' '".into()
        )),
        commander.parse("s  ").err()
    );
    assert_eq!(
        Some(ParseCommandError(
            "invalid arguments to 'sample': 'z'".into()
        )),
        commander.parse("s z").err()
    );
}

struct TestCommandParser {
    short: Option<Cow<'static, str>>,
    long: Cow<'static, str>,
    example_command: Cow<'static, str>
}

impl<T: Terminal> NamedCommandParser<TestContext, Infallible, T> for TestCommandParser {
    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<TestContext, Infallible, T>>, ParseCommandError> {
        usize::from_str(s).map_err(ParseCommandError::convert)?;
        Ok(Box::new(SampleCommand))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        self.short.clone()
    }

    fn name(&self) -> Cow<'static, str> {
        self.long.clone()
    }

    fn description(&self) -> Description {
        Description {
            purpose: Cow::default(),
            usage: Cow::default(),
            examples: vec![
                Example {
                    scenario: "sample scenario".into(),
                    command: self.example_command.clone()
                }
            ]
        }
    }
}

#[test]
fn commander_duplicate_short() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> = vec![
        Box::new(TestCommandParser {
            short: Some("g".into()),
            long: "g1".into(),
            example_command: "42".into()
        }),
        Box::new(TestCommandParser {
            short: Some("g".into()),
            long: "g2".into(),
            example_command: "42".into()
        }),
    ];
    assert_eq!(
        Some(InvalidCommandParserSpec("duplicate command parser for 'g'".into())),
        Commander::try_from(parsers).err()
    );
}

#[test]
fn commander_duplicate_long() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> = vec![
        Box::new(TestCommandParser {
            short: Some("g".into()),
            long: "gg".into(),
            example_command: "42".into()
        }),
        Box::new(TestCommandParser {
            short: Some("h".into()),
            long: "gg".into(),
            example_command: "42".into()
        }),
    ];
    assert_eq!(
        Some(InvalidCommandParserSpec(
            "duplicate command parser for 'gg'".into()
        )),
        Commander::try_from(parsers).err()
    );
}

#[test]
fn commander_duplicate_short_in_long() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> = vec![
        Box::new(TestCommandParser {
            short: Some("gg".into()),
            long: "hh".into(),
            example_command: "42".into()
        }),
        Box::new(TestCommandParser {
            short: Some("hh".into()),
            long: "ii".into(),
            example_command: "42".into()
        }),
    ];
    assert_eq!(
        Some(InvalidCommandParserSpec(
            "duplicate command parser for 'hh'".into()
        )),
        Commander::try_from(parsers).err()
    );
}

#[test]
fn commander_duplicate_long_in_short() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> = vec![
        Box::new(TestCommandParser {
            short: Some("gg".into()),
            long: "hh".into(),
            example_command: "42".into()
        }),
        Box::new(TestCommandParser {
            short: Some("ii".into()),
            long: "gg".into(),
            example_command: "42".into()
        }),
    ];
    assert_eq!(
        Some(InvalidCommandParserSpec(
            "duplicate command parser for 'gg'".into()
        )),
        Commander::try_from(parsers).err()
    );
}

#[test]
fn commander_long_name_too_short() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> =
        vec![Box::new(TestCommandParser {
            short: Some("g".into()),
            long: "h".into(),
            example_command: "42".into()
        })];
    assert_eq!(
        Some(InvalidCommandParserSpec(
            "invalid command name 'h': must contain at least 2 characters".into()
        )),
        Commander::try_from(parsers).err()
    );
}

#[test]
fn commander_unparsable_example() {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, TestTerminal>>> =
        vec![Box::new(TestCommandParser {
            short: Some("g".into()),
            long: "ggg".into(),
            example_command: "foo".into()
        })];
    assert_eq!(
        Some(InvalidCommandParserSpec(
            "unparsable example command 'foo': invalid digit found in string".into()
        )),
        Commander::try_from(parsers).err()
    );
}

fn application_error() -> ApplyCommandError<&'static str> {
    ApplyCommandError::Application("data")
}

fn access_terminal_error() -> ApplyCommandError::<Infallible> {
    ApplyCommandError::AccessTerminal(AccessTerminalError("data".into()))
}

#[test]
fn apply_command_error_implements_display() {
    assert_eq!("application: data", application_error().to_string());
    assert_eq!("access terminal: data", access_terminal_error().to_string());
}

#[test]
fn apply_command_error_variants() {
    assert_eq!(Some("data"), application_error().application());
    assert_eq!(None, application_error().access_terminal());

    assert_eq!(Some(AccessTerminalError("data".into())), access_terminal_error().access_terminal());
    assert_eq!(None, access_terminal_error().application());
}
