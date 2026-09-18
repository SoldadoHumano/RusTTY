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

#![windows_subsystem = "windows"]

mod app;
mod config;
mod debug;
mod net;
mod terminal;
mod terminal_app;
mod ui;
mod update;
mod webview_app;

use iced::{Application, Settings};
use app::RusTTYApp;
use terminal_app::run_terminal;

fn main() -> iced::Result {
    // Carrega configurações do cliente para verificar debug_mode antes de qualquer operação
    let _client_cfg = config::client::load_client_config();

    let args: Vec<String> = std::env::args().collect();
    let is_child = std::env::var("RUSTTY_CHILD").is_ok()
        || args.iter().any(|a| a == "--terminal" || a == "--quick-ssh" || a == "--bridge-terminal");

    // Inicializa logger debug (anexa se for processo filho, trunca se for o gerenciador principal)
    debug::init_debug_logger(is_child);
    debug_log!(
        "INFO",
        "Processo RusTTY iniciado (PID: {}, role: {}, args: {:?})",
        std::process::id(),
        if is_child { "Terminal/SSH Child" } else { "Manager" },
        args
    );

    // Spawna terminal de debug APENAS a partir do processo principal/gerenciador
    if !is_child {
        debug::spawn_debug_terminal();
    }

    // Detecta modo terminal salvo: `rustty --terminal <host_name>`
    if let Some(pos) = args.iter().position(|a| a == "--terminal") {
        let host_name = args.get(pos + 1).cloned().unwrap_or_default();
        debug_log!("INFO", "Roteando para Terminal Salvo: host='{}'", host_name);
        return run_terminal(terminal_app::TerminalInit::SavedHost(host_name));
    }

    // Detecta modo Quick Connect: `rustty --quick-ssh <address> <port> <user> <pass>`
    if let Some(pos) = args.iter().position(|a| a == "--quick-ssh") {
        let address = args.get(pos + 1).cloned().unwrap_or_default();
        let port = args.get(pos + 2).and_then(|p| p.parse().ok()).unwrap_or(22);
        let user = args.get(pos + 3).cloned().unwrap_or_default();
        let pass = args.get(pos + 4).cloned().unwrap_or_else(|| "none".to_string());
        
        debug_log!("INFO", "Roteando para Quick SSH: {}@{}:{}", user, address, port);
        return run_terminal(terminal_app::TerminalInit::QuickSsh {
            address,
            port,
            user,
            pass,
        });
    }

    // Detecta modo bridge: `rustty --bridge-terminal <id>`
    if let Some(pos) = args.iter().position(|a| a == "--bridge-terminal") {
        let id_str = args.get(pos + 1).cloned().unwrap_or_default();
        debug_log!("INFO", "Roteando para Bridge Terminal: id='{}'", id_str);
        return run_terminal(terminal_app::TerminalInit::Bridge(id_str));
    }

    // Modo padrão: gerenciador de conexões
    if _client_cfg.experimental_webview_ui {
        debug_log!("INFO", "Iniciando Gerenciador RusTTY com interface Webview UI");
        return webview_app::run();
    }

    debug_log!("INFO", "Iniciando Gerenciador RusTTY com interface Iced Nativa");

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
        antialiasing: _client_cfg.antialiasing,
        ..Settings::default()
    })
}
