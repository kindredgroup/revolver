// $coverage:ignore-start

use std::cell::RefCell;
use crate::terminal::{Invocation, Mock, mock, Terminal, AccessTerminalError, ReadLineInput, PrintOutput};

#[test]
fn invocation_variants() {
    let inv = Invocation::ReadLine(Ok("foo".into()));
    assert!(inv.read_line().is_some());
    assert!(inv.print().is_none());
    assert_eq!("foo", inv.read_line().unwrap_input());

    let inv = Invocation::Print("foo".into(), Ok(()));
    assert!(inv.read_line().is_none());
    assert!(inv.print().is_some());
    assert_eq!("foo", inv.print().unwrap_output());
}

#[test]
fn default_delegates() {
    let mut mock = Mock::default();
    assert_eq!(0, mock.invocations().len());

    mock.print("hello").unwrap();
    assert_eq!("", mock.read_line().unwrap());

    assert_eq!(vec![
        Invocation::Print("hello".into(), Ok(())),
        Invocation::ReadLine(Ok("".into())),
    ], mock.invocations());

    assert_eq!(Some(("hello", &Ok(()))), mock.invocations()[0].print());
    assert_eq!(Some(&Ok("".into())), mock.invocations()[1].read_line());
}

#[test]
fn custom_delegates() {
    let print_invocations = RefCell::new(0);
    let read_line_invocations = RefCell::new(0);
    let mut mock = Mock::default()
        .on_print(|_| {
            *print_invocations.borrow_mut() += 1;
            Ok(())
        })
        .on_read_line(|| {
            *read_line_invocations.borrow_mut() += 1;
            Ok((*read_line_invocations.borrow()).to_string())
        });

    mock.print("first").unwrap();
    assert_eq!(1, *print_invocations.borrow());
    assert_eq!(0, *read_line_invocations.borrow());

    mock.print("second").unwrap();
    assert_eq!(2, *print_invocations.borrow());
    assert_eq!(0, *read_line_invocations.borrow());

    assert_eq!("1", mock.read_line().unwrap());
    assert_eq!(2, *print_invocations.borrow());
    assert_eq!(1, *read_line_invocations.borrow());

    assert_eq!("2", mock.read_line().unwrap());
    assert_eq!(2, *print_invocations.borrow());
    assert_eq!(2, *read_line_invocations.borrow());
}

#[test]
fn custom_delegates_with_error() {
    let mut mock = Mock::default()
        .on_print(|_| {
            Err(AccessTerminalError("broken pipe".into()))
        })
        .on_read_line(|| {
            Err(AccessTerminalError("already exists".into()))
        });
    assert_eq!(AccessTerminalError("broken pipe".into()), mock.print("hello").unwrap_err());
    assert_eq!(AccessTerminalError("already exists".into()), mock.read_line().unwrap_err());

    assert_eq!(vec![
        Invocation::Print("hello".into(), Err("broken pipe".into())),
        Invocation::ReadLine(Err("already exists".into())),
    ], mock.invocations());
}

#[test]
fn lines() {
    let mut mock = Mock::default()
        .on_read_line(mock::lines(&["one", "two", "three"]));

    assert_eq!("one", mock.read_line().unwrap());
    assert_eq!("two", mock.read_line().unwrap());
    assert_eq!("three", mock.read_line().unwrap());
    assert_eq!(AccessTerminalError("no more lines".into()), mock.read_line().err().unwrap());
}

#[test]
fn invocation_implements_debug() {
    let inv = Invocation::Print("test".into(), Ok(()));
    let s = format!("{inv:?}");
    assert!(s.contains("test"));
}