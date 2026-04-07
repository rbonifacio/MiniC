# MiniC Evolution Projects

This document proposes ten self-contained extension projects for the MiniC language.
Each project touches all three pipeline stages currently implemented— parser, type checker, and interpreter —
and requires you to reason carefully about syntax, typing rules, and runtime semantics.
They are designed to have a similar degree of complexity so that any project is a fair
choice regardless of which one you pick.


## Assignment of the Projects to the Groups

The allocation of projects to groups was performed randomly, using a reproducible procedure (i.e., with a fixed random seed). 

| Project     | Group |
|-------------|--------|
| Projeto 10  | Daniel Silvestre de França e Souza<br>João Pedro Barbosa Marins<br>Marcelo Arcoverde Neves Britto de Rezende<br>Rafael Paz Fernandes<br>Vinícius Lima Sá de Melo |
| Projeto 7   | Enzo Gurgel Bissoli<br>Shellyda de Fatima Silva Barbosa<br>Rodrigo Santos Batista<br>Juliana Serafim da Silva<br>Anderson Vitor Leoncio de Lima |
| Projeto 8   | Alberto Guevara de Araujo Franca<br>Davi Gonzaga Guerreiro Barboza<br>Fábio Pereira de Miranda<br>Felipe Torres de Macedo |
| Projeto 2   | Ian Medeiros Melo<br>Victor Mendonça Aguiar<br>Rafael Alves de Azevedo Silva<br>João Victor Fellows Rabelo<br>Guilherme Montenegro de Albuquerque |
| Projeto 5   | Álvaro Cavalcante Negromonte<br>Gabriel Valença Mayerhofer<br>Henrique César Higino Holanda Cordeiro<br>João Victor Nascimento Lima<br>Vinicius de Souza Rodrigues |
| Projeto 4   | Bruno Antonio dos Santos Bezerra<br>Luan de Oliveira Romancini Leite<br>Leônidas Dantas de Castro Netto<br>Pedro Gabriel Alves da Silva |

The following projects were not assigned.

   * Project 1
   * Project 3
   * Project 6
   * Project 9
   
---

## Project 1 — Struct Types

Add user-defined record types that group named fields under a single type name.

### Proposed syntax

```c
struct Point {
    int x;
    int y
}

void main() {
    Point p = Point { x: 0, y: 1 };
    p.x = 42;
    print(p.x)
}
```


### Learning objectives
- Understand how composite types are represented in an AST and a type environment.
- Experience the interplay between a type registry and the symbol table.
- Practice extending all three pipeline stages for a single new feature.

---

## Project 2 — For Loop

Add a traditional three-part `for` statement as a more expressive alternative to `while`.

### Proposed syntax

```c
void main() {
    int sum = 0;
    for (int i = 0; i < 10; i = i + 1) {
        sum = sum + i
    };
    print(sum)
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | `for (init; condition; update) body` where *init* is a declaration or assignment, *condition* is an expression, and *update* is an assignment |
| **Type checker** | The *condition* must have type `bool`; *init* introduces a scoped variable visible only inside the loop; *update* must be a well-typed assignment |
| **Interpreter** | Execute *init* once; evaluate *condition* before each iteration; execute *body* then *update* each iteration; clean up the loop-scoped variable on exit |

### Learning objectives
- Model scoping rules that are tighter than a plain block.
- Understand how desugaring a `for` into a `while` can simplify the interpreter at the cost of a less informative AST.
- Practise adding a new `Statement` variant without breaking existing code.

---

## Project 3 — Switch Statement

Add a multi-way branching construct that dispatches on the value of an integer or boolean expression.

### Proposed syntax

```c
void main() {
    int x = readInt();
    switch (x) {
        case 1: print("one");
        case 2: print("two");
        default: print("other")
    }
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | `switch (expr) { case literal: stmt; … default: stmt }` where each case label is an integer or boolean literal |
| **Type checker** | All case labels must share the same type as the switch expression; the `default` branch is mandatory; each branch body is type-checked independently |
| **Interpreter** | Evaluate the switch expression; find the first matching case label; execute its statement; if no case matches, execute `default`; detect duplicate labels as an error |

### Learning objectives
- Implement a new compound statement with a variable number of branches.
- Enforce exhaustiveness via a required `default` clause.
- Explore the trade-offs between a `switch` and a chain of `if`/`else if`.

---

## Project 4 — Extended String Standard Library

Enrich the built-in string support with a comprehensive set of operations and a string-concatenation operator.

### Proposed additions

```c
void main() {
    str greeting = "Hello, " ++ "world";  // ++ is the new concat operator
    print(len(greeting));                  // 13
    print(substr(greeting, 0, 5));         // "Hello"
    print(toUpper(greeting));              // "HELLO, WORLD"
    int n = strToInt("42");
    print(contains(greeting, "world"))     // true
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | New binary operator `++` for string concatenation with appropriate precedence |
| **Type checker** | `++` requires both operands to be `str` and returns `str`; add type signatures for new stdlib functions: `len(str) → int`, `substr(str, int, int) → str`, `toUpper(str) → str`, `toLower(str) → str`, `strToInt(str) → int`, `strToFloat(str) → float`, `contains(str, str) → bool` |
| **Interpreter** | Implement each stdlib function in Rust; register it in `NativeRegistry`; implement the `++` operator in the expression evaluator; add runtime errors for out-of-range `substr` indices and unparseable `strToInt` input |

### Learning objectives
- Practice extending the standard library following the existing `NativeRegistry` pattern.
- Add an operator to the parser without disturbing the existing precedence chain.
- Handle runtime errors gracefully at the interpreter level.

---

## Project 5 — First-Class Functions

Allow functions to be stored in variables, passed as arguments, and returned from other functions.

### Proposed syntax

```c
int apply(fn(int) -> int f, int x) {
    return f(x)
}

void main() {
    fn(int) -> int double = fn(int x) -> int { return x * 2 };
    print(apply(double, 21))   // 42
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | Function-type expressions `fn(T, …) -> T`; lambda expressions `fn(params) -> T body`; function-typed parameters and variable declarations |
| **Type checker** | New `Type::Fn(Vec<Type>, Box<Type>)` variant; type-check lambda bodies like regular functions; verify that function types match at call sites and assignments |
| **Interpreter** | Lambda values already partially exist as `Value::Fn(FnValue::UserDefined(…))`; capture the *creation-time* environment snapshot so that lambdas act as closures; call through a variable holding a function value |

### Learning objectives
- Understand the relationship between function types and first-class values.
- Implement lexical scoping (closures) by capturing environments.
- See how the type system must be extended when functions become just another type.

---

## Project 6 — Enum Declarations

Add user-defined enumeration types whose variants can be used as constants and compared for equality.

### Proposed syntax

```c
enum Direction { North, South, East, West }

Direction turn(Direction d) {
    switch (d) {
        case Direction::North: return Direction::East;
        case Direction::East:  return Direction::South;
        case Direction::South: return Direction::West;
        case Direction::West:  return Direction::North;
        default: return d
    }
}

void main() {
    Direction d = Direction::North;
    print(turn(d))
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | `enum Name { Variant, … }` top-level declarations; qualified name expressions `Name::Variant`; extend `switch` (or add a dedicated `match`) to support enum values as case labels |
| **Type checker** | New `Type::Enum(name)` variant; an enum registry mapping names to variant lists; check that `Name::Variant` belongs to the declared enum; ensure switch/match cases cover only valid variants of the matched type |
| **Interpreter** | New `Value::Enum { type_name, variant }` variant; compare enum values by variant name; print enum values as their qualified name |

### Learning objectives
- Learn how nominal types (where identity matters, not just structure) are represented.
- Understand exhaustiveness checking and how a compiler can warn about missing cases.
- Coordinate a new type registry with the existing symbol table.

---

## Project 7 — Constant Declarations

Add an immutable binding form that the type checker enforces, preventing any subsequent assignment.

### Proposed syntax

```c
const int MAX_SIZE = 100;
const float PI = 3.14159;

void main() {
    const int n = readInt();
    // n = 5  <-- type error: cannot assign to constant 'n'
    print(n * 2)
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | `const T name = expr` as a new statement form; constants should be allowed both at function scope and at the top level (as global constants) |
| **Type checker** | Track mutability alongside type in the symbol table (e.g., `(Type, Mutability)` pairs); reject any assignment whose left-hand side resolves to a `Const` binding; top-level constants are visible inside all functions |
| **Interpreter** | Constants are stored as ordinary values; enforcement is purely static — no runtime changes needed, but top-level constants must be evaluated before any function runs |

### Learning objectives
- Extend the symbol table to carry more than just type information.
- Distinguish between a type error that is purely static and one that requires runtime support.
- Reason about the evaluation order of top-level definitions.

---

## Project 8 — Built-in Unit Testing Framework

Add a first-class `test` declaration and an `assert` statement, and a `--test` CLI mode that runs all tests and reports results.

### Proposed syntax

```c
int add(int a, int b) { return a + b }

test "add returns correct sum" {
    assert add(2, 3) == 5;
    assert add(0, 0) == 0;
    assert add(-1, 1) == 0
}

test "add is commutative" {
    int x = 3;
    int y = 7;
    assert add(x, y) == add(y, x)
}

void main() { print("production code") }
```

Running `minic --test program.minic` should output:

```
PASS  add returns correct sum
PASS  add is commutative
2 passed, 0 failed
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | `test "name" { stmts }` as a new top-level declaration; `assert expr` as a new statement |
| **Type checker** | `assert` requires a `bool` expression; test bodies are type-checked like function bodies but with no return type; detect duplicate test names |
| **Interpreter / CLI** | Add a `--test` flag; in test mode, execute each test block, catch assertion failures (with a descriptive message including the test name), and print a summary; `void main()` is not invoked in test mode |

### Learning objectives
- See how a language can embed a testing discipline directly in its syntax.
- Implement a new execution mode that co-exists with the regular `--run` mode.
- Handle partial failure (some tests pass, others fail) and produce meaningful output.

---

## Project 9 — Multiple Return Values

Allow functions to return more than one value using a lightweight tuple type, with destructuring on the receiving side.

### Proposed syntax

```c
(int, int) divmod(int a, int b) {
    return (a / b, a - (a / b) * b)
}

void main() {
    (int q, int r) = divmod(17, 5);
    print(q);   // 3
    print(r)    // 2
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | Tuple-type expressions `(T, T, …)`; tuple-literal expressions `(expr, expr, …)`; destructuring declaration `(T name, T name, …) = expr` as a new statement |
| **Type checker** | New `Type::Tuple(Vec<Type>)` variant; check that the right-hand side of a destructuring declaration has a matching tuple type; check that a function whose return type is a tuple always returns a tuple of the right arity and element types |
| **Interpreter** | New `Value::Tuple(Vec<Value>)` variant; evaluate tuple literals; destructuring declarations unpack the tuple and bind each name in the current scope |

### Learning objectives
- Introduce a structural composite type (unlike the nominal `struct`).
- Implement destructuring as a statement that creates multiple bindings at once.
- Understand why tuple and struct types, though similar, have different identity rules.

---

## Project 10 — Pointer and Reference Types

Add reference types so that a variable can hold the address of another variable, enabling explicit mutation through a reference.

### Proposed syntax

```c
void increment(int* p) {
    *p = *p + 1
}

void main() {
    int x = 10;
    increment(&x);
    print(x)       // 11
}
```

### What to implement

| Stage | Work |
|-------|------|
| **Parser** | Pointer-type notation `T*`; address-of expression `&name`; dereference expression `*expr`; dereference assignment `*expr = expr` |
| **Type checker** | New `Type::Ptr(Box<Type>)` variant; `&name` has type `T*` when `name` has type `T`; `*expr` has type `T` when `expr` has type `T*`; only variables (not arbitrary expressions) may be addressed with `&` |
| **Interpreter** | Model references as *mutable handles* into the environment (e.g., wrap the variable name and environment reference); `&x` captures the binding; `*p` reads through it; `*p = v` writes back into the original variable's slot |

### Learning objectives
- Understand the distinction between a value and a location.
- Implement a form of aliasing and see how it complicates reasoning about mutation.
- Confront the fundamental challenge of pointers: the interpreter must track *where* a value lives, not just what it is.

---

## Choosing a Project

All ten projects require changes to the parser, the type checker, and the interpreter.
A good starting point for any of them is:

1. Write two or three example MiniC programs that exercise the new feature.
2. Decide how the new syntax will be represented as AST nodes.
3. Add the AST nodes and adjust the parser to produce them.
4. Extend the type checker to validate the new nodes.
5. Extend the interpreter to execute them.
6. Write unit tests at each stage (parser, type checker, interpreter) and end-to-end tests using the shelltest fixtures.


## Deadlines

### First Milestone: 

   * Review the concrete syntax (grammar) and the AST and Parser components.
   * Deadline: 19/04
   
### Second Milestone: 

   * Review the implementation of the type checker and interpreter
   * Deadline: 11/05

### Third Milestone: 

   * Review the implementation of the type code generator
   * Deadline: 30/06

The outcomes of the projects must be submitted via pull-requests.
