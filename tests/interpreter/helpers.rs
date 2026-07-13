use mini_c::interpreter::interpret;
use mini_c::parser::program;
use mini_c::semantic::type_check;

pub fn run(src: &str) -> Result<(), String> {
    let unchecked = program(src)
        .map_err(|e| format!("parse error: {:?}", e))
        .map(|(_, p)| p)?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}
