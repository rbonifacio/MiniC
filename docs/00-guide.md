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
1. Fazer parsing do código-fonte em uma AST.
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
struct Point {
    x: i64,
    y: i64,
}
```

MiniC usa structs como wrappers da AST (nós de expressão com metadados).

(Rust Book: <https://doc.rust-lang.org/book/ch05-00-structs.html>)

### 3) Enums (Uma de Várias Variantes)

```rust
enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
}
```

Uma enum pode conter diferentes formas sob um tipo. Isso é central para AST e valores em tempo de execução.

(Rust Book: <https://doc.rust-lang.org/book/ch06-00-enums.html>)

### 4) match (Ramificação por Variante)

```rust
match value {
    Value::Int(n) => println!("int: {}", n),
    Value::Bool(b) => println!("bool: {}", b),
    Value::Str(s) => println!("str: {}", s),
}
```

Verificação de tipos e interpretação de MiniC são principalmente grandes instruções `match` explícitas.

(Rust Book: <https://doc.rust-lang.org/book/ch06-02-match.html>)

### 5) Result para Tratamento de Erros

```rust
fn parse_number(s: &str) -> Result<i64, String> {
    s.parse::<i64>().map_err(|e| e.to_string())
}
```

`Result<T, E>` significa um de dois casos:
- `Ok(T)` sucesso
- `Err(E)` falha

Parser, verificador de tipos e interpretador de MiniC todos dependem desse estilo.

(Rust Book: <https://doc.rust-lang.org/book/ch09-00-error-handling.html>)

### 6) Box para Árvores Recursivas

Enums recursivas precisam de indireção:

```rust
enum Expr {
    Int(i64),
    Add(Box<Expr>, Box<Expr>),
}
```

Sem `Box`, Rust não consegue determinar o tamanho recursivo em tempo de compilação.

(Rust Book: <https://doc.rust-lang.org/book/ch15-01-box.html>)

### 7) Generics (Parametrização da AST através de Fases)

Nós da AST de MiniC são genéricos sobre um parâmetro de tipo `Ty` que carrega metadados específicos de cada fase. A mesma estrutura de árvore é usada após parsing e após type-checking:
- Imediatamente após parsing: `Ty = ()` (sem informação de tipo)
- Após type-checking: `Ty = Type` (com informação de tipo para cada sub-expressão)

Esse design previne acidentalmente misturar AST verificada com AST não verificada em tempo de compilação, pois são tipos diferentes (`Expr<()>` vs `Expr<Type>`).

(Rust Book: generics <https://doc.rust-lang.org/book/ch10-00-generics.html>, ownership and borowing <https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html>)

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

É por isso que a composição de parsers funciona naturalmente.

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
- `alt((a, b, c))`: tenta alternativas em ordem.
- `tuple((a, b, c))`: faz parsing de sequência.
- `preceded(a, b)`: faz parsing de `a` depois `b`, mantém `b`.
- `delimited(a, b, c)`: faz parsing de `a b c`, mantém `b`.
- `map(p, f)`: transforma saída de parsing em nó de AST.
- `opt(p)`: parsing opcional (`Some` ou `None`).
- `many0(p)`: repete zero ou mais vezes.
- `separated_list0(sep, item)`: faz parsing de itens de lista separados por um separador.
- `verify(p, pred)`: faz parsing com `p`, depois aplica predicado.

Quando não tiver certeza qual combinator usar, consulte o guia de escolha: <https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md>.

### 4) Três Comportamentos Importantes do Nom

#### alt é ordenado

`alt((a, b, c))` tenta da esquerda para a direita e retorna o primeiro sucesso.

Então a ordem de branches indica diretamente o comportamento da linguagem.

#### Sucesso do parser não implica consumo completo

`many0(p)` coleta enquanto `p` sucede e para em falha recuperável. Similarmente, parsers individuais retornam `Ok((rest, value))` onde `rest` pode não estar vazio.

A responsabilidade de verificar consumo completo da entrada é do chamador, não do parser. Use `all_consuming` em testes ou validação explícita em produção.

Referências API úteis: `alt` <https://docs.rs/nom/latest/nom/branch/fn.alt.html>, `many0` <https://docs.rs/nom/latest/nom/multi/fn.many0.html>, `separated_list0` <https://docs.rs/nom/latest/nom/multi/fn.separated_list0.html>, `all_consuming` <https://docs.rs/nom/latest/nom/combinator/fn.all_consuming.html>.

## Parte C: Arquitetura do Parser de MiniC

Módulos do parser são divididos por categoria de gramática:
- `src/parser/identifiers.rs`
- `src/parser/literals.rs`
- `src/parser/expressions.rs`
- `src/parser/statements.rs`
- `src/parser/functions.rs`
- `src/parser/program.rs`

Essa separação reflete a organização da gramática, permite que cada módulo tenha responsabilidade clara, facilita localizar code relacionado e mapeia intuitivamente da linguagem formal para o código Rust.

Veja notas sobre arquitetura do parser em [docs/04-parser.md](04-parser.md), depois compare diretamente com [src/parser/mod.rs](../src/parser/mod.rs).

### 1) Identificadores

Padrão:
1. Fazer parsing da forma de identificador.
2. Rejeitar palavras-chave reservadas com `verify`.

Isso separa claramente forma léxica de política de palavras-chave.

(Nom `verify`: <https://docs.rs/nom/latest/nom/combinator/fn.verify.html>)

### 2) Literais

Faz parsing de int, float, string, bool.

Escolhas de implementação notáveis:
- Parser inteiro rejeita `12.34` como inteiro.
- Parser de string suporta escapes (`\\`, `\"`, `\n`, `\t`) via combinators de escape.

(Referências de parse de texto do Nom: `escaped_transform` <https://docs.rs/nom/latest/nom/bytes/complete/fn.escaped_transform.html>, tabela de parser-chooser de parse de texto: <https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md>)

### 3) Expressões (Precedência + Associatividade)

MiniC codifica precedência via camadas de função (função por nível), em ordem decrescente de precedência:
- ou lógico (precedência mais baixa)
- e lógico
- não
- relacional
- aditivo
- multiplicativo
- unário
- primário
- atômico (precedência mais alta)

Cada camada chama a camada anterior quando precisa de um operando. **Todos os operadores binários de MiniC são associativos à esquerda**, implementados com um loop de acumulador em cada nível: parseia o operando esquerdo, depois enquanto o operador esperado não falha, parseia o operador e o operando direito (que se torna o novo operando esquerdo).

Exemplo: `1 - 2 - 3` se torna `(1 - 2) - 3` porque o primeiro `2` é consumido como direito, o resultado `(1 - 2)` vira o novo esquerdo, e `3` é consumido como novo direito.

Código concreto: [src/parser/expressions.rs](../src/parser/expressions.rs) e testes em [tests/parser.rs](../tests/parser.rs).

### 4) Declarações

Parser de declaração é um conjunto ordenado de alternativas:
- bloco
- if
- while
- return
- declaração
- declaração de chamada
- atribuição

A ordem é deliberada, especialmente para formas com prefixos sobrepostos.

### 5) Funções e Tipos

Parser de tipo inclui formas escalares e de array.

Porque `alt` é ordenado, prefixos mais longos (como formas de array 2D) devem vir listados antes dos mais curtos (formas 1D).

(Nom `alt`: <https://docs.rs/nom/latest/nom/branch/fn.alt.html>)

### 6) Parser de Programa

Parser de nível superior usa repetição sobre declarações de função.

Tradeoff pedagógico:
- Código simples
- Mas você deve raciocinar cuidadosamente sobre comportamento de consumo parcial

Se quiser melhorar esse comportamento ou diagnósticos, os guias de construção de parsers e erros do Nom são o próximo passo certo: <https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md> e <https://github.com/rust-bakery/nom/blob/main/doc/error_management.md>.

## Parte D: AST, Verificação de Tipos e Interpretação

### 1) Design da AST

Nós de AST representam:
- Expressões
- Declarações
- Declarações de função
- Programa

MiniC usa decorações genéricas de nó para que parser e verificador de tipo compartilhem a mesma forma.

Referências do projeto: [docs/03-ast.md](03-ast.md), [src/ir/ast.rs](../src/ir/ast.rs).

### 2) Responsabilidades do Verificador de Tipos

Verificação de tipos valida:
- Assinatura `main` obrigatória
- Tipos de declaração e atribuição
- Contagem/tipos de argumento de chamada de função
- Digitação de operador de expressão
- Indexação de array e consistência de elementos
- cCorreção de tipo de retorno

Usa um ambiente mapeando nomes para tipos.

Referências do projeto: [docs/05-type-checker.md](05-type-checker.md), [src/semantic/type_checker.rs](../src/semantic/type_checker.rs).

### 3) Responsabilidades do Interpretador

Interpretador executa AST verificada:
- Avaliação de expressão em valores em tempo de execução
- Execução de declaração (incluindo fluxo de controle)
- Chamadas de função (definidas pelo usuário e nativas)
- Erros em tempo de execução (ex., fora dos limites)

Usa um ambiente mapeando nomes para valores em tempo de execução.

Referências do projeto: [docs/06-interpreter.md](06-interpreter.md), [src/interpreter/eval_expr.rs](../src/interpreter/eval_expr.rs), [src/interpreter/exec_stmt.rs](../src/interpreter/exec_stmt.rs).

### 4) Equivalência do Ambiente

As duas fases tem a mesma abstração central:
- Ambiente semântico (`name -> Type`)
- Ambiente de tempo de execução (`name -> Value`)

## Parte E: Como Adicionar Funcionalidades (Fluxo de Trabalho do Aluno)

Para cada nova funcionalidade de linguagem, use esta checklist:
1. Estender AST.
2. Estender parser.
3. Estender verificador de tipos.
4. Estender interpretador.
5. Adicionar testes.
6. Atualizar docs.

Se você pular um passo, a funcionalidade fica incompleta.

### Exemplo: Adicionar um Novo Operador Binário

1. Adicionar variante de expressão nova à AST.
2. Adicionar regra de parser na camada de precedência correta.
3. Adicionar regra de tipo.
4. Adicionar regra de avaliação em tempo de execução.
5. Adicionar testes de precedência do parser + testes de tipo + testes de interpretador.

### Exemplo: Adicionar uma Nova Declaração

1. Adicionar variante de declaração.
2. Adicionar branch de parser na ordem correta.
3. Adicionar branch de verificação de tipos.
4. Adicionar branch de execução.
5. Adicionar testes de escopo de bloco e testes de integração.

Para adições de funcionalidade que tocam builtins ou fiação de tempo de execução, consulte [docs/07-stdlib.md](07-stdlib.md), [docs/08-testing.md](08-testing.md), e [src/stdlib/mod.rs](../src/stdlib/mod.rs).

## Parte F: Estratégia de Teste Que Você Deveria Seguir

Camadas de teste de MiniC:
- Testes do parser
- Testes do verificador de tipos
- Testes do interpretador
- Testes CLI

Use todas as quatro camadas ao adicionar funcionalidades não triviais.

Regra prática:
- um teste unitário para cada regra local
- um teste ponta-a-ponta para cada comportamento visível ao usuário

Pontos de entrada de teste úteis:
- Testes de parser: [tests/parser.rs](../tests/parser.rs)
- Testes de programa: [tests/program.rs](../tests/program.rs)
- Testes de verificador de tipos: [tests/type_checker.rs](../tests/type_checker.rs)
- Testes de interpretador: [tests/interpreter.rs](../tests/interpreter.rs)
- Testes de stdlib: [tests/stdlib.rs](../tests/stdlib.rs)
- Testes CLI: [tests/cli](../tests/cli)

Detalhes de estratégia de teste e convenções shelltest são documentados em [docs/08-testing.md](08-testing.md).

## Parte G: Ordem de Leitura para Este Repositório

Comece com docs:
1. [docs/01-pipeline.md](01-language.md)
2. [docs/02-pipeline.md](02-pipeline.md)
3. [docs/03-ast.md](03-ast.md)
4. [docs/04-parser.md](04-parser.md)
5. [docs/05-type-checker.md](05-type-checker.md)
6. [docs/06-interpreter.md](06-interpreter.md)
7. [docs/07-stdlib.md](07-stdlib.md)
8. [docs/08-testing.md](08-testing.md)

Depois leia código nesta ordem:
1. [src/ir/ast.rs](../src/ir/ast.rs)
2. [src/parser/mod.rs](../src/parser/mod.rs) e submódulos do parser
3. [src/semantic/type_checker.rs](../src/semantic/type_checker.rs)
4. [src/interpreter/eval_expr.rs](../src/interpreter/eval_expr.rs)
5. [src/interpreter/exec_stmt.rs](../src/interpreter/exec_stmt.rs)
6. [src/stdlib/mod.rs](../src/stdlib/mod.rs)

## Conclusão

Você pode pensar em MiniC como quatro problemas de ensino conectados:
1. Fazer parsing de sintaxe (combinadores Nom).
2. Validar significado (verificador de tipos).
3. Executar comportamento (interpretador).
4. Preservar confiança (testes).

Uma vez que consiga rastrear uma funcionalidade através de todas as quatro, você consegue estender a linguagem com confiança.
