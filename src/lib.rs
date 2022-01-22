use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

mod error;
mod state;

pub use crate::error::CorgError;
use crate::state::Parser;

pub struct Corg {
    input: PathBuf,
    output: Option<PathBuf>,
    replace_input: bool,
    delete_blocks: bool,
    warn_if_no_blocks: bool,
    omit_output: bool,
    check_only: bool,

    start_block_marker: String,
    end_block_marker: String,
    end_output_marker: String,
}

impl Default for Corg {
    fn default() -> Self {
        Self {
            input: PathBuf::new(),
            replace_input: false,
            output: None,
            delete_blocks: true,
            warn_if_no_blocks: false,
            omit_output: false,
            check_only: false,

            start_block_marker: "[[[#!".to_string(),
            end_block_marker: "]]]".to_string(),
            end_output_marker: "[[[end]]]".to_string(),
        }
    }
}

impl Corg {
    pub fn input(mut self, input: PathBuf) -> Self {
        self.input = input;
        self
    }

    pub fn output(mut self, output: Option<PathBuf>) -> Self {
        self.output = output;
        self
    }

    pub fn replace_input(mut self, replace_input: bool) -> Self {
        self.replace_input = replace_input;
        self
    }

    pub fn delete_blocks(mut self, delete_blocks: bool) -> Self {
        self.delete_blocks = delete_blocks;
        self
    }

    pub fn warn_if_no_blocks(mut self, warn_if_no_blocks: bool) -> Self {
        self.warn_if_no_blocks = warn_if_no_blocks;
        self
    }

    pub fn omit_output(mut self, omit_output: bool) -> Self {
        self.omit_output = omit_output;
        self
    }

    pub fn check_only(mut self, check_only: bool) -> Self {
        self.check_only = check_only;
        self
    }

    pub fn override_markers(
        mut self,
        start_block_marker: String,
        end_block_marker: String,
        end_output_marker: String,
    ) -> Self {
        self.start_block_marker = start_block_marker;
        self.end_block_marker = end_block_marker;
        self.end_output_marker = end_output_marker;
        self
    }

    pub fn execute(&self) -> Result<(), CorgError> {
        let content = self.get_file_contents()?;

        let result = Parser::evaluate(&content, &self)?;
        let output = result.get_output();

        if self.warn_if_no_blocks && !result.found_blocks() {
            return Err(CorgError::NoBlocksDetected);
        }

        if self.check_only && content != output {
            return Err(CorgError::CheckFailed((content, output.to_string())));
        }

        let out_file = if self.replace_input {
            Some(&self.input)
        } else {
            self.output.as_ref()
        };

        if let Some(out_file) = out_file {
            if let Some(parent) = out_file.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let mut file = File::create(out_file)?;
            file.write_all(output.as_bytes())?;
        } else {
            let mut stdout = std::io::stdout();
            stdout.write_all(output.as_bytes())?;
        };

        Ok(())
    }

    fn get_file_contents(&self) -> Result<String, CorgError> {
        let mut buffer = String::new();

        let path = &self.input;
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
}
