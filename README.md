<div align="center">
    <h1> RLox </h1>
    <h6> A <a href="https://craftinginterpreters.com/the-lox-language.html">Lox</a> interpreter written in Rust </h6>
    </br>
</div>

This is an implementation of an interpreter for the Lox programming language form the [Crafting Inerpreters](https://craftinginterpreters.com) book. This tries to follow the while also attempting to write the interpreter using idiomatic Rust patterns and features.

This also draws inspiration from the [rust compiler](https://github.com/rust-lang/rust/).

## Usage

There are two ways to use this interpreter 

### Interactive interpreter 

Run with `cargo run` and a [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) with pop for in interactive experience. You can quit the language shell with `Ctrl+D`.

### Script interpreter

This also works as a CLI program. Run with the `--help` option to see the usage message:

```
Usage: rlox [SCRIPT]

Arguments:
  [SCRIPT]  Script to run

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Grammar

This is the current grammar the interpreter supports

```
program        -> declaration* EOF ;
declaration    -> classDecl 
               | funDecl
               | varDecl
               | statement ;
statement      -> exprStmt
               | forStmt
               | ifStmt
               | jumpStmt
               | printStmt
               | returnStmt
               | whileStmt
               | block ;
classDecl      -> "class" IDENTIFIER ( "<" IDENTIFIER )? "{" function* "}" ;
funDecl        -> "fun" function ;
function       -> IDENTIFIER "(" paremeters? ")" block ;
parameters     -> IDENTIFIER ( "," IDENTIFIER )* ;
exprStmt       -> expression ";" ;
jumpStmt       -> ( "break" | "continue" ) ";" ;
forStmt        -> "for" "(" ( varDecl | exprStmt | ";" )
                 expression? ";"
                 expression? ")" statement ;
ifStmt         -> "if" "(" expression ")" statement
               ( "else" statement )? ;
printStmt      -> "print" expression ";" ; 
returnStmt     -> "return" expression? ";" ;
whileStmt      -> "while" "(" expression ")" statement ;
block          -> "{" declaration* "}" ;
varDecl        -> "var" IDENTIFIER ( "=" expression )? ";" ;
expression     -> comma ;
comma          -> assignment ( "," assignment )* ;
assignment     -> ( call "." )? IDENTIFIER "=" assignment
               | logic_or ( "?" expression ":" assignment )?
               | logic_or ;
logic_or       -> logic_and ( "or" logic_and )* ;
logic_and      -> equality ( "and" equality )* ;
equality       -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison     -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           -> factor ( ( "-" | "+" ) factor )* ;
factor         -> unary ( ( "/" | "*" ) unary )* ;
unary          -> ( "!" | "-" ) unary
               | call ;
call           -> primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
arguments      -> assignment ( "," assignment )* ;
primary        -> "true" | "false" | "nil"
               | NUMBER | STRING | IDENTIFIER | "(" expression ")"
               | "super" "." IDENTIFIER ;
```

## Notes

See [NOTES.md](./NOTES.md) for some notes on development and places this implementation diverges from the book.

## Correctness 

I tested `rlox` against the tests provided in [craftinginterpreters' repo](https://github.com/munificent/craftinginterpreters) in the `jlox` version. There are only 7 tests that don't pass and I have a section in the notes explaining why they are left like that.

## Benchmarks

Initial benchmark showed a surprising (for me) result. `jlox` beats `rlox` (almost) everytime and runs ~3 times faster. 

```
» for TEST in test/benchmark/*; do hyperfine "./jlox $TEST" "rlox $TEST"; done
Benchmark 1: ./jlox test/benchmark/binary_trees.lox
  Time (mean ± σ):     10.317 s ±  0.440 s    [User: 11.291 s, System: 0.378 s]
  Range (min … max):    9.852 s … 11.152 s    10 runs

Benchmark 2: rlox test/benchmark/binary_trees.lox
  Time (mean ± σ):     30.628 s ±  1.642 s    [User: 30.532 s, System: 0.054 s]
  Range (min … max):   29.540 s … 34.320 s    10 runs

  Warning: The first benchmarking run for this command was significantly slower than the rest (34.320 s). This could be caused by (filesystem) caches that were not filled until after the first run. You should consider using the '--warmup' option to fill those caches before the actual benchmark. Alternatively, use the '--prepare' option to clear the caches before each timing run.

Summary
  ./jlox test/benchmark/binary_trees.lox ran
    2.97 ± 0.20 times faster than rlox test/benchmark/binary_trees.lox
Benchmark 1: ./jlox test/benchmark/equality.lox
  Time (mean ± σ):      8.067 s ±  0.201 s    [User: 8.225 s, System: 0.143 s]
  Range (min … max):    7.783 s …  8.403 s    10 runs

Benchmark 2: rlox test/benchmark/equality.lox
  Time (mean ± σ):     30.578 s ±  0.272 s    [User: 30.541 s, System: 0.009 s]
  Range (min … max):   30.122 s … 31.107 s    10 runs

Summary
  ./jlox test/benchmark/equality.lox ran
    3.79 ± 0.10 times faster than rlox test/benchmark/equality.lox
Benchmark 1: ./jlox test/benchmark/fib.lox
  Time (mean ± σ):      6.323 s ±  2.152 s    [User: 6.553 s, System: 0.169 s]
  Range (min … max):    5.215 s … 10.432 s    10 runs

  Warning: The first benchmarking run for this command was significantly slower than the rest (10.375 s). This could be caused by (filesystem) caches that were not filled until after the first run. You should consider using the '--warmup' option to fill those caches before the actual benchmark. Alternatively, use the '--prepare' option to clear the caches before each timing run.

Benchmark 2: rlox test/benchmark/fib.lox
  Time (mean ± σ):     14.976 s ±  0.070 s    [User: 14.952 s, System: 0.008 s]
  Range (min … max):   14.850 s … 15.075 s    10 runs

Summary
  ./jlox test/benchmark/fib.lox ran
    2.37 ± 0.81 times faster than rlox test/benchmark/fib.lox
Benchmark 1: ./jlox test/benchmark/instantiation.lox
  Time (mean ± σ):      1.975 s ±  0.048 s    [User: 2.342 s, System: 0.182 s]
  Range (min … max):    1.915 s …  2.071 s    10 runs

Benchmark 2: rlox test/benchmark/instantiation.lox
  Time (mean ± σ):      8.280 s ±  0.095 s    [User: 8.268 s, System: 0.006 s]
  Range (min … max):    8.182 s …  8.526 s    10 runs

Summary
  ./jlox test/benchmark/instantiation.lox ran
    4.19 ± 0.11 times faster than rlox test/benchmark/instantiation.lox
Benchmark 1: ./jlox test/benchmark/invocation.lox
  Time (mean ± σ):      1.770 s ±  0.017 s    [User: 2.169 s, System: 0.173 s]
  Range (min … max):    1.753 s …  1.801 s    10 runs

Benchmark 2: rlox test/benchmark/invocation.lox
  Time (mean ± σ):      5.754 s ±  0.058 s    [User: 5.747 s, System: 0.002 s]
  Range (min … max):    5.673 s …  5.819 s    10 runs

Summary
  ./jlox test/benchmark/invocation.lox ran
    3.25 ± 0.05 times faster than rlox test/benchmark/invocation.lox
Benchmark 1: ./jlox test/benchmark/method_call.lox
  Time (mean ± σ):      3.188 s ±  0.070 s    [User: 4.161 s, System: 0.170 s]
  Range (min … max):    3.093 s …  3.349 s    10 runs

Benchmark 2: rlox test/benchmark/method_call.lox
  Time (mean ± σ):      3.281 s ±  0.023 s    [User: 3.275 s, System: 0.003 s]
  Range (min … max):    3.251 s …  3.322 s    10 runs

Summary
  ./jlox test/benchmark/method_call.lox ran
    1.03 ± 0.02 times faster than rlox test/benchmark/method_call.lox
Benchmark 1: ./jlox test/benchmark/properties.lox
  Time (mean ± σ):      5.939 s ±  0.033 s    [User: 6.373 s, System: 0.172 s]
  Range (min … max):    5.887 s …  6.004 s    10 runs

Benchmark 2: rlox test/benchmark/properties.lox
  Time (mean ± σ):      8.160 s ±  0.129 s    [User: 8.147 s, System: 0.003 s]
  Range (min … max):    8.039 s …  8.483 s    10 runs

Summary
  ./jlox test/benchmark/properties.lox ran
    1.37 ± 0.02 times faster than rlox test/benchmark/properties.lox
Benchmark 1: ./jlox test/benchmark/string_equality.lox
  Time (mean ± σ):      8.091 s ±  0.230 s    [User: 8.621 s, System: 0.067 s]
  Range (min … max):    7.848 s …  8.631 s    10 runs

Benchmark 2: rlox test/benchmark/string_equality.lox
  Time (mean ± σ):     26.179 s ±  0.255 s    [User: 26.146 s, System: 0.009 s]
  Range (min … max):   25.753 s … 26.716 s    10 runs

Summary
  ./jlox test/benchmark/string_equality.lox ran
    3.24 ± 0.10 times faster than rlox test/benchmark/string_equality.lox
Benchmark 1: ./jlox test/benchmark/trees.lox
  Time (mean ± σ):     31.649 s ±  0.293 s    [User: 31.802 s, System: 1.118 s]
  Range (min … max):   31.346 s … 32.063 s    10 runs

Benchmark 2: rlox test/benchmark/trees.lox
  Time (mean ± σ):     45.109 s ±  0.290 s    [User: 44.880 s, System: 0.199 s]
  Range (min … max):   44.651 s … 45.566 s    10 runs

Summary
  ./jlox test/benchmark/trees.lox ran
    1.43 ± 0.02 times faster than rlox test/benchmark/trees.lox
Benchmark 1: ./jlox test/benchmark/zoo_batch.lox
  Time (mean ± σ):     10.131 s ±  0.015 s    [User: 10.591 s, System: 0.171 s]
  Range (min … max):   10.117 s … 10.160 s    10 runs

Benchmark 2: rlox test/benchmark/zoo_batch.lox
  Time (mean ± σ):     10.019 s ±  0.011 s    [User: 10.003 s, System: 0.005 s]
  Range (min … max):   10.006 s … 10.036 s    10 runs

Summary
  rlox test/benchmark/zoo_batch.lox ran
    1.01 ± 0.00 times faster than ./jlox test/benchmark/zoo_batch.lox
Benchmark 1: ./jlox test/benchmark/zoo.lox
  Time (mean ± σ):      5.725 s ±  0.048 s    [User: 6.180 s, System: 0.174 s]
  Range (min … max):    5.644 s …  5.800 s    10 runs

Benchmark 2: rlox test/benchmark/zoo.lox
  Time (mean ± σ):      5.772 s ±  0.049 s    [User: 5.763 s, System: 0.004 s]
  Range (min … max):    5.686 s …  5.839 s    10 runs

Summary
  ./jlox test/benchmark/zoo.lox ran
    1.01 ± 0.01 times faster than rlox test/benchmark/zoo.lox
```
This surprised me because, well..., it's Rust vs. Java. I also had ran a benchmark while implementing functions using [the fibonacci example from the book](https://craftinginterpreters.com/functions.html#return-statements) and `rlox` was way faster and it still is:

```
craftinginterpreters (master*) » cat ~/test.lox
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

for (var i = 0; i < 20; i = i + 1) {
  print fib(i);
}
craftinginterpreters (master*) » hyperfine "./jlox ~/test.lox"  "rlox ~/test.lox"
Benchmark 1: ./jlox ~/test.lox
  Time (mean ± σ):     204.1 ms ±  28.5 ms    [User: 349.7 ms, System: 46.7 ms]
  Range (min … max):   181.5 ms … 292.5 ms    16 runs

Benchmark 2: rlox ~/test.lox
  Time (mean ± σ):      19.0 ms ±   1.4 ms    [User: 18.4 ms, System: 0.6 ms]
  Range (min … max):    17.8 ms …  29.3 ms    96 runs

  Warning: The first benchmarking run for this command was significantly slower than the rest (29.3 ms). This could be caused by (filesystem) caches that were not filled until after the first run. You should consider using the '--warmup' option to fill those caches before the actual benchmark. Alternatively, use the '--prepare' option to clear the caches before each timing run.

Summary
  rlox ~/test.lox ran
   10.74 ± 1.68 times faster than ./jlox ~/test.lox
```

This is exacly the same program as the `fib.lox` benchmark but with a lower `n`. 

I did some profiling and from there I guess that the main issue is allocation and deallocation costs on every recursive call, something that the JVM wouldn't probably do. I tried some drop-in-replacement recomendations like `mimalloc` reserving the 3 huge OS pages of 1GiB, but it didn't improve the performance at all, maybe I did it wrong, I don't know. 

