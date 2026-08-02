//! Módulo principal da aplicação Iced — define estado, mensagens e views.
//!
//! Arquitetura da UI:
//!   - `RusTTYApp` é a Application Iced (estado global + estado transitório do formulário)
//!   - `View` define qual tela está ativa (sem View::Terminal — terminal abre em nova janela)
//!   - `Message` é o enum de eventos da UI (puro, sem side-effects)
//!   - Views são métodos separados por responsabilidade (SoC)
//!   - Validação de endereço está isolada em `validate_address`
//!   - `OpenTerminal` spawna um novo processo filho `rustty --terminal <host_name>`
//!   - `delete_confirm` guarda o índice do host aguardando confirmação de deleção

use iced::{
    executor, theme, time,
    widget::{
        button, checkbox, column, container, mouse_area, row, scrollable, text, text_input,
        Space,
    },
    Alignment, Application, Color, Command, Element, Length, Theme, Subscription,
};
use iced_aw::Modal;
use std::collections::HashMap;

use crate::ui::icons::{icon, icon_sized, icon_colored, LucideIcon};
use crate::config::{
    load_config, save_config, AppConfig, AuthType, ConfigNode, HostProfile,
    client::{load_client_config, save_client_config, ClientConfig},
};

// ─── Estado do formulário de novo host ───────────────────────────────────────

/// Estado transitório do formulário de criação de host SSH.
///
/// Invariante: `error` é `None` quando os campos são válidos.
#[derive(Debug, Clone, Default)]
pub struct NewHostForm {
    /// Apelido/nome de exibição do host (ex: "Servidor Prod")
    pub name: String,
    /// Endereço — IPv4 por padrão; domínio apenas se `allow_domain = true`
    pub address: String,
    /// Porta SSH (string para permitir edição parcial; padrão "22")
    pub port: String,
    /// Nome de usuário SSH
    pub username: String,
    /// Senha SSH (em texto claro apenas em memória; criptografada ao salvar)
    pub password: String,
    /// Se `true`, aceita domínios além de IPs numéricos
    pub allow_domain: bool,
    /// Controla visibilidade da senha no campo
    pub show_password: bool,
    /// Se deve realizar o teste de ICMP periodicamente
    pub enable_icmp: bool,
    /// Mensagem de erro de validação atual
    pub error: Option<String>,
}

impl NewHostForm {
    fn new() -> Self {
        Self {
            port: "22".to_string(),
            enable_icmp: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickConnectProtocol {
    Ssh,
    Telnet,
    Serial,
}

#[derive(Debug, Clone)]
pub struct QuickConnectForm {
    pub protocol: QuickConnectProtocol,
    pub address: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub allow_domain: bool,
    pub show_password: bool,
    pub error: Option<String>,
}

impl Default for QuickConnectForm {
    fn default() -> Self {
        Self {
            protocol: QuickConnectProtocol::Ssh,
            address: String::new(),
            port: "22".to_string(),
            username: String::new(),
            password: String::new(),
            allow_domain: false,
            show_password: false,
            error: None,
        }
    }
}

// ─── Estado de Formulários de Personalização ─────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CustomizationViewMode {
    List,
    EditIpv4,
    EditIpv6,
    EditKeyword(Option<usize>), // None = Novo, Some(idx) = Editar existente
}

#[derive(Debug, Clone)]
pub struct CustomizationState {
    pub mode: CustomizationViewMode,
    // Campos para Keyword
    pub kw_keyword: String,
    pub kw_color: String,
    pub kw_case_insensitive: bool,
    pub kw_error: Option<String>,
    // Campos para IPv4 / IPv6
    pub ip_split: bool,
    pub ip_unified_color: String,
    pub ip_public_color: String,
    pub ip_private_color: String,
}

impl Default for CustomizationState {
    fn default() -> Self {
        Self {
            mode: CustomizationViewMode::List,
            kw_keyword: String::new(),
            kw_color: "#FF7300".to_string(),
            kw_case_insensitive: false,
            kw_error: None,
            ip_split: false,
            ip_unified_color: "#FF7300".to_string(),
            ip_public_color: "#33D980".to_string(),
            ip_private_color: "#F24C4C".to_string(),
        }
    }
}

// ─── Estado Global ────────────────────────────────────────────────────────────

pub struct RusTTYApp {
    config: AppConfig,
    client_config: ClientConfig,
    current_view: View,
    /// Estado do formulário — habitado apenas quando `current_view == View::NewHost`
    new_host_form: NewHostForm,
    /// Estado do formulário de conexão rápida
    quick_connect_form: QuickConnectForm,
    /// Estado da aba de personalização
    customization_state: CustomizationState,
    /// Índice do host aguardando confirmação de deleção permanente.
    /// `None` quando nenhuma deleção está pendente.
    delete_confirm: Option<usize>,
    /// Erro ao tentar abrir terminal (ex: exe não encontrado)
    spawn_error: Option<String>,
    /// Índice do host sendo editado atualmente.
    editing_host: Option<usize>,
    
    // Estado da tela de Configurações
    settings_scrollback_input: String,
    settings_scroll_lines_input: String,
    settings_command_palette_key_input: String,

    // Status de ICMP por host (índice -> resultado)
    icmp_status: HashMap<usize, Option<bool>>,
}

// ─── View Enum ────────────────────────────────────────────────────────────────

/// Telas disponíveis no gerenciador de conexões.
/// O terminal SSH NÃO é uma view aqui — abre em processo separado.
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Home,
    NewHost,
    QuickConnect,
    Settings,
    Customization,
    Documentation(Option<String>),
}

// ─── Mensagens ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    SwitchView(View),

    // Formulário de novo host
    FormNameChanged(String),
    FormAddressChanged(String),
    FormPortChanged(String),
    FormUsernameChanged(String),
    FormPasswordChanged(String),
    FormToggleDomain(bool),
    FormToggleIcmp(bool),
    FormTogglePassword,
    FormSave,
    FormCancel,

    // Formulário de conexão rápida
    QcProtocolSelected(QuickConnectProtocol),
    QcAddressChanged(String),
    QcPortChanged(String),
    QcUsernameChanged(String),
    QcPasswordChanged(String),
    QcToggleDomain(bool),
    QcTogglePassword,
    QcConnect,
    QcCancel,

    // Terminal em janela separada
    /// Spawna `rustty --terminal <host_name>` em novo processo.
    OpenTerminal(String),

    // Deleção e Edição de hosts
    /// Solicita confirmação de deleção para o host no índice `usize`.
    RequestDeleteHost(usize),
    /// Confirma a deleção permanente do host.
    ConfirmDeleteHost,
    /// Cancela a intenção de deleção.
    CancelDeleteHost,
    /// Edita o host no índice `usize`.
    EditHost(usize),

    // Configurações Locais
    SettingsMaxScrollbackChanged(String),
    SettingsScrollLinesChanged(String),
    SettingsCommandPaletteKeyChanged(String),
    SettingsPerformanceModeToggled(bool),
    SettingsGlobalIcmpToggled(bool),
    SettingsCustomizationToggled(bool),

    // Customization
    CustomizationOpen(CustomizationViewMode),
    CustomizationClose,
    KwKeywordChanged(String),
    KwColorChanged(String),
    KwCaseInsensitiveToggled(bool),
    KwSave,
    KwDelete(usize),
    IpSplitToggled(bool),
    IpUnifiedColorChanged(String),
    IpPublicColorChanged(String),
    IpPrivateColorChanged(String),
    IpSave,

    // Link do desenvolvedor
    OpenDeveloperWebsite,

    // Background Tasks
    Tick(()),
    IcmpResult(usize, bool),
}

// ─── Design Tokens ───────────────────────────────────────────────────────────

pub const BACKGROUND_COLOR: Color = Color::from_rgb(0.08, 0.08, 0.08);
pub const SIDEBAR_COLOR: Color    = Color::from_rgb(0.12, 0.12, 0.12);
pub const TEXT_COLOR: Color       = Color::from_rgb(0.9, 0.9, 0.9);
pub const PRIMARY_ORANGE: Color   = Color::from_rgb(1.0, 0.45, 0.0);
pub const ERROR_COLOR: Color      = Color::from_rgb(0.95, 0.3, 0.3);
pub const MUTED_COLOR: Color      = Color::from_rgb(0.5, 0.5, 0.5);
pub const WARNING_BG: Color       = Color::from_rgba(0.95, 0.3, 0.3, 0.12);
pub const SUCCESS_COLOR: Color    = Color::from_rgb(0.2, 0.85, 0.5);

// ─── Application ─────────────────────────────────────────────────────────────

impl Application for RusTTYApp {
    type Executor = executor::Default;
    type Message  = Message;
    type Theme    = Theme;
    type Flags    = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = load_config();
        let client_config = load_client_config();
        let settings_scrollback_input = client_config.max_scrollback_lines.to_string();
        let settings_scroll_lines_input = client_config.scroll_lines.to_string();
        let settings_command_palette_key_input = client_config.command_palette_key.to_string();

        (
            Self {
                config,
                client_config,
                current_view: View::Home,
                new_host_form: NewHostForm::default(),
                quick_connect_form: QuickConnectForm::default(),
                customization_state: CustomizationState::default(),
                delete_confirm: None,
                spawn_error: None,
                editing_host: None,
                settings_scrollback_input,
                settings_scroll_lines_input,
                settings_command_palette_key_input,
                icmp_status: HashMap::new(),
            },
            Command::perform(async {}, |_| Message::Tick(())),
        )
    }

    fn title(&self) -> String {
        match &self.current_view {
            View::NewHost => "RusTTY — Novo Host".to_string(),
            View::Documentation(_) => "RusTTY — Documentação".to_string(),
            _             => "RusTTY".to_string(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(60))
            .map(|_| Message::Tick(()))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SwitchView(view) => {
                if view == View::NewHost {
                    self.new_host_form = NewHostForm::new();
                }
                // Fechar modal de confirmação ao trocar de view
                self.delete_confirm = None;
                self.spawn_error = None;
                self.editing_host = None;
                self.current_view = view;
            }

            // ── Campos do formulário ────────────────────────────────────────
            Message::FormNameChanged(v)    => { self.new_host_form.name = v; self.new_host_form.error = None; }
            Message::FormAddressChanged(v) => { self.new_host_form.address = v; self.new_host_form.error = None; }
            Message::FormPortChanged(v) => {
                if v.chars().all(|c| c.is_ascii_digit()) && v.len() <= 5 {
                    self.new_host_form.port = v;
                }
                self.new_host_form.error = None;
            }
            Message::FormUsernameChanged(v) => { self.new_host_form.username = v; self.new_host_form.error = None; }
            Message::FormPasswordChanged(v) => { self.new_host_form.password = v; self.new_host_form.error = None; }
            Message::FormToggleDomain(v) => {
                self.new_host_form.allow_domain = v;
                self.new_host_form.error = None;
            }
            Message::FormToggleIcmp(v) => {
                self.new_host_form.enable_icmp = v;
                self.new_host_form.error = None;
            }
            Message::FormTogglePassword => {
                self.new_host_form.show_password = !self.new_host_form.show_password;
            }

            // ── Salvar host ─────────────────────────────────────────────────
            Message::FormSave => {
                let form = &mut self.new_host_form;

                if form.name.trim().is_empty() {
                    form.error = Some("O nome/apelido do host é obrigatório.".to_string());
                    return Command::none();
                }
                if form.address.trim().is_empty() {
                    form.error = Some("O endereço é obrigatório.".to_string());
                    return Command::none();
                }
                if let Err(e) = validate_address(&form.address, form.allow_domain) {
                    form.error = Some(e);
                    return Command::none();
                }
                if form.username.trim().is_empty() {
                    form.error = Some("O nome de usuário é obrigatório.".to_string());
                    return Command::none();
                }

                let port: u16 = match form.port.parse() {
                    Ok(p) if p > 0 => p,
                    _ => {
                        form.error = Some("Porta inválida (1–65535).".to_string());
                        return Command::none();
                    }
                };

                let profile = HostProfile {
                    name: form.name.trim().to_string(),
                    address: form.address.trim().to_string(),
                    port,
                    username: form.username.trim().to_string(),
                    auth: if form.password.is_empty() {
                        AuthType::None
                    } else {
                        AuthType::Password(
                            crate::config::ProtectedMemory::new(&form.password)
                                .unwrap_or_else(|_| crate::config::ProtectedMemory::new("").unwrap())
                        )
                    },
                    enable_icmp: form.enable_icmp,
                };

                if let Some(idx) = self.editing_host {
                    self.config.root_nodes[idx] = ConfigNode::Host(profile);
                    self.editing_host = None;
                } else {
                    self.config.root_nodes.push(ConfigNode::Host(profile));
                }

                match save_config(&self.config) {
                    Ok(()) => {
                        self.current_view = View::Home;
                        self.new_host_form = NewHostForm::new();
                        return Command::perform(async {}, |_| Message::Tick(()));
                    }
                    Err(e) => {
                        self.new_host_form.error = Some(format!("Erro ao salvar: {}", e));
                    }
                }
            }

            Message::FormCancel => {
                self.new_host_form = NewHostForm::new();
                self.editing_host = None;
                self.current_view = View::Home;
            }

            // ── Conexão Rápida ──────────────────────────────────────────────
            Message::QcProtocolSelected(p) => {
                self.quick_connect_form.protocol = p;
                self.quick_connect_form.port = match self.quick_connect_form.protocol {
                    QuickConnectProtocol::Ssh => "22".to_string(),
                    QuickConnectProtocol::Telnet => "23".to_string(),
                    QuickConnectProtocol::Serial => String::new(),
                };
            }
            Message::QcAddressChanged(v) => { self.quick_connect_form.address = v; self.quick_connect_form.error = None; }
            Message::QcPortChanged(v) => {
                if v.chars().all(|c| c.is_ascii_digit()) && v.len() <= 5 {
                    self.quick_connect_form.port = v;
                }
                self.quick_connect_form.error = None;
            }
            Message::QcUsernameChanged(v) => { self.quick_connect_form.username = v; self.quick_connect_form.error = None; }
            Message::QcPasswordChanged(v) => { self.quick_connect_form.password = v; self.quick_connect_form.error = None; }
            Message::QcToggleDomain(v) => { self.quick_connect_form.allow_domain = v; self.quick_connect_form.error = None; }
            Message::QcTogglePassword => { self.quick_connect_form.show_password = !self.quick_connect_form.show_password; }
            Message::QcConnect => {
                let form = &mut self.quick_connect_form;
                
                if form.protocol != QuickConnectProtocol::Ssh {
                    form.error = Some("Telnet e Serial ainda estão em desenvolvimento.".to_string());
                    return Command::none();
                }
                
                if form.address.trim().is_empty() {
                    form.error = Some("O endereço é obrigatório.".to_string());
                    return Command::none();
                }
                if let Err(e) = validate_address(&form.address, form.allow_domain) {
                    form.error = Some(e);
                    return Command::none();
                }
                if form.username.trim().is_empty() {
                    form.error = Some("O nome de usuário é obrigatório.".to_string());
                    return Command::none();
                }
                let port: u16 = match form.port.parse() {
                    Ok(p) if p > 0 => p,
                    _ => {
                        form.error = Some("Porta inválida (1–65535).".to_string());
                        return Command::none();
                    }
                };

                let pass = if form.password.is_empty() {
                    "none".to_string()
                } else {
                    form.password.clone()
                };

                self.spawn_error = None;
                match std::env::current_exe() {
                    Ok(exe_path) => {
                        if let Err(e) = std::process::Command::new(&exe_path)
                            .args([
                                "--quick-ssh",
                                form.address.trim(),
                                &port.to_string(),
                                form.username.trim(),
                                &pass
                            ])
                            .spawn()
                        {
                            self.spawn_error = Some(format!("Falha ao abrir terminal: {}", e));
                        }
                    }
                    Err(e) => {
                        self.spawn_error = Some(format!("Não foi possível localizar o executável: {}", e));
                    }
                }
            }
            Message::QcCancel => {
                self.quick_connect_form = QuickConnectForm::default();
                self.current_view = View::Home;
            }

            // ── Terminal em nova janela ─────────────────────────────────────
            Message::OpenTerminal(host_name) => {
                self.spawn_error = None;
                match std::env::current_exe() {
                    Ok(exe_path) => {
                        if let Err(e) = std::process::Command::new(&exe_path)
                            .args(["--terminal", &host_name])
                            .spawn()
                        {
                            self.spawn_error = Some(format!(
                                "Falha ao abrir terminal: {}",
                                e
                            ));
                        }
                    }
                    Err(e) => {
                        self.spawn_error = Some(format!(
                            "Não foi possível localizar o executável: {}",
                            e
                        ));
                    }
                }
            }

            // ── Deleção de hosts ────────────────────────────────────────────
            Message::RequestDeleteHost(idx) => {
                self.delete_confirm = Some(idx);
            }

            Message::CancelDeleteHost => {
                self.delete_confirm = None;
            }

            Message::ConfirmDeleteHost => {
                if let Some(idx) = self.delete_confirm.take() {
                    if idx < self.config.root_nodes.len() {
                        self.config.root_nodes.remove(idx);
                        if let Err(e) = save_config(&self.config) {
                            // Erro não-fatal: log e exibe na UI
                            self.spawn_error = Some(format!("Erro ao salvar após deleção: {}", e));
                        }
                    }
                }
                self.delete_confirm = None;
            }

            Message::EditHost(idx) => {
                if let Some(ConfigNode::Host(host)) = self.config.root_nodes.get(idx) {
                    let mut form = NewHostForm::new();
                    form.name = host.name.clone();
                    form.address = host.address.clone();
                    form.port = host.port.to_string();
                    form.username = host.username.clone();
                    if let AuthType::Password(p) = &host.auth {
                        use secrecy::ExposeSecret;
                        form.password = p.unprotect().map(|s| s.expose_secret().to_string()).unwrap_or_default();
                    }
                    form.allow_domain = host.address.parse::<std::net::IpAddr>().is_err();
                    form.enable_icmp = host.enable_icmp;
                    self.new_host_form = form;
                    self.editing_host = Some(idx);
                    self.current_view = View::NewHost;
                }
            }

            // ── Configurações ───────────────────────────────────────────────
            Message::SettingsMaxScrollbackChanged(val) => {
                let clean_val: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                if clean_val.is_empty() {
                    self.settings_scrollback_input = String::new();
                } else if let Ok(num) = clean_val.parse::<usize>() {
                    let clamped = num.min(100_000);
                    self.settings_scrollback_input = clamped.to_string();
                    self.client_config.max_scrollback_lines = clamped;
                    let _ = save_client_config(&self.client_config);
                }
            }
            Message::SettingsScrollLinesChanged(val) => {
                let clean_val: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                if clean_val.is_empty() {
                    self.settings_scroll_lines_input = String::new();
                } else if let Ok(num) = clean_val.parse::<usize>() {
                    let clamped = num.clamp(1, 16);
                    self.settings_scroll_lines_input = clamped.to_string();
                    self.client_config.scroll_lines = clamped;
                    let _ = save_client_config(&self.client_config);
                }
            }
            Message::SettingsCommandPaletteKeyChanged(val) => {
                let mut clean_val = val.trim().to_lowercase();
                if clean_val.is_empty() {
                    // fall back to default visually or keep empty? keep empty.
                    self.settings_command_palette_key_input = String::new();
                } else {
                    let first_char = clean_val.chars().next().unwrap();
                    self.settings_command_palette_key_input = first_char.to_string();
                    self.client_config.command_palette_key = first_char;
                    let _ = save_client_config(&self.client_config);
                }
            }
            Message::SettingsPerformanceModeToggled(val) => {
                self.client_config.performance_mode = val;
                crate::config::client::PERFORMANCE_MODE.store(val, std::sync::atomic::Ordering::Relaxed);
                let _ = save_client_config(&self.client_config);
            }
            Message::SettingsGlobalIcmpToggled(val) => {
                self.client_config.global_icmp = val;
                let _ = save_client_config(&self.client_config);
                if !val {
                    self.icmp_status.clear();
                } else {
                    return Command::perform(async {}, |_| Message::Tick(()));
                }
            }
            Message::SettingsCustomizationToggled(val) => {
                self.client_config.enable_customization = val;
                let _ = save_client_config(&self.client_config);
            }

            // ── Customization Messages ──────────────────────────────────────
            Message::CustomizationOpen(mode) => {
                self.customization_state.mode = mode.clone();
                if let CustomizationViewMode::EditKeyword(Some(idx)) = mode {
                    if let Some(kw) = self.client_config.customization_data.keywords.get(idx) {
                        self.customization_state.kw_keyword = kw.keyword.clone();
                        self.customization_state.kw_case_insensitive = kw.case_insensitive;
                        self.customization_state.kw_color = kw.color.clone();
                    }
                } else if mode == CustomizationViewMode::EditKeyword(None) {
                    self.customization_state.kw_keyword = String::new();
                    self.customization_state.kw_case_insensitive = false;
                    self.customization_state.kw_color = "#FF7300".to_string();
                }
            }
            Message::CustomizationClose => {
                self.customization_state.mode = CustomizationViewMode::List;
            }
            Message::KwKeywordChanged(val) => {
                self.customization_state.kw_keyword = val;
            }
            Message::KwColorChanged(color) => {
                self.customization_state.kw_color = color;
            }
            Message::KwCaseInsensitiveToggled(val) => {
                self.customization_state.kw_case_insensitive = val;
            }
            Message::KwSave => {
                let kw_str = self.customization_state.kw_keyword.trim().to_string();
                if kw_str.is_empty() {
                    return Command::none(); // could show error
                }
                
                let kw = crate::config::client::KeywordTheme {
                    id: uuid::Uuid::new_v4(),
                    keyword: kw_str,
                    color: self.customization_state.kw_color.trim().to_string(),
                    case_insensitive: self.customization_state.kw_case_insensitive,
                };
                
                if let CustomizationViewMode::EditKeyword(Some(idx)) = self.customization_state.mode {
                    if idx < self.client_config.customization_data.keywords.len() {
                        self.client_config.customization_data.keywords[idx] = kw;
                    }
                } else {
                    self.client_config.customization_data.keywords.push(kw);
                }
                let _ = crate::config::client::save_client_config(&self.client_config);
                self.customization_state.mode = CustomizationViewMode::List;
            }
            Message::KwDelete(idx) => {
                if idx < self.client_config.customization_data.keywords.len() {
                    self.client_config.customization_data.keywords.remove(idx);
                    let _ = crate::config::client::save_client_config(&self.client_config);
                }
            }
            Message::IpSplitToggled(val) => {
                self.customization_state.ip_split = val;
            }
            Message::IpUnifiedColorChanged(c) => self.customization_state.ip_unified_color = c,
            Message::IpPublicColorChanged(c) => self.customization_state.ip_public_color = c,
            Message::IpPrivateColorChanged(c) => self.customization_state.ip_private_color = c,
            Message::IpSave => {
                let data = if self.customization_state.ip_split {
                    crate::config::client::IpCustomization::Split {
                        public: self.customization_state.ip_public_color.trim().to_string(),
                        private: self.customization_state.ip_private_color.trim().to_string(),
                    }
                } else {
                    crate::config::client::IpCustomization::Unified(self.customization_state.ip_unified_color.trim().to_string())
                };
                if self.customization_state.mode == CustomizationViewMode::EditIpv4 {
                    self.client_config.customization_data.ipv4 = Some(data);
                } else if self.customization_state.mode == CustomizationViewMode::EditIpv6 {
                    self.client_config.customization_data.ipv6 = Some(data);
                }
                let _ = crate::config::client::save_client_config(&self.client_config);
                self.customization_state.mode = CustomizationViewMode::List;
            }

            Message::OpenDeveloperWebsite => {
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "https://byvitor.com.br/"])
                    .spawn();
                
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open")
                    .arg("https://byvitor.com.br/")
                    .spawn();

                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open")
                    .arg("https://byvitor.com.br/")
                    .spawn();
            }

            Message::Tick(_) => {
                if !self.client_config.global_icmp {
                    self.icmp_status.clear();
                    return Command::none();
                }

                let mut commands = Vec::new();
                for (idx, node) in self.config.root_nodes.iter().enumerate() {
                    if let ConfigNode::Host(h) = node {
                        if h.enable_icmp {
                            let ip = h.address.clone();
                            commands.push(Command::perform(
                                async move { crate::net::icmp::check_icmp(&ip).await },
                                move |res| Message::IcmpResult(idx, res)
                            ));
                        }
                    }
                }
                return Command::batch(commands);
            }
            Message::IcmpResult(idx, res) => {
                self.icmp_status.insert(idx, Some(res));
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = match &self.current_view {
            View::Documentation(page) => self.view_docs_sidebar(page.clone()),
            _ => self.view_sidebar(),
        };

        let content: Element<Message> = match &self.current_view {
            View::Home => self.view_home(),
            View::NewHost => self.view_new_host(),
            View::QuickConnect => self.view_quick_connect(),
            View::Settings => self.view_settings(),
            View::Customization => crate::ui::customization::view(
                self.client_config.enable_customization,
                &self.customization_state,
                &self.client_config.customization_data,
            ),
            View::Documentation(page) => self.view_documentation(page),
        };

        let main_content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(theme::Container::Custom(Box::new(MainContentStyle)));
            
        let base_ui: Element<Message> = row![sidebar, main_content].into();

        if let Some(idx) = self.delete_confirm {
            let host_display_name = self.config.root_nodes
                .get(idx)
                .and_then(|n| match n {
                    ConfigNode::Host(h) => Some(h.name.as_str()),
                    _ => None,
                })
                .unwrap_or("este host");

            let modal_box = container(
                column![
                    row![
                        icon_sized::<Message>(LucideIcon::AlertTriangle, 24),
                        text(format!("  Deletar \"{}\"?", host_display_name))
                            .size(18)
                            .style(theme::Text::Color(ERROR_COLOR)),
                        Space::with_width(Length::Fill),
                        button(icon::<Message>(LucideIcon::X))
                            .on_press(Message::CancelDeleteHost)
                            .style(theme::Button::Text)
                    ]
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(16.0)),
                    text("Esta ação é permanente e irreversível.")
                        .size(14)
                        .style(theme::Text::Color(MUTED_COLOR)),
                    Space::with_height(Length::Fixed(24.0)),
                    row![
                        Space::with_width(Length::Fill),
                        button(
                            row![
                                text("Cancelar").size(14),
                            ]
                            .align_items(Alignment::Center)
                        )
                        .on_press(Message::CancelDeleteHost)
                        .style(theme::Button::Text)
                        .padding([8, 16]),
                        Space::with_width(Length::Fixed(12.0)),
                        button(
                            row![
                                icon::<Message>(LucideIcon::Trash2),
                                text("  Deletar permanentemente").size(14),
                            ]
                            .align_items(Alignment::Center)
                        )
                        .on_press(Message::ConfirmDeleteHost)
                        .style(theme::Button::Destructive)
                        .padding([8, 16]),
                    ]
                    .align_items(Alignment::Center),
                ]
                .padding(24)
            )
            .width(Length::Fixed(480.0))
            .style(theme::Container::Custom(Box::new(DeleteConfirmStyle)));

            let modal_content = container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y();

            let modal_element: Element<Message> = modal_content.into();

            Modal::new(base_ui, Some(modal_element))
                .backdrop(Message::CancelDeleteHost)
                .on_esc(Message::CancelDeleteHost)
                .into()
        } else {
            base_ui
        }
    }
}

// ─── Views ────────────────────────────────────────────────────────────────────

impl RusTTYApp {
    /// Sidebar de navegação com ícones Lucide.
    fn view_sidebar(&self) -> Element<'_, Message> {
        let nav = column![
            text("RusTTY")
                .size(28)
                .style(theme::Text::Color(PRIMARY_ORANGE)),

            button(
                row![
                    icon::<Message>(LucideIcon::Home),
                    text("  Início").size(15),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::SwitchView(View::Home))
            .style(theme::Button::Text)
            .width(Length::Fill),

            button(
                row![
                    icon::<Message>(LucideIcon::Settings),
                    text("  Configurações").size(15),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::SwitchView(View::Settings))
            .style(theme::Button::Text)
            .width(Length::Fill),

            button(
                row![
                    icon::<Message>(LucideIcon::Paintbrush),
                    text("  Personalização").size(15),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::SwitchView(View::Customization))
            .style(theme::Button::Text)
            .width(Length::Fill),

            button(
                row![
                    icon::<Message>(LucideIcon::BookOpenCheck),
                    text("  Documentação").size(15),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::SwitchView(View::Documentation(None)))
            .style(theme::Button::Text)
            .width(Length::Fill),
        ]
        .spacing(12);

        let footer = button(
            text("Developed by Vitor")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(theme::Text::Color(MUTED_COLOR))
        )
        .on_press(Message::OpenDeveloperWebsite)
        .style(theme::Button::Custom(Box::new(InvisibleButtonStyle)))
        .width(Length::Fill);

        let sidebar_content = column![
            nav,
            Space::with_height(Length::Fill),
            footer,
        ]
        .padding([20, 20, 0, 20]);

        container(sidebar_content)
            .width(Length::Fixed(220.0))
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(SidebarStyle)))
            .into()
    }

    /// View principal: lista de conexões salvas + modal de confirmação de deleção.
    fn view_home(&self) -> Element<'_, Message> {
        let header = row![
            icon_sized::<Message>(LucideIcon::Server, 22),
            text("  Suas Conexões")
                .size(26)
                .style(theme::Text::Color(PRIMARY_ORANGE)),
        ]
        .align_items(Alignment::Center);

        let add_btn = button(
            row![
                icon::<Message>(LucideIcon::ServerPlus),
                text("  Novo Host").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::SwitchView(View::NewHost))
        .padding([8, 14]);

        let qc_btn = button(
            row![
                icon::<Message>(LucideIcon::Plug),
                text("  Conexão rápida").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::SwitchView(View::QuickConnect))
        .style(theme::Button::Custom(Box::new(OrangeButtonStyle)))
        .padding([8, 14]);

        let header_actions = row![qc_btn, add_btn].spacing(10);
        let header_row = row![
            header,
            Space::with_width(Length::Fill),
            header_actions
        ].align_items(Alignment::Center);

        let mut host_list = column![header_row].spacing(16);

        // Exibe erro de spawn, se houver
        if let Some(err) = &self.spawn_error {
            host_list = host_list.push(
                container(
                    row![
                        icon::<Message>(LucideIcon::AlertTriangle),
                        text(format!("  {}", err))
                            .size(13)
                            .style(theme::Text::Color(ERROR_COLOR)),
                    ]
                    .align_items(Alignment::Center)
                )
                .padding([8, 12])
                .style(theme::Container::Custom(Box::new(ErrorBoxStyle)))
            );
        }

        // A renderização do modal agora é feita em `view()` para cobrir a tela toda

        if self.config.root_nodes.is_empty() {
            host_list = host_list.push(
                text("Nenhum host cadastrado. Clique em \"Novo Host\" para adicionar.")
                    .size(14)
                    .style(theme::Text::Color(MUTED_COLOR)),
            );
        }

        for (idx, node) in self.config.root_nodes.iter().enumerate() {
            match node {
                ConfigNode::Host(host) => {
                    let auth_icon = match &host.auth {
                        AuthType::Key { .. } => icon::<Message>(LucideIcon::Key),
                        AuthType::Password(_) => icon::<Message>(LucideIcon::Lock),
                        AuthType::None => icon::<Message>(LucideIcon::Shield),
                    };

                    let terminal_icon = if host.enable_icmp && self.client_config.global_icmp {
                        match self.icmp_status.get(&idx) {
                            Some(Some(true)) => icon_colored::<Message>(LucideIcon::Terminal, SUCCESS_COLOR),
                            Some(Some(false)) => icon_colored::<Message>(LucideIcon::Terminal, ERROR_COLOR),
                            _ => icon::<Message>(LucideIcon::Terminal),
                        }
                    } else {
                        icon::<Message>(LucideIcon::Terminal)
                    };

                    // Botão principal do host — abre terminal em nova janela
                    let host_btn = button(
                        row![
                            terminal_icon,
                            text(format!("  {}  ", host.name)).size(15),
                            auth_icon,
                            text(format!("  {}@{}:{}", host.username, host.address, host.port))
                                .size(12)
                                .style(theme::Text::Color(MUTED_COLOR)),
                        ]
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::OpenTerminal(host.name.clone()))
                    .width(Length::Fill)
                    .padding(12)
                    .style(theme::Button::Text);

                    let host_with_menu = iced_aw::ContextMenu::new(
                        host_btn,
                        move || {
                            container(
                                column![
                                    button(
                                        row![
                                            icon::<Message>(LucideIcon::Edit),
                                            text("  Editar host").size(14),
                                            Space::with_width(Length::Fill),
                                        ]
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::EditHost(idx))
                                    .style(theme::Button::Text)
                                    .padding([8, 12])
                                    .width(Length::Fill),
                                    button(
                                        row![
                                            icon::<Message>(LucideIcon::Trash2),
                                            text("  Remover host").size(14),
                                            Space::with_width(Length::Fill),
                                        ]
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::RequestDeleteHost(idx))
                                    .style(theme::Button::Destructive)
                                    .padding([8, 12])
                                    .width(Length::Fill),
                                ]
                                .spacing(2)
                            )
                            .width(Length::Fixed(160.0))
                            .style(theme::Container::Custom(Box::new(ContextMenuStyle)))
                            .into()
                        }
                    );

                    let host_row = row![
                        host_with_menu,
                    ]
                    .align_items(Alignment::Center)
                    .spacing(4);

                    host_list = host_list.push(
                        container(host_row)
                            .style(theme::Container::Custom(Box::new(HostItemStyle)))
                    );
                }
                ConfigNode::Folder { name, .. } => {
                    let folder_item = row![
                        icon::<Message>(LucideIcon::Folder),
                        text(format!("  {}", name))
                            .size(15)
                            .style(theme::Text::Color(TEXT_COLOR)),
                    ]
                    .align_items(Alignment::Center);
                    host_list = host_list.push(folder_item);
                }
            }
        }

        scrollable(host_list.padding(4)).into()
    }

    /// View de formulário para criar novo host SSH.
    fn view_new_host(&self) -> Element<'_, Message> {
        let form = &self.new_host_form;

        // ── Cabeçalho ─────────────────────────────────────────────────────────
        let header = row![
            button(
                row![
                    icon::<Message>(LucideIcon::Undo2),
                    text("  Voltar").size(14),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::FormCancel)
            .style(theme::Button::Text),

            Space::with_width(Length::Fixed(16.0)),

            icon_sized::<Message>(LucideIcon::ServerPlus, 22),
            text(if self.editing_host.is_some() {
                "  Editar Host SSH"
            } else {
                "  Novo Host"
            })
                .size(24)
                .style(theme::Text::Color(PRIMARY_ORANGE)),
        ]
        .align_items(Alignment::Center)
        .spacing(4);

        // ── Campo: Apelido ────────────────────────────────────────────────────
        let name_input = column![
            row![
                icon::<Message>(LucideIcon::Monitor),
                text("  Apelido / Nome").size(13).style(theme::Text::Color(MUTED_COLOR)),
            ]
            .align_items(Alignment::Center),
            text_input("Ex: Servidor Prod", &form.name)
                .on_input(Message::FormNameChanged)
                .padding(10)
                .size(15)
                .id(text_input::Id::new("host_name")),
        ]
        .spacing(6);

        // ── Campo: Endereço + checkbox domínio ───────────────────────────────
        let addr_label = row![
            icon::<Message>(LucideIcon::Network),
            text("  Endereço").size(13).style(theme::Text::Color(MUTED_COLOR)),
        ]
        .align_items(Alignment::Center);

        let addr_hint = if form.allow_domain {
            "Ex: meu.servidor.com ou 192.168.1.1"
        } else {
            "Ex: 192.168.1.1 (somente IP numérico)"
        };

        let domain_checkbox = checkbox(
            "Permitir domínio",
            form.allow_domain,
        )
        .on_toggle(Message::FormToggleDomain)
        .size(16)
        .text_size(13);

        let icmp_checkbox = checkbox(
            "Habilitar teste ICMP",
            form.enable_icmp,
        )
        .on_toggle(Message::FormToggleIcmp)
        .size(16)
        .text_size(13);

        let address_input = column![
            addr_label,
            text_input(addr_hint, &form.address)
                .on_input(Message::FormAddressChanged)
                .padding(10)
                .size(15)
                .id(text_input::Id::new("host_address")),
            row![domain_checkbox, icmp_checkbox].spacing(16),
        ]
        .spacing(6);

        // ── Campo: Porta ──────────────────────────────────────────────────────
        let port_input = column![
            row![
                icon::<Message>(LucideIcon::Plug),
                text("  Porta SSH").size(13).style(theme::Text::Color(MUTED_COLOR)),
            ]
            .align_items(Alignment::Center),
            text_input("22", &form.port)
                .on_input(Message::FormPortChanged)
                .padding(10)
                .size(15)
                .width(Length::Fixed(100.0))
                .id(text_input::Id::new("host_port")),
        ]
        .spacing(6);

        // ── Campo: Usuário ────────────────────────────────────────────────────
        let user_input = column![
            row![
                icon::<Message>(LucideIcon::User),
                text("  Usuário SSH").size(13).style(theme::Text::Color(MUTED_COLOR)),
            ]
            .align_items(Alignment::Center),
            text_input("Ex: root, admin", &form.username)
                .on_input(Message::FormUsernameChanged)
                .padding(10)
                .size(15)
                .id(text_input::Id::new("host_user")),
        ]
        .spacing(6);

        // ── Campo: Senha + toggle visibilidade ────────────────────────────────
        let eye_icon = if form.show_password {
            icon::<Message>(LucideIcon::EyeOff)
        } else {
            icon::<Message>(LucideIcon::Eye)
        };

        let pass_field = text_input("Senha SSH (opcional)", &form.password)
            .on_input(Message::FormPasswordChanged)
            .secure(!form.show_password)
            .padding(10)
            .size(15)
            .id(text_input::Id::new("host_pass"));

        let password_input_col = column![
            row![
                icon::<Message>(LucideIcon::Lock),
                text("  Senha").size(13).style(theme::Text::Color(MUTED_COLOR)),
            ]
            .align_items(Alignment::Center),
            row![
                pass_field,
                Space::with_width(Length::Fixed(8.0)),
                button(eye_icon)
                    .on_press(Message::FormTogglePassword)
                    .style(theme::Button::Text)
                    .padding(8),
            ]
            .align_items(Alignment::Center),
            text("Deixe em branco para usar autenticação por chave.")
                .size(11)
                .style(theme::Text::Color(MUTED_COLOR)),
        ]
        .spacing(6);
        
        let password_input: Element<Message> = if self.editing_host.is_none() {
            password_input_col.into()
        } else {
            column![
                row![
                    icon::<Message>(LucideIcon::Lock),
                    text("  Senha oculta por segurança").size(13).style(theme::Text::Color(MUTED_COLOR)),
                ]
                .align_items(Alignment::Center),
                text("Para alterar a senha, remova este host e crie um novo.")
                    .size(11)
                    .style(theme::Text::Color(MUTED_COLOR)),
            ]
            .spacing(6)
            .into()
        };

        // ── Aviso de segurança ────────────────────────────────────────────────
        let security_note = row![
            icon_sized::<Message>(LucideIcon::Shield, 13),
            text("  Dados armazenados criptografados (AES-256-GCM) em %AppData%\\ByVitor\\RusTTY")
                .size(11)
                .style(theme::Text::Color(MUTED_COLOR)),
        ]
        .align_items(Alignment::Center);

        // ── Mensagem de erro ──────────────────────────────────────────────────
        let error_widget: Element<Message> = if let Some(err) = &form.error {
            container(
                row![
                    icon::<Message>(LucideIcon::X),
                    text(format!("  {}", err))
                        .size(13)
                        .style(theme::Text::Color(ERROR_COLOR)),
                ]
                .align_items(Alignment::Center)
            )
            .padding([8, 12])
            .style(theme::Container::Custom(Box::new(ErrorBoxStyle)))
            .into()
        } else {
            iced::widget::Container::new(iced::widget::Space::with_width(Length::Fixed(0.0))).into()
        };

        // ── Botões de ação ────────────────────────────────────────────────────
        let save_btn = button(
            row![
                icon::<Message>(LucideIcon::Save),
                text("  Salvar Host").size(15),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::FormSave)
        .padding([10, 20])
        .style(theme::Button::Primary);

        let cancel_btn = button(
            row![
                icon::<Message>(LucideIcon::X),
                text("  Cancelar").size(15),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::FormCancel)
        .padding([10, 20])
        .style(theme::Button::Text);

        let action_row = row![save_btn, Space::with_width(Length::Fixed(12.0)), cancel_btn]
            .align_items(Alignment::Center);

        // ── Montagem final ────────────────────────────────────────────────────
        scrollable(
            column![
                header,
                Space::with_height(Length::Fixed(24.0)),
                name_input,
                Space::with_height(Length::Fixed(16.0)),
                address_input,
                Space::with_height(Length::Fixed(16.0)),
                port_input,
                Space::with_height(Length::Fixed(16.0)),
                user_input,
                Space::with_height(Length::Fixed(16.0)),
                password_input,
                Space::with_height(Length::Fixed(24.0)),
                error_widget,
                Space::with_height(Length::Fixed(8.0)),
                action_row,
                Space::with_height(Length::Fixed(16.0)),
                security_note,
            ]
            .spacing(0)
            .max_width(600),
        )
        .into()
    }

    /// View de formulário para Conexão Rápida.
    fn view_quick_connect(&self) -> Element<'_, Message> {
        let form = &self.quick_connect_form;

        // ── Cabeçalho ─────────────────────────────────────────────────────────
        let header = row![
            button(
                row![
                    icon::<Message>(LucideIcon::Undo2),
                    text("  Voltar").size(14),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::QcCancel)
            .style(theme::Button::Text),

            Space::with_width(Length::Fixed(16.0)),
            icon_sized::<Message>(LucideIcon::Plug, 24),
            text("  Conexão Rápida")
                .size(24)
                .style(theme::Text::Color(PRIMARY_ORANGE)),
        ]
        .align_items(Alignment::Center);

        // ── Protocolo (Radio Buttons) ─────────────────────────────────────────
        use iced::widget::radio;
        let protocol_row = row![
            radio(
                "SSH",
                QuickConnectProtocol::Ssh,
                Some(form.protocol.clone()),
                Message::QcProtocolSelected
            ).size(18),
            radio(
                "Telnet (Em desenv.)",
                QuickConnectProtocol::Telnet,
                Some(form.protocol.clone()),
                Message::QcProtocolSelected
            ).size(18),
            radio(
                "Serial (Em desenv.)",
                QuickConnectProtocol::Serial,
                Some(form.protocol.clone()),
                Message::QcProtocolSelected
            ).size(18),
        ].spacing(20);

        // ── Campos de entrada ────────────────────────────────────────────────
        let mut form_col = column![
            text("Protocolo").size(14).style(theme::Text::Color(MUTED_COLOR)),
            protocol_row,
            Space::with_height(Length::Fixed(8.0)),
        ].spacing(8);

        let addr_input = text_input("Ex: 192.168.1.100 ou meuservidor.com", &form.address)
            .on_input(Message::QcAddressChanged)
            .padding(10);
        let port_input = text_input("Ex: 22", &form.port)
            .on_input(Message::QcPortChanged)
            .padding(10)
            .width(Length::Fixed(100.0));
        let user_input = text_input("Ex: root", &form.username)
            .on_input(Message::QcUsernameChanged)
            .padding(10);
        let pass_input = if form.show_password {
            text_input("Senha (opcional)", &form.password)
                .on_input(Message::QcPasswordChanged)
                .padding(10)
        } else {
            text_input("Senha (opcional)", &form.password)
                .on_input(Message::QcPasswordChanged)
                .secure(true)
                .padding(10)
        };

        form_col = form_col.push(
            row![
                column![
                    text("Endereço / Host").size(14).style(theme::Text::Color(MUTED_COLOR)),
                    addr_input,
                ].spacing(6).width(Length::FillPortion(3)),
                column![
                    text("Porta").size(14).style(theme::Text::Color(MUTED_COLOR)),
                    port_input,
                ].spacing(6).width(Length::FillPortion(1)),
            ].spacing(16)
        );

        let is_ssh = form.protocol == QuickConnectProtocol::Ssh;
        let is_telnet = form.protocol == QuickConnectProtocol::Telnet;
        
        if is_ssh || is_telnet {
            form_col = form_col.push(
                row![
                    column![
                        text("Usuário").size(14).style(theme::Text::Color(MUTED_COLOR)),
                        user_input,
                    ].spacing(6).width(Length::Fill),
                    column![
                        text("Senha").size(14).style(theme::Text::Color(MUTED_COLOR)),
                        pass_input,
                    ].spacing(6).width(Length::Fill),
                ].spacing(16)
            );
        }

        if is_ssh {
            form_col = form_col.push(
                checkbox("Permitir resolução de domínio (DNS)", form.allow_domain)
                    .on_toggle(Message::QcToggleDomain)
                    .size(18)
                    .text_size(14)
            );
            form_col = form_col.push(
                checkbox("Exibir senha", form.show_password)
                    .on_toggle(|_| Message::QcTogglePassword)
                    .size(18)
                    .text_size(14)
            );
        }

        let main_card = container(form_col.padding(24));

        // ── Rodapé (Erro + Botão) ─────────────────────────────────────────────
        let mut footer = row![]
            .align_items(Alignment::Center)
            .width(Length::Fill);

        if let Some(err) = &form.error {
            footer = footer.push(
                row![
                    icon::<Message>(LucideIcon::AlertTriangle),
                    text(format!("  {}", err))
                        .size(14)
                        .style(theme::Text::Color(ERROR_COLOR)),
                ]
                .align_items(Alignment::Center)
                .width(Length::Fill)
            );
        } else {
            footer = footer.push(Space::with_width(Length::Fill));
        }

        let connect_btn = button(
            row![
                icon::<Message>(LucideIcon::GlobeLock),
                text("  Conectar").size(16),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::QcConnect)
        .padding([10, 24])
        .style(theme::Button::Custom(Box::new(OrangeButtonStyle)));

        footer = footer.push(connect_btn);

        // ── Layout Final ──────────────────────────────────────────────────────
        let content = column![
            header,
            main_card,
            footer,
        ]
        .spacing(20)
        .max_width(600.0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(40)
            .center_x()
            .into()
    }

    /// View de configurações (agora em src/ui/settings.rs).
    fn view_settings(&self) -> Element<'_, Message> {
        crate::ui::settings::view(
            &self.settings_scrollback_input,
            &self.settings_scroll_lines_input,
            &self.settings_command_palette_key_input,
            self.client_config.performance_mode,
            self.client_config.global_icmp,
            self.client_config.enable_customization,
        )
    }

    fn view_docs_sidebar(&self, current_page: Option<String>) -> Element<'_, Message> {
        let mut nav = column![
            text("Documentação")
                .size(24)
                .style(theme::Text::Color(PRIMARY_ORANGE)),
            Space::with_height(Length::Fixed(16.0)),
        ].spacing(8);

        for page in crate::ui::documentation::PAGES {
            let is_active = current_page.as_deref() == Some(page.id) || (current_page.is_none() && page.id == crate::ui::documentation::PAGES[0].id);
            let btn = button(
                text(page.title).size(15)
                    .style(if is_active { theme::Text::Color(PRIMARY_ORANGE) } else { theme::Text::Color(TEXT_COLOR) })
            )
            .on_press(Message::SwitchView(View::Documentation(Some(page.id.to_string()))))
            .style(theme::Button::Text)
            .width(Length::Fill);
            nav = nav.push(btn);
        }

        nav = nav.push(Space::with_height(Length::Fixed(24.0)));
        nav = nav.push(
            button(
                row![
                    icon::<Message>(LucideIcon::Undo2),
                    text("  Voltar ao App").size(15),
                ]
                .align_items(Alignment::Center)
            )
            .on_press(Message::SwitchView(View::Home))
            .style(theme::Button::Text)
            .width(Length::Fill)
        );

        let sidebar_content = column![
            nav,
            Space::with_height(Length::Fill),
        ]
        .padding([20, 20, 0, 20]);

        container(sidebar_content)
            .width(Length::Fixed(220.0))
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(SidebarStyle)))
            .into()
    }

    fn view_documentation(&self, current_page: &Option<String>) -> Element<'_, Message> {
        let page = if let Some(id) = current_page {
            crate::ui::documentation::PAGES.iter().find(|p| p.id == id).unwrap_or(&crate::ui::documentation::PAGES[0])
        } else {
            &crate::ui::documentation::PAGES[0]
        };

        crate::ui::documentation::render_markdown(page.content)
    }
}

// ─── Validação ────────────────────────────────────────────────────────────────

/// Valida o endereço do host.
///
/// # Contratos
/// - Se `allow_domain = false`: aceita apenas IPv4 ou IPv6
/// - Se `allow_domain = true`: aceita também domínios válidos (RFC 1123)
///
/// Retorna `Ok(())` se válido, `Err(mensagem_legível)` caso contrário.
fn validate_address(address: &str, allow_domain: bool) -> Result<(), String> {
    let addr = address.trim();

    if addr.is_empty() {
        return Err("Endereço não pode ser vazio.".to_string());
    }

    if addr.parse::<std::net::IpAddr>().is_ok() {
        return Ok(());
    }

    if !allow_domain {
        return Err(
            "Somente IPs numéricos são aceitos. Marque \"Permitir domínio\" para usar nomes DNS."
                .to_string(),
        );
    }

    if addr.len() > 253 {
        return Err("Domínio excede o limite de 253 caracteres.".to_string());
    }

    let labels: Vec<&str> = addr.split('.').collect();
    if labels.len() < 2 {
        return Err("Domínio inválido — deve ter pelo menos um ponto (ex: host.example.com).".to_string());
    }

    for label in &labels {
        if label.is_empty() || label.len() > 63 {
            return Err(format!("Label de domínio inválida: \"{}\"", label));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(format!("Label \"{}\" não pode começar ou terminar com hífen.", label));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(format!("Label \"{}\" contém caracteres inválidos.", label));
        }
    }

    Ok(())
}

// ─── Estilos ──────────────────────────────────────────────────────────────────

struct SidebarStyle;
impl container::StyleSheet for SidebarStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(SIDEBAR_COLOR.into()),
            text_color: Some(TEXT_COLOR),
            ..Default::default()
        }
    }
}

struct MainContentStyle;
impl container::StyleSheet for MainContentStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(BACKGROUND_COLOR.into()),
            text_color: Some(TEXT_COLOR),
            ..Default::default()
        }
    }
}

struct InvisibleButtonStyle;
impl button::StyleSheet for InvisibleButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            text_color: MUTED_COLOR,
            ..Default::default()
        }
    }
    
    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }
    
    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }
    
    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }
}

struct ErrorBoxStyle;
impl container::StyleSheet for ErrorBoxStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgba(0.95, 0.3, 0.3, 0.1).into()),
            border: iced::Border {
                color: ERROR_COLOR,
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Some(ERROR_COLOR),
            ..Default::default()
        }
    }
}

/// Estilo do modal de confirmação de deleção.
struct DeleteConfirmStyle;
impl container::StyleSheet for DeleteConfirmStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.10, 0.10, 0.10))),
            border: iced::Border {
                color: Color::from_rgba(0.95, 0.3, 0.3, 0.4),
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.8),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 20.0,
            },
            text_color: Some(TEXT_COLOR),
        }
    }
}

/// Estilo de card para cada item de host na lista.
struct OrangeButtonStyle;
impl button::StyleSheet for OrangeButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(PRIMARY_ORANGE.into()),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut app = self.active(style);
        app.background = Some(Color::from_rgb(1.0, 0.55, 0.1).into());
        app
    }
}

struct HostItemStyle;
impl container::StyleSheet for HostItemStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ContextMenuStyle;
impl container::StyleSheet for ContextMenuStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(TEXT_COLOR),
            ..Default::default()
        }
    }
}

// ─── Helpers de Cor ───────────────────────────────────────────────────────────

pub fn color_to_hex(color: iced::Color) -> String {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn hex_to_color(hex: &str) -> Result<iced::Color, ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
        Ok(iced::Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
    } else {
        Err(())
    }
}
