//! Ponto de entrada do RusTTY v2.1.0.
//!
//! # Modos de Operação
//!
//! ```text
//! rustty                       → Abre a interface principal moderna (Webview UI)
//! rustty --terminal <host>     → Abre terminal SSH nativo Win32 DirectWrite
//! ```

#![windows_subsystem = "windows"]

mod config;
mod debug;
mod net;
mod security;
mod terminal;
mod update;
mod webview_app;

use terminal::TerminalInit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Aplica proteções nativas de processo do Windows (DACL restritiva, mitigações, anti-debug)
    security::apply_process_security();

    // Carrega configurações do cliente para verificar debug_mode antes de qualquer operação
    let _client_cfg = config::client::load_client_config();

    let args: Vec<String> = std::env::args().collect();
    let is_child = std::env::var("RUSTTY_CHILD").is_ok()
        || args.iter().any(|a| {
            a == "--terminal" || a == "--quick-ssh" || a == "--bridge-terminal"
                || a == "--win32-terminal" || a == "--win32-quick-ssh" || a == "--win32-bridge-terminal"
        });

    // Inicializa logger debug (anexa se for processo filho, trunca se for o gerenciador principal)
    debug::init_debug_logger(is_child);
    debug_log!(
        "INFO",
        "Processo RusTTY v2.1.0 iniciado (PID: {}, role: {}, args: {:?})",
        std::process::id(),
        if is_child { "Terminal/SSH Child" } else { "Manager" },
        args
    );

    // Spawna terminal de debug APENAS a partir do processo principal/gerenciador
    if !is_child {
        debug::spawn_debug_terminal();
    }

    // Detecta modo terminal salvo: `rustty --terminal <host_name>` ou `rustty --win32-terminal <host_name>`
    if let Some(pos) = args.iter().position(|a| a == "--terminal" || a == "--win32-terminal") {
        let host_name = args.get(pos + 1).cloned().unwrap_or_default();
        debug_log!("INFO", "Roteando para Terminal Win32: host='{}'", host_name);
        #[cfg(windows)]
        terminal::win32::run_win32_terminal(TerminalInit::SavedHost(host_name))?;
        return Ok(());
    }

    // Detecta modo Quick Connect: `rustty --quick-ssh ...` ou `rustty --win32-quick-ssh ...`
    if let Some(pos) = args.iter().position(|a| a == "--quick-ssh" || a == "--win32-quick-ssh") {
        let address = args.get(pos + 1).cloned().unwrap_or_default();
        let port = args.get(pos + 2).and_then(|p| p.parse().ok()).unwrap_or(22);
        let user = args.get(pos + 3).cloned().unwrap_or_default();
        let raw_pass = args.get(pos + 4).cloned().unwrap_or_else(|| "-".to_string());
        
        let pass = if raw_pass == "-" {
            // Lê a senha de forma segura via stdin (evitando exposição no PEB/linha de comando)
            use std::io::Read;
            use zeroize::Zeroize;
            let mut stdin_buf = String::new();
            let _ = std::io::stdin().read_to_string(&mut stdin_buf);
            let final_pass = stdin_buf.trim_end_matches(&['\r', '\n'][..]).to_string();
            stdin_buf.zeroize();
            final_pass
        } else {
            raw_pass
        };

        debug_log!("INFO", "Roteando para Quick SSH Win32: {}@{}:{}", user, address, port);
        let init = TerminalInit::QuickSsh {
            address,
            port,
            user,
            pass,
        };
        #[cfg(windows)]
        terminal::win32::run_win32_terminal(init)?;
        return Ok(());
    }

    // Detecta modo bridge: `rustty --bridge-terminal <id>` ou `rustty --win32-bridge-terminal <id>`
    if let Some(pos) = args.iter().position(|a| a == "--bridge-terminal" || a == "--win32-bridge-terminal") {
        let id_str = args.get(pos + 1).cloned().unwrap_or_default();
        debug_log!("INFO", "Roteando para Bridge Terminal Win32: id='{}'", id_str);
        let init = TerminalInit::Bridge(id_str);
        #[cfg(windows)]
        terminal::win32::run_win32_terminal(init)?;
        return Ok(());
    }

    // Modo padrão: Gerenciador de conexões via Webview UI
    debug_log!("INFO", "Iniciando Gerenciador RusTTY v2.1.0 com interface Webview UI");
    webview_app::run()
}
