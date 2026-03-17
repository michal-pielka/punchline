use chrono::{DateTime, Local};
use std::sync::mpsc::Sender;

use crate::style::Style;

pub struct PeerInfo {
    pub alias: Option<String>,
    pub public_key: String,
    pub addr: String,
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
}
