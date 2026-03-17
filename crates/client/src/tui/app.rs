use crate::tui::AppEvent;
use ratatui::DefaultTerminal;
use std::sync::mpsc::Receiver;

use chrono::{DateTime, Local};
use std::sync::mpsc::Sender;

use crate::style::Style;

pub struct PeerInfo {
    pub alias: Option<String>,
    pub public_key: String,
    pub addr: String,
}

pub struct ConnectInfo {
    pub own_public_key: String,
    pub target_key: String,
    pub target_alias: Option<String>,
    pub stun_addr: String,
    pub signal_addr: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Done,
    Failed,
}

pub struct ConnectionStep {
    pub label: &'static str,
    pub status: StepStatus,
    pub detail: String,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub peer: Option<PeerInfo>,
    pub style: Style,
    pub phase: Phase,
    pub tx_out: Option<Sender<String>>,
    pub peer_disconnected: bool,
    pub should_quit: bool,
    pub connect_info: Option<ConnectInfo>,
    pub steps: Vec<ConnectionStep>,
}

pub struct ChatMessage {
    pub text: String,
    pub sender: MessageSender,
    pub timestamp: DateTime<Local>,
}

pub enum MessageSender {
    Me,
    Peer,
}

impl ChatMessage {
    pub fn new(text: String, sender: MessageSender) -> Self {
        ChatMessage {
            text,
            sender,
            timestamp: Local::now(),
        }
    }
}

#[derive(PartialEq)]
pub enum Phase {
    Connecting,
    Connected,
}

impl App {
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

    pub fn new(style: Style, connect_info: ConnectInfo) -> Self {
        let steps = vec![
            ConnectionStep {
                label: "stun discovery",
                status: StepStatus::InProgress,
                detail: String::new(),
            },
            ConnectionStep {
                label: "signal server",
                status: StepStatus::Pending,
                detail: String::new(),
            },
            ConnectionStep {
                label: "waiting for peer",
                status: StepStatus::Pending,
                detail: String::new(),
            },
            ConnectionStep {
                label: "hole punch",
                status: StepStatus::Pending,
                detail: String::new(),
            },
            ConnectionStep {
                label: "noise handshake",
                status: StepStatus::Pending,
                detail: String::new(),
            },
        ];

        App {
            messages: Vec::new(),
            input: String::new(),
            peer: None,
            style,
            phase: Phase::Connecting,
            tx_out: None,
            peer_disconnected: false,
            should_quit: false,
            connect_info: Some(connect_info),
            steps,
        }
    }

    pub fn advance_step(&mut self, step: usize, detail: String) {
        if let Some(s) = self.steps.get_mut(step) {
            s.status = StepStatus::Done;
            s.detail = detail;
        }
        if let Some(next) = self.steps.get_mut(step + 1) {
            next.status = StepStatus::InProgress;
        }
    }

    pub fn fail_step(&mut self, step: usize, detail: String) {
        if let Some(s) = self.steps.get_mut(step) {
            s.status = StepStatus::Failed;
            s.detail = detail;
        }
    }
}
