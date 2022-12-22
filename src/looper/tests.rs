// $coverage:ignore-start

use crate::command::{
    quit, ApplyCommandError, ApplyOutcome, Command, Commander, Description, NamedCommandParser,
    ParseCommandError,
};
use crate::looper::{Looper, RunFlag};
use crate::terminal::Invocation::ReadLine;
use crate::terminal::{lines, AccessTerminalError, Invocation, Mock, Terminal};
use std::borrow::Cow;
use std::str::FromStr;
use thiserror::Error;
use Invocation::Print;

#[derive(Default, Debug, PartialEq, Eq)]
struct TestContext {
    state: usize
}

#[derive(Debug, Clone, Error)]
#[error("{0}")]
struct TestError(String);

#[derive(Debug)]
struct Echo {
    num: usize,
}

impl<T: Terminal> Command<T> for Echo {
    type Context = TestContext;
    type Error = TestError;

    fn apply(
        &mut self,
        looper: &mut Looper<Self::Context, Self::Error, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<TestError>> {
        looper
            .terminal
            .print_line(&format!("the number is {}", self.num))?;
        Ok(ApplyOutcome::Applied)
    }
}

struct EchoParser;

impl<T: Terminal> NamedCommandParser<T> for EchoParser {
    type Context = TestContext;
    type Error = TestError;

    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<T, Context = Self::Context, Error = Self::Error>>, ParseCommandError> {
        let num = usize::from_str(s).map_err(ParseCommandError::convert)?;
        Ok(Box::new(Echo { num }))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("e".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "echo".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: Cow::default(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[derive(Debug)]
struct Respond {
    val: Result<ApplyOutcome, ApplyCommandError<TestError>>,
}

impl<T: Terminal> Command<T> for Respond {
    type Context = TestContext;
    type Error = TestError;

    fn apply(
        &mut self,
        _: &mut Looper<Self::Context, Self::Error, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<Self::Error>> {
        self.val.clone()
    }
}

struct RespondParser {
    val: Result<ApplyOutcome, ApplyCommandError<TestError>>,
}

impl<T: Terminal> NamedCommandParser<T> for RespondParser {
    type Context = TestContext;
    type Error = TestError;

    fn parse(
        &self,
        s: &str,
    ) -> Result<Box<dyn Command<T, Context = Self::Context, Error = Self::Error>>, ParseCommandError> {
        assert!(s.is_empty());
        let val = self.val.clone();
        Ok(Box::new(Respond { val }))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("r".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "respond".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: Cow::default(),
            usage: Cow::default(),
            examples: Vec::default()
        }
    }
}

#[test]
fn get_context() {
    let mut term = Mock::default();
    let commander = Commander::new(vec![Box::new(EchoParser)]);
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    assert_eq!(&TestContext::default(), looper.context());
    looper.context.state = 1;
}

#[test]
fn echo_applied() {
    let mut term = Mock::default().on_read_line(lines(&["echo 1", "echo 2", "quit"]));
    let commander = Commander::new(vec![Box::new(EchoParser), Box::new(quit::Parser::default())]);
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    looper.run().unwrap();

    assert_eq!(
        &[
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("echo 1".into())),
            Print("the number is 1\n".into(), Ok(())),
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("echo 2".into())),
            Print("the number is 2\n".into(), Ok(())),
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("quit".into())),
            Print("Exiting.\n".into(), Ok(())),
        ],
        term.invocations()
    );
}

#[test]
fn echo_with_parse_error() {
    let mut term = Mock::default().on_read_line(lines(&["echo x", "echo 2", "quit"]));
    let commander = Commander::new(vec![Box::new(EchoParser), Box::new(quit::Parser::default())]);
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    looper.run().unwrap();

    assert_eq!(
        &[
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("echo x".into())),
            Print(
                "Invalid input: invalid digit found in string.\n".into(),
                Ok(())
            ),
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("echo 2".into())),
            Print("the number is 2\n".into(), Ok(())),
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("quit".into())),
            Print("Exiting.\n".into(), Ok(())),
        ],
        term.invocations()
    );
}

#[test]
fn respond_skip() {
    let mut term = Mock::default().on_read_line(lines(&["respond", "quit"]));
    let commander = Commander::new(vec![
        Box::new(RespondParser {
            val: Ok(ApplyOutcome::Skipped),
        }),
        Box::new(quit::Parser::default()),
    ]);
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    looper.run().unwrap();

    assert_eq!(
        &[
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("respond".into())),
            Print("->> ".into(), Ok(())),
            ReadLine(Ok("quit".into())),
            Print("Exiting.\n".into(), Ok(())),
        ],
        term.invocations()
    );
}

#[test]
fn respond_application_error() {
    let mut term = Mock::default().on_read_line(lines(&["respond", "quit"]));
    let commander = Commander::new(vec![
        Box::new(RespondParser {
            val: Err(ApplyCommandError::Application(TestError(
                "cooling pump exploded".into(),
            ))),
        }),
        Box::new(quit::Parser::default()),
    ]);
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    looper.run().unwrap();

    assert_eq!(
        &[
            Print("+>> ".into(), Ok(())),
            ReadLine(Ok("respond".into())),
            Print("Command error: cooling pump exploded.\n".into(), Ok(())),
            Print("!>> ".into(), Ok(())),
            ReadLine(Ok("quit".into())),
            Print("Exiting.\n".into(), Ok(())),
        ],
        term.invocations()
    );
}

#[test]
fn respond_terminal_error() {
    let mut term = Mock::default().on_read_line(lines(&["respond", "quit"]));
    let commander = Commander::new(vec![Box::new(RespondParser {
        val: Err(ApplyCommandError::AccessTerminal(AccessTerminalError(
            "terminal meltdown".into(),
        ))),
    })]);
    let mut context = TestContext::default();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    assert_eq!(
        AccessTerminalError("terminal meltdown".into()),
        looper.run().unwrap_err()
    );

    assert_eq!(
        &[Print("+>> ".into(), Ok(())), ReadLine(Ok("respond".into()))],
        term.invocations()
    );
}

#[test]
fn run_flag_implements_debug() {
    let flag = RunFlag::Running;
    assert_eq!("Running", format!("{flag:?}"));
}
