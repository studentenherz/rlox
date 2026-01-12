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
declaration    -> funDecl
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
assignment     -> IDENTIFIER "=" assignment
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
call           -> primary ( "(" arguments? ")" )* ;
arguments      -> assignment ( "," assignment )* ;
primary        -> "true" | "false" | "nil"
               | NUMBER | STRING | IDENTIFIER
               | "(" expression ")" ;
```

## Notes

See [NOTES.md](./NOTES.md) for some notes on development and places this implementation diverges from the book.
