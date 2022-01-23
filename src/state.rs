use regex::{Match, Regex};

use std::io::Write;
use std::iter::Peekable;
use std::mem;
use std::process::{Command, Stdio};
use std::str::Lines;

use crate::error::CorgError;
use crate::Corg;

pub struct Parser;

impl Parser {
    pub fn evaluate(input: &str, corg: &Corg) -> Result<OutputState, CorgError> {
        let mut input: Peekable<Lines> = input.lines().peekable();

        let raw_re_start_block = format!("^.*{}(.*)$", regex::escape(&corg.start_block_marker));
        let re_start_block: Regex = Regex::new(&raw_re_start_block).unwrap();

        let raw_re_end_block = format!("^.*{}.*$", regex::escape(&corg.end_block_marker));
        let re_end_block: Regex = Regex::new(&raw_re_end_block).unwrap();

        let raw_re_end_output = format!(
            r"^(.*)({})\s*(?:\(checksum: ([a-z0-9]+)\))?\s*(.*?)$",
            regex::escape(&corg.end_output_marker)
        );
        let re_end_output: Regex = Regex::new(&raw_re_end_output).unwrap();

        let mut state = ParseStates::default();
        loop {
            state = match state {
                ParseStates::RawText(raw_text) => {
                    raw_text.consume_raw_text(&mut input, &re_start_block)
                }
                ParseStates::CodePending(code_pending) => {
                    code_pending.consume_code(&mut input, &re_start_block, &re_end_block, corg)
                }
                ParseStates::OutputPending(output_pending) => {
                    output_pending.produce_output(&mut input, &re_end_output, corg)?
                }
                ParseStates::Done(state) => return Ok(state),
            }
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
struct RawText {
    state: OutputState,
}

impl RawText {
    fn consume_raw_text(
        mut self,
        input: &mut Peekable<Lines>,
        re_start_block: &Regex,
    ) -> ParseStates {
        loop {
            if let Some(line) = input.peek() {
                if re_start_block.is_match(line) {
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

#[derive(Debug)]
struct CodePending {
    state: OutputState,
}

impl CodePending {
    fn consume_code(
        mut self,
        input: &mut Peekable<Lines>,
        re_start_block: &Regex,
        re_end_block: &Regex,
        corg: &Corg,
    ) -> ParseStates {
        self.state.blocks_found = true;

        let line = input.next().unwrap();
        let captures = re_start_block.captures(line).unwrap();
        let shebang = captures[1].to_string();
        self.add_meta_line(corg, line);

        let mut code = String::new();

        loop {
            let line = match input.peek() {
                Some(line) => line.to_string(),
                None => return ParseStates::done(self.state),
            };

            self.add_meta_line(corg, &line);

            if re_end_block.is_match(&line) {
                return ParseStates::output_pending(self.state, shebang, code);
            }

            input.next().unwrap();
            code.push_str(&format!("{line}\n"));
        }
    }

    fn add_meta_line(&mut self, corg: &Corg, line: &str) {
        if !corg.delete_blocks {
            self.state.output.push_str(&format!("{line}\n"));
        }
    }
}

#[derive(Debug)]
struct OutputPending {
    state: OutputState,
    shebang: String,
    code: String,
}

impl OutputPending {
    fn produce_output(
        mut self,
        input: &mut Peekable<Lines>,
        re_end_output: &Regex,
        corg: &Corg,
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

            if let Some(capture) = re_end_output.captures(line) {
                let output = self.execute_code_block()?;
                if !corg.omit_output {
                    self.state.output.push_str(&output);
                }

                if !corg.delete_blocks {
                    let before = capture[1].to_string();
                    let output_marker = capture[2].to_string();
                    let after = capture[4].to_string();
                    let checksum_capture = capture.get(3);

                    let checksum_part = self.output_checksum(corg, output, checksum_capture)?;

                    self.state
                        .output
                        .push_str(&format!("{before}{output_marker}{checksum_part}{after}\n"));
                }

                return Ok(ParseStates::raw_text(self.state));
            }
        }
    }

    #[cfg(not(feature = "checksum"))]
    fn output_checksum(
        &self,
        _corg: &Corg,
        _output: String,
        _checksum_capture: Option<Match>,
    ) -> Result<String, CorgError> {
        Ok(String::new())
    }

    #[cfg(feature = "checksum")]
    fn output_checksum(
        &self,
        corg: &Corg,
        output: String,
        checksum_capture: Option<Match>,
    ) -> Result<String, CorgError> {
        Ok(if corg.checksum {
            let new_checksum = format!("{:x}", md5::compute(output));

            if let Some(old_checksum) = checksum_capture {
                let old_checksum = old_checksum.as_str().to_string();
                if new_checksum != old_checksum {
                    return Err(CorgError::ChecksumMismatch((old_checksum, new_checksum)));
                }
            }
            format!(" (checksum: {new_checksum}) ")
        } else {
            String::new()
        })
    }

    fn execute_code_block(&self) -> Result<String, CorgError> {
        let (program, args) = shlex::split(&self.shebang)
            .and_then(|parts| {
                let mut iter = parts.into_iter();
                iter.next().map(|program| (program, iter.collect()))
            })
            .unwrap_or_else(|| (self.shebang.clone(), vec![]));

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
