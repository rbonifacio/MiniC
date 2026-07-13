use mini_c::ir::ast::CheckedProgram;
use mini_c::parser::program;
use mini_c::semantic::type_check;
use nom::combinator::all_consuming;

pub fn parse_and_type_check(src: &str) -> Result<CheckedProgram, mini_c::semantic::TypeError> {
    let (_, prog) = all_consuming(program)(src).map_err(|_| mini_c::semantic::TypeError {
        message: "parse failed".to_string(),
    })?;
    type_check(&prog)
}
