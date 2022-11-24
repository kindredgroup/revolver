// $coverage:ignore-start

use crate::terminal::{AccessTerminalError, Invocation, Mock, mock, Terminal};

#[test]
fn read_from_str_valid() {
    let mut mock = Mock::default()
        .on_read_line(mock::lines(&["0.123"]));

    let result = mock.read_from_str::<f64>(">>> ");
    assert_eq!(0.123, result.unwrap());

    assert_eq!(&[
        Invocation::Print(">>> ".into(), Ok(())),
        Invocation::ReadLine(Ok("0.123".into())),
    ], mock.invocations());
}

#[test]
fn read_from_str_invalid_corrected() {
    let mut mock = Mock::default()
        .on_read_line(mock::lines(&["foo", "0.123"]));

    let result = mock.read_from_str::<f64>(">>> ");
    assert_eq!(0.123, result.unwrap());

    assert_eq!(&[
        Invocation::Print(">>> ".into(), Ok(())),
        Invocation::ReadLine(Ok("foo".into())),
        Invocation::Print("Invalid input: invalid float literal.\n".into(), Ok(())),
        Invocation::Print(">>> ".into(), Ok(())),
        Invocation::ReadLine(Ok("0.123".into())),
    ], mock.invocations());
}

#[test]
fn read_from_str_error() {
    let mut mock = Mock::default()
        .on_read_line(|| Err(AccessTerminalError("invalid input".into())));

    let result = mock.read_from_str::<f64>(">>> ");
    assert_eq!(AccessTerminalError("invalid input".into()), result.unwrap_err());

    assert_eq!(&[
        Invocation::Print(">>> ".into(), Ok(())),
        Invocation::ReadLine(Err("invalid input".into())),
    ], mock.invocations());
}

#[test]
fn read_from_str_default_empty() {
    let mut mock = Mock::default()
        .on_read_line(mock::lines(&[""]));

    let result = mock.read_from_str_default::<f64>(">>> ");
    assert_eq!(0.0, result.unwrap());

    assert_eq!(&[
        Invocation::Print(">>> ".into(), Ok(())),
        Invocation::ReadLine(Ok("".into())),
    ], mock.invocations());
}

#[test]
fn read_from_str_default_valid() {
    let mut mock = Mock::default()
        .on_read_line(mock::lines(&["3.14"]));

    let result = mock.read_from_str_default::<f64>(">>> ");
    assert_eq!(3.14, result.unwrap());

    assert_eq!(&[
        Invocation::Print(">>> ".into(), Ok(())),
        Invocation::ReadLine(Ok("3.14".into())),
    ], mock.invocations());
}