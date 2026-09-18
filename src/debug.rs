//! Módulo de logging debug do RusTTY.
//!
//! # Arquitetura
//! - `debug_log!()` — macro de logging condicional (zero-cost quando debug desativado)
//! - `init_debug_logger(is_child: bool)` — inicializa ou anexa ao arquivo de log centralizado
//! - `spawn_debug_terminal()` — abre um ÚNICO terminal PowerShell com tail do log
//!
//! # Segurança
//! - Nenhum dado sensível (senhas, chaves) é escrito nos logs

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
pub fn log_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ByVitor");
    path.push("RusTTY");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("debug.log");
    path
}

/// Inicializa o logger debug.
///
/// Se `is_child` for false (processo principal/gerenciador), cria/trunca o arquivo
/// e escreve o cabeçalho de inicialização.
/// Se `is_child` for true (sessão de terminal spawnada), apenas anexa (append) ao
/// arquivo compartilhado existente sem truncá-lo nem spawnar novas janelas.
pub fn init_debug_logger(is_child: bool) {
    if !DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let path = log_path();
    let file_result = if is_child {
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
    };

    match file_result {
        Ok(file) => {
            if let Ok(mut guard) = LOG_FILE.lock() {
                *guard = Some(file);
            }
            let pid = std::process::id();
            if !is_child {
                write_log_entry("INFO", "================================================================================");
                write_log_entry("INFO", &format!("RusTTY v1.2.0 — Sessão do Gerenciador Iniciada (PID: {})", pid));
                write_log_entry("INFO", &format!("Arquivo de Log: {}", path.display()));
                write_log_entry("INFO", "================================================================================");
            } else {
                write_log_entry("INFO", &format!(">>> Processo de Terminal Conectado ao Log Central (PID: {}) <<<", pid));
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Falha ao abrir arquivo de log: {}", e);
        }
    }
}

/// Escreve uma entrada formatada no arquivo de log.
///
/// Formato: `[HH:MM:SS.mmm] [LEVEL] [PID:xxxx] mensagem`
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
    let hours = (total_secs / 3600) % 24;
    let mins = (total_secs / 60) % 60;
    let secs = total_secs % 60;
    let pid = std::process::id();

    let entry = format!(
        "[{:02}:{:02}:{:02}.{:03}] [{:<5}] [PID:{}] {}\n",
        hours, mins, secs, millis, level, pid, message
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
/// Garante que NUNCA seja aberto mais de um terminal de diagnóstico ao mesmo tempo.
/// Se uma janela com o título 'RusTTY Debug Log' já existir, a nova instância fecha
/// imediatamente.
pub fn spawn_debug_terminal() {
    if !DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let path = log_path();
    let path_str = path.to_string_lossy().to_string();

    let ps_command = format!(
        "$isNew = $false; \
         $mutex = New-Object System.Threading.Mutex($true, 'Global\\RusTTY_Debug_Terminal_Mutex', [ref]$isNew); \
         if (-not $isNew -and -not $mutex.WaitOne(200, $false)) {{ exit }}; \
         $title = 'RusTTY Debug Log'; \
         $existing = Get-Process powershell, pwsh -ErrorAction SilentlyContinue | Where-Object {{ $_.MainWindowTitle -like '*RusTTY Debug Log*' -and $_.Id -ne $PID }}; \
         if ($existing) {{ exit }}; \
         $host.ui.RawUI.WindowTitle = $title; \
         Clear-Host; \
         Write-Host '========================================================================' -ForegroundColor DarkYellow; \
         Write-Host '                 RusTTY v1.2.0 — Console de Diagnóstico                 ' -ForegroundColor Yellow; \
         Write-Host '========================================================================' -ForegroundColor DarkYellow; \
         Get-Content -Path '{}' -Wait -Tail 100",
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
#[macro_export]
macro_rules! debug_log {
    ($level:expr, $($arg:tt)*) => {
        if $crate::config::client::DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::debug::write_log_entry($level, &format!($($arg)*));
        }
    };
}
