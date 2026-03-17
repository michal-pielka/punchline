use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style as RatStyle,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::tui::app::App;

impl App {
    pub fn render_connecting(&self, f: &mut Frame) {
        let colors = &self.style.colors;

        let horizontal = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(81),
            Constraint::Min(0),
        ])
        .split(f.area());

        let vertical = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(13),
            Constraint::Min(0),
        ])
        .split(horizontal[1]);

        let lines = vec![
            Line::raw(
                r"                                         /$$       /$$ /$$                     ",
            ),
            Line::raw(
                r"                                        | $$      | $$|__/                     ",
            ),
            Line::raw(
                r"  /$$$$$$  /$$   /$$ /$$$$$$$   /$$$$$$$| $$$$$$$ | $$ /$$ /$$$$$$$   /$$$$$$  ",
            ),
            Line::raw(
                r" /$$__  $$| $$  | $$| $$__  $$ /$$_____/| $$__  $$| $$| $$| $$__  $$ /$$__  $$ ",
            ),
            Line::raw(
                r"| $$  \ $$| $$  | $$| $$  \ $$| $$      | $$  \ $$| $$| $$| $$  \ $$| $$$$$$$$ ",
            ),
            Line::raw(
                r"| $$  | $$| $$  | $$| $$  | $$| $$      | $$  | $$| $$| $$| $$  | $$| $$_____/ ",
            ),
            Line::raw(
                r"| $$$$$$$/|  $$$$$$/| $$  | $$|  $$$$$$$| $$  | $$| $$| $$| $$  | $$|  $$$$$$$ ",
            ),
            Line::raw(
                r"| $$____/  \______/ |__/  |__/ \_______/|__/  |__/|__/|__/|__/  |__/ \_______/ ",
            ),
            Line::raw(
                r"| $$                                                                           ",
            ),
            Line::raw(
                r"| $$                                                                           ",
            ),
            Line::raw(
                r"|__/                                                                           ",
            ),
        ];

        let dialog = Paragraph::new(lines).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border)),
        );

        f.render_widget(dialog, vertical[1]);
    }
}
