//! A simple calculator REPL with just three commands:
//!
//! * `add` -- Adds a value to the register and prints its contents.
//! * `subtract` -- Subtracts a value from the register and prints its contents.
//! * `print` -- Prints the contents of the register, leaving it unchanged.
//!
//! The example also includes the `help` and `quit` built-in commands.

use revolver::command;
use revolver::command::Commander;
use revolver::command::NamedCommandParser;
use revolver::looper::Looper;
use revolver::terminal::{AccessTerminalError, Streaming, Terminal};
use std::convert::Infallible;

#[derive(Debug, Default)]
struct Register {
    value: f64,
}

impl Register {
    /// Prints the contents of the register.
    ///
    /// # Errors
    /// [`AccessTerminalError`] if a terminal I/O error occurs.
    fn print(&self, terminal: &mut impl Terminal) -> Result<(), AccessTerminalError> {
        terminal.print_line(&format!("{:?}", self))
    }
}

/// Creates a new [`Commander`] instance.
fn commander<T: Terminal>() -> Commander<Register, Infallible, T> {
    let parsers: Vec<Box<dyn NamedCommandParser<_, _, _>>> = vec![
        Box::new(add::Parser),
        Box::new(print::Parser),
        Box::new(subtract::Parser),
        Box::new(command::help::Parser),
        Box::new(command::quit::Parser),
    ];
    Commander::new(parsers)
}

fn main() {
    let mut terminal = Streaming::default();
    let commander = commander();
    let mut register = Register::default();
    let mut looper = Looper::new(&mut terminal, &commander, &mut register);
    looper.run().unwrap();
}

mod add {
    //! A command and parser pair for adding a value to the register.

    use crate::Register;
    use revolver::command::{
        ApplyCommandError, ApplyOutcome, Command, Description, Example, NamedCommandParser,
        ParseCommandError,
    };
    use revolver::looper::Looper;
    use revolver::terminal::Terminal;
    use std::borrow::Cow;
    use std::convert::Infallible;
    use std::str::FromStr;

    struct Add {
        value: f64,
    }

    impl<T: Terminal> Command<Register, Infallible, T> for Add {
        fn apply(
            &mut self,
            looper: &mut Looper<Register, Infallible, T>,
        ) -> Result<ApplyOutcome, ApplyCommandError<Infallible>> {
            let (terminal, _, register) = looper.split();
            register.value += self.value;
            register.print(terminal)?;
            Ok(ApplyOutcome::Applied)
        }
    }

    pub struct Parser;

    impl<T: Terminal> NamedCommandParser<Register, Infallible, T> for Parser {
        fn parse(
            &self,
            s: &str,
        ) -> Result<Box<dyn Command<Register, Infallible, T>>, ParseCommandError> {
            let value = f64::from_str(s).map_err(ParseCommandError::convert)?;
            Ok(Box::new(Add { value }))
        }

        fn shorthand(&self) -> Option<Cow<'static, str>> {
            Some("a".into())
        }

        fn name(&self) -> Cow<'static, str> {
            "add".into()
        }

        fn description(&self) -> Description {
            Description {
                purpose: "Adds a value to the register.".into(),
                usage: "<value>".into(),
                examples: vec![Example {
                    scenario: "adds 1.5 to the register".into(),
                    command: "1.5".into(),
                }],
            }
        }
    }
}

mod subtract {
    //! A command and parser pair for subtracting a value from the register.

    use crate::Register;
    use revolver::command::{
        ApplyCommandError, ApplyOutcome, Command, Description, Example, NamedCommandParser,
        ParseCommandError,
    };
    use revolver::looper::Looper;
    use revolver::terminal::Terminal;
    use std::borrow::Cow;
    use std::convert::Infallible;
    use std::str::FromStr;

    struct Subtract {
        value: f64,
    }

    impl<T: Terminal> Command<Register, Infallible, T> for Subtract {
        fn apply(
            &mut self,
            looper: &mut Looper<Register, Infallible, T>,
        ) -> Result<ApplyOutcome, ApplyCommandError<Infallible>> {
            let (terminal, _, register) = looper.split();
            register.value -= self.value;
            register.print(terminal)?;
            Ok(ApplyOutcome::Applied)
        }
    }

    pub struct Parser;

    impl<T: Terminal> NamedCommandParser<Register, Infallible, T> for Parser {
        fn parse(
            &self,
            s: &str,
        ) -> Result<Box<dyn Command<Register, Infallible, T>>, ParseCommandError> {
            let value = f64::from_str(s).map_err(ParseCommandError::convert)?;
            Ok(Box::new(Subtract { value }))
        }

        fn shorthand(&self) -> Option<Cow<'static, str>> {
            Some("s".into())
        }

        fn name(&self) -> Cow<'static, str> {
            "subtract".into()
        }

        fn description(&self) -> Description {
            Description {
                purpose: "Subtracts a value from the register.".into(),
                usage: "<value>".into(),
                examples: vec![Example {
                    scenario: "subtracts 1.5 from the register".into(),
                    command: "1.5".into(),
                }],
            }
        }
    }
}

mod print {
    //! A command and parser pair for printing the contents of the register.
    use crate::Register;
    use revolver::command::{
        ApplyCommandError, ApplyOutcome, Command, Description, NamedCommandParser,
        ParseCommandError,
    };
    use revolver::looper::Looper;
    use revolver::terminal::Terminal;
    use std::borrow::Cow;
    use std::convert::Infallible;

    struct Print;

    impl<T: Terminal> Command<Register, Infallible, T> for Print {
        fn apply(
            &mut self,
            looper: &mut Looper<Register, Infallible, T>,
        ) -> Result<ApplyOutcome, ApplyCommandError<Infallible>> {
            let (terminal, _, register) = looper.split();
            register.print(terminal)?;
            Ok(ApplyOutcome::Applied)
        }
    }

    pub struct Parser;

    impl<T: Terminal> NamedCommandParser<Register, Infallible, T> for Parser {
        fn parse(
            &self,
            s: &str,
        ) -> Result<Box<dyn Command<Register, Infallible, T>>, ParseCommandError> {
            self.parse_no_args(s, || Print)
        }

        fn shorthand(&self) -> Option<Cow<'static, str>> {
            Some("p".into())
        }

        fn name(&self) -> Cow<'static, str> {
            "print".into()
        }

        fn description(&self) -> Description {
            Description {
                purpose: "Prints the contents of the register.".into(),
                usage: "".into(),
                examples: Vec::default(),
            }
        }
    }
}