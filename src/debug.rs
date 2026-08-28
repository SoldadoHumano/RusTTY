//! Módulo de logging debug do RusTTY.
//!
//! # Arquitetura
//! - `debug_log!()` — macro de logging condicional (zero-cost quando debug desativado)
//! - `init_debug_logger()` — inicializa o arquivo de log
//! - `spawn_debug_terminal()` — abre terminal PowerShell com tail do log
//!
//! # Segurança
//! - Nenhum dado sensível (senhas, chaves) é escrito nos logs
//! - O arquivo de log é truncado a cada inicialização para evitar acúmulo

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::config::client::DEBUG_MODE;

/// Handle global do arquivo de log, protegido por Mutex para acesso thread-safe.
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

/// Retorna o caminho absoluto do arquivo de log debug.
///
/// Localização: `%APPDATA%/ByVitor/RusTTY/debug.log`
fn log_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ByVitor");
    path.push("RusTTY");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("debug.log");
    path
}

/// Inicializa o logger debug, truncando o arquivo de log anterior.
///
/// Deve ser chamada uma vez no início do programa (`main()`).
/// Se `debug_mode` estiver desativado, o arquivo não é criado.
pub fn init_debug_logger() {
    if !DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let path = log_path();
    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
    {
        Ok(file) => {
            if let Ok(mut guard) = LOG_FILE.lock() {
                *guard = Some(file);
            }
            // Escreve header inicial
            write_log_entry("INFO", "RusTTY Debug Logger inicializado");
            write_log_entry("INFO", &format!("Log: {}", path.display()));
        }
        Err(e) => {
            eprintln!("[DEBUG] Falha ao criar arquivo de log: {}", e);
        }
    }
}

/// Escreve uma entrada formatada no arquivo de log.
///
/// Formato: `[HH:MM:SS.mmm] [LEVEL] mensagem`
///
/// Thread-safe via Mutex. Se o lock falhar ou o arquivo não estiver
/// inicializado, a entrada é silenciosamente descartada.
pub fn write_log_entry(level: &str, message: &str) {
    if !DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = now.as_secs();
    let millis = now.subsec_millis();
    // Horário local aproximado (UTC offset não é crítico para debug)
    let hours = (total_secs / 3600) % 24;
    let mins = (total_secs / 60) % 60;
    let secs = total_secs % 60;

    let entry = format!(
        "[{:02}:{:02}:{:02}.{:03}] [{}] {}\n",
        hours, mins, secs, millis, level, message
    );

    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let _ = file.write_all(entry.as_bytes());
            let _ = file.flush();
        }
    }
}

/// Abre um terminal PowerShell que faz tail do arquivo de log em tempo real.
///
/// O terminal é um processo independente (`cmd.exe` → `powershell`) que
/// sobrevive ao ciclo de vida do RusTTY. Fecha automaticamente quando
/// o usuário fecha a janela do terminal de debug.
pub fn spawn_debug_terminal() {
    if !DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let path = log_path();
    let path_str = path.to_string_lossy().to_string();

    // Usa cmd.exe para abrir PowerShell com Get-Content -Wait (equivalente a tail -f)
    let ps_command = format!(
        "Get-Content -Path '{}' -Wait -Tail 50",
        path_str
    );

    let _ = std::process::Command::new("cmd.exe")
        .args([
            "/C",
            "start",
            "RusTTY Debug Log",
            "powershell.exe",
            "-NoExit",
            "-Command",
            &ps_command,
        ])
        .spawn();
}

/// Macro de logging condicional para o modo debug.
///
/// # Uso
/// ```ignore
/// debug_log!("INFO", "Conectando a {}:{}", host, port);
/// debug_log!("ERROR", "Falha de autenticação: {}", err);
/// ```
///
/// # Performance
/// Quando `DEBUG_MODE` está desativado, a macro verifica o `AtomicBool`
/// (operação ~1ns) e retorna imediatamente sem formatação de string.
#[macro_export]
macro_rules! debug_log {
    ($level:expr, $($arg:tt)*) => {
        if $crate::config::client::DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::debug::write_log_entry($level, &format!($($arg)*));
        }
    };
}
