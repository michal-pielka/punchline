use chrono::{DateTime, Local};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
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
        let chunks = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

        // Header
        let key_short = self.truncated_peer_key();
        let peer_name = self.peer.alias.as_deref().unwrap_or(&key_short);
        let header_top = " PUNCHLINE │ Noise_IK·ChaCha20·SHA256".to_string();
        let header_bot = format!(" peer: {peer_name} │ {key_short} │ {}", self.peer.addr);
        let header = Paragraph::new(vec![Line::raw(header_top), Line::raw(header_bot)]).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        );

        // Messages
        let text: Vec<Line> = self
            .messages
            .iter()
            .map(|m| {
                let prefix = match m.sender {
                    MessageSender::Me => "You",
                    MessageSender::Peer => "Peer",
                };
                Line::raw(format!(
                    " [{}] {prefix}: {}",
                    m.timestamp.format("%H:%M"),
                    m.text
                ))
            })
            .collect();
        let messages = Paragraph::new(text).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        );

        // Input
        let input = Paragraph::new(format!(" > {}", self.input)).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        );

        f.render_widget(header, chunks[0]);
        f.render_widget(messages, chunks[1]);
        f.render_widget(input, chunks[2]);
    }

    fn truncated_peer_key(&self) -> String {
        let k = &self.peer.public_key;
        format!("{}..{}", &k[..8], &k[k.len() - 8..])
    }
}
