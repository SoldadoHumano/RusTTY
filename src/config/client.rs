use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub static PERFORMANCE_MODE: AtomicBool = AtomicBool::new(false);
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

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
    /// Permite abrir múltiplas conexões para o mesmo host.
    #[serde(default)]
    pub allow_multiple_access_to_same_host: bool,
    /// Dados das personalizações criadas pelo usuário.
    #[serde(default)]
    pub customization_data: CustomizationConfig,
    /// Habilita atualização automática do RusTTY.
    #[serde(default = "default_true")]
    pub enable_auto_update: bool,
    /// Tamanho da fonte do terminal SSH
    #[serde(default = "default_terminal_font_size")]
    pub terminal_font_size: u8,
    /// Modo debug: abre terminal de logging com informações detalhadas de conexão.
    #[serde(default)]
    pub debug_mode: bool,
    /// Habilita antialiasing no Iced.
    #[serde(default)]
    pub antialiasing: bool,
    /// [Experimental] Habilita a nova interface baseada em webview.
    #[serde(default)]
    pub experimental_webview_ui: bool,
}

fn default_scroll_lines() -> usize { 1 }
fn default_command_palette_key() -> char { '.' }
fn default_terminal_font_size() -> u8 { 14 }

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            max_scrollback_lines: 4000,
            performance_mode: false,
            global_icmp: true,
            scroll_lines: 1,
            command_palette_key: '.',
            enable_customization: true,
            allow_multiple_access_to_same_host: false,
            customization_data: CustomizationConfig::default(),
            enable_auto_update: true,
            terminal_font_size: 14,
            debug_mode: false,
            antialiasing: false,
            experimental_webview_ui: false,
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
    DEBUG_MODE.store(cfg.debug_mode, Ordering::Relaxed);
    cfg
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    pub key: String,
    pub label: String,
    pub description: String,
    pub setting_type: String, // "boolean", "number", "text", "char"
    pub category: String,
    pub webview_only: bool,
}

impl SettingDefinition {
    pub fn new(key: &str, label: &str, desc: &str, t: &str, cat: &str, wo: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            description: desc.to_string(),
            setting_type: t.to_string(),
            category: cat.to_string(),
            webview_only: wo,
        }
    }
}

pub fn get_settings_schema() -> Vec<SettingDefinition> {
    vec![
        SettingDefinition::new("experimental_webview_ui", "Interface Webview (Beta)", "Usar a nova interface moderna construída com tecnologias web.", "boolean", "Geral", true),
        SettingDefinition::new("enable_auto_update", "Atualização Automática", "Busca e instala atualizações silenciosamente em segundo plano.", "boolean", "Geral", false),
        SettingDefinition::new("command_palette_key", "Tecla da Command Palette", "Ex: '.' para Ctrl+.", "char", "Geral", false),
        SettingDefinition::new("allow_multiple_access_to_same_host", "Múltiplas Conexões", "Permitir abrir o mesmo Host várias vezes simultaneamente.", "boolean", "Geral", false),
        
        SettingDefinition::new("global_icmp", "ICMP Global", "Monitorar o status online/offline dos Hosts automaticamente via Ping.", "boolean", "Rede", false),
        
        SettingDefinition::new("terminal_font_size", "Tamanho da Fonte", "Tamanho da fonte renderizada no terminal (padrão 14).", "number", "Terminal", false),
        SettingDefinition::new("max_scrollback_lines", "Linhas de Histórico", "Máximo de linhas retidas no buffer para rolagem para cima.", "number", "Terminal", false),
        SettingDefinition::new("scroll_lines", "Sensibilidade do Scroll", "Quantidade de linhas puladas a cada rolagem do mouse.", "number", "Terminal", false),
        SettingDefinition::new("antialiasing", "Antialiasing (Suavização)", "Suavizar renderização de fontes e formas. Pode impactar o consumo da CPU no Iced.", "boolean", "Terminal", false),
        SettingDefinition::new("performance_mode", "Modo Performance", "Reduz as atualizações da UI para maximizar a fluidez no terminal legacy.", "boolean", "Terminal", false),
        
        SettingDefinition::new("enable_customization", "Personalização (Highlighter)", "Habilitar destaque inteligente de IPs e palavras customizadas.", "boolean", "Personalização", false),
        SettingDefinition::new("debug_mode", "Modo Debug", "Habilitar modo de diagnóstico extra (Requer reinício).", "boolean", "Avançado", false),
    ]
}
