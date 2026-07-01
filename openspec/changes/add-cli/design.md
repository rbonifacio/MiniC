## Context

`src/main.rs` currently implements a single-mode pipeline: parse → type-check → interpret. There is no way to halt after type-checking. The change replaces the positional-argument interface with explicit `--check` and `--run` flags, keeping all library code untouched.

## Goals / Non-Goals

**Goals:**
- Introduce `--check <file>` and `--run <file>` flags to the binary
- Print a clear usage message when invoked incorrectly
- Exit with code `0` on success and non-zero on any error (parse, type, runtime)
- Keep the change entirely within `src/main.rs`

**Non-Goals:**
- Adding a third-party argument-parsing crate (e.g., `clap`, `argh`)
- Changing any library module (`parser`, `semantic`, `interpreter`, etc.)
- Supporting multiple input files, stdin input, or verbose/quiet flags

## Decisions

**Manual argument parsing over a crate**
Rationale: the interface has exactly two flags and one positional per invocation. A hand-rolled `match` on `args[1]` is ten lines and introduces zero new dependencies. Adding `clap` for this surface area would be premature complexity.

Alternative considered: `clap` — rejected because it adds a compile-time dependency and generated help text that would need its own maintenance for a two-flag CLI.

**Flag-first, file-second argument order (`--check <file>`)**
Rationale: mirrors the convention used by tools like `rustc`, `tsc`, and `gcc` (`--flag <target>`). Users familiar with those tools will find the interface natural.

Alternative considered: file-first (`<file> --check`) — rejected because it is less idiomatic and complicates the match logic.

**Single exit-code strategy: 0 = success, 1 = any failure**
Rationale: simple and consistent. Callers only need to check `if [ $? -ne 0 ]`. Distinguishing parse errors from type errors from runtime errors at the exit-code level is not a stated requirement.

## Risks / Trade-offs

**Breaking change to existing callers** → Any script using `minic <file>` must be updated to `minic --run <file>`. Mitigated by clear documentation in the usage message and proposal.

**No color / structured output** → Error messages go to stderr as plain text. Acceptable for the current audience (students/educators); easy to add later without interface changes.

## Migration Plan

1. Rewrite `src/main.rs` to implement the new flag-based dispatch
2. Update any documentation or examples that reference the old `minic <file>` invocation
3. No database migrations, feature flags, or multi-step deploys needed — single binary, local tool
