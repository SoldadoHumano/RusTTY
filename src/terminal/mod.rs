//! Módulo de emulação de terminal.
pub mod grid;
pub use grid::{TerminalState, TerminalGrid, Cell, CellColor, MouseMode};

#[cfg(windows)]
pub mod win32;

#[derive(Debug, Clone)]
pub enum TerminalInit {
    SavedHost(String),
    QuickSsh {
        address: String,
        port: u16,
        user: String,
        pass: String,
    },
    Bridge(String),
}

impl Default for TerminalInit {
    fn default() -> Self {
        TerminalInit::SavedHost(String::new())
    }
}
