# Corg

A [cog-like](https://nedbatchelder.com/code/cog) tool, written in Rust.

The primary difference between Cog and Corg is how Corg executes code blocks.
Being written in Rust, it cannot as easily assume specific python implementation
details.

As such Corg uses a shebang-like mechanism instead, and as such it can execute
any available program, such as python, bash, etc!

## Example

Given:

<!-- [[[#!/usr/bin/env bash
echo
cat example.md
echo
]]] -->

```md
<!-- [[[#!/usr/bin/env bash
cargo run --features cli -- --help
]]] -->
```

<!-- [[[end]]] -->

Output:

<!-- [[[#!/usr/bin/env bash
echo '```'
cargo run --features cli -- --help
echo '```'
]]] -->

```
corg 0.1.0
A Cog-like tool

USAGE:
    corg [OPTIONS] <INPUT>

ARGS:
    <INPUT>

OPTIONS:
        --check
    -d, --delete-block       Delete the generator code from the output file
    -e                       Warn if a file has no cog code in it
    -h, --help               Print help information
    -o, --output <OUTPUT>    Write the output to a file instead of stdout
    -r, --replace            Write the output to the original input file, supercedes `--output`
    -V, --version            Print version information
    -x, --omit-output        Omit all the generated output without running the generators
```

<!-- [[[end]]] -->
