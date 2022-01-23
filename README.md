# Corg

<p align="center">
<img src="https://img.shields.io/crates/l/corg.svg" alt="license">
<a href="https://crates.io/crates/corg">
<img src="https://img.shields.io/crates/v/corg.svg?colorB=319e8c" alt="Version info">
</a>
<a href="https://github.com/DanCardin/corg/actions?query=workflow%3ATest">
<img src="https://github.com/DanCardin/corg/workflows/Test/badge.svg" alt="Build Status">
</a> <a href="https://codecov.io/gh/DanCardin/corg">
<img src="https://codecov.io/gh/DanCardin/corg/branch/main/graph/badge.svg?token=U7NQIWXWKW"/>
</a><br>
</p>

A [cog](https://nedbatchelder.com/code/cog)-like tool, written in Rust.

Straight from Ned:

> Cog is a file generation tool. It lets you use pieces of Python code as generators in your source files to generate whatever text you need.

Being written in python, Cog naturally executes python and integrates more deeply
with it than is possible here.

Instead, Corg allows one to choose any executable (python, bash, etc) which
accepts piped input. Shown below, Corg uses a shebang-looking mechanism instead.

## Example

The most obvious motivating example which comes to mind for this tool is for
keeping documentation up-to-date with their sources of truth. Be that, verifying
CLI help text, or executing code examples.

This document is a great example. Embedded below at the [CLI](#CLI) section
is the following block.

<!-- [[[#!bash
echo
cat example.md
echo
]]] -->

```md
<!-- [[[#!/usr/bin/env bash
cargo run --features cli -- --help
]]] -->
```

<!-- [[[end]]] (checksum: 9503506b397b9716def5152b41695181) -->

And as you can see below, it outputs the help text. See the raw README source,
and you'll notice the invisible, commented out sections which are used to produce
the output!

The CI for this repo then runs: `cargo run --features cli -- README.md -r --check --checksum`
to verify that it remains in sync.

### Installation

#### With Cargo

```bash
cargo install corg --features=cli
```

#### Download Release

- Download a pre-built binary from [Releases](https://github.com/DanCardin/corg/releases)

## CLI

Using itself to produce output below!

<!-- [[[#!/usr/bin/env bash
echo
echo '```'
cargo run --features cli -- --help
echo '```'
echo
]]] -->

```
corg 0.1.0
A cog-like tool

USAGE:
    corg [OPTIONS] <INPUT>

ARGS:
    <INPUT>    The input file

OPTIONS:
    -c, --checksum             Checksum the output to protect it against accidental change
        --check                Check that the files would not change if run again
    -d, --delete-blocks        Delete the generator code from the output file
    -e                         Warn if a file has no cog code in it
    -h, --help                 Print help information
        --markers <MARKERS>    The patterns surrounding cog inline instructions. Should include
                               three values separated by spaces, the start, end, and end-output
                               markers
    -o, --output <OUTPUT>      Write the output to a file instead of stdout
    -r, --replace              Write the output to the original input file, supercedes `--output`
    -V, --version              Print version information
    -x, --omit-output          Omit all the generated output without running the generators
```

<!-- [[[end]]] (checksum: 22deee7b210466aedf32e7eb677409d3) -->

## Library

The above CLI is a thin wrapper over the corg internal structures. It's not entirely
obvious whether this is useful or not, but it's available!
