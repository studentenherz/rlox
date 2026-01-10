use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustyLineResult};

use clap::Parser as ClapParser;

mod common;
mod constants;
mod environments;
mod expressions;
mod interpreter;
mod lexer;
mod parser;
mod statements;
mod values;

use self::interpreter::*;
use interpreter::InterpreterCtx;
use parser::Parser as LoxParser;

#[derive(ClapParser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Script to run
    script: Option<PathBuf>,
}

fn run(source: &str, ctx: &mut InterpreterCtx, repl: bool) {
    match LoxParser::parse(source, repl) {
        Ok(statements) => {
            for stmt in statements {
                match stmt.evaluate(ctx) {
                    Ok(Some(value)) => {
                        if repl {
                            println!("{:?}", value);
                        }
                    }
                    Err(err) => println!("{}", err),
                    _ => {}
                }
            }
        }
        Err(error) => println!("{}", error),
    }
}

fn run_script(path: PathBuf) -> std::io::Result<()> {
    let content = read_to_string(path)?;
    let mut ctx = InterpreterCtx::new();
    run(&content, &mut ctx, false);
    Ok(())
}

fn welcome_message() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");
    let quit = "q!";
    format!("Welcome to {name} version {version} by {authors}\nUse {quit} to quit")
}

fn run_prompt() -> RustyLineResult<()> {
    let mut rl = DefaultEditor::new()?;

    let history_file_path =
        env::home_dir().map_or(PathBuf::from(".rlox_history"), |p| p.join(".rlox_history"));

    let _ = rl.load_history(&history_file_path);

    println!("{}", welcome_message());

    let mut ctx = InterpreterCtx::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                if line == "q!" {
                    break;
                }
                run(&line, &mut ctx, true);
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&history_file_path)?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let _ = match cli.script {
        Some(path) => {
            if let Err(err) = run_script(path) {
                eprintln!("{}", err);
            }
        }
        None => {
            if let Err(err) = run_prompt() {
                eprintln!("{}", err);
            }
        }
    };
}
