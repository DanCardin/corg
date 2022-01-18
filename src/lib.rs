use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

mod error;
mod options;
mod state;

pub use crate::error::CorgError;
pub use crate::options::Options;
use crate::state::Parser;

pub struct Corg {
    options: Options,
}

impl Corg {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    pub fn execute(&self) -> Result<(), CorgError> {
        let content = self.get_file_contents()?;

        let result = Parser::evaluate(&content, &self.options)?;
        let output = result.get_output();

        if self.options.warn_if_no_blocks && !result.found_blocks() {
            return Err(CorgError::NoBlocksDetected);
        }

        if self.options.check && content != output {
            return Err(CorgError::CheckFailed((content, output.to_string())));
        }

        self.output(output)?;
        Ok(())
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
}
