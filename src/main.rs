//! Ponto de entrada do RusTTY.
//!
//! # Modos de Operação
//!
//! ```text
//! rustty                       → Abre o gerenciador de conexões (RusTTYApp)
//! rustty --terminal <host>     → Abre janela de terminal SSH para o host (TerminalApp)
//! ```
//!
//! O roteamento por args permite que o gerenciador spawne janelas de terminal
//! independentes sem exigir multi-janela no mesmo processo, contornando a
//! limitação do Iced 0.12.

mod app;
mod config;
mod net;
mod terminal;
mod terminal_app;
mod ui;

use iced::{Application, Settings};
use app::RusTTYApp;
use terminal_app::run_terminal;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();

    // Detecta modo terminal salvo: `rustty --terminal <host_name>`
    if let Some(pos) = args.iter().position(|a| a == "--terminal") {
        let host_name = args.get(pos + 1).cloned().unwrap_or_default();
        return run_terminal(terminal_app::TerminalInit::SavedHost(host_name));
    }

    // Detecta modo Quick Connect: `rustty --quick-ssh <address> <port> <user> <pass>`
    if let Some(pos) = args.iter().position(|a| a == "--quick-ssh") {
        let address = args.get(pos + 1).cloned().unwrap_or_default();
        let port = args.get(pos + 2).and_then(|p| p.parse().ok()).unwrap_or(22);
        let user = args.get(pos + 3).cloned().unwrap_or_default();
        let pass = args.get(pos + 4).cloned().unwrap_or_else(|| "none".to_string());
        
        return run_terminal(terminal_app::TerminalInit::QuickSsh {
            address,
            port,
            user,
            pass,
        });
    }

    // Modo padrão: gerenciador de conexões
    RusTTYApp::run(Settings {
        // Registra a fonte Lucide para que o renderer possa exibir os ícones.
        // O TTF está embutido no binário via include_bytes! em ui::icons.
        fonts: vec![],
        window: iced::window::Settings {
            size: iced::Size::new(800.0, 600.0),
            min_size: Some(iced::Size::new(600.0, 400.0)),
            icon: crate::ui::icons::load_window_icon(),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
