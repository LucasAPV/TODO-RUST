//! Renderização da interface (ratatui).

use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::parser::Level;
use crate::theme::{self, badge, bold, dim, level_color, level_icon};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_header(f, app, header);

    let [list_area, detail_area] = if area.width >= 90 {
        Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)]).areas(body)
    } else {
        Layout::vertical([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(body)
    };
    draw_list(f, app, list_area);
    draw_detail(f, app, detail_area);
    draw_footer(f, app, footer);

    if app.show_help {
        draw_help(f, area);
    }
}

fn panel(title: &str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(title.to_string(), bold(theme::ACCENT)),
            Span::raw(" "),
        ]))
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let counts = app.counts();
    let mut title = vec![
        Span::styled(" ✔ TODO ", badge(theme::ACCENT)),
        Span::raw("  "),
        Span::styled(app.source.clone(), bold(theme::FG)),
        Span::styled(
            format!(
                "   {} arquivo(s) analisado(s) · {} TODO(s)",
                app.files_scanned,
                app.todos.len()
            ),
            dim(),
        ),
    ];
    if !app.query.is_empty() {
        title.push(Span::styled(format!("   busca: \"{}\"", app.query), bold(theme::ACCENT)));
    }

    let visible: Vec<Level> = Level::ALL.iter().copied().filter(|l| counts[l.rank()] > 0).collect();
    // Largura aproximada do modo completo; se não couber, os chips inativos
    // mostram só ícone e contagem.
    let full_width: usize = 11
        + visible
            .iter()
            .map(|l| format!(" 0 {} {} {} ", level_icon(*l), l.label(), counts[l.rank()]).chars().count() + 1)
            .sum::<usize>();
    let compact = full_width > area.width.saturating_sub(4) as usize;

    let mut chips = vec![chip("0 Todos", app.todos.len(), theme::ACCENT, app.filter.is_none())];
    for level in visible {
        let n = counts[level.rank()];
        let active = app.filter == Some(level);
        let num = level.rank() + 1;
        let label = if compact && !active {
            format!("{num} {}", level_icon(level))
        } else {
            format!("{num} {} {}", level_icon(level), level.label())
        };
        chips.push(chip(&label, n, level_color(level), active));
    }
    let chips: Vec<Span> = chips.into_iter().flatten().collect();

    f.render_widget(
        Paragraph::new(vec![Line::from(title), Line::from(chips)]).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER))
                .padding(Padding::horizontal(1)),
        ),
        area,
    );
}

fn chip(label: &str, count: usize, color: ratatui::style::Color, active: bool) -> Vec<Span<'static>> {
    let text = format!(" {label} {count} ");
    let style = if active { badge(color) } else { Style::default().fg(color) };
    vec![Span::styled(text, style), Span::raw(" ")]
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let title = format!("Lista ({}/{})", app.view.len(), app.todos.len());
    let block = panel(&title);
    let inner = block.inner(area);
    app.page_size = (inner.height / 3).max(1) as usize;

    if app.view.is_empty() {
        let (head, hint) = if app.todos.is_empty() {
            ("Nenhum TODO encontrado", "Use o formato:  [TODO]  title:\"...\", desc=\"...\", relevance=...")
        } else {
            ("Nenhum resultado", "Esc limpa filtros e busca")
        };
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(head, bold(theme::FG))),
            Line::from(Span::styled(hint, dim())),
        ]);
        f.render_widget(
            Paragraph::new(text).alignment(Alignment::Center).wrap(Wrap { trim: true }).block(block),
            area,
        );
        return;
    }

    let width = inner.width.saturating_sub(5) as usize;
    let items: Vec<ListItem> = app
        .view
        .iter()
        .map(|&i| {
            let t = &app.todos[i];
            let color = level_color(t.level);
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{} ", level_icon(t.level)), Style::default().fg(color)),
                    Span::styled(truncate(&t.title, width), bold(theme::FG)),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(t.label(), Style::default().fg(color)),
                    Span::styled(" · ", dim()),
                    Span::styled(truncate(&format!("{}:{}", t.file, t.line), width.saturating_sub(14)), dim()),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(theme::SELECTED_BG))
        .highlight_symbol("▌ ");
    f.render_stateful_widget(list, area, &mut app.list);

    if app.view.len() <= app.page_size {
        return;
    }
    let mut sb = ScrollbarState::new(app.view.len()).position(app.list.selected().unwrap_or(0));
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(Style::default().fg(theme::BORDER)),
        area.inner(Margin { vertical: 1, horizontal: 0 }),
        &mut sb,
    );
}

fn rule(width: u16, label: &str) -> Line<'static> {
    let used = label.chars().count() + 1;
    let rest = (width as usize).saturating_sub(used);
    Line::from(vec![
        Span::styled(label.to_string(), bold(theme::DIM)),
        Span::styled(format!(" {}", "─".repeat(rest.saturating_sub(1))), Style::default().fg(theme::BORDER)),
    ])
}

fn draw_detail(f: &mut Frame, app: &mut App, area: Rect) {
    let mut block = panel("Detalhes").padding(Padding::horizontal(1));
    let inner = block.inner(area);

    let Some(t) = app.selected_todo().cloned() else {
        f.render_widget(block, area);
        return;
    };
    let color = level_color(t.level);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            t.title.clone(),
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!(" {} {} ", level_icon(t.level), t.label().to_uppercase()), badge(color)),
            Span::raw("  "),
            Span::styled(t.file.clone(), Style::default().fg(theme::ACCENT)),
            Span::styled(format!(":{}", t.line), dim()),
        ]),
        Line::from(""),
        rule(inner.width, "DESCRIÇÃO"),
        Line::from(""),
    ];
    if t.desc.is_empty() {
        lines.push(Line::from(Span::styled(
            "(sem descrição)",
            dim().add_modifier(Modifier::ITALIC),
        )));
    } else {
        for l in t.desc.lines() {
            lines.push(Line::from(Span::styled(l.to_string(), Style::default().fg(theme::FG))));
        }
    }
    lines.push(Line::from(""));
    lines.push(rule(inner.width, "ORIGEM"));
    lines.push(Line::from(""));
    let code_w = (inner.width as usize).saturating_sub(7);
    for s in &t.snippet {
        let (bar, style) = if s.in_tag {
            ("▌", Style::default().fg(color))
        } else {
            ("│", dim())
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{:>4} ", s.no), dim()),
            Span::styled(format!("{bar} "), style),
            Span::styled(truncate(&s.text, code_w), if s.in_tag { Style::default().fg(theme::FG) } else { dim() }),
        ]));
    }

    // Estimativa da altura após quebra de linha, para limitar a rolagem.
    let w = inner.width.max(1) as usize;
    let total: usize = lines.iter().map(|l| l.width().max(1).div_ceil(w)).sum();
    let max_scroll = total.saturating_sub(inner.height as usize) as u16;
    app.detail_scroll = app.detail_scroll.min(max_scroll);

    if max_scroll > 0 {
        block = block.title_bottom(
            Line::from(Span::styled(" J/K rolar ", dim())).alignment(Alignment::Right),
        );
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0)),
        area,
    );
}

fn key(k: &str, desc: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!(" {k} "), badge(theme::ACCENT)),
        Span::styled(format!(" {desc}  "), Style::default().fg(theme::FG)),
    ]
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let bg = Style::default().bg(theme::BAR_BG);
    let left: Vec<Span> = if app.mode == Mode::Search {
        vec![
            Span::styled(" / ", badge(theme::ACCENT)),
            Span::styled(format!(" {}", app.query), Style::default().fg(theme::FG)),
            Span::styled("█", Style::default().fg(theme::ACCENT)),
            Span::styled("   Enter confirma · Esc cancela", dim()),
        ]
    } else {
        [key("q", "sair"), key("j/k", "mover"), key("/", "buscar"), key("Tab", "filtro"), key("?", "ajuda")]
            .concat()
    };
    let pos = match app.list.selected() {
        Some(i) => format!(" {}/{} ", i + 1, app.view.len()),
        None => " 0/0 ".to_string(),
    };
    let [l, r] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(pos.chars().count() as u16)]).areas(area);
    f.render_widget(Paragraph::new(Line::from(left)).style(bg), l);
    f.render_widget(Paragraph::new(Span::styled(pos, bold(theme::ACCENT))).style(bg), r);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let rows = [
        ("j / ↓   k / ↑", "próximo / anterior"),
        ("g / Home", "primeiro"),
        ("G / End", "último"),
        ("Espaço / PgDn", "página abaixo"),
        ("b / PgUp", "página acima"),
        ("Ctrl-d / Ctrl-u", "meia página"),
        ("J / K", "rolar detalhes"),
        ("Tab / Shift-Tab", "alternar filtro de relevância"),
        ("0 … 6", "filtrar por relevância"),
        ("/", "buscar (título, descrição, arquivo)"),
        ("Esc", "limpar filtro e busca"),
        ("q", "sair"),
    ];
    let mut lines = vec![Line::from("")];
    for (k, d) in rows {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<18}"), bold(theme::ACCENT)),
            Span::styled(d, Style::default().fg(theme::FG)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Pressione qualquer tecla para fechar", dim())));

    let w = 58.min(area.width);
    let h = (lines.len() as u16 + 2).min(area.height);
    let popup = Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).block(panel("Atalhos").border_style(Style::default().fg(theme::ACCENT))),
        popup,
    );
}
