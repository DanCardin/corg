#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use colored::Colorize;

#[cfg(feature = "cli")]
use difference::{Changeset, Difference};

use corg::{Corg, CorgError, Options};
use std::io::Write;

fn main() -> Result<(), CorgError> {
    let options = Options::parse();
    let corg = Corg::new(options);

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
