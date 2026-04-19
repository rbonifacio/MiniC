use proptest::prelude::*;
use proptest::string as pstring;
use proptest::collection as pcollection;
use proptest::sample as psample;

pub struct ConstAssignment {
    pub _const: String,
    pub _type: String,
    pub _identifier: String,
    pub _rvalue: String
}

#[derive(Debug, Clone, Copy)]
pub enum OpType {
    Arithmetic,
    Bitwise,
    Comparison,
    Increment,
    Logical,
    Assignment,
}

#[derive(Debug, Clone, Copy)]
pub enum RvalueType {
    Identifier,
    U64,
    I64,
    F64,
    Char,
    Hex,
    Octal,
    Binary,
    Str,
    FnCall,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprTypes {
    UnaryExpr,
    BinaryExpr,
    TernaryExpr,
}

macro_rules! gen_expr_strategy {
    // 1 argument (unary)
    ($op_type:expr, $a:expr) => {
        {
            let args = $a.prop_map(|a| vec![a]);
            args.prop_flat_map(|args_gen| compose_args_strategy(args_gen, $op_type))
        }
    };
    
    // 2 arguments (binary)
    ($op_type:expr, $a:expr, $b:expr) => {
        {
            let args = ($a, $b).prop_map(|(a, b)| vec![a, b]);
            args.prop_flat_map(|args_gen| compose_args_strategy(args_gen, $op_type))
        }
    };
    
    // 3 arguments (ternary)
    ($op_type:expr, $a:expr, $b:expr, $c:expr) => {
        {
            let args = ($a, $b, $c).prop_map(|(a, b, c)| vec![a, b, c]);
            args.prop_flat_map(|args_gen| compose_args_strategy(args_gen, $op_type))
        }
    };
}

fn gen_identifier_strategy() -> impl Strategy<Value = String> {
    pstring::string_regex("^[_a-zA-Z]([_a-zA-Z0-9])*").unwrap()
}

fn gen_hex_strategy() -> impl Strategy<Value = String> {
    pstring::string_regex("^0x[0-9A-F]+").unwrap()
}

fn gen_octal_strategy() -> impl Strategy<Value = String> {
    pstring::string_regex("^0[0-7]+").unwrap()
}

fn gen_binary_strategy() -> impl Strategy<Value = String> {
    pstring::string_regex("^0b[0-1]+").unwrap()
}

fn gen_string_literal() -> impl Strategy<Value = String> {
    let content = "[a-zA-Z0-9 ,.!?-]{0,20}";
    content.prop_map(|s: String| format!("\"{}\"", s))
}

fn wrap_parens_strategy(expr: String) -> impl Strategy<Value = String> {
    Just(format!("({})", expr)).boxed()
}

fn format_commas_strategy(args: Vec<String>) -> impl Strategy<Value = String> {
    Just(args.join(", ")).boxed()
}

fn compose_args_strategy(args: Vec<String>, op_type: OpType) -> impl Strategy<Value = String> {
    static ARITHMETIC: &'static [&str] = &["+", "-", "*", "/", "%"];
    static BITWISE: &'static [&str] = &["&", "|", "^"];
    static COMPARISON: &'static [&str] = &["<", ">"];
    static LOGICAL: &'static [&str] = &["&&", "||"];
    static ASSIGNMENT: &'static [&str] = &["=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="];
    static UNARY_ARITHMETIC: &'static [&str] = &["+", "-"];
    static UNARY_LOGICAL: &'static [&str] = &["!"];
    static UNARY_BITWISE: &'static [&str] = &["~"];
    static UNARY_ADDRESS: &'static [&str] = &["&", "*"];
    static UNARY_INCREMENT: &'static [&str] = &["++", "--"];

    let (binary_ops, unary_ops) = match op_type {
        OpType::Arithmetic => (ARITHMETIC, UNARY_ARITHMETIC),
        OpType::Bitwise => (BITWISE, UNARY_BITWISE),
        OpType::Comparison => (COMPARISON, &[]),
        OpType::Logical => (LOGICAL, UNARY_LOGICAL),
        OpType::Increment => (&[], UNARY_INCREMENT),
        OpType::Assignment => (ASSIGNMENT, &[]),
    };

    match args.as_slice() {
        [a] => {
            if !unary_ops.is_empty() {
                prop::sample::select(unary_ops)
                    .prop_map(move |op| format!("{}{}", op, a))
                    .boxed()
            } else {
                Just(a.clone()).boxed()
            }
        }
        [first, second] => {
            prop::sample::select(binary_ops)
                .prop_map(move |op| format!("({} {} {})", first, op, second))
                .boxed()
        }
        [first, rest @ ..] => {
            prop::sample::select(binary_ops)
                .prop_flat_map(move |op| {
                    compose_args_strategy(rest.to_vec(), op_type)
                        .prop_map(move |rest_str| format!("({} {} {})", first, op, rest_str))
                })
                .boxed()
        }
        [] => Just(String::new()).boxed(),
    }
}

fn gen_expression_call() -> impl Strategy<Value = String> {
    let identifier = gen_identifier_strategy();
    let quantity = pcollection::SizeRange::new(0..=10);
    let args = pcollection::vec(gen_identifier_strategy(), quantity)
        .prop_flat_map(|args_gen| format_commas_strategy(args_gen));

    (identifier, args)
        .prop_flat_map(|(id, args)| Just(format!("{}({})", id, args)))
}


fn gen_primitive_by_type(rtype: RvalueType) -> impl Strategy<Value = String> {
    match rtype {
        RvalueType::FnCall => gen_expression_call(),
        RvalueType::Identifier => gen_identifier_strategy(),
        RvalueType::U64 => any::<u64>().prop_map(|n| n.to_string()),
        RvalueType::I64 => any::<i64>().prop_map(|n| n.to_string()),
        RvalueType::Char => any::<char>().prop_map(|c| format!("'{}'", c)),
        RvalueType::Hex => gen_hex_strategy(),
        RvalueType::Octal => gen_octal_strategy(),
        RvalueType::Binary => gen_binary_strategy(),
        RvalueType::Str => gen_string_literal(),
    }
}

fn gen_binary_expr() -> impl Strategy<Value = String> {
    let rvalue_types = vec![ 
        RvalueType::Identifier,
        RvalueType::U64,
        RvalueType::I64,
        RvalueType::F64,
        RvalueType::Char,
        RvalueType::Hex,
        RvalueType::Octal,
        RvalueType::Binary,
        RvalueType::FnCall,
    ];

    // Pick left and right types independently
    let left_type = psample::select(rvalue_types.clone());
    let right_type = psample::select(rvalue_types);
    
    (left_type, right_type).prop_flat_map(|(left_rtype, right_rtype)| {
        let left = gen_primitive_by_type(left_rtype);
        let right = gen_primitive_by_type(right_rtype);
        
        // Determine allowed operations based on both types
        let is_left_numeric = matches!(left_rtype, 
            RvalueType::U64 | RvalueType::I64 | RvalueType::F64 | 
            RvalueType::Hex | RvalueType::Octal | RvalueType::Binary | 
            RvalueType::Char);
        let is_right_numeric = matches!(right_rtype,
            RvalueType::U64 | RvalueType::I64 | RvalueType::F64 | 
            RvalueType::Hex | RvalueType::Octal | RvalueType::Binary | 
            RvalueType::Char);
        
        let is_left_integer = matches!(left_rtype,
            RvalueType::U64 | RvalueType::I64 | 
            RvalueType::Hex | RvalueType::Octal | RvalueType::Binary);
        let is_right_integer = matches!(right_rtype,
            RvalueType::U64 | RvalueType::I64 | 
            RvalueType::Hex | RvalueType::Octal | RvalueType::Binary);
        
        let mut ops: Vec<OpType> = vec![];
        
        // Arithmetic: both numeric
        if is_left_numeric && is_right_numeric {
            ops.push(OpType::Arithmetic);
        }
        
        // Bitwise: both integer
        if is_left_integer && is_right_integer {
            ops.push(OpType::Bitwise);
        }
        
        // Comparison: any non-string (or same type)
        if !matches!(left_rtype, RvalueType::Str) && !matches!(right_rtype, RvalueType::Str) {
            ops.push(OpType::Comparison);
        }
        
        // Logical: any (in C, any non-zero is true)
        ops.push(OpType::Logical);
        
        if ops.is_empty() {
            // Fallback: just concatenate
            (left, right).prop_map(|(l, r)| format!("({}, {})", l, r)).boxed()
        } else {
            let op = psample::select(ops);
            op.prop_flat_map(move |op_type| {
                gen_expr_strategy!(op_type, left.clone(), right.clone())
            }).boxed()
        }
    })
}

fn gen_unary_expr() -> impl Strategy<Value = String> {
    let rvalue_types = vec![ 
        RvalueType::Identifier,
        RvalueType::U64,
        RvalueType::I64,
        RvalueType::F64,
        RvalueType::Char,
        RvalueType::Hex,
        RvalueType::Octal,
        RvalueType::Binary,
        RvalueType::Str,
        RvalueType::FnCall,
    ];

    psample::select(rvalue_types).prop_flat_map(|rtype| {
        let p = gen_primitive_by_type(rtype);

        match rtype {
            // Numbers → arithmetic, bitwise
            RvalueType::U64 | RvalueType::I64 | RvalueType::Hex | 
                RvalueType::Octal | RvalueType::Binary => {
                    prop_oneof![
                        gen_expr_strategy!(OpType::Arithmetic, p.clone()),
                        gen_expr_strategy!(OpType::Bitwise, p.clone()),
                    ]
                }
            // Float → arithmetic only
            RvalueType::F64 => {
                gen_expr_strategy!(OpType::Arithmetic, p)
            }
            // Char → arithmetic, bitwise
            RvalueType::Char => {
                prop_oneof![
                    gen_expr_strategy!(OpType::Arithmetic, p.clone()),
                    gen_expr_strategy!(OpType::Bitwise, p),
                ]
            }
            // Identifier/FnCall → everything
            RvalueType::Identifier | RvalueType::FnCall => {
                prop_oneof![
                    gen_expr_strategy!(OpType::Arithmetic, p.clone()),
                    gen_expr_strategy!(OpType::Logical, p.clone()),
                    gen_expr_strategy!(OpType::Bitwise, p.clone()),
                    gen_expr_strategy!(OpType::Comparison, p),
                ]
            }
            // String → just the string literal
            RvalueType::Str => {
                p  // Just the string literal, no operators
            }
        }
})
}

fn gen_expression_by_type(expr: ExprType) -> impl Strategy<Value = String> {
    match ExprType {
        ExprType::Unary => gen_unary_expr(),
        ExprType::Binary => gen_binary_expr(),
        ExprType::Ternary => gen_ternary_expr(),
    }
}

fn gen_expression_cexpr(quantity: impl Into<SizeRange>) -> impl Strategy<Value = String> {
    let quantity = quantity.into();
    let expr_type = vec![
        ExprTypes::UnaryExpr,
        ExprTypes::BinaryExpr,
        ExprTypes::TernaryExpr,
    ];
    
    let expression = proptest::collection::vec(
        psample::select(expr_type)
            .prop_flat_map(|expr| gen_expression_type(expr)),
    );
    
    gen_expr_strategy!(OpType::Arithmetic, primitives)
}

impl Arbitrary for ConstAssignment {
    type Parameters = ();
    type Strategy = impl Strategy<Value = Self>;

    fn arbitrary_with((): ()) -> Self::Strategy {
        let const_kw = Just("const".to_string());
        let type_kw = pstring::string_regex(
            "(int|char|float|double|void|"
            "unsigned int|unsigned char|unsigned short|unsigned long|unsigned long long|"
            "signed int|signed char|signed short|signed long|signed long long|"
            "short|long|long long)"
        ).unwrap();
        let identifier = gen_identifier_strategy();
        let rvalue_exp = gen_any_expression();
        
        (const_kw, type_kw, identifier, rvalue_exp)
            .prop_map(|(_const, _type, _identifier, _rvalue)| ConstAssignment {
                _const,
                _type,
                _identifier,
                _rvalue,
            })
    }
}


proptest! {
    #[test]
    fn test_shitty(s in ".*") {
        assert!(s.len() >= 0);
    }
}
