// $coverage:ignore-start

use crate::command::{
    quit, ApplyCommandError, ApplyOutcome, Command, Commander, Description, Example,
    NamedCommandParser, ParseCommandError,
};
use crate::looper::Looper;
use crate::terminal::{lines, Mock, Terminal};
use std::borrow::Cow;
use std::convert::Infallible;
use stanza::renderer::console::{Console, Decor};
use stanza::renderer::Renderer;
use crate::command::help::commands;

#[derive(Debug)]
struct SampleCommand;

impl<T: Terminal> Command<T> for SampleCommand {
    type Context = ();
    type Error = Infallible;

    fn apply(
        &mut self,
        _: &mut Looper<(), Infallible, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<Infallible>> {
        unimplemented!()
    }
}

struct SampleParser;

impl<T: Terminal> NamedCommandParser<T> for SampleParser {
    type Context = ();
    type Error = Infallible;

    fn parse(&self, _: &str) -> Result<Box<dyn Command<T, Context = Self::Context, Error = Self::Error>>, ParseCommandError> {
        Ok(Box::new(SampleCommand))
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("z".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "sample".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "A sample command.".into(),
            usage: "<alpha> <beta>".into(),
            examples: vec![Example {
                scenario: "do something great".into(),
                command: "foo bar".into(),
            }],
        }
    }
}

#[test]
fn invoke() {
    let mut term = Mock::default().on_read_line(lines(&["help", "quit"]));
    let commander = Commander::<_, Infallible, _>::new(vec![
        Box::new(super::Parser::default()),
        Box::new(quit::Parser::default()),
        Box::new(SampleParser),
    ]);
    let mut context = ();
    let mut looper = Looper::new(&mut term, &commander, &mut context);
    looper.run().unwrap();

    let (output, _) = term.invocations()[2].print().unwrap();
    assert!(output.contains("h, help"));
    assert!(output.contains("z, sample"));
    assert!(output.contains("q, quit"));
    assert!(output.contains("A sample command."));
    assert!(output.contains("usage: sample <alpha> <beta>"));
    assert!(output.contains("example - do something great:"));
    assert!(output.contains("sample foo bar"));
}

#[test]
fn commands_content() {
    let commander = Commander::<_, _, Mock>::new(vec![
        Box::new(super::Parser::default()),
        Box::new(quit::Parser::default()),
        Box::new(SampleParser),
    ]);

    let renderer = Console(
        Decor::default()
            .suppress_escape_codes()
            .suppress_inner_horizontal_border(),
    );

    let s = renderer.render(&commands(&commander)).to_string();
    assert_eq!("\
    ╔═══════════════╤═════════════════════════════════════════════════════════════════╗\n\
    ║Command        │Description                                                      ║\n\
    ║h, help        │Displays a list of commands, their usage syntax and examples.    ║\n\
    ║               │usage: help                                                      ║\n\
    ║q, quit        │Exits the program.                                               ║\n\
    ║               │usage: quit                                                      ║\n\
    ║z, sample      │A sample command.                                                ║\n\
    ║               │usage: sample <alpha> <beta>                                     ║\n\
    ║               │example - do something great:                                    ║\n\
    ║               │    sample foo bar                                               ║\n\
    ╚═══════════════╧═════════════════════════════════════════════════════════════════╝", s);
}

#[test]
fn parse_error() {
    assert_eq!(
        ParseCommandError("invalid arguments to 'help': 'foo'".into()),
        NamedCommandParser::<Mock>::parse(&super::Parser::<(), Infallible>::default(), "foo").err().unwrap()
    );
}
