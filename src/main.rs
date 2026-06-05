use std::{env, fs, process};

use mini_c::{interpreter::{interpret, run_tests}, parser::program, semantic::type_check};

fn usage() -> ! {
    eprintln!("Usage: minic --check <file.minic>");
    eprintln!("       minic --run   <file.minic>");
    eprintln!("       minic --test  <file.minic>");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Task 1.1: require exactly 3 args (binary, flag, file)
    if args.len() != 3 {
        usage();
    }

    let flag = &args[1];
    let path = &args[2];

    // Task 1.3: read source file
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    // Task 2.1 / 3.1: parse
    let unchecked = match program(&source) {
        Ok((_, prog)) => prog,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            process::exit(1);
        }
    };

    // Task 2.2 / 3.1: type-check
    let checked = match type_check(&unchecked) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Type error: {}", e);
            process::exit(1);
        }
    };

    match flag.as_str() {
        // Task 2.3: --check succeeds after parse + type-check
        "--check" => {
            println!("'{}' is well-typed.", path);
        }
        // Task 3.2 / 3.3: --run interprets after check
        "--run" => {
            if let Err(e) = interpret(&checked) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        "--test" => {
            if let Err(e) = run_tests(&checked) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        // Task 1.1: unknown flag
        _ => usage(),
    }
}
