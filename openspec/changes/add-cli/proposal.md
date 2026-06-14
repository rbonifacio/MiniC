## Why

The current `main.rs` entry point always parses, type-checks, and interprets a MiniC program with no way to stop early. Users need the ability to validate a program (parse + type-check only) without running it, and to explicitly choose between checking and running from the command line.

## What Changes

- Replace the single positional-argument interface (`minic <file>`) with a two-flag interface:
  - `--check <file>`: parse the source file and run the type checker; report errors or confirm success
  - `--run <file>`: parse, type-check, and interpret the source file (full pipeline)
- Both flags produce clear, user-friendly error messages on failure and exit with a non-zero code
- Invoking the binary with no arguments, unknown flags, or missing file path prints usage and exits with an error

## Capabilities

### New Capabilities
- `cli`: Command-line interface for the MiniC binary — argument parsing, flag dispatch (`--check` / `--run`), error reporting, and exit codes

### Modified Capabilities
<!-- None — existing pipeline modules (parser, type-checker, interpreter) are unchanged -->

## Impact

- `src/main.rs`: full rewrite of the entry point to handle the new flag-based interface
- No changes to library code (`parser`, `semantic`, `interpreter`, `environment`, etc.)
- No new dependencies required (stdlib `std::env` and `std::fs` are sufficient)
- Binary interface change: **existing scripts calling `minic <file>` directly will break** — they must be updated to use `minic --run <file>`
