// $coverage:ignore-start

use std::fs::File;
use std::{io};
use std::io::{BufRead, BufReader, Cursor, ErrorKind, Write};
use flanker_temp::TempPath;
use stdio_override::{StdinOverride};
use crate::terminal::{AccessTerminalError, InputAdapter, OutputAdapter, Streaming, Terminal};

#[test]
fn default_print_no_locking() {
    let mut term_1 = Streaming::default();
    let mut term_2 = Streaming::default();
    term_1.print_line("hello 1").unwrap();
    term_2.print_line("hello 2").unwrap();
}

#[test]
fn default_read_line_intercepted() {
    let temp = TempPath::with_extension("txt");
    {
        let mut file = File::create(&temp).unwrap();
        file.write_all(b"line one\nline two").unwrap();
    }

    let guard = StdinOverride::override_file(temp);
    let mut term = Streaming::default();
    let line = term.read_line().unwrap();
    assert_eq!("line one\n", line);
    let line = term.read_line().unwrap();
    assert_eq!("line two", line);
    drop(guard);
}

#[test]
fn input_output() {
    let mut read = BufReader::new(Cursor::new("hello".as_bytes()));
    let mut write = Cursor::new(Vec::new());
    let input = InputAdapter::new(|| {
        let mut buf = String::default();
        read.read_line(&mut buf)?;
        Ok(buf)
    });
    let output = OutputAdapter::new(|str| Ok(write!(write, "{}", str)?));
    let mut term = Streaming {
        input, output
    };
    assert_eq!("hello", term.read_line().unwrap());
    term.print("printed").unwrap();
    drop(term);

    let written = String::from_utf8(write.into_inner()).unwrap();
    assert_eq!("printed", written);
}

#[test]
fn implements_from_io_error() {
    let io_error = io::Error::new(ErrorKind::BrokenPipe, "broken pipe");
    let access_error = AccessTerminalError::from(io_error);
    assert!(access_error.to_string().contains("broken pipe"));
}