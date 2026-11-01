//! Extração de blocos `[TODO]` de um texto.
//!
//! Formato aceito (dentro ou fora de comentários, qualquer linguagem):
//!
//! ```text
//! // [TODO]
//! // title:"Titulo",
//! // desc="Descricao",
//! // relevance=importante
//! ```
//!
//! Os separadores `:` e `=` são ambos aceitos, as aspas são opcionais para
//! valores simples e strings podem ocupar várias linhas.

/// Quantas linhas após a tag serão examinadas, no máximo.
const MAX_BLOCK_LINES: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Importante,
    Fazendo,
    Pendente,
    Baixa,
    Outro,
    Feito,
}

impl Level {
    /// Ordem usada na listagem e nos filtros.
    pub const ALL: [Level; 6] = [
        Level::Importante,
        Level::Fazendo,
        Level::Pendente,
        Level::Baixa,
        Level::Outro,
        Level::Feito,
    ];

    pub fn rank(self) -> usize {
        Self::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            Level::Importante => "Importante",
            Level::Fazendo => "Fazendo",
            Level::Pendente => "Pendente",
            Level::Baixa => "Baixa",
            Level::Outro => "Outro",
            Level::Feito => "Feito",
        }
    }

    /// Interpreta o texto livre de `relevance` em um nível conhecido.
    pub fn from_text(text: &str) -> Level {
        let n = normalize(text);
        let has = |pats: &[&str]| pats.iter().any(|p| n.contains(p));
        if has(&["feito", "done", "conclu", "resolv", "finaliz"]) {
            Level::Feito
        } else if has(&["fazendo", "doing", "andamento", "progress", "wip", "working"]) {
            Level::Fazendo
        } else if has(&["import", "urgent", "alta", "high", "critic", "asap"]) {
            Level::Importante
        } else if has(&["baix", "low", "minor", "menor", "talvez", "maybe"]) {
            Level::Baixa
        } else if has(&["pend", "todo", "open", "abert", "normal", "media", "medium", "fazer"]) {
            Level::Pendente
        } else {
            Level::Outro
        }
    }
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct SnippetLine {
    pub no: usize,
    pub text: String,
    pub in_tag: bool,
}

#[derive(Clone, Debug)]
pub struct Todo {
    pub title: String,
    pub desc: String,
    /// Texto original de `relevance`, exatamente como o usuário escreveu.
    pub relevance: String,
    pub level: Level,
    pub file: String,
    /// Linha (1-based) onde a tag `[TODO]` aparece.
    pub line: usize,
    pub snippet: Vec<SnippetLine>,
}

impl Todo {
    /// Rótulo exibido: nome do nível, ou o texto livre quando é desconhecido.
    pub fn label(&self) -> String {
        match self.level {
            Level::Outro if !self.relevance.is_empty() => self.relevance.clone(),
            Level::Outro => "Indefinida".to_string(),
            other => other.label().to_string(),
        }
    }
}

#[derive(Default)]
struct Fields {
    title: Option<String>,
    desc: Option<String>,
    relevance: Option<String>,
}

/// Procura todos os blocos `[TODO]` em `content`.
pub fn extract(file: &str, content: &str) -> Vec<Todo> {
    let raw: Vec<&str> = content.lines().collect();
    let mut todos = Vec::new();

    for (i, line) in raw.iter().enumerate() {
        // `to_ascii_uppercase` preserva os índices de byte, então `pos` vale
        // também para a linha original.
        let Some(pos) = line.to_ascii_uppercase().find("[TODO]") else {
            continue;
        };
        let after = clean_tail(&line[pos + "[TODO]".len()..]);

        let mut block = vec![after.clone()];
        for next in raw.iter().skip(i + 1).take(MAX_BLOCK_LINES) {
            if next.to_ascii_uppercase().contains("[TODO]") {
                break;
            }
            block.push(clean_line(next));
        }

        let text = block.join("\n");
        let (fields, end) = parse_fields(&text);
        let found_any = fields.title.is_some() || fields.desc.is_some() || fields.relevance.is_some();

        let (title, consumed) = if found_any {
            let consumed = text.chars().take(end).filter(|c| *c == '\n').count();
            (fields.title.unwrap_or_default(), consumed)
        } else {
            (after.trim().to_string(), 0)
        };
        let title = if title.is_empty() { "(sem título)".to_string() } else { title };
        let relevance = fields.relevance.unwrap_or_default();

        let first = i.saturating_sub(1);
        let last = (i + consumed + 1).min(raw.len() - 1);
        let snippet = (first..=last)
            .map(|n| SnippetLine {
                no: n + 1,
                text: raw[n].replace('\t', "    ").trim_end().to_string(),
                in_tag: n >= i && n <= i + consumed,
            })
            .collect();

        todos.push(Todo {
            title,
            desc: fields.desc.unwrap_or_default(),
            level: Level::from_text(&relevance),
            relevance,
            file: file.to_string(),
            line: i + 1,
            snippet,
        });
    }
    todos
}

const PREFIXES: [&str; 9] = ["//!", "///", "//", "/*", "<!--", "--", "#", "*", "%"];

/// Remove o prefixo de comentário (`//`, `#`, `*`, ...) de uma linha.
fn clean_line(line: &str) -> String {
    let mut s = line.trim_start();
    // Duas passadas cobrem casos como `/**` ou `# //`.
    for _ in 0..2 {
        for p in PREFIXES {
            if let Some(rest) = s.strip_prefix(p) {
                s = rest.trim_start();
                break;
            }
        }
    }
    clean_tail(s)
}

/// Remove fechamentos de comentário (`*/`, `-->`) no fim da linha.
fn clean_tail(line: &str) -> String {
    let mut s = line.trim_end();
    for suffix in ["*/", "-->"] {
        if let Some(rest) = s.strip_suffix(suffix) {
            s = rest.trim_end();
        }
    }
    s.trim().to_string()
}

fn parse_fields(text: &str) -> (Fields, usize) {
    let c: Vec<char> = text.chars().collect();
    let mut f = Fields::default();
    let (mut i, mut end) = (0, 0);

    loop {
        while i < c.len() && (c[i].is_whitespace() || c[i] == ',') {
            i += 1;
        }
        let start = i;
        while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
            i += 1;
        }
        if start == i {
            break;
        }
        let key: String = c[start..i].iter().collect::<String>().to_lowercase();
        let slot = match key.as_str() {
            "title" | "titulo" | "título" | "name" | "nome" => 0,
            "desc" | "description" | "descricao" | "descrição" => 1,
            "relevance" | "relevancia" | "relevância" | "priority" | "prioridade" | "status" => 2,
            _ => break,
        };

        while i < c.len() && (c[i] == ' ' || c[i] == '\t') {
            i += 1;
        }
        if i >= c.len() || (c[i] != ':' && c[i] != '=') {
            break;
        }
        i += 1;
        while i < c.len() && c[i].is_whitespace() {
            i += 1;
        }

        let value = if i < c.len() && (c[i] == '"' || c[i] == '\'') {
            let quote = c[i];
            i += 1;
            let mut v = String::new();
            while i < c.len() && c[i] != quote {
                match c[i] {
                    '\\' if i + 1 < c.len() => {
                        i += 1;
                        v.push(match c[i] {
                            'n' => '\n',
                            't' => ' ',
                            other => other,
                        });
                    }
                    '\n' => v.push(' '),
                    ch => v.push(ch),
                }
                i += 1;
            }
            i = (i + 1).min(c.len());
            v
        } else {
            let s = i;
            while i < c.len() && c[i] != ',' && c[i] != '\n' {
                i += 1;
            }
            c[s..i].iter().collect()
        };

        let value = collapse(&value);
        match slot {
            0 => f.title = Some(value),
            1 => f.desc = Some(value),
            _ => f.relevance = Some(value),
        }
        end = i;
    }
    (f, end)
}

/// Normaliza espaços em cada linha, preservando quebras explícitas (`\n`).
fn collapse(s: &str) -> String {
    s.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_comment_style() {
        let src = "int x;\n// [TODO]\n// title:\"Corrigir\",\n// desc=\"Vazamento\n//   de memoria\",\n// relevance=importante\nint y;\n";
        let t = extract("a.c", src);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].title, "Corrigir");
        assert_eq!(t[0].desc, "Vazamento de memoria");
        assert_eq!(t[0].level, Level::Importante);
        assert_eq!(t[0].line, 2);
        assert!(t[0].snippet.iter().filter(|s| s.in_tag).count() == 5);
    }

    #[test]
    fn falls_back_to_inline_text() {
        let t = extract("a.py", "# [TODO] ajustar isso\n");
        assert_eq!(t[0].title, "ajustar isso");
    }

    #[test]
    fn stops_at_code() {
        let src = "/* [TODO]\n title: \"A\",\n relevance: fazendo */\nx = 5;\n";
        let t = extract("a.js", src);
        assert_eq!(t[0].level, Level::Fazendo);
        assert_eq!(t[0].desc, "");
    }
}
