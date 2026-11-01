mod app;
mod parser;
mod scanner;
mod theme;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use app::App;

/// Encontra tags [TODO] no seu código e as exibe em uma TUI.
///
/// Formato da tag (em qualquer linguagem, geralmente dentro de comentários):
///
///   [TODO]
///   title:"titulo",
///   desc="descricao",
///   relevance=importante
#[derive(Parser)]
#[command(name = "todo", version, verbatim_doc_comment)]
struct Cli {
    /// Arquivo para analisar (pode ser repetido)
    #[arg(short = 'f', long = "file", value_name = "ARQUIVO")]
    files: Vec<PathBuf>,

    /// Pasta para analisar recursivamente (respeita .gitignore)
    #[arg(short = 'd', long = "dir", value_name = "PASTA")]
    dirs: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let mut cli = Cli::parse();
    if cli.files.is_empty() && cli.dirs.is_empty() {
        cli.dirs.push(PathBuf::from("."));
    }

    for f in &cli.files {
        if !f.is_file() {
            eprintln!("erro: '{}' não é um arquivo válido", f.display());
            return ExitCode::from(2);
        }
    }
    for d in &cli.dirs {
        if !d.is_dir() {
            eprintln!("erro: '{}' não é uma pasta válida", d.display());
            return ExitCode::from(2);
        }
    }

    let result = scanner::scan(&cli.files, &cli.dirs);
    let source = cli
        .files
        .iter()
        .chain(cli.dirs.iter())
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let mut app = App::new(result.todos, source, result.files_scanned);

    // `ratatui::init` entra na tela alternativa (como less/vim) e instala um
    // hook de pânico que restaura o terminal.
    let mut terminal = ratatui::init();
    let outcome = run(&mut terminal, &mut app);
    ratatui::restore();

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while !app.quit {
        terminal.draw(|f| ui::draw(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.on_key(key);
            }
        }
    }
    Ok(())
}
