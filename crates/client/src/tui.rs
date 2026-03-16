use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    widgets::{Block, BorderType, Borders, Widget},
};
use std::sync::mpsc::{Receiver, Sender};

pub struct App {
    messages: Vec<String>,
    input: String,
    pub should_quit: bool,
    // pub state: AppState,
}

pub struct AppState {
    pub phase: Phase,
    pub steps: Vec<Step>,
    pub my_key: String,
    pub peer_key: String,
    pub peer_alias: Option<String>,
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            messages: Vec::new(),
            input: String::new(),
            should_quit: false,
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

            match event {
                AppEvent::MessageReceived(msg) | AppEvent::MessageSent(msg) => {
                    self.messages.push(msg);
                }

                AppEvent::Key(k) => {
                    if k.kind != KeyEventKind::Press {
                        continue;
                    }

                    match k.code {
                        KeyCode::Esc => self.should_quit = true,
                        KeyCode::Enter => {
                            if self.input.is_empty() {
                                continue;
                            }

                            let msg: String = self.input.drain(..).collect();
                            self.messages.push(msg.clone());
                            let _ = tx_out.send(msg);
                        }
                        _ => todo!(),
                    }
                }

                _ => todo!(),
            }

            terminal.draw(|f| self.render(f))?;

            // if let Event::Key(key) = event::read()? {
            //     self.handle_key(key);
            // }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            // Quitting the TUI
            KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let app_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        app_block.render(f.area(), f.buffer_mut());
    }
}
