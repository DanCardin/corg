use clap::Parser;
use colored::Colorize;
use corg::{Corg, CorgError, Options};
use difference::{Changeset, Difference};
use std::io::Write;

fn main() -> Result<(), CorgError> {
    let options = Options::parse();

    let corg = Corg::new(options);

    let exit_code = match corg.run() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{e}");

            match e {
                CorgError::NoBlocksDetected => 0,
                CorgError::IOError(_) => 1,
                CorgError::BlockExecutionError(_) => 3,
                CorgError::CheckFailed((content, output)) => {
                    let Changeset { diffs, .. } = Changeset::new(&output, &content, "");

                    let mut out = std::io::stderr();
                    for c in &diffs {
                        let colored = match *c {
                            Difference::Same(ref z) => z.white(),
                            Difference::Rem(ref z) => z.red(),
                            Difference::Add(ref z) => z.green(),
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
