use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

mod lexer;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Script to run
    script: Option<PathBuf>,
}

fn run(source: &str) {
    for token in lexer::tokenize(source) {
        match token {
            lexer::Token::Whitespace => {}
            _ => {
                println!("{:?}", token);
            }
        }
    }
}

fn run_script(path: PathBuf) -> std::io::Result<()> {
    let content = read_to_string(path)?;
    run(&content);
    Ok(())
}

fn run_prompt() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");
    let quit = "q!";

    stdout.write_all(
        format!("Welcome to {name} version {version} by {authors}\nUse {quit} to quit\n")
            .as_bytes(),
    )?;

    loop {
        let mut line = String::new();
        stdout.write_all(b"> ")?;
        stdout.flush()?;
        let bytes = stdin.read_line(&mut line)?;
        if line == format!("{quit}\n") || bytes == 0 {
            break;
        }
        run(&line);
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let _ = match cli.script {
        Some(path) => run_script(path),
        None => run_prompt(),
    };
}
