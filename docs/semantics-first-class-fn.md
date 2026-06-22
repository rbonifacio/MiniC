# Semântica: Funções de Primeira Classe e Closures (Milestone 2)

Objetivo curto
- Formalizar como funções passam a ser valores em MiniC, qual a representação em runtime e a semântica de captura (closures).

Decisão principal
- Captura por snapshot no momento da criação: uma lambda carrega uma cópia do *ambiente de execução* (snapshot do `Environment`) quando é criada. Ao chamar a closure, o intérprete restaura esse ambiente capturado, vincula os parâmetros e executa o corpo.

Representação sugerida (estrutura em Rust-like, acho que era assim que o professor fez)
```
// representação conceitual
struct ClosureValue {
    params: Vec<Param>,                 // nomes + tipos
    return_type: Type,
    body: Statement,                    // AST do corpo
    captured_env: Environment<Value>,   // snapshot por criação
}

// em runtime stored as
// Value::Fn(FnValue::Closure { decl, captured })
```

O que é capturado
- Capturamos um *snapshot completo* do `Environment` (mapa nome → `Value`) no momento da criação da lambda. Isso significa:
  - variáveis livres ficam com o valor que tinham na criação;
  - a closure não observa mudanças subsequentes em bindings fora do seu escopo local.

Semântica da chamada (ideia em pseudocódigo)
```
fn call_closure(closure, args, env) {
    // 1. snapshot do caller
    caller_snapshot = env.snapshot()

    // 2. entra no ambiente capturado (lexical scope)
    env.restore(closure.captured_env.clone())

    // 3. bind dos parâmetros com os argumentos (sobrepondo nomes capturados)
    for (param, arg) in zip(closure.params, args) {
        env.declare(param.name.clone(), arg)
    }

    // 4. executar o corpo
    result = exec_stmt(&closure.body, env)

    // 5. restaurar ambiente do caller
    env.restore(caller_snapshot)

    return result.unwrap_or(Value::Void)
}
```

Decisão sobre declarações sem inicializador
- Não declarar uma variável de tipo função sem init (a não ser que queiram mais trabalho, ai só avisar no grupo pra gente rever isso).

Invariantes de tipo
- Uma `Expr::Lambda` tem tipo `Type::Fun(param_types, Box::new(return_type))` no type checker.
- `types_compatible` deve comparar aridade e compatibilidade por posição de `Type::Fun`.

Mensagens de erro (sugeridas)
- Chamar algo não-função: `RuntimeError: 'X' is not a function` ou `attempting to call a non-function value`.
- Número de argumentos errado: `function 'f' expects N arguments, got M`.
- Atribuição de tipo inválida: `assignment to f: expected fn(...)->..., got ...`.

Testes obrigatórios (mínimos) — mapeados
- Pessoa 2 (Sistema de Tipos):
  - `fn(int) -> int f;` — válido
  - `fn(float) -> int f; f = fn(int x) -> int { return x; }` — inválido (TypeError)

- Pessoa 3 (Lambdas / Chamadas — type-checker):
  - `fn(int x) -> int { return x * 2; }` — type-check OK
  - `f(true)` com `f: fn(int)->int` — inválido (TypeError)

- Pessoa 4 (Runtime: função como valor):
  - declarar `f = fn(int x) -> int { return x * 2 }; print(f(21));` → saída `42`

- Pessoa 5 (Closures):
  - `int y = 10; fn(int)->int f = fn(int x)->int { return x + y; }; y = 20; print(f(1));` → saída `11` (captura por snapshot)

---

# TAC: contrato do Milestone 3

Objetivo
- Fechar a representação de funções de primeira classe no TAC sem misturar chamada direta, valor funcional e chamada indireta.

Novos Address
- `Address::FunctionLabel(String)` para representar o código de uma função/lambda como valor.

Novas instruções
- `Instruction::CallIndirect(Option<Address>, Address, usize)` para chamadas via variável ou expressão que produza um valor funcional.

Decisão de geração
- `Expr::Lambda` gera uma função interna com nome único, por exemplo `lambda_1`.
- O valor da lambda é `Address::FunctionLabel("lambda_1")`.
- `Expr::CallExpr` usa `CallIndirect` quando o callee não é um nome direto de função.
- Chamada direta continua usando `Instruction::Call`.

Exemplos TAC
```text
fn(int) -> int double = fn(int x) -> int { return x * 2; }

JMP Label1:
Label lambda_1:
  temp1 = x * 2
  return temp1
Label1:
  double = lambda_1

double(21)

param 21
call_indirect double
```

Integração
- A Pessoa 1 valida consistência entre AST, type checker, interpreter e TAC.
- A Pessoa 1 também revisa PRs para garantir que o modelo de `FunctionLabel` e `CallIndirect` permaneça único em todo o grupo.
