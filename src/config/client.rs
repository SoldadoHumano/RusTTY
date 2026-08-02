use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub static PERFORMANCE_MODE: AtomicBool = AtomicBool::new(false);

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpCustomization {
    Unified(String),
    Split { public: String, private: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordTheme {
    pub id: uuid::Uuid,
    pub keyword: String,
    pub color: String,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomizationConfig {
    pub ipv4: Option<IpCustomization>,
    pub ipv6: Option<IpCustomization>,
    pub keywords: Vec<KeywordTheme>,
}

/// Configuração local do cliente RusTTY, que controla aspectos visuais e de
/// funcionamento do próprio emulador do terminal (ex: limite de histórico).
/// Essa configuração é salva em texto claro localmente, pois não possui credenciais.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// O número máximo de linhas de scrollback (histórico retido) no terminal.
    /// Se for 0, o scrollback fica desabilitado. O limite do usize é o teto, mas na prática
    /// o máximo deve ser ditado pela memória ou UI.
    pub max_scrollback_lines: usize,
    /// Modo de performance que desativa a renderização customizada de vetores de alta qualidade.
    pub performance_mode: bool,
    /// Habilita ou desabilita globalmente o teste de ICMP.
    #[serde(default = "default_true")]
    pub global_icmp: bool,
    /// Quantidade de linhas que o terminal irá rolar por cada "scroll" do mouse.
    #[serde(default = "default_scroll_lines")]
    pub scroll_lines: usize,
    /// Tecla de atalho usada com Ctrl para abrir a barra de comandos local (Command Palette).
    #[serde(default = "default_command_palette_key")]
    pub command_palette_key: char,
    /// Habilita ou desabilita o recurso de personalização.
    #[serde(default = "default_true")]
    pub enable_customization: bool,
    /// Dados das personalizações criadas pelo usuário.
    #[serde(default)]
    pub customization_data: CustomizationConfig,
}

fn default_scroll_lines() -> usize { 1 }
fn default_command_palette_key() -> char { '.' }

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            max_scrollback_lines: 4000,
            performance_mode: false,
            global_icmp: true,
            scroll_lines: 1,
            command_palette_key: '.',
            enable_customization: true,
            customization_data: CustomizationConfig::default(),
        }
    }
}

/// Retorna o caminho absoluto do arquivo de configuração local (client.rtty).
pub fn get_client_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ByVitor");
    path.push("RusTTY");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("client.rtty");
    path
}

/// Salva as configurações do client localmente em formato JSON (plano).
pub fn save_client_config(config: &ClientConfig) -> Result<(), String> {
    let json_bytes = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    let path = get_client_config_path();
    fs::write(path, json_bytes).map_err(|e| e.to_string())?;
    Ok(())
}

/// Lê as configurações do client, caso não exista retorna o padrão (default).
pub fn load_client_config() -> ClientConfig {
    let path = get_client_config_path();
    let cfg = if !path.exists() {
        ClientConfig::default()
    } else {
        match fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => ClientConfig::default(),
        }
    };
    
    PERFORMANCE_MODE.store(cfg.performance_mode, Ordering::Relaxed);
    cfg
}
