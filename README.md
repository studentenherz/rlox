<div align="center">
<h1> RLox </h1>
<h6> A <a href="https://craftinginterpreters.com/the-lox-language.html">Lox</a> interpreter written in Rust </h6>
</br>
</div>

This is an implementation of an interpreter for the Lox programming language form the [Crafting Inerpreters](https://craftinginterpreters.com) book. This tries to follow the book while also attempting to write the interpreter using idiomatic Rust patterns and features.

This also draws inspiration (to put it mildly) from the [rust compiler](https://github.com/rust-lang/rust/).

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
declaration    -> varDecl | statement ;
statement      -> exprStmt 
               | printStmt 
               | block ;
exprStmt       -> expression ";" ;
printStmt      -> "print" expression ";" ; 
block          -> "{" declaration* "}" ;
varDecl        -> "var" IDENTIFIER ( "=" expression )? ";" ;
expression     -> comma ;
comma          -> assignment ( "," assignment )* ;
assignment     -> IDENTIFIER "=" assignment
               | equality ( "?" expression ":" assignment )? ; 
equality       -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison     -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           -> factor ( ( "-" | "+" ) factor )* ;
factor         -> unary ( ( "/" | "*" ) unary )* ;
unary          -> ( "!" | "-" ) unary
               | primary ;
primary        -> "true" | "false" | "nil"
               | NUMBER | STRING | IDENTIFIER
               | "(" expression ")" ;

```
