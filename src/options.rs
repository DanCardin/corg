use std::path::PathBuf;

#[cfg(feature = "cli")]
use clap::Parser;

/// A Cog-like tool
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
    pub delete_block: bool,

    /// Warn if a file has no cog code in it
    #[cfg_attr(feature = "cli", clap(short = 'e'))]
    pub warn_if_no_blocks: bool,

    /// Omit all the generated output without running the generators
    #[cfg_attr(feature = "cli", clap(short = 'x', long))]
    pub omit_output: bool,

    /// Check that the files would not change if run again
    #[cfg_attr(feature = "cli", clap(long, short))]
    pub check: bool,
}
