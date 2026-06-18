// This file is responsible for "generating" a type defined by ConstAssignment, providing an
// implementation of proptest's Arbitrary trait. The implementation is based on **combinators** —
// composing small, reusable strategy-building functions to create complex generators.
//
// An Arbitrary implementation implies the
// generation is "canonical" — it can be further customized if needed via Parameters.
//
// A valid instance of Arbitrary must provide at least the method `arbitrary_with`. Such method
// returns a `Strategy` type. The Strategy trait is what allows other types to be "generated". This
// library is heavily inspired by Haskell's QuickCheck, and the Strategy trait is equivalent to the
// Gen monad (a computation that produces random values).
//
// Any valid instance of Strategy — that is, any generated type — can be further "shrunk" during
// failed tests to find the minimal input that still reproduces the failure. This works by treating
// the generated attributes as dimensions in a search space, where each type defines its own
// "simpler" variants via the Shrink trait.
//
// The objective of shrinking is to reach a fixed point: a value that always fails the test and
// cannot be further shrunk (all simpler variants pass). If a shrunk value still fails, the
// shrinker continues; if it passes, the shrinker backtracks and tries a different path. The
// process is analogous to a multi-dimensional binary search, exploring a tree of possible
// simplifications until the minimal failing case is found.
//
// For complex types (like recursive C expressions), shrinking composes recursively: each
// sub-expression shrinks independently, and the shrinker explores combinations of simpler
// sub-values. Proptest's state machine tracks the current best failure and systematically tries
// all shrinking strategies defined by the Shrink implementations.
use proptest::prelude::*;
use std::ops::RangeInclusive;
use proptest::string as pstring;
use proptest::sample as psample;
use proptest::strategy::BoxedStrategy;


#[derive(Debug, Clone)]
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
    // Str,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprTypes {
    UnaryExpr,
    BinaryExpr,
    TernaryExpr,
    FuncallExpr,
}

/// Parameters for configuring ConstAssignment generation.
///
/// These parameters control the complexity and shape of generated const declarations:
/// - `param_complexity`: How many expressions are combined with operators (1 = single expression,
///   2+ = expressions composed with operators like +, -, &&, etc.)
/// - `nesting_complexity`: How deep expressions can nest (0 = primitives only, 3 = heavily nested)
/// - `modifier_kw`: Optional type modifier keyword (e.g., "const", "static"). If None, no keyword.
/// - `identifier`: Optional identifier name. If None, a random identifier is generated.
///
/// # Example
/// ```
/// // Generate simple declarations (no operators, no nesting)
/// let params = ConstAssignmentParams {
///     param_complexity: 1..=1,
///     nesting_complexity: 0..=0,
///     modifier_kw: Some("const".to_string()),
///     identifier: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConstAssignmentParams {
    pub param_complexity: RangeInclusive<usize>,
    pub nesting_complexity: RangeInclusive<usize>,
    pub modifier_kw: Option<String>,
    pub identifier: BoxedStrategy<String>,
}

impl Default for ConstAssignmentParams {
    fn default() -> Self {
        Self {
            modifier_kw: Some("const".to_string()),
            param_complexity: 1..=5,
            nesting_complexity: 0..=3,
            identifier: gen_identifier_strategy().boxed(),
        }
    }
}

/// Composes arguments with operators based on arity.
///
/// This macro takes an operator type and 1-3 argument strategies, then:
/// - Unary (1 arg): applies operators like `!`, `~`, `++`, `--`
/// - Binary (2 args): applies infix operators like `+`, `-`, `*`, `&&`, `||`
/// - Ternary (3 args): applies all three (though C doesn't have ternary operators that
///   take operators between them — this is for completeness)
///
/// The macro creates a strategy that generates strings like:
/// - Unary: `!x`, `~foo`, `++bar`
/// - Binary: `(a + b)`, `(x < y)`, `(foo && bar)`
/// - Ternary: N/A in C (would be `a ? b : c` but that's handled separately)
///
/// # Note
/// The actual operator selection and formatting is delegated to `compose_args_strategy`,
/// which handles operator precedence and parentheses.
macro_rules! gen_expr_strategy {
    ($op_type:expr, $a:expr) => {
        {
            let args = $a.prop_map(|a| vec![a]);
            args.prop_flat_map(move |args_gen| compose_args_strategy(args_gen, $op_type))
        }
    };
    
    ($op_type:expr, $a:expr, $b:expr) => {
        {
            let args = ($a, $b).prop_map(|(a, b)| vec![a, b]);
            args.prop_flat_map(move |args_gen| compose_args_strategy(args_gen, $op_type))
        }
    };
    
    ($op_type:expr, $a:expr, $b:expr, $c:expr) => {
        {
            let args = ($a, $b, $c).prop_map(|(a, b, c)| vec![a, b, c]);
            args.prop_flat_map(move |args_gen| compose_args_strategy(args_gen, $op_type))
        }
    };
}

fn gen_identifier_strategy() -> BoxedStrategy<String> {
    pstring::string_regex("[_a-zA-Z]([_a-zA-Z0-9])*").unwrap().boxed()
}

fn gen_hex_strategy() -> BoxedStrategy<String> {
    pstring::string_regex("0x[0-9A-Fa-f]+").unwrap().boxed()
}

fn gen_octal_strategy() -> BoxedStrategy<String> {
    pstring::string_regex("0[0-7]+").unwrap().boxed()
}

fn gen_binary_strategy() -> BoxedStrategy<String> {
    pstring::string_regex("0b[01]+").unwrap().boxed()
}

fn gen_string_literal() -> BoxedStrategy<String> {
    let content = "[a-zA-Z0-9 ,.!?-]{0,20}";
    content.prop_map(|s: String| format!("\"{}\"", s)).boxed()
}

fn compose_args_strategy(args: Vec<String>, op_type: OpType) -> BoxedStrategy<String> {
    static ARITHMETIC: &'static [&str] = &["+", "-", "*", "/", "%"];
    static BITWISE: &'static [&str] = &["&", "|", "^"];
    static COMPARISON: &'static [&str] = &["<", ">"];
    static LOGICAL: &'static [&str] = &["&&", "||"];
    static UNARY_ARITHMETIC: &'static [&str] = &["+", "-"];
    static UNARY_LOGICAL: &'static [&str] = &["!"];
    static UNARY_BITWISE: &'static [&str] = &["~"];
    static UNARY_INCREMENT: &'static [&str] = &["++", "--"];

    let (binary_ops, unary_ops) = match op_type {
        OpType::Arithmetic => (ARITHMETIC, UNARY_ARITHMETIC),
        OpType::Bitwise => (BITWISE, UNARY_BITWISE),
        OpType::Comparison => (COMPARISON, &[] as &[&str]),
        OpType::Logical => (LOGICAL, UNARY_LOGICAL),
        OpType::Increment => (&[] as &[&str], UNARY_INCREMENT),
        OpType::Assignment => (ARITHMETIC, &[] as &[&str]),
    };

    match args.len() {
        0 => Just(String::new()).boxed(),
        1 => {
            let a = args[0].clone();
            if !unary_ops.is_empty() {
                prop::sample::select(unary_ops)
                    .prop_map(move |op| format!("{}{}", op, a))
                    .boxed()
            } else {
                Just(a).boxed()
            }
        }
        _ => {
            let mut result: BoxedStrategy<String> = Just(args[0].clone()).boxed();
            for i in 1..args.len() {
                let next_val = args[i].clone();
                let op_strategy = prop::sample::select(binary_ops);
                result = result
                    .prop_flat_map(move |left| {
                        let next = next_val.clone();
                        op_strategy.clone()
                            .prop_map(move |op| format!("({} {} {})", left, op, next))
                    })
                    .boxed();
            }
            result
        }
    }
}

fn gen_primitive_by_type(rtype: RvalueType) -> BoxedStrategy<String> {
    match rtype {
        RvalueType::Identifier => gen_identifier_strategy().boxed(),
        RvalueType::U64 => any::<u64>().prop_map(|n| n.to_string()).boxed(),
        RvalueType::I64 => any::<i64>().prop_map(|n| n.to_string()).boxed(),
        RvalueType::F64 => any::<f64>().prop_map(|n| n.to_string()).boxed(),
        RvalueType::Char => {
            // ASCII only, no Unicode nightmares
            (32u8..127u8)  // Printable ASCII range
                .prop_map(|c| format!("'{}'", c as char))
                .boxed()
        }
        RvalueType::Hex => {
            // Limit hex to 64-bit range
            (0u64..=u64::MAX)
                .prop_map(|n| format!("0x{:x}", n))
                .boxed()
        }
        RvalueType::Octal => {
            // Limit octal to 64-bit range
            (0u64..=u64::MAX)
                .prop_map(|n| format!("0{:o}", n))
                .boxed()
        }
        RvalueType::Binary => {
            // Binary constants are C23 only, keep simple
            (0u64..=u64::MAX)
                .prop_map(|n| format!("0b{:b}", n))
                .boxed()
        }
        // RvalueType::Str => gen_string_literal(),
    }
}

fn gen_primitive_any() -> BoxedStrategy<String> {
    let rvalue_types = vec![
        RvalueType::Identifier,
        RvalueType::U64,
        RvalueType::I64,
        RvalueType::F64,
        RvalueType::Char,
        RvalueType::Hex,
        RvalueType::Octal,
        RvalueType::Binary,
        // RvalueType::Str,
    ];
    

    psample::select(rvalue_types)
        .prop_flat_map(move |rtype| gen_primitive_by_type(rtype))
        .boxed()
}

fn gen_expression_core(depth: u32, ) -> BoxedStrategy<String> {
    if depth == 0 {
        return gen_primitive_any();
    }
    
    let expr = prop_oneof![
        gen_primitive_any(),
        gen_unary_expr(depth - 1),
        gen_binary_expr(depth - 1),
        gen_ternary_expr(depth - 1),
        gen_funcall_expr(depth - 1),
    ];

    prop_oneof![
        expr.clone(),
        expr.prop_map(|e| format!("({})", e)),
    ].boxed()
}

fn gen_funcall_expr(depth: u32, ) -> BoxedStrategy<String> {
    let name = gen_identifier_strategy();
    let num_args = 0..=5;
    
    // Generate each argument as either expression OR string literal
    let args_strat = proptest::collection::vec(
        prop_oneof![
            gen_expression_core(depth ).boxed(),
            gen_string_literal().boxed(),
        ],
        num_args,
    );
    
    (name, args_strat)
        .prop_map(|(n, a): (String, Vec<String>)| format!("{}({})", n, a.join(", ")))
        .boxed()
}

fn gen_ternary_expr(depth: u32 ) -> BoxedStrategy<String> {
    let cond = gen_expression_core(depth);  // Just a single value, no operators!
    let then_expr = gen_expression_core(depth);
    let else_expr = gen_expression_core(depth);
    
    (cond, then_expr, else_expr)
        .prop_map(|(c, t, e)| format!("{} ? {} : {}", c, t, e))
        .boxed()
}

fn gen_binary_expr(depth: u32) -> BoxedStrategy<String> {
    let left = gen_expression_core(depth);
    let right = gen_expression_core(depth);
    let op_type = psample::select(vec![
        OpType::Arithmetic,
        OpType::Bitwise,
        OpType::Comparison,
        OpType::Logical,
    ]);
    
    // Clone left and right before moving into the closure
    let left_clone = left.clone();
    let right_clone = right.clone();
    
    op_type.prop_flat_map(move |ot| {
        gen_expr_strategy!(ot, left_clone.clone(), right_clone.clone())
    }).boxed()
}

fn gen_unary_expr(depth: u32,) -> BoxedStrategy<String> {
    // Don't wrap operand here
    let operand = gen_expression_core(depth);
    let op_type = psample::select(vec![
        OpType::Arithmetic, 
        OpType::Logical,    
        OpType::Bitwise,    
        OpType::Increment,  
    ]);
    
    let operand_clone = operand.clone();
    
    op_type.prop_flat_map(move |ot| {
        // Wrap the RESULT of the macro in parentheses
        let expr = gen_expr_strategy!(ot, operand_clone.clone());
        expr.prop_map(|e| format!("({})", e))  // ← Add parentheses HERE
    }).boxed()
}

fn gen_expression_by_type(expr_type: ExprTypes,nesting_complexity: RangeInclusive<usize>) -> BoxedStrategy<String> {
    // Use 1..=3u32 to avoid type mismatch
    (nesting_complexity).prop_flat_map(move |depth| {
        let depth = depth as u32;
        match expr_type {
            ExprTypes::UnaryExpr => gen_unary_expr(depth),
            ExprTypes::BinaryExpr => gen_binary_expr(depth),
            ExprTypes::TernaryExpr => gen_ternary_expr(depth),
            ExprTypes::FuncallExpr => gen_funcall_expr(depth),
        }
    }).boxed()
}

fn gen_expression_cexpr(nesting_complexity: RangeInclusive<usize>) -> BoxedStrategy<String> {
    let expr_types = vec![
        ExprTypes::UnaryExpr,
        ExprTypes::BinaryExpr,
        ExprTypes::TernaryExpr,
        ExprTypes::FuncallExpr,
    ];

    psample::select(expr_types)
        .prop_flat_map(move |expr_type| gen_expression_by_type(expr_type,nesting_complexity.clone()))
        .boxed()
}

/// Top-level expression generator for C.
///
/// Generates valid C expressions with the following constraints:
/// - 1 expression: can be any Unary, Binary, Ternary, Funcall, OR a String literal
/// - 2-5 expressions: combined with operators (Arithmetic, Bitwise, Comparison, Logical)
/// - Strings do NOT compose with operators (C doesn't support `"hello" + "world"`)
/// - Nesting depth is controlled by `gen_expression_cexpr` (0-3 levels deep)
///
/// # Why Strings are special
/// In C, string literals are arrays, not first-class values. They cannot be used with
/// arithmetic or logical operators. They are only allowed:
/// - As standalone expressions (rare but valid)
/// - As function arguments (handled in `gen_funcall_expr`)
///
/// # Expression composition
/// When multiple expressions (2-5) are generated, they are combined left-to-right with
/// randomly selected operators, each sub-expression wrapped in parentheses to enforce
/// explicit precedence.
///
/// # Shrinking behavior
/// Proptest will automatically shrink this generator by:
/// 1. Reducing the number of combined expressions (5 → 4 → 3 → 2 → 1)
/// 2. Simplifying operators (e.g., `&&` → `&` → `+` → etc.)
/// 3. Replacing complex sub-expressions with simpler primitives
/// 4. Eventually reaching a single primitive value
fn gen_any_expression(param_complexity: RangeInclusive<usize>,nesting_complexity: RangeInclusive<usize>) -> BoxedStrategy<String> {
    let exprs = proptest::collection::vec(
        gen_expression_cexpr(nesting_complexity),
        param_complexity,
    );

    exprs.prop_flat_map(|exprs: Vec<String>| {
        if exprs.len() == 1 {
            // Return BoxedStrategy<String>, not Flatten<...>
            return prop_oneof![
                Just(exprs[0].clone()).boxed(),
                gen_string_literal().boxed(),
            ].boxed();
        }
        
        let op_type = psample::select(vec![
            OpType::Arithmetic,
            OpType::Bitwise,
            OpType::Comparison,
            OpType::Logical,
        ]);

        op_type.prop_flat_map(move |ot| {
            compose_args_strategy(exprs.clone(), ot)
        }).boxed()
    }).boxed()
}

impl Arbitrary for ConstAssignment {
    type Parameters = ConstAssignmentParams;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        let const_kw = match &args.modifier_kw {
            Some(kw) => Just(kw.clone()).boxed(),
            None => Just("".to_string()).boxed(),
        };
         let type_kw = proptest::sample::select(vec![
            "int",
            "char", 
            "float",
            "double",
            "void",
            "short",
            "long",
            "long long",
            "unsigned int",
            "unsigned char",
            "unsigned short",
            "unsigned long",
            "unsigned long long",
            "signed int",
            "signed char",
            "signed short",
            "signed long",
            "signed long long",
         ]).prop_map(|s| s.to_string());
         let identifier = args.identifier;
         let rvalue_exp = gen_any_expression(
             args.param_complexity,
             args.nesting_complexity,
         );
        
        (const_kw, type_kw, identifier, rvalue_exp)
            .prop_map(|(_const, _type, _identifier, _rvalue)| ConstAssignment {
                _const,
                _type,
                _identifier,
                _rvalue,
            })
            .boxed()
    }
}
