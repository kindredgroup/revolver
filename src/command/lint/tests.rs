// $coverage:ignore-start

use std::borrow::Cow;
use std::convert::Infallible;
use crate::command::{Command, Description, Example, lint, Lint, NamedCommandParser, ParseCommandError};
use crate::terminal::Mock;

struct Parser {
    name: &'static str,
    description: Description
}

impl<T> NamedCommandParser<T> for Parser {
    type Context = ();
    type Error = Infallible;

    fn parse(&self, _: &str) -> Result<Box<dyn Command<T, Context = Self::Context, Error = Self::Error>>, ParseCommandError> {
        unimplemented!()
    }

    fn shorthand(&self) -> Option<Cow<'static, str>> {
        unimplemented!()
    }

    fn name(&self) -> Cow<'static, str> {
        self.name.into()
    }

    fn description(&self) -> Description {
        self.description.clone()
    }
}

fn assert_description_pedantic(command_name: &'static str, description: Description) {
    assert_description(command_name, description, &[])
}

fn assert_description(command_name: &'static str, description: Description, exclusions: &[Lint]) {
    lint::assert::<_, _, Mock>(&Parser {
        name: command_name,
        description
    }, exclusions);
}

#[test]
fn validate_description_empty_usage_passes() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump.".into(),
        usage: Cow::default(),
        examples: Vec::default()
    });
}

#[test]
fn validate_description_with_usage_passes() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump.".into(),
        usage: "<pump_id> <flow_rate>".into(),
        examples: Vec::default()
    });
}

#[test]
fn validate_description_with_usage_and_example_passes() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump.".into(),
        usage: "<pump_id> <flow_rate>".into(),
        examples: vec! [
            Example {
                scenario: "set flow rate of 50% for pump 2".into(),
                command: "2 0.5".into()
            }
        ]
    });
}

#[test]
fn validate_description_purpose_trailing_whitespace_with_exclusions_passes() {
    assert_description("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump. ".into(),
        usage: Cow::default(),
        examples: Vec::default()
    }, &[Lint::PurposeHasExcessWhitespace, Lint::PurposeDoesNotEndWithPeriod]);
}

#[test]
#[should_panic(expected = "failed lint: PurposeHasExcessWhitespace")]
fn validate_description_purpose_trailing_whitespace_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump. ".into(),
        usage: Cow::default(),
        examples: Vec::default()
    });
}

#[test]
#[should_panic(expected = "failed lint: PurposeIsEmpty")]
fn validate_description_empty_purpose_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "".into(),
        usage: Cow::default(),
        examples: Vec::default()
    });
}

#[test]
#[should_panic(expected = "failed lint: PurposeDoesNotBeginWithUppercase")]
fn validate_description_purpose_starts_with_lowercase_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "frobnicates.".into(),
        usage: Cow::default(),
        examples: Vec::default()
    });
}

#[test]
#[should_panic(expected = "failed lint: PurposeDoesNotEndWithPeriod")]
fn validate_description_purpose_does_not_end_with_period_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates".into(),
        usage: Cow::default(),
        examples: Vec::default()
    });
}

#[test]
#[should_panic(expected = "failed lint: UsageHasExcessWhitespace")]
fn validate_description_usage_trailing_whitespace_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump.".into(),
        usage: "foo ".into(),
        examples: Vec::default()
    });
}

#[test]
#[should_panic(expected = "failed lint: UsageBeginsWithCommandName")]
fn validate_description_usage_begins_with_command_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Frobnicates the gogomobile's auxiliary fuel pump.".into(),
        usage: "frobnicate foo".into(),
        examples: Vec::default()
    });
}

#[test]
#[should_panic(expected = "failed lint: ExampleScenarioHasExcessWhitespace")]
fn validate_description_example_scenario_trailing_whitespace_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Valid purpose.".into(),
        usage: Cow::default(),
        examples: vec! [
            Example {
                scenario: "set flow rate of 50% for pump 2 ".into(),
                command: "2 0.5".into()
            }
        ]
    });
}

#[test]
#[should_panic(expected = "failed lint: ExampleScenarioBeginsWithUppercase")]
fn validate_description_example_scenario_starts_with_capital_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Valid purpose.".into(),
        usage: Cow::default(),
        examples: vec! [
            Example {
                scenario: "Set flow rate of 50% for pump 2".into(),
                command: "2 0.5".into()
            }
        ]
    });
}

#[test]
#[should_panic(expected = "failed lint: ExampleScenarioEndsWithPeriod")]
fn validate_description_example_scenario_ends_with_period_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Valid purpose.".into(),
        usage: Cow::default(),
        examples: vec! [
            Example {
                scenario: "set flow rate of 50% for pump 2.".into(),
                command: "2 0.5".into()
            }
        ]
    });
}

#[test]
#[should_panic(expected = "failed lint: ExampleCommandHasExcessWhitespace")]
fn validate_description_example_command_trailing_whitespace_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Valid purpose.".into(),
        usage: Cow::default(),
        examples: vec! [
            Example {
                scenario: "set flow rate of 50% for pump 2".into(),
                command: "2 0.5 ".into()
            }
        ]
    });
}

#[test]
#[should_panic(expected = "failed lint: ExampleCommandIsEmpty")]
fn validate_description_example_empty_command_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Valid purpose.".into(),
        usage: Cow::default(),
        examples: vec! [
            Example {
                scenario: "set flow rate of 50% for pump 2".into(),
                command: "".into()
            }
        ]
    });
}

#[test]
#[should_panic(expected = "failed lint: ExampleCommandBeginsWithCommandName")]
fn validate_description_example_command_begins_with_command_fails() {
    assert_description_pedantic("frobnicate", Description {
        purpose: "Valid purpose.".into(),
        usage: Cow::default(),
        examples: vec! [
            Example {
                scenario: "set flow rate of 50% for pump 2".into(),
                command: "frobnicate 2 0.5".into()
            }
        ]
    });
}