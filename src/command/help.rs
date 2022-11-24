//! A self-help guide, outlining the available commands and how to use them.

use crate::command::{
    ApplyCommandError, ApplyOutcome, Command, Commander, Description, NamedCommandParser,
    ParseCommandError,
};
use crate::looper::Looper;
use crate::terminal::{AccessTerminalError, Terminal};
use stanza::renderer::console::{Console, Decor};
use stanza::renderer::Renderer;
use stanza::style::{Bold, Header, MaxWidth, MinWidth, Palette16, Styles, TextFg};
use stanza::table::{Cell, Col, Row, Table};
use std::borrow::{Borrow, Cow};

/// The `help` command. The list of available commands is obtained by interrogating the [`Commander`]. The output
/// of the help command is a rendered [Stanza](https://github.com/obsidiandynamics/stanza) table, enumerating
/// each of the available commands, their name (incl. shorthand, if set) and description (incl. any examples).
pub struct Help;

impl<C, E, T: Terminal> Command<C, E, T> for Help {
    fn apply(
        &mut self,
        looper: &mut Looper<C, E, T>,
    ) -> Result<ApplyOutcome, ApplyCommandError<E>> {
        let (terminal, commander, _) = looper.split();
        print_commands(commander, terminal)?;
        Ok(ApplyOutcome::Applied)
    }
}

/// Parser for [`Help`].
pub struct Parser;

impl<C, E, T: Terminal> NamedCommandParser<C, E, T> for Parser {
    fn parse(&self, s: &str) -> Result<Box<dyn Command<C, E, T>>, ParseCommandError> {
        self.parse_no_args(s, || Help)
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        Some("h".into())
    }

    fn name(&self) -> Cow<'static, str> {
        "help".into()
    }

    fn description(&self) -> Description {
        Description {
            purpose: "Displays a list of commands, their usage syntax and examples.".into(),
            usage: Cow::default(),
            examples: Vec::default(),
        }
    }
}

fn commands<C, E, T: Terminal>(commander: &Commander<C, E, T>) -> Table {
    let mut table = Table::default()
        .with_cols(vec![
            Col::new(Styles::default().with(MinWidth(15))),
            Col::new(Styles::default().with(MinWidth(65)).with(MaxWidth(120))),
        ])
        .with_row(Row::new(
            Styles::default()
                .with(Header(true))
                .with(Bold(true))
                .with(TextFg(Palette16::Yellow)),
            vec!["Command".into(), "Description".into()],
        ));

    for parser in commander.parsers() {
        let mut command = String::new();
        if let Some(shorthand) = parser.shorthand() {
            command.push_str(shorthand.borrow());
            command.push_str(", ");
        }
        command.push_str(&parser.name());

        let description = parser.description();
        let mut desc_buf = String::new();
        desc_buf.push_str(&format!("{}\n", description.purpose));
        desc_buf.push_str(&format!("usage: {} {}\n", parser.name(), description.usage));
        for example in &description.examples {
            desc_buf.push_str(&format!("example - {}:\n", example.scenario));
            desc_buf.push_str(&format!("    {} {}\n", parser.name(), example.command));
        }

        table.push_row(Row::new(
            Styles::default(),
            vec![
                Cell::new(
                    Styles::default().with(TextFg(Palette16::BrightGreen)),
                    command.into(),
                ),
                Cell::new(Styles::default().with(Bold(true)), desc_buf.into()),
            ],
        ));
    }

    table
}

fn print_commands<C, E, T: Terminal>(
    commander: &Commander<C, E, T>,
    terminal: &mut T,
) -> Result<(), AccessTerminalError> {
    let renderer = Console(
        Decor::default()
            .suppress_all_lines()
            .suppress_outer_border(),
    );
    terminal.print_line(&renderer.render(&commands(commander)))
}

#[cfg(test)]
mod tests;
