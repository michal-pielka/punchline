use ratatui::DefaultTerminal;
use std::sync::mpsc::Receiver;

pub mod app;
pub mod events;
pub mod render;

pub use app::{App, ConnectInfo, PeerInfo, Phase};
pub use events::AppEvent;

impl app::App {
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
}
