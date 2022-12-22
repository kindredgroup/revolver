// $coverage:ignore-start

use std::convert::Infallible;
use crate::command::{NamedCommandParser, ParseCommandError};
use crate::terminal::Mock;

#[test]
fn parse_error() {
    assert_eq!(
        ParseCommandError("invalid arguments to 'quit': 'foo'".into()),
        NamedCommandParser::<Mock>::parse(&super::Parser::<(), Infallible>::default(), "foo").err().unwrap()
    );
}