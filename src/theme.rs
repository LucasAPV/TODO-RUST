//! Paleta de cores e estilos compartilhados pela interface.

use ratatui::style::{Color, Modifier, Style};

use crate::parser::Level;

pub const ACCENT: Color = Color::Rgb(137, 180, 250);
pub const FG: Color = Color::Rgb(205, 214, 244);
pub const DIM: Color = Color::Rgb(127, 132, 156);
pub const BORDER: Color = Color::Rgb(69, 71, 90);
pub const SELECTED_BG: Color = Color::Rgb(49, 50, 68);
pub const BAR_BG: Color = Color::Rgb(30, 30, 46);
pub const BLACK: Color = Color::Rgb(17, 17, 27);

pub fn level_color(level: Level) -> Color {
    match level {
        Level::Importante => Color::Rgb(243, 139, 168),
        Level::Fazendo => Color::Rgb(249, 226, 175),
        Level::Pendente => Color::Rgb(137, 180, 250),
        Level::Baixa => Color::Rgb(166, 173, 200),
        Level::Outro => Color::Rgb(203, 166, 247),
        Level::Feito => Color::Rgb(166, 227, 161),
    }
}

pub fn level_icon(level: Level) -> &'static str {
    match level {
        Level::Importante => "▲",
        Level::Fazendo => "◐",
        Level::Pendente => "●",
        Level::Baixa => "▽",
        Level::Outro => "◆",
        Level::Feito => "✔",
    }
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn bold(color: Color) -> Style {
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

/// Estilo de "etiqueta" (texto escuro sobre fundo colorido).
pub fn badge(bg: Color) -> Style {
    Style::default()
        .fg(BLACK)
        .bg(bg)
        .add_modifier(Modifier::BOLD)
}
