use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::{Block, BorderType, Borders, Widget},
};

pub struct App {
    pub should_quit: bool,
    pub state: AppState,
}

pub struct AppState {
    pub phase: Phase,
    pub steps: Vec<Step>,
    pub my_key: String,
    pub peer_key: String,
    pub peer_alias: Option<String>,
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
    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
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
