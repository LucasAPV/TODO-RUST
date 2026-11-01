//! Estado da aplicação e tratamento de teclado.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::parser::{Level, Todo};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Normal,
    Search,
}

pub struct App {
    pub todos: Vec<Todo>,
    /// Índices (em `todos`) que passam pelo filtro/busca atuais.
    pub view: Vec<usize>,
    pub list: ListState,
    pub filter: Option<Level>,
    pub query: String,
    pub mode: Mode,
    pub show_help: bool,
    pub detail_scroll: u16,
    pub source: String,
    pub files_scanned: usize,
    pub quit: bool,
    /// Atualizado pela UI a cada frame (itens visíveis na lista).
    pub page_size: usize,
}

impl App {
    pub fn new(mut todos: Vec<Todo>, source: String, files_scanned: usize) -> Self {
        todos.sort_by(|a, b| {
            (a.level.rank(), &a.file, a.line).cmp(&(b.level.rank(), &b.file, b.line))
        });
        let mut app = App {
            todos,
            view: Vec::new(),
            list: ListState::default(),
            filter: None,
            query: String::new(),
            mode: Mode::Normal,
            show_help: false,
            detail_scroll: 0,
            source,
            files_scanned,
            quit: false,
            page_size: 5,
        };
        app.refilter();
        app
    }

    pub fn selected_todo(&self) -> Option<&Todo> {
        self.list.selected().and_then(|i| self.view.get(i)).map(|&i| &self.todos[i])
    }

    pub fn counts(&self) -> [usize; 6] {
        let mut c = [0; 6];
        for t in &self.todos {
            c[t.level.rank()] += 1;
        }
        c
    }

    fn refilter(&mut self) {
        let keep = self.list.selected().and_then(|i| self.view.get(i)).copied();
        let q = self.query.to_lowercase();
        self.view = self
            .todos
            .iter()
            .enumerate()
            .filter(|(_, t)| self.filter.map_or(true, |l| t.level == l))
            .filter(|(_, t)| {
                q.is_empty()
                    || [&t.title, &t.desc, &t.file, &t.label()]
                        .iter()
                        .any(|s| s.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect();
        let pos = keep.and_then(|k| self.view.iter().position(|&i| i == k));
        self.select(if self.view.is_empty() { None } else { Some(pos.unwrap_or(0)) });
    }

    fn select(&mut self, idx: Option<usize>) {
        if self.list.selected() != idx {
            self.detail_scroll = 0;
        }
        self.list.select(idx);
    }

    fn move_by(&mut self, delta: isize) {
        if self.view.is_empty() {
            return;
        }
        let cur = self.list.selected().unwrap_or(0) as isize;
        let next = (cur + delta).clamp(0, self.view.len() as isize - 1);
        self.select(Some(next as usize));
    }

    fn cycle_filter(&mut self, forward: bool) {
        let counts = self.counts();
        let mut options: Vec<Option<Level>> = vec![None];
        options.extend(Level::ALL.iter().filter(|l| counts[l.rank()] > 0).map(|l| Some(*l)));
        let cur = options.iter().position(|o| *o == self.filter).unwrap_or(0);
        let n = options.len();
        self.filter = options[if forward { (cur + 1) % n } else { (cur + n - 1) % n }];
        self.refilter();
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.show_help {
            self.show_help = false;
            return;
        }
        match self.mode {
            Mode::Search => self.on_key_search(key),
            Mode::Normal => self.on_key_normal(key),
        }
    }

    fn on_key_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.mode = Mode::Normal,
            KeyCode::Esc => {
                self.query.clear();
                self.mode = Mode::Normal;
                self.refilter();
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.refilter();
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.refilter();
            }
            _ => {}
        }
    }

    fn on_key_normal(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let page = self.page_size.max(1) as isize;
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Esc => {
                if self.filter.is_some() || !self.query.is_empty() {
                    self.filter = None;
                    self.query.clear();
                    self.refilter();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_by(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_by(-1),
            KeyCode::Char('d') if ctrl => self.move_by(page / 2),
            KeyCode::Char('u') if ctrl => self.move_by(-(page / 2)),
            KeyCode::Char(' ') | KeyCode::PageDown | KeyCode::Char('f') => self.move_by(page),
            KeyCode::Char('b') | KeyCode::PageUp => self.move_by(-page),
            KeyCode::Char('g') | KeyCode::Home => self.select((!self.view.is_empty()).then_some(0)),
            KeyCode::Char('G') | KeyCode::End => {
                self.select(self.view.len().checked_sub(1));
            }
            KeyCode::Char('J') => self.detail_scroll = self.detail_scroll.saturating_add(2),
            KeyCode::Char('K') => self.detail_scroll = self.detail_scroll.saturating_sub(2),
            KeyCode::Tab => self.cycle_filter(true),
            KeyCode::BackTab => self.cycle_filter(false),
            KeyCode::Char(c @ '0'..='6') => {
                let n = c.to_digit(10).unwrap() as usize;
                self.filter = if n == 0 { None } else { Some(Level::ALL[n - 1]) };
                self.refilter();
            }
            KeyCode::Char('/') => self.mode = Mode::Search,
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }
}
