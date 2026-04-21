use super::conststrategy::ConstAssignment;
use proptest::prelude::*;
use mini_c::parser::statements::const_statement;

macro_rules! const_source {
    ($assign:expr) => {
        format!("{} {} {} = {};", 
            $assign._const, 
            $assign._type, 
            $assign._identifier, 
            $assign._rvalue
        )
    };
}

// Property: All generated const declarations must parse completely
proptest! {
    #[test]
    fn test_const_parsing(assign in any::<ConstAssignment>()) {
        let source = const_source!(assign);
        
        let result = const_statement(&source);
        assert!(result.is_ok(), "Failed to parse: {}", source);
        
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining, "", "Parser didn't consume everything: '{}'", remaining);
    }
}
