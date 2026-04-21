use super::conststrategy::ConstAssignment;
use proptest::prelude::*;
use proptest::string as pstring;
use proptest::sample as psample;
use mini_c::parser::statements::const_statement;
use crate::conststrategy::ConstAssignmentParams;

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

macro_rules! const_params {
    // Empty -> default
    () => {
        ConstAssignmentParams::default()
    };
    
    // Only param
    (param = $param:expr) => {
        ConstAssignmentParams {
            param_complexity: $param,
            ..Default::default()
        }
    };
    
    // Only nesting
    (nesting = $nesting:expr) => {
        ConstAssignmentParams {
            nesting_complexity: $nesting,
            ..Default::default()
        }
    };
    
    // Only modifier
    (modifier = $modifier:expr) => {
        ConstAssignmentParams {
            modifier_kw: Some($modifier.to_string()),  // ← Just Some(String), NOT boxed
            ..Default::default()
        }
    };
    
    // Only ident (strategy)
    (ident = $ident:expr) => {
        ConstAssignmentParams {
            identifier: $ident.boxed(),
            ..Default::default()
        }
    };
    
    // param + nesting
    (param = $param:expr, nesting = $nesting:expr) => {
        ConstAssignmentParams {
            param_complexity: $param,
            nesting_complexity: $nesting,
            ..Default::default()
        }
    };
    
    // param + nesting + modifier
    (param = $param:expr, nesting = $nesting:expr, modifier = $modifier:expr) => {
        ConstAssignmentParams {
            param_complexity: $param,
            nesting_complexity: $nesting,
            modifier_kw: Some($modifier.to_string()),  // ← Some(String), NOT boxed
            ..Default::default()
        }
    };
    
    // param + nesting + modifier + ident
    (param = $param:expr, nesting = $nesting:expr, modifier = $modifier:expr, ident = $ident:expr) => {
        ConstAssignmentParams {
            param_complexity: $param,
            nesting_complexity: $nesting,
            modifier_kw: Some($modifier.to_string()),  // ← Some(String), NOT boxed
            identifier: $ident.boxed(),
        }
    };
}

// Property: Const generation without rvalue should never be parsed 
proptest! {
    #[test]
    fn test_nonexistent_rvalue(assign in any_with::<ConstAssignment>(
        const_params!(param = 0..=0, nesting = 0..=0)
    )) {
        let source = const_source!(assign);
        
        let result = const_statement(&source);
        
        // param_complexity: 0..=0 generates empty rvalue
        // Source becomes "const int x = ;" (invalid)
        assert!(result.is_err(), "Invalid code parsed successfully: {}", source);
    }
}

// Property: Const generation with other modifier that is not const should never be parsed
proptest! {
    #[test]
    fn test_invalid_modifier(assign in any_with::<ConstAssignment>(
        const_params!(param = 1..=1, nesting = 0..=0,modifier = "static")
    )) {
        let source = const_source!(assign);
        
        let result = const_statement(&source);
        
        assert!(result.is_err(), "Invalid code parsed successfully: {}", source);
    }
}

// Property: Const generation must be parsed when the identifier is "simple"
proptest! {
    #[test]
    fn test_valid_modifier(assign in any_with::<ConstAssignment>(
        const_params!(param = 1..=1, nesting = 0..=0, modifier = "const",
            ident = pstring::string_regex("[a-zA-Z]([_a-zA-Z0-9])*").unwrap()
        )
    )) {
        let source = const_source!(assign);
        
        let result = const_statement(&source);
        
        // Valid C syntax should parse successfully
        assert!(result.is_ok(), "Failed to parse valid C: {}", source);
    }
}

// Property: Const generation must be parsed when the identifier is "simple" and one level of nesting
proptest! {
    #[test]
    fn test_valid_with_simple_nesting(assign in any_with::<ConstAssignment>(
        const_params!(param = 1..=1, nesting = 1..=1, modifier = "const",
            ident = pstring::string_regex("[a-zA-Z]([_a-zA-Z0-9])*").unwrap()
        )
    )) {
        let source = const_source!(assign);
        
        let result = const_statement(&source);
        
        // Valid C syntax should parse successfully
        assert!(result.is_ok(), "Failed to parse valid C: {}", source);
    }
}

fn gen_invalid_identifier() -> BoxedStrategy<String> {
    let c_keywords = vec![
        "int", "const", "return", "if", "else", "while", "for",
    ];
    prop_oneof![
        psample::select(c_keywords).prop_map(|s| s.to_string()),
        Just("".to_string()),                           // Empty
        pstring::string_regex("[0-9][a-zA-Z]*").unwrap(), // Starts with number
        Just("123".to_string()),                        // All numbers
    ].boxed()
}

// Property: Const generation should not accept invalid identifiers
proptest! {
    #[test]
    fn test_invalid_identifiers(assign in any_with::<ConstAssignment>(
        ConstAssignmentParams {
            param_complexity: 1..=1,
            nesting_complexity: 0..=0,
            modifier_kw: Some("const".to_string()),
            identifier: gen_invalid_identifier(),
        }
    )) {
        let source = const_source!(assign);
        let result = const_statement(&source);
        assert!(result.is_err(), "Invalid C parsed: {}", source);
    }
}

// Property: Const generation should not break under "moderate nesting" and single expressions
proptest! {
    #[test]
    fn test_moderate_nesting_single_expression(assign in any_with::<ConstAssignment>(
            const_params!(param = 1..=1, nesting = 2..=2 
            )
    )) {
        let source = const_source!(assign);
        let result = const_statement(&source);
        assert!(result.is_ok(), "Failed to parse valid C : {}", source);
    }
}


// Property: Const generation should not break under "simple nesting" and composite expressions
proptest! {
    #[test]
    fn test_composite_expressions_single_nesting(assign in any_with::<ConstAssignment>(
            const_params!(param = 1..=5, nesting = 1..=1
            )
    )) {
        let source = const_source!(assign);
        let result = const_statement(&source);
        assert!(result.is_ok(), "Failed to parse valid C : {}", source);
    }
}

// Property: Const generation should not break under "complex nesting" and composite expressions
proptest! {
    #[test]
    fn test_composite_expressions_complex_nesting(assign in any_with::<ConstAssignment>(
            const_params!(param = 1..=5, nesting = 1..=3
            )
    )) {
        let source = const_source!(assign);
        let result = const_statement(&source);
        assert!(result.is_ok(), "Failed to parse valid C : {}", source);
    }
}

