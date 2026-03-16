use chrono::{DateTime, Local};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Spacing},
    style::Style as RatStyle,
    symbols::merge::MergeStrategy,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::sync::mpsc::{Receiver, Sender};

pub struct PeerInfo {
    pub alias: Option<String>,
    pub public_key: String,
    pub addr: String,
}

pub struct App {
    messages: Vec<ChatMessage>,
    input: String,
    peer: PeerInfo,
    style: crate::style::Style,
    pub should_quit: bool,
}

pub struct AppState {
    pub phase: Phase,
    pub steps: Vec<Step>,
    pub my_key: String,
    pub peer_key: String,
    pub peer_alias: Option<String>,
}

struct ChatMessage {
    text: String,
    sender: MessageSender,
    timestamp: DateTime<Local>,
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

enum MessageSender {
    Me,
    Peer,
}

pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Resize(u16, u16),
    StepUpdate {
        step: ConnectionStep,
        status: StepStatus,
        detail: String,
    },
    MessageReceived(String),
    MessageSent(String),
    Error(String),
}

pub enum Phase {
    Connecting,
    Connected,
    Disconnected,
}

pub struct Step {
    pub con_step: ConnectionStep,
    pub status: StepStatus,
    pub detail: String,
}

pub enum ConnectionStep {
    IdentityLoaded,
    StunResolved,
    SignalingComplete,
    HolePunching,
    KeyExchange,
    SecureChannel,
}

pub enum StepStatus {
    Pending,
    Active,
    Done,
    Failed,
}

impl App {
    pub fn new(peer: PeerInfo, style: crate::style::Style) -> Self {
        App {
            messages: Vec::new(),
            input: String::new(),
            peer,
            style,
            should_quit: false,
        }
    }

    fn handle_event(&mut self, event: AppEvent, tx_out: &Sender<String>) {
        match event {
            AppEvent::MessageReceived(msg) => {
                let chat_message = ChatMessage::new(msg, MessageSender::Peer);
                self.messages.push(chat_message);
            }

            AppEvent::MessageSent(msg) => {
                let chat_message = ChatMessage::new(msg, MessageSender::Me);
                self.messages.push(chat_message);
            }

            AppEvent::Key(key) => self.handle_key(key, tx_out),

            _ => {}
        }
    }

    pub fn run(
        mut self,
        mut terminal: DefaultTerminal,
        rx: Receiver<AppEvent>,
        tx_out: Sender<String>,
    ) -> anyhow::Result<()> {
        while !self.should_quit {
            let event = rx.recv()?;
            self.handle_event(event, &tx_out);

            terminal.draw(|f| self.render(f))?;
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, tx_out: &Sender<String>) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Enter => {
                if self.input.is_empty() {
                    return;
                }

                let msg: String = self.input.drain(..).collect();
                let chat_message = ChatMessage::new(msg.clone(), MessageSender::Me);
                self.messages.push(chat_message);
                let _ = tx_out.send(msg);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame) {
        // Main: top area (chat + sidebar) and bottom (input)
        let main_chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)])
            .spacing(Spacing::Overlap(1))
            .split(f.area());

        // Top: chat left, sidebar right
        let top_chunks = Layout::horizontal([Constraint::Min(1), Constraint::Length(31)])
            .spacing(Spacing::Overlap(1))
            .split(main_chunks[0]);

        // Sidebar: peer + crypto panels
        let sidebar_chunks = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .spacing(Spacing::Overlap(1))
        .split(top_chunks[1]);

        // Sidebar filler
        let sidebar_fill = Block::new()
            .borders(Borders::RIGHT)
            .border_type(BorderType::Plain)
            .border_style(RatStyle::new().fg(self.style.colors.border))
            .merge_borders(MergeStrategy::Exact);

        f.render_widget(self.render_messages(), top_chunks[0]);
        f.render_widget(self.render_peer_panel(), sidebar_chunks[0]);
        f.render_widget(self.render_crypto_panel(), sidebar_chunks[1]);
        f.render_widget(self.render_input(), main_chunks[1]);
        f.render_widget(sidebar_fill, sidebar_chunks[2]);
    }

    fn render_messages(&self) -> Paragraph<'_> {
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
        Paragraph::new(text).block(
            Block::new()
                .title(" punchline ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        )
    }

    fn render_peer_panel(&self) -> Paragraph<'_> {
        let colors = &self.style.colors;
        let key_short = self.truncated_peer_key();
        let peer_name = self.peer_display_name();
        Paragraph::new(vec![
            Line::raw(""),
            Line::raw(format!(" alias: {peer_name}")),
            Line::raw(format!(" key: {key_short}")),
            Line::raw(format!(" addr: {}", self.peer.addr)),
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
            Line::raw(" pattern: Noise IK"),
            Line::raw(" dh: X25519"),
            Line::raw(" cipher: ChaCha20Poly1305"),
            Line::raw(" hash: SHA-256"),
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
        Paragraph::new(format!(" > {}", self.input)).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(RatStyle::new().fg(colors.border))
                .merge_borders(MergeStrategy::Exact),
        )
    }

    fn peer_display_name(&self) -> &str {
        self.peer.alias.as_deref().unwrap_or("unknown")
    }

    fn truncated_peer_key(&self) -> String {
        let k = &self.peer.public_key;
        format!("{}..{}", &k[..8], &k[k.len() - 8..])
    }
}
