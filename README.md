Revolver
===
A library for building REPL applications.

[![Crates.io](https://img.shields.io/crates/v/revolver?style=flat-square&logo=rust)](https://crates.io/crates/revolver)
[![docs.rs](https://img.shields.io/badge/docs.rs-revolver-blue?style=flat-square&logo=docs.rs)](https://docs.rs/revolver)
[![Build Status](https://img.shields.io/github/workflow/status/kindredgroup/revolver/Cargo%20build?style=flat-square&logo=github)](https://github.com/kindredgroup/revolver/actions/workflows/master.yml)
[![codecov](https://img.shields.io/codecov/c/github/kindredgroup/revolver/master?style=flat-square&logo=codecov)](https://codecov.io/gh/kindredgroup/revolver)

# Concepts
## Command
The `Command` trait is a specification of an executable command — the 'execute' part of a REPL application. A command will typically be accompanied by a `NamedCommandParser` implementation for converting command strings into `Command` objects.

`Command`s and `NamedCommandParser`s are the only two traits that you must implement. Everything else is just configuration.

### Commander
A `Commander` decodes user input (typically a line read from a terminal interface) into a dynamic `Command` object, using a preconfigured map of `NamedCommandParser`s.

### Built-in commands
Revolver comes with two useful built-in commands that can be used out-of-the-box.

* `help` — A self-help guide, outlining the available commands and how to use them.
* `quit` — Terminates the REPL. (It only exits the loop; it does not terminate the application.)

These commands are opt-in, meaning that you must explicitly include their parsers in your `Commander` to enable them.

## Terminal
The `Terminal` trait represents a text-based interface with the user. It fulfils the 'read' and 'print' parts of a REPL application.

Revolver is currently bundled with two `Terminal` implementations:

* `Streaming` — A terminal device that composes over I/O streams using `Input` and `Output` traits. Out-of-the-box adapters exist for `stdin` and `stdout` streams. Adapters may be written to interface with nonstandard streams by supplying a custom closure.
* `Mock` — A way of mocking a terminal device for feeding input, capturing output, and performing various assertions.

## Looper
`Looper` is a mechanism for iteratively running commands based on successive user input. It fulfils the 'loop' part of a REPL application.

# Getting started
## Add dependency
```sh
cargo add revolver
```

## An example
See `examples/calculator.rs` for a simple calculator REPL. 