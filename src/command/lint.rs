//! Linting/validation for [`NamedCommandParser`] types.

#[allow(clippy::enum_glob_use)]
use Lint::*;
use crate::command::{Description, Example, NamedCommandParser};

/// Lints that indicate problems during validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lint {
    PurposeHasExcessWhitespace,
    PurposeIsEmpty,
    PurposeDoesNotBeginWithUppercase,
    PurposeDoesNotEndWithPeriod,
    UsageHasExcessWhitespace,
    UsageBeginsWithCommandName,
    ExampleScenarioHasExcessWhitespace,
    ExampleScenarioIsEmpty,
    ExampleScenarioBeginsWithUppercase,
    ExampleScenarioEndsWithPeriod,
    ExampleCommandHasExcessWhitespace,
    ExampleCommandIsEmpty,
    ExampleCommandBeginsWithCommandName,
}

impl Lint {
    fn assert(self, condition: bool, failed: &mut Vec<Lint>) -> bool {
        if !condition {
            failed.push(self);
        }
        condition
    }
}

/// Asserts that there are no lints raised during validation.
///
/// # Panics
/// If any lint is raised. The panic message contains the description of the first failed lint
/// (possibly among many).
pub fn assert_pedantic<C, E, T>(parser: &impl NamedCommandParser<T, Context = C , Error = E>) {
    assert(parser, &[]);
}

/// Asserts that the validation of a parser does not raise a lint that is outside the
/// exclusions list.
///
/// # Panics
/// If a lint is raised that isn't covered by `exclusions`. The panic message contains the
/// description of the first failed lint (possibly among many).
pub fn assert<C, E, T>(parser: &impl NamedCommandParser<T, Context = C , Error = E>, exclusions: &[Lint]) {
    let failed = validate(parser);
    for failed_lint in failed {
        assert!(exclusions.contains(&failed_lint), "failed lint: {failed_lint:?}");
    }
}

/// Ensures that the parser is correctly specified, returning a vector of failed lints.
pub fn validate<C, E, T>(parser: &impl NamedCommandParser<T, Context = C , Error = E>) -> Vec<Lint> {
    let mut failed = vec![];
    validate_description(&parser.name(), &parser.description(), &mut failed);
    failed
}

/// Ensures that the description fields are correctly formulated.
fn validate_description(command_name: &str, desc: &Description, failed: &mut Vec<Lint>) {
    let (purpose, usage) = (&desc.purpose[..], &desc.usage[..]);

    no_excess_whitespace(purpose, PurposeHasExcessWhitespace, failed);
    if PurposeIsEmpty.assert(!purpose.is_empty(), failed) {
        PurposeDoesNotBeginWithUppercase.assert(purpose.chars().next().unwrap().is_uppercase(), failed);
        PurposeDoesNotEndWithPeriod.assert(purpose.chars().rev().next().unwrap() == '.', failed);
    }

    no_excess_whitespace(usage, UsageHasExcessWhitespace, failed);
    if !usage.is_empty() {
        UsageBeginsWithCommandName.assert(!usage.starts_with(command_name), failed);
    }

    for ex in &desc.examples {
        validate_example(command_name, ex, failed);
    }
}

/// Ensures that the example fields are correctly formulated.
fn validate_example(command_name: &str, ex: &Example, failed: &mut Vec<Lint>) {
    let (scenario, command) = (&ex.scenario[..], &ex.command[..]);

    no_excess_whitespace(scenario, ExampleScenarioHasExcessWhitespace, failed);
    if ExampleScenarioIsEmpty.assert(!scenario.is_empty(), failed) {
        ExampleScenarioBeginsWithUppercase.assert(!scenario.chars().next().unwrap().is_uppercase(), failed);
        ExampleScenarioEndsWithPeriod.assert(scenario.chars().rev().next().unwrap() != '.', failed);
    }

    no_excess_whitespace(command, ExampleCommandHasExcessWhitespace, failed);
    if ExampleCommandIsEmpty.assert(!command.is_empty(), failed) {
        ExampleCommandBeginsWithCommandName.assert(!command.starts_with(command_name), failed);
    }
}

fn no_excess_whitespace(s: &str, lint: Lint, failed: &mut Vec<Lint>) {
    lint.assert(s.trim() == s, failed);
}

#[cfg(test)]
mod tests;
