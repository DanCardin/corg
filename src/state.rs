use std::io::Write;
use std::iter::Peekable;
use std::mem;
use std::process::{Command, Stdio};
use std::str::Lines;

use lazy_static::lazy_static;
use regex::Regex;

use crate::error::CorgError;
use crate::options::Options;

lazy_static! {
    static ref RE_START_BLOCK: Regex = Regex::new(r".*\[\[\[\#!(.*)$").unwrap();
    static ref RE_END_BLOCK: Regex = Regex::new(r"\]\]\](.*)").unwrap();
    static ref RE_END_OUTPUT: Regex = Regex::new(r"^.*\[\[\[\s*end\s*\]\]\].*$").unwrap();
}

pub struct Parser;

impl Parser {
    pub fn evaluate(input: &str, options: &Options) -> Result<OutputState, CorgError> {
        let mut input: Peekable<Lines> = input.lines().peekable();

        let mut state = ParseStates::default();
        loop {
            state = match state {
                ParseStates::RawText(raw_text) => raw_text.consume_raw_text(&mut input),
                ParseStates::CodePending(code_pending) => {
                    code_pending.consume_code(&mut input, options)
                }
                ParseStates::OutputPending(output_pending) => {
                    output_pending.produce_output(&mut input, options)?
                }
                ParseStates::Done(state) => return Ok(state),
            }
        }
    }
}

pub struct OutputState {
    output: String,
    blocks_found: bool,
}

impl OutputState {
    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn found_blocks(&self) -> bool {
        self.blocks_found
    }
}

enum ParseStates {
    RawText(RawText),
    CodePending(CodePending),
    OutputPending(OutputPending),
    Done(OutputState),
}

impl ParseStates {
    fn raw_text(state: OutputState) -> Self {
        ParseStates::RawText(RawText { state })
    }

    fn code_pending(state: OutputState) -> Self {
        ParseStates::CodePending(CodePending { state })
    }

    fn output_pending(state: OutputState, shebang: String, code: String) -> Self {
        ParseStates::OutputPending(OutputPending {
            state,
            shebang,
            code,
        })
    }

    fn done(state: OutputState) -> Self {
        ParseStates::Done(state)
    }
}

impl Default for ParseStates {
    fn default() -> Self {
        Self::RawText(RawText {
            state: OutputState {
                output: String::new(),
                blocks_found: false,
            },
        })
    }
}

struct RawText {
    state: OutputState,
}

impl RawText {
    fn consume_raw_text(mut self, input: &mut Peekable<Lines>) -> ParseStates {
        loop {
            if let Some(line) = input.peek() {
                if RE_START_BLOCK.is_match(line) {
                    return ParseStates::code_pending(self.state);
                }
            } else {
                return ParseStates::done(self.state);
            }

            let line = input.next().unwrap();
            self.state.output.push_str(&format!("{line}\n"));
        }
    }
}

struct CodePending {
    state: OutputState,
}

impl CodePending {
    fn consume_code(mut self, input: &mut Peekable<Lines>, options: &Options) -> ParseStates {
        self.state.blocks_found = true;

        let line = input.next().unwrap();
        let captures = RE_START_BLOCK.captures(line).unwrap();
        let shebang = captures[1].to_string();
        self.add_meta_line(options, line);

        let mut code = String::new();

        loop {
            let line = match input.peek() {
                Some(line) => line.to_string(),
                None => return ParseStates::done(self.state),
            };

            self.add_meta_line(options, &line);

            if RE_END_BLOCK.is_match(&line) {
                return ParseStates::output_pending(self.state, shebang, code);
            }

            input.next().unwrap();
            code.push_str(&format!("{line}\n"));
        }
    }

    fn add_meta_line(&mut self, options: &Options, line: &str) {
        if !options.delete_block {
            self.state.output.push_str(&format!("{line}\n"));
        }
    }
}

struct OutputPending {
    state: OutputState,
    shebang: String,
    code: String,
}

impl OutputPending {
    fn produce_output(
        mut self,
        input: &mut Peekable<Lines>,
        options: &Options,
    ) -> Result<ParseStates, CorgError> {
        loop {
            let line = match input.next() {
                Some(line) => line,
                None => {
                    let output = self.execute_code_block()?;
                    self.state.output.push_str(&output);
                    return Ok(ParseStates::done(self.state));
                }
            };

            if RE_END_OUTPUT.is_match(line) {
                if !options.omit_output {
                    let output = self.execute_code_block()?;
                    self.state.output.push_str(&output);
                }

                if !options.delete_block {
                    self.state.output.push_str(&format!("{line}\n"));
                }

                return Ok(ParseStates::raw_text(self.state));
            }
        }
    }

    fn execute_code_block(&self) -> Result<String, CorgError> {
        let mut parts = self.shebang.split(' ');
        let (program, args) = if let Some(first_part) = parts.next() {
            (first_part, parts.collect())
        } else {
            (self.shebang.as_str(), vec![])
        };

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(self.code.as_bytes())?;
        mem::drop(stdin);

        let child = child.wait_with_output()?;
        if !child.status.success() {
            let err = String::from_utf8_lossy(&child.stderr).to_string();
            return Err(CorgError::BlockExecutionError(err));
        }
        Ok(String::from_utf8_lossy(&child.stdout).to_string())
    }
}
