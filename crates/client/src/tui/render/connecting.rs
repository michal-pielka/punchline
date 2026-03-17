use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Spacing},
    style::Style as RatStyle,
    symbols::merge::MergeStrategy,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::tui::app::{App, StepStatus};

impl App {
    pub fn render_connecting(&self, f: &mut Frame) {
        let colors = &self.style.colors;

        // Padding
        let vertical = Layout::vertical([
            Constraint::Length(self.style.padding.chat_vertical),
            Constraint::Min(0),
            Constraint::Length(self.style.padding.chat_vertical),
        ])
        .split(f.area());

        let area = Layout::horizontal([
            Constraint::Length(self.style.padding.chat_horizontal),
            Constraint::Min(0),
            Constraint::Length(self.style.padding.chat_horizontal),
        ])
        .split(vertical[1])[1];

        // Top
        let chunks = Layout::vertical([Constraint::Length(17), Constraint::Min(0)])
            .spacing(Spacing::Overlap(1))
            .split(area);

        let art_lines = vec![
            Line::raw(""),
            Line::raw(""),
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
            Line::raw(""),
            Line::raw(""),
        ];

        let art = Paragraph::new(art_lines)
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(RatStyle::new().fg(colors.border))
                    .merge_borders(MergeStrategy::Exact),
            );

        f.render_widget(art, chunks[0]);

        // Bottom panel: split into progress and right
        let bottom_chunks = Layout::horizontal([Constraint::Min(1), Constraint::Length(31)])
            .spacing(Spacing::Overlap(1))
            .split(chunks[1]);

        // Connection progress steps
        let mut step_lines: Vec<Line> = vec![Line::raw("")];
        let max_label_len = self.steps.iter().map(|s| s.label.len()).max().unwrap_or(0);

        for step in &self.steps {
            let (icon, style) = match step.status {
                StepStatus::Pending => (" . ", RatStyle::new().fg(colors.sidebar_key)),
                StepStatus::InProgress => (" * ", RatStyle::new().fg(colors.input_text)),
                StepStatus::Done => (" + ", RatStyle::new().fg(colors.my_text)),
                StepStatus::Failed => (" x ", RatStyle::new().fg(colors.peer_text)),
            };

            let mut spans = vec![
                Span::styled(icon, style),
                Span::styled(
                    format!("{:<width$}", step.label, width = max_label_len),
                    style,
                ),
            ];

            if !step.detail.is_empty() {
                spans.push(Span::styled(
                    format!("  {}", step.detail),
                    RatStyle::new().fg(colors.sidebar_value),
                ));
            }

            step_lines.push(Line::from(spans));
        }
        step_lines.push(Line::raw(""));

        let progress = Paragraph::new(step_lines).block(
            Block::new()
                .title("── progress ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        );

        f.render_widget(progress, bottom_chunks[0]);

        // Info panel
        let info_chunks = Layout::vertical([Constraint::Length(7), Constraint::Min(0)])
            .spacing(Spacing::Overlap(1))
            .split(bottom_chunks[1]);

        // Identity panel
        let own_key = self
            .connect_info
            .as_ref()
            .map(|i| {
                let k = &i.own_public_key;
                format!("{}..{}", &k[..8], &k[k.len() - 8..])
            })
            .unwrap_or_else(|| "—".into());

        let identity_lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled(" key: ", RatStyle::new().fg(colors.sidebar_key)),
                Span::styled(own_key, RatStyle::new().fg(colors.sidebar_value)),
            ]),
            Line::from(vec![
                Span::styled(" stun: ", RatStyle::new().fg(colors.sidebar_key)),
                Span::styled(
                    self.connect_info
                        .as_ref()
                        .map(|i| i.stun_addr.as_str())
                        .unwrap_or("—"),
                    RatStyle::new().fg(colors.sidebar_value),
                ),
            ]),
            Line::from(vec![
                Span::styled(" signal: ", RatStyle::new().fg(colors.sidebar_key)),
                Span::styled(
                    self.connect_info
                        .as_ref()
                        .map(|i| i.signal_addr.as_str())
                        .unwrap_or("—"),
                    RatStyle::new().fg(colors.sidebar_value),
                ),
            ]),
            Line::raw(""),
        ];

        let identity_panel = Paragraph::new(identity_lines).block(
            Block::new()
                .title("── identity ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        );

        f.render_widget(identity_panel, info_chunks[0]);

        // Target peer panel
        let target_key = self
            .connect_info
            .as_ref()
            .map(|i| {
                let k = &i.target_key;
                format!("{}..{}", &k[..8], &k[k.len() - 8..])
            })
            .unwrap_or_else(|| "—".into());

        let target_alias = self
            .connect_info
            .as_ref()
            .and_then(|i| i.target_alias.as_deref())
            .unwrap_or("—");

        let target_lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled(" alias: ", RatStyle::new().fg(colors.sidebar_key)),
                Span::styled(target_alias, RatStyle::new().fg(colors.sidebar_value)),
            ]),
            Line::from(vec![
                Span::styled(" key: ", RatStyle::new().fg(colors.sidebar_key)),
                Span::styled(target_key, RatStyle::new().fg(colors.sidebar_value)),
            ]),
            Line::raw(""),
        ];

        let target_panel = Paragraph::new(target_lines).block(
            Block::new()
                .title("── target ")
                .title_bottom("── Esc to cancel ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        );

        f.render_widget(target_panel, info_chunks[1]);
    }
}
