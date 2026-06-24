# MiniC + Nom

Este guia introduz o projeto MiniC da disciplina **sem assumir experiência prévia com Rust** ou **com Nom**.

Tem um objetivo: levar você de "sei o que são combinadores de parsing em teoria" até "consigo ler e estender essa implementação real de linguagem em Rust".

Use este documento como seu ponto de partida e consulte as referências vinculadas quando precisar de ajuda.

Referências rápidas:
- Rust Book: <https://doc.rust-lang.org/book/>
- Documentação da crate Nom: <https://docs.rs/nom/latest/nom/>
- Guias do Nom: <https://github.com/rust-bakery/nom/tree/main/doc>


## O que é MiniC

MiniC é uma pequena linguagem similar a C implementada em Rust para aprender construção de compiladores.

Um programa MiniC é uma lista de declarações de funções:

```c
int factorial(int n) {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}

void main() {
  int result = factorial(10);
  print(result);
}
```

Pipeline do MiniC:
1. Fazer parsing do código-fonte para uma AST.
2. Verificar tipos da AST.
3. Interpretar (executar) a AST verificada.

Por que isso importa pedagogicamente:
- Cada etapa tem uma responsabilidade clara.
- Você pode testar cada etapa independentemente.
- O trabalho de extensão é estruturado e previsível.

Para mais informações sobre a linguagem e o pipeline, veja [docs/01-language.md](01-language.md) e [docs/02-pipeline.md](02-pipeline.md).

## Parte A: Introdução Rápida a Rust (Apenas o Necessário)

Você **não** precisa de todo o livro de Rust para trabalhar em MiniC. Você só precisa de um pequeno subconjunto.

### 1) Variáveis e Funções

Estrutura de função em Rust:

```rust
fn add(x: i64, y: i64) -> i64 {
    x + y
}
```

- `fn` começa uma função.
- `x: i64` é o nome do parâmetro e tipo.
- `-> i64` é o tipo de retorno.
- A última expressão sem `;` é retornada.

(Rust Book: <https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html>)

### 2) Structs (Dados com Campos Nomeados)

```rust
struct Example {
    x: i64,
    y: i64,
}
```

(Rust Book: <https://doc.rust-lang.org/book/ch05-00-structs.html>)

### 3) Enums (Uma de Várias Variantes)

Enums em Rust têm três estilos principais de variante:
- Tuple-like (`Name(T1, T2)`): campos posicionais, útil quando a ordem importa (ex.: operandos).
- Struct-like (`Name { f1: T1, f2: T2 }`): campos nomeados, mais explícito e resistente à reordenação.
- Unit-like (`Name`): etiqueta sem dados, para estados/flags.

```rust
enum Example {
    Tuple(i64, i64),           // tuple-like (posicional)
    Record { x: i64, y: i64 }, // struct-like (campos nomeados)
    Unit,                      // unit-like (sem dados)
}
```

(Rust Book: <https://doc.rust-lang.org/book/ch06-00-enums.html>)

### 4) match (Ramificação por Variante)

Use `match` ou `if let` para desestruturar enums:

```rust
match some_enum {
    Expr::Tuple(a, b) => println!("posicional: {} {}", a, b),
    Expr::Record { x, .. } => println!("campo x = {}", x),
    Expr::Unit => println!("unit"),
}
```

O compilador garante que o `match` seja exaustivo sobre todas as possibilidades do enum. Para ter um caso "catch-all", podemos usar `_ => {}` como o último match.

(Rust Book: <https://doc.rust-lang.org/book/ch06-02-match.html>)

### 5) Box para Árvores Recursivas

Tipos recursivos (como nós de expressão) precisam de `Box` para que o compilador saiba calcular seu tamanho:

```rust
enum Expr {
    Int(i64),
    Add(Box<Expr>, Box<Expr>),
}

// Construindo um nó Add:
let left = Expr::Int(1);
let right = Expr::Int(2);
let add = Expr::Add(Box::new(left), Box::new(right));

// Desestruturando com `match`:
match expr_example {
    Expr::Add(box Expr::Int(l), box Expr::Int(r)) => {
        println!("Add node: {} + {} = {}", l, r, l + r);
    }
    _ => {}
}
```

(Rust Book: <https://doc.rust-lang.org/book/ch15-01-box.html>)

### 6) Result para Tratamento de Erros

`Result<T, E>` significa um de dois casos:
- `Ok(T)` sucesso
- `Err(E)` falha

Ao chamar uma função que retorna `Result`, podemos:
- Desempacotá-lo para lidar com ambos os casos, ou
- Propagar o erro caso a função atual também retorne `Result`. 

```rust
fn parse_to_number(input: &str) -> Result<i32, std::num::ParseIntError> {
    input.parse::<i32>()
}

// chamando e lidando com o resultado explicitamente
fn try_parse_number() {
    match parse_number("42") {
        Ok(n) => println!("numero: {}", n),
        Err(e) => eprintln!("erro ao parsear: {}", e),
    }
}

// propagando erros com `?`
fn try_double(s: &str) -> Result<i64, String> {
    let n = parse_number(s)?;
    Ok(n * 2)
}
```

(Rust Book: <https://doc.rust-lang.org/book/ch09-00-error-handling.html>)

### 7) Generics

Generics são parâmetros de tipo: permitem escrever uma definição uma única vez e instanciá-la com diferentes tipos.

No MiniC, a [AST](../src/ir/ast.rs#L214) usa `Ty` como o tipo genérico nos nós. O parser produz `ExprD<()>` (sem tipos) e o type-checker produz `ExprD<Type>` (cada nó carrega seu `Type`).

(Rust Book: <https://doc.rust-lang.org/book/ch10-00-generics.html>)

### 8) Macros `#[derive(...)]`

Na [AST](../src/ir/ast.rs), muitas structs/enums usam `#[derive(...)]`. Derives geram código boilerplate automaticamente durante a compilação.

- `Debug`: permite imprimir o nó para debugging/tests (`{:?}`).
- `Clone`: gera uma implementação automática de `clone()` para copiar nós quando necessário.
- `PartialEq`/`Eq`: permitem comparar nós (útil em testes e transformações).
- `Hash`: permite usar o valor como chave em `HashMap`/`HashSet`.

(Rust Book: <https://doc.rust-lang.org/book/appendix-03-derivable-traits.html>)

### 9) Ownership e Borrowing

Breve resumo das regras essenciais do Rust aplicáveis ao projeto:

- O borrow-checker é um verificador em tempo de compilação que evita dangling references e condições de corrida sem custo em tempo de execução.
- Cada valor tem um dono. Quando o dono sai de escopo, o valor é liberado.
- `&T` e `&mut T` são empréstimos (borrows). O *borrow-checker* garante que essas referências não ultrapassem o tempo de vida do dono e impede usos concorrentes inválidos.
- Use `Box<T>` para tipos recursivos (p.ex., nós de AST).
- Prefira `&str` para views de string e faça `clone()` só quando necessário.

(Rust Book: <https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html>)

## Parte B: Introdução Rápida a Nom (Apenas o Necessário)

Nom é uma biblioteca de combinadores de parsers para Rust.

Conforme a documentação do Nom, a forma central de um parsing é:

```rust
fn parser(input: I) -> IResult<I, O>
```

Para MiniC, tipicamente:
- tipo de entrada: `&str`
- tipo de saída: fragmento de AST

(Visão geral do Nom: <https://docs.rs/nom/latest/nom/>, guia "fazendo um novo parser": <https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md>)

### 1) O que IResult Significa

`IResult<I, O>` é essencialmente:
- `Ok((remaining_input, output_value))`
- ou `Err(...)`

Então um parser retorna ambos:
1. O que foi parseado.
2. O que restou sem parsear.

(Referência: <https://docs.rs/nom/latest/nom/type.IResult.html>)

Modelo de erro do Nom (importante ao debugar comportamento de parsing): 
- `Err::Error` é recuperável (então `alt` pode tentar outro branch)
- `Err::Failure` é irrecuperável (branch confirmado)
- `Err::Incomplete` significa que mais entrada é necessária em modo streaming (mas nunca ocorre em modo complete). 

O combinator `cut` muda erros recuperáveis para falhas quando você sabe que está no branch correto. Veja <https://github.com/rust-bakery/nom/blob/main/doc/error_management.md> e <https://docs.rs/nom/latest/nom/combinator/fn.cut.html>.

### 2) Complete vs Streaming

Nom tem variantes `complete` e `streaming`.

MiniC usa parsers `complete` porque arquivos de código estão totalmente disponíveis na memória.

Conforme a documentação do Nom:
- streaming pode retornar `Incomplete` para buffers parciais.
- complete trata dados faltantes como um erro.

Para parsing de linguagem baseado em arquivo, complete é o padrão certo.

(Referência e exemplos: <https://docs.rs/nom/latest/nom/#streaming--complete>)

### 3) Combinators Principais Usados em MiniC

- `tag("if")`: correspondência exata de string.
- `char('(')`: correspondência exata de caractere.
- `alt((a, b, c))`: tenta parsers em ordem, aplica o primeiro que funcionar.
- `tuple((a, b, c))`: aplica parsers em sequência.
- `preceded(a, b)`: retorna `b` se for precedido por `a`. Descarta `a`. Usado para descartar whitespace.
- `delimited(a, b, c)`: retorna `b` se estiver entre `a` e `c`. Descarta `a` e `c`. Usado para encontrar "{}", "()" e "[]".
- `opt(p)`: pode ou não estar presente (p?).
- `many0(p)`: se repete zero ou mais vezes (p*).
- `many1(p)`: se repete um ou mais vezes (p+).
- `separated_list0(sep, item)`: uma lista de `item` separada por `sep`. (Ex. parâmetros de função)
- `verify(p, pred)`: verifica a condição `pred` sobre o resultado do parser `p`.
- `map(p, f)`: aplica `f` sobre o resultado do parser `p`. Usado para transformar a saída dos parsers em nós da AST.

Quando não tiver certeza qual combinator usar, consulte o guia de escolha: <https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md>.

### 4) Comportamentos Importantes do Nom

#### Sucesso do parser não implica consumo completo

Um parser em Nom retorna `Ok((rest, value))`. Mesmo quando `value` foi reconhecido com sucesso, parte da entrada pode sobrar em `rest`. Se você precisa garantir que o parser consuma toda a entrada (útil nos testes unitários), envolva-o com `all_consuming`:

```rust
let all = all_consuming(tag("str"));
assert!(all("str").is_ok());      // consome tudo
assert!(all("struct").is_err());  // sobra entrada -> erro
```

(Nom:`all_consuming` <https://docs.rs/nom/latest/nom/combinator/fn.all_consuming.html>)

#### alt é ordenado

`alt((a, b, c))` tenta da esquerda para a direita e retorna o primeiro sucesso. A ordem dos branches afeta diretamente comportamento da linguagem.

Exemplo onde a ordem importa (um branch é prefixo de outro):

```rust
let parser_wrong = alt((tag("str"), tag("struct")));
assert_eq!(parser_wrong("struct"), Ok(("uct", "str")));
// branch curto primeiro -> escolhe "str" e deixa "uct" sobrando

let parser_right = alt((tag("struct"), tag("str")));
assert_eq!(parser_right("struct"), Ok(("", "struct")));
// branch longo primeiro -> escolhe "struct" como esperado
```

(Nom: `alt` <https://docs.rs/nom/latest/nom/branch/fn.alt.html>)

## Parte C: Arquitetura do Parser de MiniC

Módulos do parser são divididos por categoria de gramática.

Veja notas sobre arquitetura do parser em [docs/04-parser.md](04-parser.md), depois compare diretamente com a implementação.

### 1) [Identificadores](../src/parser/identifiers.rs)

Nomes de variáveis, funções, etc. Rejeita palavras-chave reservadas com `verify`.

(Nom `verify`: <https://docs.rs/nom/latest/nom/combinator/fn.verify.html>)

### 2) [Literais](../src/parser/literals.rs)

Faz parsing de int, float, string, bool. Parser de string suporta escapes (`\\`, `\"`, `\n`, `\t`).

(Nom: `escaped_transform` <https://docs.rs/nom/latest/nom/bytes/complete/fn.escaped_transform.html>)

### 3) [Expressões (Precedência + Associatividade)](../src/parser/expressions.rs)

- ou lógico (precedência mais baixa)
- e lógico
- não
- relacional
- aditivo
- multiplicativo
- unário
- primário
- atômico (precedência mais alta)

MiniC codifica precedência de operadores via camadas de função. Cada camada chama a camada anterior quando precisa de um operando. **Todos os operadores binários de MiniC são associativos à esquerda**, implementados com um loop de acumulador em cada nível: parseia o operando esquerdo, depois enquanto o operador esperado não falha, parseia o operador e o operando direito (que se torna o novo operando esquerdo).

Exemplo: `1 - 2 - 3` se torna `(1 - 2) - 3` porque o primeiro `2` é consumido como direito, o resultado `(1 - 2)` vira o novo esquerdo, e `3` é consumido como novo direito.

### 4) [Declarações](../src/parser/statements.rs)

- bloco
- if 
- while
- return
- declaração de variável
- chamada de função
- atribuição

A ordem é deliberada, especialmente para formas com prefixos sobrepostos.

### 5) [Funções e Tipos](../src/parser/functions.rs)

Declaração de função e tipos. 

Parser de tipo inclui formas escalares e de array. Porque `alt` é ordenado, prefixos mais longos (como formas de array 2D) são listados listados antes dos mais curtos (formas 1D).

### 6) [Parser de Programa](../src/parser/program.rs)

Parser de declarações top-level. Usa repetição sobre declarações de função.

## Parte D: AST, Verificação de Tipos e Interpretação

### 1) Design da AST

Um nó da AST é uma unidade da árvore que representa uma construção sintática (ex.: expressão, declaração) e agrupa os campos necessários para representá-la.

Nós seguem o padrão `Object<Ty>` + `ObjectD<Ty>`:
- `Object<Ty>` descreve a forma de um objeto.
- `ObjectD<Ty>` agrupa o objeto com o tipo que ele carrega para execução.

Na prática: o parser produz `ObjectD<()>` (sem tipos) e o type-checker produz `ObjectD<Type>` (com tipos). Isso reaproveita a mesma forma estrutural entre fases sem duplicação.

Arquivos: [docs/03-ast.md](03-ast.md), [src/ir/ast.rs](../src/ir/ast.rs).

### 2) Responsabilidades do Verificador de Tipos

Verificação de tipos valida:
- Assinatura `main` obrigatória
- Tipos de declaração e atribuição
- Contagem/tipos de argumento de chamada de função
- Digitação de operador de expressão
- Indexação de array e consistência de elementos
- Correção de tipo de retorno

Usa um ambiente mapeando nomes para tipos.

Arquivos: [docs/05-type-checker.md](05-type-checker.md), [src/semantic/type_checker.rs](../src/semantic/type_checker.rs).

### 3) Responsabilidades do Interpretador

Interpretador executa AST verificada:
- Avaliação de expressão em valores em tempo de execução
- Execução de declaração (incluindo fluxo de controle)
- Chamadas de função (definidas pelo usuário e nativas)
- Erros em tempo de execução (ex., fora dos limites)

Usa um ambiente mapeando nomes para valores em tempo de execução.

Arquivos: [docs/06-interpreter.md](06-interpreter.md), [src/interpreter/eval_expr.rs](../src/interpreter/eval_expr.rs), [src/interpreter/exec_stmt.rs](../src/interpreter/exec_stmt.rs).

## Parte E: Como Adicionar Funcionalidades

Para cada nova funcionalidade de linguagem, o ideal é fazer nessa ordem:
1. Estender AST.
2. Estender parser.
3. Adicionar testes de parser/programa.
4. Estender type-checker.
5. Adicionar testes de type-checker.
6. Estender interpretador.
7. Adicionar testes de interpretador.
8. Verificar testes de stdlib e CLI
8. Atualizar docs.

Para adição de funcionalidades que impactam nas funções builtin em tempo de execução, consulte [docs/07-stdlib.md](07-stdlib.md) e [src/stdlib/mod.rs](../src/stdlib/mod.rs).

## Parte F: Estratégia de Teste Que Você Deveria Seguir

Camadas de teste de MiniC:
- Testes do parser / programa
- Testes do type-checker
- Testes do interpretador
- Testes CLI

Regra prática:
- em cada camada, pelo menos um teste unitário para cada regra de funcionamento da funcionalidade adicionada
- um teste CLI para cada comportamento visível ao usuário

Arquivos:
- [tests/parser.rs](../tests/parser.rs)
- [tests/program.rs](../tests/program.rs)
- [tests/type_checker.rs](../tests/type_checker.rs)
- [tests/interpreter.rs](../tests/interpreter.rs)
- [tests/stdlib.rs](../tests/stdlib.rs)
- [tests/cli](../tests/cli)

Detalhes de estratégia de teste e convenções shelltest são documentados em [docs/08-testing.md](08-testing.md).
