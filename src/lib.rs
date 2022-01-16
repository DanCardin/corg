use lazy_static::lazy_static;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use thiserror::Error;

use regex::Regex;

#[cfg(feature = "cli")]
use clap::Parser;

lazy_static! {
    static ref RE_START_BLOCK: Regex = Regex::new(r".*\[\[\[\#!(.*)$").unwrap();
    static ref RE_END_BLOCK: Regex = Regex::new(r"\]\]\](.*)").unwrap();
    static ref RE_END_OUTPUT: Regex = Regex::new(r"^.*\[\[\[\s*end\s*\]\]\].*$").unwrap();
}

/// A Cog-like tool
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", clap(author, version, about))]
pub struct Options {
    input: PathBuf,

    /// Write the output to a file instead of stdout
    #[cfg_attr(feature = "cli", clap(long, short))]
    output: Option<PathBuf>,

    /// Write the output to the original input file, supercedes `--output`
    #[cfg_attr(feature = "cli", clap(long, short))]
    replace: bool,

    /// Delete the generator code from the output file.
    #[cfg_attr(feature = "cli", clap(long, short))]
    delete_block: bool,

    /// Warn if a file has no cog code in it.
    #[cfg_attr(feature = "cli", clap(short = 'e'))]
    warn_if_no_blocks: bool,

    /// Omit all the generated output without running the generators.
    #[cfg_attr(feature = "cli", clap(short = 'x', long))]
    omit_output: bool,

    // Check that the files would not change if run again.
    #[cfg_attr(feature = "cli", clap(long))]
    check: bool,
}

#[derive(Error, Debug)]
pub enum CorgError {
    #[error("No code blocks detected")]
    NoBlocksDetected,

    #[error("{0}")]
    IOError(#[from] std::io::Error),

    #[error("Error occured during block execution")]
    BlockExecutionError(String),

    #[error("Generated dutput did not match the existing content")]
    CheckFailed((String, String)),
}

pub struct Corg {
    options: Options,
}

impl Corg {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    fn get_file_contents(&self) -> Result<String, CorgError> {
        let mut buffer = String::new();

        let path = &self.options.input;
        if path == &PathBuf::from("-") {
            std::io::stdin()
                .read_to_string(&mut buffer)
                .expect("Could not read stdin");
        } else {
            let mut file = File::open(&path)?;
            file.read_to_string(&mut buffer)?;
        }
        Ok(buffer)
    }

    fn process_content(&self, content: &str) -> Result<String, CorgError> {
        let mut output = String::new();

        let mut blocks_exist = false;
        let mut code_pending = false;
        let mut output_pending = false;
        let mut shebang = String::new();
        let mut code = String::new();
        for line in content.lines() {
            let full_line = format!("{line}\n");
            if RE_END_OUTPUT.is_match(line) {
                self.add_meta_line(&mut output, &full_line);

                code_pending = false;
                output_pending = false;
            } else {
                if output_pending {
                    continue;
                }

                if let Some(captures) = RE_START_BLOCK.captures(line) {
                    self.add_meta_line(&mut output, &full_line);

                    shebang = captures.get(1).unwrap().as_str().to_string();

                    blocks_exist = true;
                    code_pending = true;
                    output_pending = false;
                } else if RE_END_BLOCK.is_match(line) {
                    self.add_meta_line(&mut output, &full_line);

                    code_pending = false;
                    output_pending = true;

                    if !self.options.omit_output {
                        let block_output = self.execute_block(&shebang, &code)?;
                        output.push_str(&block_output);
                    }

                    code.clear();
                } else {
                    if code_pending {
                        code.push_str(&full_line);

                        self.add_meta_line(&mut output, &full_line);
                    } else if !output_pending {
                        output.push_str(&full_line);
                    }
                }
            }
        }

        if self.options.warn_if_no_blocks && !blocks_exist {
            Err(CorgError::NoBlocksDetected)
        } else {
            Ok(output)
        }
    }

    fn add_meta_line(&self, buffer: &mut String, line: &str) {
        if !self.options.delete_block {
            buffer.push_str(&line);
        }
    }

    fn execute_block(&self, shebang: &str, code: &str) -> Result<String, CorgError> {
        let mut parts = shebang.split(' ');
        let (program, args) = if let Some(first_part) = parts.next() {
            (first_part, parts.collect())
        } else {
            (shebang, vec![])
        };

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin.write(code.as_bytes())?;
        mem::drop(stdin);

        let child = child.wait_with_output()?;
        Ok(String::from_utf8_lossy(&child.stdout).to_string())
    }

    fn output(&self, processed_content: &str) -> Result<(), CorgError> {
        let out_file = if self.options.replace {
            Some(&self.options.input)
        } else {
            self.options.output.as_ref()
        };

        if let Some(out_file) = out_file {
            if let Some(parent) = out_file.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let mut file = File::create(out_file)?;
            file.write_all(processed_content.as_bytes())?;
        } else {
            let mut stdout = std::io::stdout();
            stdout.write_all(processed_content.as_bytes())?;
        };

        Ok(())
    }

    pub fn run(&self) -> Result<(), CorgError> {
        let content = self.get_file_contents()?;
        let processed_content = self.process_content(&content)?;

        if self.options.check && content != processed_content {
            Err(CorgError::CheckFailed((content, processed_content)))
        } else {
            self.output(&processed_content)?;
            Ok(())
        }
    }
}
