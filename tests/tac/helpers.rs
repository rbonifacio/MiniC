use mini_c::ir::ast::{CheckedProgram, UncheckedProgram};
use mini_c::parser::program;
use mini_c::semantic::type_check;
use nom::combinator::all_consuming;
use std::path::Path;

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn parse_fixture(name: &str) -> UncheckedProgram {
    let source = std::fs::read_to_string(fixtures_dir().join(name)).expect("fixture should exist");
    let result = all_consuming(program)(source.trim())
        .map_err(|e| e.map_input(String::from))
        .expect("fixture should parse");
    result.1
}

pub fn type_check_fixture(name: &str) -> Result<CheckedProgram, mini_c::semantic::TypeError> {
    type_check(&parse_fixture(name))
}
