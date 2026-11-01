//! Varredura de arquivos e diretórios em busca de TODOs.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::parser::{self, Todo};

pub struct ScanResult {
    pub todos: Vec<Todo>,
    pub files_scanned: usize,
}

const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024;

pub fn scan(files: &[PathBuf], dirs: &[PathBuf]) -> ScanResult {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut todos = Vec::new();
    let mut files_scanned = 0;

    let mut visit = |path: &Path| {
        let key = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !seen.insert(key) {
            return;
        }
        if let Some(content) = read_text(path) {
            files_scanned += 1;
            todos.extend(parser::extract(&display(path), &content));
        }
    };

    for f in files {
        visit(f);
    }
    for d in dirs {
        let walker = WalkBuilder::new(d)
            .filter_entry(|e| !matches!(e.file_name().to_str(), Some("target" | "node_modules")))
            .build();
        for entry in walker.flatten() {
            if entry.file_type().map_or(false, |t| t.is_file()) {
                visit(entry.path());
            }
        }
    }

    ScanResult { todos, files_scanned }
}

fn display(path: &Path) -> String {
    path.strip_prefix("./").unwrap_or(path).to_string_lossy().into_owned()
}

/// Lê o arquivo como texto; ignora binários e arquivos enormes.
fn read_text(path: &Path) -> Option<String> {
    if fs::metadata(path).ok()?.len() > MAX_FILE_SIZE {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if bytes.iter().take(8000).any(|b| *b == 0) {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}
