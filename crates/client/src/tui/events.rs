use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::sync::mpsc::Sender;

use crate::tui::app::{App, ChatMessage, MessageSender, Phase};

pub enum AppEvent {
    Key(KeyEvent),
    Resize,
    StepComplete {
        step: usize,
        detail: String,
    },
    StepFailed {
        step: usize,
        detail: String,
    },
    Connected {
        peer: crate::tui::app::PeerInfo,
        tx_out: Sender<String>,
    },
    MessageReceived(String),
    PeerDisconnected,
    Error(String),
}

impl App {
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StepComplete { step, detail } => {
                self.advance_step(step, detail);
            }

            AppEvent::StepFailed { step, detail } => {
                self.fail_step(step, detail);
            }

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
                let msg = ChatMessage::new("disconnected.".to_string(), MessageSender::Peer);
                self.messages.push(msg);
                self.peer_disconnected = true;
            }

            AppEvent::Error(_) => {}

            AppEvent::Resize => {}

            AppEvent::Key(key) => self.handle_key(key),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
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
}
