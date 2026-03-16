use chrono::{DateTime, Local};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Spacing},
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
    pub fn new(peer: PeerInfo) -> Self {
        App {
            messages: Vec::new(),
            input: String::new(),
            peer,
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
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .spacing(Spacing::Overlap(1))
        .split(top_chunks[1]);

        // Messages
        let text: Vec<Line> = self
            .messages
            .iter()
            .map(|m| {
                let prefix = match m.sender {
                    MessageSender::Me => "Me",
                    MessageSender::Peer => self.peer_display_name(),
                };
                Line::raw(format!(
                    " [{}] <{prefix}> {}",
                    m.timestamp.format("%H:%M"),
                    m.text
                ))
            })
            .collect();
        let messages = Paragraph::new(text).block(
            Block::new()
                .title(" PUNCHLINE ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .merge_borders(MergeStrategy::Exact),
        );

        // Peer panel
        let key_short = self.truncated_peer_key();
        let peer_name = self.peer_display_name();
        let peer = Paragraph::new(vec![
            Line::raw(format!(" ALIAS: {peer_name}")),
            Line::raw(format!(" KEY: {key_short}")),
            Line::raw(format!(" ADDR: {}", self.peer.addr)),
        ])
        .block(
            Block::new()
                .title("── PEER ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .merge_borders(MergeStrategy::Exact),
        );

        // Crypto panel
        let crypto = Paragraph::new(vec![
            Line::raw(" PATTERN: Noise IK"),
            Line::raw(" DH: X25519"),
            Line::raw(" CIPHER: ChaCha20Poly1305"),
            Line::raw(" HASH: SHA-256"),
        ])
        .block(
            Block::new()
                .title("── CRYPTO ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .merge_borders(MergeStrategy::Exact),
        );

        // Input
        let input = Paragraph::new(format!(" > {}", self.input)).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .merge_borders(MergeStrategy::Exact),
        );

        f.render_widget(messages, top_chunks[0]);
        f.render_widget(peer, sidebar_chunks[0]);
        f.render_widget(crypto, sidebar_chunks[1]);
        f.render_widget(input, main_chunks[1]);
    }

    fn peer_display_name(&self) -> &str {
        self.peer.alias.as_deref().unwrap_or("unknown")
    }

    fn truncated_peer_key(&self) -> String {
        let k = &self.peer.public_key;
        format!("{}..{}", &k[..8], &k[k.len() - 8..])
    }
}
