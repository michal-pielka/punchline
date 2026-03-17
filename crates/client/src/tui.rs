use chrono::{DateTime, Local};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Spacing},
    style::Style as RatStyle,
    symbols::merge::MergeStrategy,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::sync::mpsc::{Receiver, Sender};

use crate::style::Style;

pub struct PeerInfo {
    pub alias: Option<String>,
    pub public_key: String,
    pub addr: String,
}

pub struct App {
    messages: Vec<ChatMessage>,
    input: String,
    peer: Option<PeerInfo>,
    style: Style,
    phase: Phase,
    tx_out: Option<Sender<String>>,
    peer_disconnected: bool,
    pub should_quit: bool,
}

struct ChatMessage {
    text: String,
    sender: MessageSender,
    timestamp: DateTime<Local>,
}

enum MessageSender {
    Me,
    Peer,
}

impl ChatMessage {
    fn new(text: String, sender: MessageSender) -> Self {
        ChatMessage {
            text,
            sender,
            timestamp: Local::now(),
        }
    }
}

pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Connected {
        peer: PeerInfo,
        tx_out: Sender<String>,
    },
    MessageReceived(String),
    PeerDisconnected,
    Error(String),
}

#[derive(PartialEq)]
enum Phase {
    Connecting,
    Connected,
}

impl App {
    pub fn new(style: Style) -> Self {
        App {
            messages: Vec::new(),
            input: String::new(),
            peer: None,
            style,
            phase: Phase::Connecting,
            tx_out: None,
            peer_disconnected: false,
            should_quit: false,
        }
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connected { peer, tx_out } => {
                self.peer = Some(peer);
                self.tx_out = Some(tx_out);
                self.phase = Phase::Connected;
            }

            AppEvent::MessageReceived(msg) => {
                let chat_message = ChatMessage::new(msg, MessageSender::Peer);
                self.messages.push(chat_message);
            }

            AppEvent::PeerDisconnected => {
                let name = self.peer_display_name().to_string();
                let msg = ChatMessage::new(format!("{name} disconnected."), MessageSender::Peer);
                self.messages.push(msg);
                self.peer_disconnected = true;
            }

            AppEvent::Error(_) => {}

            AppEvent::Key(key) => self.handle_key(key),
        }
    }

    pub fn run(
        mut self,
        mut terminal: DefaultTerminal,
        rx: Receiver<AppEvent>,
    ) -> anyhow::Result<()> {
        while !self.should_quit {
            let event = rx.recv()?;
            self.handle_event(event);

            terminal.draw(|f| match self.phase {
                Phase::Connecting => self.render_connecting(f),
                Phase::Connected => self.render_chat(f),
            })?;
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Enter => {
                if self.phase != Phase::Connected {
                    return;
                }
                if self.input.is_empty() || self.peer_disconnected {
                    return;
                }

                let msg: String = self.input.drain(..).collect();
                let chat_message = ChatMessage::new(msg.clone(), MessageSender::Me);
                self.messages.push(chat_message);
                if let Some(tx_out) = &self.tx_out {
                    let _ = tx_out.send(msg);
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                if self.phase == Phase::Connected {
                    self.input.push(c);
                }
            }
            _ => {}
        }
    }

    // Connecting view
    fn render_connecting(&self, f: &mut Frame) {
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

    // Chat view
    fn render_chat(&self, f: &mut Frame) {
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

    fn render_messages(&self, height: u16) -> Paragraph<'_> {
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

    fn sidebar_line(&self, key: &str, value: &str) -> Line<'_> {
        let colors = &self.style.colors;
        Line::from(vec![
            Span::styled(format!(" {key}: "), RatStyle::new().fg(colors.sidebar_key)),
            Span::styled(value.to_string(), RatStyle::new().fg(colors.sidebar_value)),
        ])
    }

    fn render_peer_panel(&self) -> Paragraph<'_> {
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

    fn render_crypto_panel(&self) -> Paragraph<'_> {
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

    fn render_input(&self) -> Paragraph<'_> {
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

    fn peer_display_name(&self) -> &str {
        self.peer
            .as_ref()
            .and_then(|p| p.alias.as_deref())
            .unwrap_or("unknown")
    }

    fn truncated_peer_key(&self) -> String {
        match &self.peer {
            Some(p) => {
                let k = &p.public_key;
                format!("{}..{}", &k[..8], &k[k.len() - 8..])
            }
            None => "—".to_string(),
        }
    }
}
