## 1. Rewrite Entry Point

- [x] 1.1 Parse `args[1]` to detect `--check` or `--run`; print usage to stderr and exit 1 for any other value or when fewer than 3 args are provided
- [x] 1.2 Extract `args[2]` as the file path; print usage to stderr and exit 1 if it is missing
- [x] 1.3 Read the source file with `fs::read_to_string`; print a file-not-found error to stderr and exit 1 on failure

## 2. Implement --check Dispatch

- [x] 2.1 Call `parser::program` on the source; print a parse error to stderr and exit 1 on failure
- [x] 2.2 Call `semantic::type_check` on the parsed AST; print a type error to stderr and exit 1 on failure
- [x] 2.3 Print a success message to stdout and exit 0 when both steps pass

## 3. Implement --run Dispatch

- [x] 3.1 Reuse the parse and type-check steps from `--check`
- [x] 3.2 Call `interpreter::interpret` on the typed AST; print a runtime error to stderr and exit 1 on failure
- [x] 3.3 Exit 0 on successful interpretation

## 4. Verify

- [x] 4.1 Run `cargo build` and confirm the binary compiles without warnings
- [x] 4.2 Test `--check` with a valid `.minic` file; confirm exit code 0 and success output
- [x] 4.3 Test `--check` with a file containing a type error; confirm exit code 1 and error on stderr
- [x] 4.4 Test `--run` with a valid `.minic` file; confirm the program executes correctly and exits 0
- [x] 4.5 Test `--run` with a file containing a runtime error; confirm exit code 1 and error on stderr
- [x] 4.6 Test invocation with no arguments and with an unknown flag; confirm usage message and exit code 1
