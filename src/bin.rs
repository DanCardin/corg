#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use anyhow::{anyhow, Result};

#[cfg(feature = "cli")]
use colored::Colorize;

#[cfg(feature = "cli")]
use difference::{Changeset, Difference};

use corg::{Corg, CorgError};
use std::io::Write;
use std::path::PathBuf;

/// A cog-like tool
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", clap(author, version, about))]
pub struct Options {
    /// The input file
    pub input: PathBuf,

    /// Write the output to a file instead of stdout
    #[cfg_attr(feature = "cli", clap(short, long))]
    pub output: Option<PathBuf>,

    /// Write the output to the original input file, supercedes `--output`
    #[cfg_attr(feature = "cli", clap(short, long))]
    pub replace: bool,

    /// Delete the generator code from the output file
    #[cfg_attr(feature = "cli", clap(short, long))]
    pub delete_blocks: bool,

    /// Warn if a file has no cog code in it
    #[cfg_attr(feature = "cli", clap(short = 'e'))]
    pub warn_if_no_blocks: bool,

    /// Omit all the generated output without running the generators
    #[cfg_attr(feature = "cli", clap(short = 'x', long))]
    pub omit_output: bool,

    /// Check that the files would not change if run again
    #[cfg_attr(feature = "cli", clap(long, short))]
    pub check: bool,

    /// The patterns surrounding cog inline instructions. Should
    /// include three values separated by spaces, the start, end,
    /// and end-output markers.
    #[cfg_attr(feature = "cli", clap(long, parse(try_from_str = parse_markers)))]
    pub markers: Option<(String, String, String)>,

    /// Read the
    #[cfg_attr(feature = "cli", clap(long))]
    pub raw: bool,
}

pub fn parse_markers(s: &str) -> Result<(String, String, String)> {
    let mut iter = s.splitn(3, ' ');
    let start_block = iter.next();
    let end_block = iter.next();
    let end_output = iter.next();
    match (start_block, end_block, end_output) {
        (Some(start_block), Some(end_block), Some(end_output)) => Ok((
            start_block.to_string(),
            end_block.to_string(),
            end_output.to_string(),
        )),
        _ => Err(anyhow!("Invalid marker: {}", s)),
    }
}

fn main() -> Result<(), CorgError> {
    let options = Options::parse();
    let mut corg = Corg::default()
        .input(options.input)
        .delete_blocks(options.delete_blocks)
        .warn_if_no_blocks(options.warn_if_no_blocks)
        .omit_output(options.omit_output)
        .check_only(options.check);

    corg = if options.replace {
        corg.replace_input(options.replace)
    } else {
        corg.output(options.output)
    };

    if let Some(markers) = options.markers {
        corg = corg.override_markers(markers.0, markers.1, markers.2)
    }

    let exit_code = match corg.execute() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e.to_string().red());

            match e {
                CorgError::NoBlocksDetected => 0,
                CorgError::IOError(_) => 1,
                CorgError::BlockExecutionError(_) => 3,
                CorgError::CheckFailed((content, output)) => {
                    let Changeset { diffs, .. } = Changeset::new(&content, &output, "");

                    let mut out = std::io::stderr();
                    for c in &diffs {
                        let colored = match *c {
                            Difference::Same(ref z) => z.white(),
                            Difference::Rem(ref z) => z.on_red(),
                            Difference::Add(ref z) => z.on_green(),
                        };
                        write!(out, "{}", colored)?;
                    }
                    5
                }
            }
        }
    };
    std::process::exit(exit_code);
}
