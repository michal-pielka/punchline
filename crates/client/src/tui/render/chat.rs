use ratatui::{
    Frame,
    layout::{Constraint, Layout, Spacing},
    style::Style as RatStyle,
    symbols::merge::MergeStrategy,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::tui::app::{App, MessageSender};

impl App {
    pub fn render_chat(&self, f: &mut Frame) {
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

        let main_chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
            .spacing(Spacing::Overlap(1))
            .split(area);

        let top_chunks = Layout::horizontal([Constraint::Min(1), Constraint::Length(31)])
            .spacing(Spacing::Overlap(1))
            .split(main_chunks[0]);

        let sidebar_chunks = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .spacing(Spacing::Overlap(1))
        .split(top_chunks[1]);

        let sidebar_fill = Block::new()
            .borders(Borders::RIGHT)
            .border_type(BorderType::Plain)
            .border_style(RatStyle::new().fg(self.style.colors.border))
            .merge_borders(MergeStrategy::Exact);

        f.render_widget(self.render_messages(top_chunks[0].height), top_chunks[0]);
        f.render_widget(self.render_peer_panel(), sidebar_chunks[0]);
        f.render_widget(self.render_crypto_panel(), sidebar_chunks[1]);
        f.render_widget(self.render_input(), main_chunks[1]);
        f.render_widget(sidebar_fill, sidebar_chunks[2]);
    }

    pub fn render_messages(&self, height: u16) -> Paragraph<'_> {
        let colors = &self.style.colors;
        let text: Vec<Line> = self
            .messages
            .iter()
            .map(|m| {
                let (prefix, color) = match m.sender {
                    MessageSender::Me => ("Me", colors.my_text),
                    MessageSender::Peer => (self.peer_display_name(), colors.peer_text),
                };
                Line::raw(format!(
                    " [{}] <{prefix}> {}",
                    m.timestamp.format("%H:%M"),
                    m.text
                ))
                .style(RatStyle::new().fg(color))
            })
            .collect();

        let visible = height.saturating_sub(2) as usize;
        let scroll = text.len().saturating_sub(visible) as u16;

        Paragraph::new(text).scroll((scroll, 0)).block(
            Block::new()
                .title(" punchline ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        )
    }

    pub fn sidebar_line(&self, key: &str, value: &str) -> Line<'_> {
        let colors = &self.style.colors;
        Line::from(vec![
            Span::styled(format!(" {key}: "), RatStyle::new().fg(colors.sidebar_key)),
            Span::styled(value.to_string(), RatStyle::new().fg(colors.sidebar_value)),
        ])
    }

    pub fn render_peer_panel(&self) -> Paragraph<'_> {
        let colors = &self.style.colors;
        let key_short = self.truncated_peer_key();
        let peer_name = self.peer_display_name().to_string();
        Paragraph::new(vec![
            Line::raw(""),
            self.sidebar_line("alias", &peer_name),
            self.sidebar_line("key", &key_short),
            self.sidebar_line(
                "addr",
                self.peer.as_ref().map(|p| p.addr.as_str()).unwrap_or("—"),
            ),
            Line::raw(""),
        ])
        .block(
            Block::new()
                .title("── peer ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        )
    }

    pub fn render_crypto_panel(&self) -> Paragraph<'_> {
        let colors = &self.style.colors;
        Paragraph::new(vec![
            Line::raw(""),
            self.sidebar_line("pattern", "Noise IK"),
            self.sidebar_line("dh", "X25519"),
            self.sidebar_line("cipher", "ChaCha20Poly1305"),
            self.sidebar_line("hash", "SHA-256"),
            Line::raw(""),
        ])
        .block(
            Block::new()
                .title("── crypto ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        )
    }

    pub fn render_input(&self) -> Paragraph<'_> {
        let colors = &self.style.colors;
        Paragraph::new(format!(" > {}", self.input))
            .style(RatStyle::new().fg(colors.input_text))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(RatStyle::new().fg(colors.border))
                    .merge_borders(MergeStrategy::Exact),
            )
    }

    pub fn peer_display_name(&self) -> &str {
        self.peer
            .as_ref()
            .and_then(|p| p.alias.as_deref())
            .unwrap_or("unknown")
    }

    pub fn truncated_peer_key(&self) -> String {
        match &self.peer {
            Some(p) => {
                let k = &p.public_key;
                format!("{}..{}", &k[..8], &k[k.len() - 8..])
            }
            None => "—".to_string(),
        }
    }
}
