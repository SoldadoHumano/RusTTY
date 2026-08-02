//! Aplicação Iced para a janela de terminal SSH.
//!
//! # Modo de operação
//! Ativada com `rustty --terminal <host_name>`.
//!
//! # Arquitetura
//! - `Canvas` renderiza a grade de células do terminal (não widgets de texto)
//! - Teclado capturado via `iced::event::listen_with()` → bytes SSH diretos
//! - Seleção de texto via mouse no Canvas (clique + arraste)
//! - Ctrl+Shift+C → copia seleção para clipboard
//! - Ctrl+Shift+V → cola do clipboard no terminal
//! - Redimensionamento da janela → notifica PTY via `NetworkCommand::ResizePty`
//!
//! # Segurança
//! - Plaintext zerizado via `Zeroizing<Vec<u8>>` no config/crypto
//! - Senha SSH zerizada via `zeroize::Zeroize` após autenticação

use iced::{
    executor, theme,
    widget::{button, column, container, row, text, Space, text_input},
    widget::canvas::{self, Canvas, Frame, Geometry},
    Application, Alignment, Color, Command, Element, Font,
    Length, Pixels, Point, Rectangle, Size, Subscription, Theme,
};
use iced::keyboard::{self, key::Named, Modifiers};
use iced::mouse;
use iced::alignment::{Horizontal, Vertical};

use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::config::{
    load_config, AuthType, ConfigNode,
    client::load_client_config,
};
use crate::net::{NetworkCommand, NetworkEvent, SshAuth};
use crate::net::ssh::start_ssh_session;
use crate::terminal::{TerminalState, CellColor, CELL_W, CELL_H, FONT_SIZE};
use crate::ui::icons::{icon, icon_sized, LucideIcon};

// ─── Constantes ───────────────────────────────────────────────────────────────

/// Fonte monospace do terminal (Consolas já presente no Windows).
const MONOSPACE: Font = Font::with_name("Consolas");
/// Tamanho de janela padrão para cálculo inicial do PTY.
const DEFAULT_WIN_W: f32 = 900.0;
const DEFAULT_WIN_H: f32 = 400.0;

// Design tokens do header
const HEADER_BG: Color = Color::from_rgb(0.10, 0.10, 0.10);
const PRIMARY:   Color = Color::from_rgb(1.0, 0.45, 0.0);
const SUCCESS:   Color = Color::from_rgb(0.2, 0.85, 0.5);
const ERROR_C:   Color = Color::from_rgb(0.95, 0.3, 0.3);
const MUTED:     Color = Color::from_rgb(0.5, 0.5, 0.5);
const TEXT_C:    Color = Color::from_rgb(0.9, 0.9, 0.9);

// ─── Estado da Sessão ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalStatus {
    Connecting,
    Connected,
    Disconnected(String),
    Error(String),
    HostNotFound,
}

/// Canal de eventos SSH compartilhado com a Subscription.
type SharedEventReceiver = Arc<Mutex<Option<mpsc::Receiver<NetworkEvent>>>>;

// ─── App ──────────────────────────────────────────────────────────────────────

pub struct TerminalApp {
    host_name:      String,
    status:         TerminalStatus,
    terminal:       TerminalState,
    pub sel_anchor: Option<(usize, usize)>, // (row, col) absolutos na viewport atual
    pub sel_cursor: Option<(usize, usize)>,
    
    // Comando para enviar inputs para o backend PTY
    cmd_sender: Option<mpsc::Sender<NetworkCommand>>,
    event_receiver: Option<SharedEventReceiver>,
    cursor_visible: bool,

    // Controle de rolagem do scrollback
    pub scroll_offset: usize,
    pub scroll_lines: usize,

    // Command Palette
    pub command_palette_key: char,
    pub palette_open: bool,
    pub palette_lines_input: String,

    pub client_config: crate::config::client::ClientConfig,
}

// ─── Mensagens ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TerminalMessage {
    SshEvent(NetworkEvent),
    IcedEvent(iced::Event),
    /// Canvas inicia seleção
    SelectionStart(usize, usize),
    /// Canvas estende seleção
    SelectionExtend(usize, usize),
    /// Limpa a seleção atual
    ClearSelection,
    /// Botão direito pressionado no Canvas
    RightClick,
    /// Mover cursor via clique na linha de comando
    MoveCursorLeftRight(isize),
    /// Conteúdo do clipboard (para paste)
    ClipboardContent(Option<String>),
    Disconnect,
    WindowCloseRequested(iced::window::Id),
    ForceClose(iced::window::Id),
    CursorBlink(std::time::Instant),
    ScrollWheel(f32),
    Noop,
    
    // Command Palette actions
    TogglePalette,
    PaletteLinesInputChanged(String),
    PaletteActionCopyAll,
    PaletteActionCopyLast,
    PaletteActionClear,
}

#[derive(Debug, Clone)]
pub enum TerminalInit {
    SavedHost(String),
    QuickSsh {
        address: String,
        port: u16,
        user: String,
        pass: String,
    },
}

impl Default for TerminalInit {
    fn default() -> Self {
        TerminalInit::SavedHost(String::new())
    }
}

// ─── Application ─────────────────────────────────────────────────────────────

impl Application for TerminalApp {
    type Executor = executor::Default;
    type Message  = TerminalMessage;
    type Theme    = Theme;
    type Flags    = TerminalInit;

    fn new(flags: TerminalInit) -> (Self, Command<TerminalMessage>) {
        // ── 1. Carrega config (plaintext zerizado pelo Zeroizing no crypto.rs) ─
        let config = load_config();
        let client_config = load_client_config();
        let max_scrollback = client_config.max_scrollback_lines;
        let scroll_lines = client_config.scroll_lines;
        let command_palette_key = client_config.command_palette_key;

        // ── 2. Localiza o perfil do host ou usa Quick Connect ────────────────
        let (host_name_display, host) = match flags {
            TerminalInit::SavedHost(name) => {
                let h = config.root_nodes.iter().find_map(|n| match n {
                    ConfigNode::Host(h) if h.name == name => Some(h.clone()),
                    _ => None,
                });
                (name, h)
            },
            TerminalInit::QuickSsh { address, port, user, pass } => {
                let auth = if pass == "none" {
                    AuthType::None
                } else {
                    AuthType::Password(
                        crate::config::ProtectedMemory::new(&pass).unwrap_or_else(|_| crate::config::ProtectedMemory::new("").unwrap())
                    )
                };
                
                let h = crate::config::HostProfile {
                    name: "Conexão Rápida".to_string(),
                    address: address.clone(),
                    port,
                    username: user,
                    auth,
                    enable_icmp: false,
                };
                (address, Some(h))
            }
        };

        let host = match host {
            Some(h) => h,
            None => {
                let mut term = TerminalState::new(24, 80, max_scrollback);
                term.process_bytes(
                    format!("ERRO: Host \"{}\" não encontrado na configuração.\r\n", host_name_display)
                        .as_bytes(),
                );
                return (
                    TerminalApp {
                        host_name: host_name_display,
                        status: TerminalStatus::HostNotFound,
                        terminal: term,
                        sel_anchor: None,
                        sel_cursor: None,
                        cmd_sender: None,
                        event_receiver: None,
                        cursor_visible: true,
                        scroll_offset: 0,
                        scroll_lines,
                        command_palette_key,
                        palette_open: false,
                        palette_lines_input: String::new(),
                        client_config: client_config.clone(),
                    },
                    Command::none(),
                );
            }
        };

        // ── 3. Extrai credenciais (senha zerizada em ssh.rs após auth) ─────────
        let ssh_auth = match &host.auth {
            AuthType::Password(p) => {
                match p.unprotect() {
                    Ok(secret) => SshAuth::Password(secret),
                    Err(e) => {
                        let mut term = TerminalState::new(24, 80, max_scrollback);
                        term.process_bytes(
                            format!("ERRO DE MEMORIA: {}\r\n", e).as_bytes(),
                        );
                        return (
                            TerminalApp {
                                host_name: host_name_display,
                                status: TerminalStatus::Disconnected(format!("Erro de Memória: {}", e)),
                                terminal: term,
                                sel_anchor: None,
                                sel_cursor: None,
                                cmd_sender: None,
                                event_receiver: None,
                                cursor_visible: true,
                                scroll_offset: 0,
                                scroll_lines,
                                command_palette_key,
                                palette_open: false,
                                palette_lines_input: String::new(),
                                client_config: client_config.clone(),
                            },
                            Command::none(),
                        );
                    }
                }
            },
            AuthType::Key { path, passphrase } => SshAuth::PrivateKey {
                path: path.clone(),
                passphrase: passphrase.as_ref().and_then(|p| p.unprotect().ok()),
            },
            AuthType::None => SshAuth::Password(secrecy::SecretString::new(String::new())),
        };

        let host_name = host_name_display;

        // ── 4. Calcula tamanho inicial do PTY a partir do tamanho da janela ───
        let canvas_h  = DEFAULT_WIN_H;
        let init_cols = (DEFAULT_WIN_W / CELL_W).floor() as usize;
        let init_rows = (canvas_h / CELL_H).floor() as usize;
        let pty_cols  = init_cols.max(1) as u16;
        let pty_rows  = init_rows.max(1) as u16;

        // ── 5. Cria canais ────────────────────────────────────────────────────
        let (cmd_tx, cmd_rx)     = mpsc::channel::<NetworkCommand>(64);
        let (event_tx, event_rx) = mpsc::channel::<NetworkEvent>(256);
        let event_arc: SharedEventReceiver = Arc::new(Mutex::new(Some(event_rx)));

        // ── 6. Inicializa grid com o tamanho do PTY ───────────────────────────
        let mut terminal = TerminalState::new(init_rows.max(1), init_cols.max(1), max_scrollback);
        terminal.process_bytes(b"Conectando...\r\n");

        // ── 7. Spawna a task SSH ──────────────────────────────────────────────
        let host_addr = host.address.clone();
        let host_port = host.port;
        let host_user = host.username.clone();

        let spawn_cmd = Command::perform(
            async move {
                tokio::spawn(start_ssh_session(
                    host_addr, host_port, host_user,
                    ssh_auth, pty_cols, pty_rows,
                    event_tx, cmd_rx,
                ));
            },
            |_| TerminalMessage::Noop,
        );

        let app = TerminalApp {
            host_name,
            status: TerminalStatus::Connecting,
            terminal,
            sel_anchor: None,
            sel_cursor: None,
            cmd_sender: Some(cmd_tx),
            event_receiver: Some(event_arc),
            cursor_visible: true,
            scroll_offset: 0,
            scroll_lines,
            command_palette_key,
            palette_open: false,
            palette_lines_input: String::new(),
            client_config,
        };

        (app, spawn_cmd)
    }

    fn title(&self) -> String {
        let tag = match &self.status {
            TerminalStatus::Connecting      => "⟳",
            TerminalStatus::Connected       => "●",
            TerminalStatus::Disconnected(_) => "○",
            TerminalStatus::Error(_)        => "✕",
            TerminalStatus::HostNotFound    => "✕",
        };
        format!("RusTTY — {} [{}]", self.host_name, tag)
    }

    fn theme(&self) -> Theme { Theme::Dark }

    fn subscription(&self) -> Subscription<TerminalMessage> {
        Subscription::batch(vec![
            // ── Cursor Blink Timer ───────────────────────────────────────────
            iced::time::every(std::time::Duration::from_millis(600))
                .map(TerminalMessage::CursorBlink),
            // ── Eventos SSH ──────────────────────────────────────────────────
            self.ssh_subscription(),
            // ── Eventos de teclado e janela (Iced 0.12: listen_with) ─────────
            iced::event::listen_with(|event, _| {
                match &event {
                    iced::Event::Keyboard(_) => Some(TerminalMessage::IcedEvent(event)),
                    iced::Event::Window(_, iced::window::Event::Resized { .. }) => {
                        Some(TerminalMessage::IcedEvent(event))
                    }
                    iced::Event::Window(id, iced::window::Event::CloseRequested) => {
                        Some(TerminalMessage::WindowCloseRequested(*id))
                    }
                    _ => None,
                }
            }),
        ])
    }

    fn update(&mut self, message: TerminalMessage) -> Command<TerminalMessage> {
        match message {
            // ── Eventos SSH ──────────────────────────────────────────────────
            TerminalMessage::SshEvent(event) => {
                match event {
                    NetworkEvent::Connected(msg) => {
                        self.status = TerminalStatus::Connected;
                        let banner = format!("\r\n── {} ──\r\n\r\n", msg);
                        self.terminal.process_bytes(banner.as_bytes());
                    }
                    NetworkEvent::DataReceived(bytes) => {
                        self.terminal.process_bytes(&bytes);
                        self.scroll_offset = 0;
                    }
                    NetworkEvent::Disconnected(msg) => {
                        self.status = TerminalStatus::Disconnected(msg.clone());
                        let note = format!("\r\n\r\n── {} ──\r\n", msg);
                        self.terminal.process_bytes(note.as_bytes());
                    }
                    NetworkEvent::Error(msg) => {
                        self.status = TerminalStatus::Error(msg.clone());
                        let err = format!("\r\n\x1b[31mERRO: {}\x1b[0m\r\n", msg);
                        self.terminal.process_bytes(err.as_bytes());
                    }
                }
            }

            // ── Eventos de Teclado ───────────────────────────────────────────
            TerminalMessage::IcedEvent(iced::Event::Keyboard(kb_event)) => {
                self.scroll_offset = 0;
                return self.handle_keyboard(kb_event);
            }

            // ── Resize da Janela ─────────────────────────────────────────────
            TerminalMessage::IcedEvent(iced::Event::Window(
                _,
                iced::window::Event::Resized { width, height },
            )) => {
                let canvas_h  = height as f32;
                let new_cols  = ((width as f32) / CELL_W).floor() as usize;
                let new_rows  = (canvas_h / CELL_H).floor() as usize;
                let (new_cols, new_rows) = (new_cols.max(1), new_rows.max(1));

                if new_cols != self.terminal.grid.cols || new_rows != self.terminal.grid.rows {
                    self.terminal.resize(new_rows, new_cols);
                    if let Some(tx) = self.cmd_sender.clone() {
                        return Command::perform(
                            async move {
                                tx.send(NetworkCommand::ResizePty {
                                    cols: new_cols as u16,
                                    rows: new_rows as u16,
                                })
                                .await
                                .ok();
                            },
                            |_| TerminalMessage::Noop,
                        );
                    }
                }
            }

            // ── Seleção de Texto ─────────────────────────────────────────────
            TerminalMessage::SelectionStart(r, c) => {
                self.sel_anchor = Some((r, c));
                self.sel_cursor = Some((r, c));
            }
            TerminalMessage::SelectionExtend(r, c) => {
                self.sel_cursor = Some((r, c));
            }

            // ── Clear Selection ──────────────────────────────────────────────
            TerminalMessage::ClearSelection => {
                self.sel_anchor = None;
                self.sel_cursor = None;
            }

            // ── Mover Cursor (Clique) ─────────────────────────────────────────
            TerminalMessage::MoveCursorLeftRight(diff) => {
                if diff != 0 {
                    let mut bytes = Vec::new();
                    if diff > 0 {
                        for _ in 0..diff { bytes.extend_from_slice(b"\x1b[C"); }
                    } else {
                        for _ in 0..(-diff) { bytes.extend_from_slice(b"\x1b[D"); }
                    }
                    return self.send_bytes(bytes);
                }
            }

            // ── Right Click (Copy/Paste) ──────────────────────────────────────
            TerminalMessage::RightClick => {
                let has_selection = if let (Some(a), Some(b)) = (self.sel_anchor, self.sel_cursor) {
                    a != b
                } else {
                    false
                };

                if has_selection {
                    let text = self.terminal.grid.selected_text(self.sel_anchor.unwrap(), self.sel_cursor.unwrap());
                    self.sel_anchor = None;
                    self.sel_cursor = None;
                    if !text.is_empty() {
                        return iced::clipboard::write(text);
                    }
                } else {
                    return iced::clipboard::read(TerminalMessage::ClipboardContent);
                }
            }

            // ── Paste do Clipboard ───────────────────────────────────────────
            TerminalMessage::ClipboardContent(Some(text)) => {

                if let Some(tx) = self.cmd_sender.clone() {
                    let bytes = text.into_bytes();
                    return Command::perform(
                        async move { tx.send(NetworkCommand::SendData(bytes)).await.ok(); },
                        |_| TerminalMessage::Noop,
                    );
                }
            }

            TerminalMessage::Disconnect => {
                if let Some(tx) = self.cmd_sender.clone() {
                    return Command::perform(
                        async move { tx.send(NetworkCommand::Disconnect).await.ok(); },
                        |_| TerminalMessage::Noop,
                    );
                }
            }
            TerminalMessage::WindowCloseRequested(id) => {
                if let Some(tx) = self.cmd_sender.clone() {
                    return Command::perform(
                        async move {
                            tx.send(NetworkCommand::Disconnect).await.ok();
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        },
                        move |_| TerminalMessage::ForceClose(id),
                    );
                } else {
                    return iced::window::close(id);
                }
            }
            TerminalMessage::ForceClose(id) => {
                return iced::window::close(id);
            }
            TerminalMessage::CursorBlink(_) => {
                self.cursor_visible = !self.cursor_visible;
            }
            TerminalMessage::ScrollWheel(dy) => {
                let max_offset = self.terminal.grid.scrollback.len();
                let jump = (dy.abs() as usize).max(1) * self.scroll_lines;
                
                if dy > 0.0 {
                    // Scroll up
                    self.scroll_offset = self.scroll_offset.saturating_add(jump).min(max_offset);
                } else if dy < 0.0 {
                    // Scroll down
                    self.scroll_offset = self.scroll_offset.saturating_sub(jump);
                }
                return Command::none();
            }

            // ── Command Palette ──────────────────────────────────────────────
            TerminalMessage::TogglePalette => {
                self.palette_open = !self.palette_open;
            }
            TerminalMessage::PaletteLinesInputChanged(val) => {
                let clean_val: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                self.palette_lines_input = clean_val;
            }
            TerminalMessage::PaletteActionCopyAll => {
                let total_rows = self.terminal.grid.scrollback.len() + self.terminal.grid.rows;
                if total_rows > 0 {
                    let text = self.terminal.grid.selected_text(
                        (0, 0),
                        (total_rows.saturating_sub(1), self.terminal.grid.cols.saturating_sub(1))
                    );
                    self.palette_open = false; // Fecha palette ao executar
                    if !text.is_empty() {
                        return iced::clipboard::write(text);
                    }
                }
            }
            TerminalMessage::PaletteActionCopyLast => {
                if let Ok(lines) = self.palette_lines_input.parse::<usize>() {
                    let total_rows = self.terminal.grid.scrollback.len() + self.terminal.grid.rows;
                    let start_row = total_rows.saturating_sub(lines);
                    if total_rows > 0 {
                        let text = self.terminal.grid.selected_text(
                            (start_row, 0),
                            (total_rows.saturating_sub(1), self.terminal.grid.cols.saturating_sub(1))
                        );
                        self.palette_open = false;
                        if !text.is_empty() {
                            return iced::clipboard::write(text);
                        }
                    }
                }
            }
            TerminalMessage::PaletteActionClear => {
                self.terminal.clear_all();
                self.scroll_offset = 0;
                self.palette_open = false;
            }

            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, TerminalMessage> {
        let terminal_canvas = Canvas::new(TerminalCanvas {
            grid:       &self.terminal.grid,
            sel_anchor: self.sel_anchor,
            sel_cursor: self.sel_cursor,
            cursor_visible: self.cursor_visible,
            scroll_offset: self.scroll_offset,
            client_config: &self.client_config,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let underlying = container(terminal_canvas)
            .width(Length::Fill)
            .height(Length::Fill);

        if !self.palette_open {
            return underlying.into();
        }

        let modal_content = container(
            column![
                row![
                    icon_sized::<TerminalMessage>(LucideIcon::Settings, 24),
                    Space::with_width(Length::Fixed(8.0)),
                    text("Command Palette").size(20).style(theme::Text::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                ].align_items(Alignment::Center),
                Space::with_height(Length::Fixed(24.0)),
                button(text("Copy All").size(16).horizontal_alignment(Horizontal::Center))
                    .width(Length::Fill)
                    .padding([12, 16])
                    .on_press(TerminalMessage::PaletteActionCopyAll),
                Space::with_height(Length::Fixed(12.0)),
                row![
                    button(text("Copy Last").size(16).horizontal_alignment(Horizontal::Center))
                        .width(Length::Fill)
                        .padding([12, 16])
                        .on_press(TerminalMessage::PaletteActionCopyLast),
                    Space::with_width(Length::Fixed(8.0)),
                    text_input("Linhas...", &self.palette_lines_input)
                        .on_input(TerminalMessage::PaletteLinesInputChanged)
                        .width(Length::Fixed(100.0))
                        .padding(12)
                        .size(16),
                ].align_items(Alignment::Center),
                Space::with_height(Length::Fixed(12.0)),
                button(text("Clear Terminal").size(16).horizontal_alignment(Horizontal::Center))
                    .width(Length::Fill)
                    .padding([12, 16])
                    .on_press(TerminalMessage::PaletteActionClear),
                Space::with_height(Length::Fixed(24.0)),
                button(text("Close").size(16).horizontal_alignment(Horizontal::Center))
                    .width(Length::Fill)
                    .padding([12, 16])
                    .on_press(TerminalMessage::TogglePalette)
                    .style(theme::Button::Destructive),
            ]
        )
        .width(Length::Fixed(400.0))
        .padding(32)
        .style(theme::Container::Custom(Box::new(PaletteStyle)));

        let overlay: Option<Element<'_, TerminalMessage>> = if self.palette_open {
            Some(modal_content.into())
        } else {
            None
        };

        iced_aw::Modal::new(
            underlying,
            overlay,
        )
        .on_esc(TerminalMessage::TogglePalette)
        .into()
    }
}

// ─── Keyboard Handling ────────────────────────────────────────────────────────

impl TerminalApp {
    fn handle_keyboard(&mut self, event: keyboard::Event) -> Command<TerminalMessage> {
        self.cursor_visible = true; // Mantém o cursor visível ao digitar
        // Apenas `KeyPressed` produz input — `KeyReleased` e `ModifiersChanged` são ignorados
        let (key, modifiers, text) = match event {
            keyboard::Event::KeyPressed { key, modifiers, text, .. } => (key, modifiers, text),
            _ => return Command::none(),
        };

        // ── Atalho da Command Palette ─────────────────────────────────────────
        if modifiers.control() && !modifiers.shift() {
            if let keyboard::Key::Character(ref c) = key {
                if c.as_str().to_lowercase().chars().next().unwrap_or('\0') == self.command_palette_key {
                    return Command::perform(async {}, |_| TerminalMessage::TogglePalette);
                }
            }
        }

        // ── Atalhos de Clipboard (intercepta antes de qualquer envio SSH) ─────
        if modifiers.control() && modifiers.shift() {
            if let keyboard::Key::Character(ref c) = key {
                match c.as_str().to_lowercase().as_str() {
                    "c" => {
                        // Ctrl+Shift+C → copia seleção
                        if let (Some(a), Some(b)) = (self.sel_anchor, self.sel_cursor) {
                            let selected = self.terminal.grid.selected_text(a, b);
                            if !selected.is_empty() {
                                return iced::clipboard::write(selected);
                            }
                        }
                        return Command::none(); // Não envia para SSH
                    }
                    "v" => {
                        // Ctrl+Shift+V → paste do clipboard
                        return iced::clipboard::read(TerminalMessage::ClipboardContent);
                    }
                    _ => {}
                }
            }
        }

        // Qualquer input limpa a seleção visual
        self.sel_anchor = None;
        self.sel_cursor = None;

        // ── Teclas nomeadas → sequências de escape ───────────────────────────
        if let keyboard::Key::Named(ref named) = key {
            let bytes: Option<Vec<u8>> = match named {
                Named::ArrowUp    => Some(b"\x1b[A".to_vec()),
                Named::ArrowDown  => Some(b"\x1b[B".to_vec()),
                Named::ArrowRight => Some(b"\x1b[C".to_vec()),
                Named::ArrowLeft  => Some(b"\x1b[D".to_vec()),
                Named::Home       => Some(b"\x1b[H".to_vec()),
                Named::End        => Some(b"\x1b[F".to_vec()),
                Named::PageUp     => Some(b"\x1b[5~".to_vec()),
                Named::PageDown   => Some(b"\x1b[6~".to_vec()),
                Named::Insert     => Some(b"\x1b[2~".to_vec()),
                Named::Delete     => Some(b"\x1b[3~".to_vec()),
                // Backspace → DEL (0x7F) — padrão de terminais modernos
                Named::Backspace  => Some(b"\x7f".to_vec()),
                Named::Escape     => Some(b"\x1b".to_vec()),
                Named::F1         => Some(b"\x1bOP".to_vec()),
                Named::F2         => Some(b"\x1bOQ".to_vec()),
                Named::F3         => Some(b"\x1bOR".to_vec()),
                Named::F4         => Some(b"\x1bOS".to_vec()),
                Named::F5         => Some(b"\x1b[15~".to_vec()),
                Named::F6         => Some(b"\x1b[17~".to_vec()),
                Named::F7         => Some(b"\x1b[18~".to_vec()),
                Named::F8         => Some(b"\x1b[19~".to_vec()),
                Named::F9         => Some(b"\x1b[20~".to_vec()),
                Named::F10        => Some(b"\x1b[21~".to_vec()),
                Named::F11        => Some(b"\x1b[23~".to_vec()),
                Named::F12        => Some(b"\x1b[24~".to_vec()),
                _ => None,
            };
            if let Some(data) = bytes {
                return self.send_bytes(data);
            }
        }

        // ── Texto regular (inclui Enter, Tab, Ctrl+letra) ────────────────────
        // Em Iced 0.12, o campo `text` de KeyPressed contém o caractere gerado
        // pelo OS (incluindo Ctrl+C → '\x03', Ctrl+D → '\x04', etc.)
        if let Some(t) = text {
            let s = t.as_str();
            if !s.is_empty() {
                return self.send_bytes(s.as_bytes().to_vec());
            }
        }

        Command::none()
    }

    fn send_bytes(&self, data: Vec<u8>) -> Command<TerminalMessage> {
        if self.status != TerminalStatus::Connected {
            return Command::none();
        }
        if let Some(tx) = self.cmd_sender.clone() {
            Command::perform(
                async move { tx.send(NetworkCommand::SendData(data)).await.ok(); },
                |_| TerminalMessage::Noop,
            )
        } else {
            Command::none()
        }
    }

    fn ssh_subscription(&self) -> Subscription<TerminalMessage> {
        if let Some(arc) = &self.event_receiver {
            let arc = Arc::clone(arc);
            iced::subscription::channel(
                std::any::TypeId::of::<TerminalApp>(),
                256,
                move |mut output| async move {
                    use iced::futures::SinkExt;

                    let mut rx = { arc.lock().await.take() };

                    if let Some(ref mut rx) = rx {
                        while let Some(event) = rx.recv().await {
                            if output.send(TerminalMessage::SshEvent(event)).await.is_err() {
                                break;
                            }
                        }
                    }
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                    }
                },
            )
        } else {
            Subscription::none()
        }
    }
}

// ─── Canvas do Terminal ───────────────────────────────────────────────────────

/// Programa de Canvas que renderiza a grade VTE do terminal.
struct TerminalCanvas<'a> {
    grid:       &'a crate::terminal::TerminalGrid,
    sel_anchor: Option<(usize, usize)>,
    sel_cursor: Option<(usize, usize)>,
    cursor_visible: bool,
    scroll_offset: usize,
    client_config: &'a crate::config::client::ClientConfig,
}

/// Estado de drag do canvas (gerenciado pelo widget, persiste entre frames).
#[derive(Default)]
struct CanvasDrag {
    dragging: bool,
    start_pos: Option<(usize, usize)>,
}

impl<'a> canvas::Program<TerminalMessage> for TerminalCanvas<'a> {
    type State = CanvasDrag;

    fn update(
        &self,
        state:  &mut CanvasDrag,
        event:  canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<TerminalMessage>) {
        use canvas::Event as CE;
        use mouse::Event as ME;
        use mouse::Button;
        use canvas::event::Status;

        match event {
            CE::Mouse(ME::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let col = ((pos.x / CELL_W) as usize).min(self.grid.cols.saturating_sub(1));
                    let row = ((pos.y / CELL_H) as usize).min(self.grid.rows.saturating_sub(1));
                    let abs_top = self.grid.scrollback.len().saturating_sub(self.scroll_offset);
                    let abs_row = abs_top + row;
                    state.dragging = true;
                    state.start_pos = Some((abs_row, col));
                    return (Status::Captured, Some(TerminalMessage::SelectionStart(abs_row, col)));
                }
            }
            CE::Mouse(ME::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let col = ((pos.x / CELL_W) as usize).min(self.grid.cols.saturating_sub(1));
                    let row = ((pos.y / CELL_H) as usize).min(self.grid.rows.saturating_sub(1));
                    let abs_top = self.grid.scrollback.len().saturating_sub(self.scroll_offset);
                    let abs_row = abs_top + row;
                    return (Status::Captured, Some(TerminalMessage::SelectionExtend(abs_row, col)));
                }
            }
            CE::Mouse(ME::ButtonReleased(Button::Left)) => {
                state.dragging = false;
                if let (Some(pos), Some(start_pos)) = (cursor.position_in(bounds), state.start_pos) {
                    let col = ((pos.x / CELL_W) as usize).min(self.grid.cols.saturating_sub(1));
                    let row = ((pos.y / CELL_H) as usize).min(self.grid.rows.saturating_sub(1));
                    let abs_top = self.grid.scrollback.len().saturating_sub(self.scroll_offset);
                    let abs_row = abs_top + row;
                    
                    if (abs_row, col) == start_pos {
                        // Clique simples (sem drag)
                        // Apenas consideramos mover cursor se for na viewport atual e na linha do cursor real
                        let cursor_abs_row = self.grid.scrollback.len() + self.grid.cursor_row;
                        if abs_row == cursor_abs_row {
                            let diff = col as isize - self.grid.cursor_col as isize;
                            return (Status::Captured, Some(TerminalMessage::MoveCursorLeftRight(diff)));
                        } else {
                            return (Status::Captured, Some(TerminalMessage::ClearSelection));
                        }
                    }
                }
                return (Status::Captured, None);
            }
            CE::Mouse(ME::ButtonPressed(Button::Right)) => {
                return (Status::Captured, Some(TerminalMessage::RightClick));
            }
            CE::Mouse(ME::WheelScrolled { delta }) => {
                match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        return (Status::Captured, Some(TerminalMessage::ScrollWheel(y)));
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => {
                        return (Status::Captured, Some(TerminalMessage::ScrollWheel(y / CELL_H)));
                    }
                }
            }
            _ => {}
        }
        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &CanvasDrag,
        renderer: &iced::Renderer,
        _theme:   &Theme,
        bounds:   Rectangle,
        _cursor:  mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // ── Fundo do terminal ────────────────────────────────────────────────
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            cell_color_to_iced(CellColor::DEFAULT_BG),
        );

        let grid    = self.grid;
        let has_sel = self.sel_anchor.is_some() && self.sel_cursor.is_some();

        let abs_top = grid.scrollback.len().saturating_sub(self.scroll_offset);

        // ── Renderiza células ────────────────────────────────────────────────
        for row in 0..grid.rows {
            let abs_row = abs_top + row;
            let from_scrollback = abs_row < grid.scrollback.len();

            // Computa cores customizadas da linha atual (Highlighting)
            let mut custom_colors = vec![None; grid.cols];
            if self.client_config.enable_customization {
                let mut row_str = String::with_capacity(grid.cols);
                for col in 0..grid.cols {
                    let cell = if from_scrollback {
                        &grid.scrollback[abs_row][col]
                    } else {
                        &grid.cells[abs_row - grid.scrollback.len()][col]
                    };
                    row_str.push(cell.ch);
                }

                // Palavras chave
                for kw in &self.client_config.customization_data.keywords {
                    if let Ok(color) = crate::app::hex_to_color(&kw.color) {
                        let target = if kw.case_insensitive { kw.keyword.to_lowercase() } else { kw.keyword.clone() };
                        let search_in = if kw.case_insensitive { row_str.to_lowercase() } else { row_str.clone() };
                        
                        let mut start_idx = 0;
                        while let Some(idx) = search_in[start_idx..].find(&target) {
                            let match_start = start_idx + idx;
                            let match_end = match_start + target.len();

                            let is_start_boundary = match_start == 0 || !search_in[..match_start].chars().last().unwrap().is_ascii_alphanumeric();
                            let is_end_boundary = match_end == search_in.len() || !search_in[match_end..].chars().next().unwrap().is_ascii_alphanumeric();

                            if is_start_boundary && is_end_boundary {
                                let char_start = search_in[..match_start].chars().count();
                                let char_count = target.chars().count();
                                for i in char_start..(char_start + char_count) {
                                    if i < grid.cols { custom_colors[i] = Some(color); }
                                }
                            }
                            start_idx = match_start + target.len();
                        }
                    }
                }

                // IPs
                let check_ipv4 = self.client_config.customization_data.ipv4.is_some();
                let check_ipv6 = self.client_config.customization_data.ipv6.is_some();
                if check_ipv4 || check_ipv6 {
                    let mut word_start = None;
                    let chars: Vec<char> = row_str.chars().collect();
                    for (i, &c) in chars.iter().enumerate() {
                        let is_ip_char = c.is_ascii_alphanumeric() || c == '.' || c == ':';
                        if is_ip_char {
                            if word_start.is_none() { word_start = Some(i); }
                        } else if let Some(start) = word_start {
                            let word = &row_str[start..i];
                            apply_ip_highlight(word, start, i, &mut custom_colors, &self.client_config.customization_data);
                            word_start = None;
                        }
                    }
                    if let Some(start) = word_start {
                        let word = &row_str[start..chars.len()];
                        apply_ip_highlight(word, start, chars.len(), &mut custom_colors, &self.client_config.customization_data);
                    }
                }
            }

            for col in 0..grid.cols {
                let cell = if from_scrollback {
                    &grid.scrollback[abs_row][col]
                } else {
                    let grid_row = abs_row - grid.scrollback.len();
                    &grid.cells[grid_row][col]
                };

                let x = col as f32 * CELL_W;
                let y = row as f32 * CELL_H;

                let in_sel = has_sel && grid.in_selection(
                    abs_row, col,
                    self.sel_anchor.unwrap(),
                    self.sel_cursor.unwrap(),
                );

                let eff_bg = cell.effective_bg();
                let eff_fg = cell.effective_fg();

                // Fundo da célula
                if in_sel {
                    // Highlight de seleção azul
                    frame.fill_rectangle(
                        Point::new(x, y),
                        Size::new(CELL_W, CELL_H),
                        Color::from_rgba(0.25, 0.45, 0.85, 0.6),
                    );
                } else if eff_bg != CellColor::DEFAULT_BG {
                    frame.fill_rectangle(
                        Point::new(x, y),
                        Size::new(CELL_W, CELL_H),
                        cell_color_to_iced(eff_bg),
                    );
                }

                // Caractere (pula espaços padrão para performance)
                if cell.ch != ' ' {
                    let mut fg_color = if in_sel {
                        Color::WHITE
                    } else {
                        cell_color_to_iced(eff_fg)
                    };

                    if let Some(c) = custom_colors[col] {
                        if !in_sel { fg_color = c; }
                    }

                    frame.fill_text(canvas::Text {
                        content:              cell.ch.to_string(),
                        position:             Point::new(x, y),
                        color:                fg_color,
                        size:                 Pixels(FONT_SIZE),
                        font:                 MONOSPACE,
                        horizontal_alignment: Horizontal::Left,
                        vertical_alignment:   Vertical::Top,
                        line_height:          iced::widget::text::LineHeight::Absolute(Pixels(CELL_H)),
                        shaping:              iced::widget::text::Shaping::Basic,
                    });
                }
            }
        }

        // ── Cursor ───────────────────────────────────────────────────────────
        let cr = grid.cursor_row;
        let cc = grid.cursor_col;
        
        let screen_row = cr + self.scroll_offset;
        
        if self.cursor_visible && screen_row < grid.rows && cc < grid.cols {
            let cx = cc as f32 * CELL_W;
            let cy = screen_row as f32 * CELL_H;
            // Cursor como bloco branco sólido
            frame.fill_rectangle(
                Point::new(cx, cy),
                Size::new(CELL_W, CELL_H),
                Color::from_rgba(1.0, 1.0, 1.0, 0.85),
            );
            // Caractere do cursor em cor invertida
            let cur_cell = &grid.cells[cr][cc];
            if cur_cell.ch != ' ' {
                frame.fill_text(canvas::Text {
                    content:              cur_cell.ch.to_string(),
                    position:             Point::new(cx, cy),
                    color:                cell_color_to_iced(CellColor::DEFAULT_BG),
                    size:                 Pixels(FONT_SIZE),
                    font:                 MONOSPACE,
                    horizontal_alignment: Horizontal::Left,
                    vertical_alignment:   Vertical::Top,
                    line_height:          iced::widget::text::LineHeight::Absolute(Pixels(CELL_H)),
                    shaping:              iced::widget::text::Shaping::Basic,
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state:  &CanvasDrag,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::Text
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Converte `CellColor` para `iced::Color`.
#[inline]
fn cell_color_to_iced(c: CellColor) -> Color {
    Color::from_rgb8(c.r, c.g, c.b)
}

fn apply_ip_highlight(
    word: &str,
    start: usize,
    end: usize,
    custom_colors: &mut Vec<Option<Color>>,
    config: &crate::config::client::CustomizationConfig,
) {
    if let Ok(ip) = std::net::IpAddr::from_str(word) {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                if let Some(ip_cfg) = &config.ipv4 {
                    let color = match ip_cfg {
                        crate::config::client::IpCustomization::Unified(c) => crate::app::hex_to_color(c).unwrap_or(Color::WHITE),
                        crate::config::client::IpCustomization::Split { public, private } => {
                            let octets = ipv4.octets();
                            let is_private = octets[0] == 10 || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) || (octets[0] == 192 && octets[1] == 168) || ipv4.is_loopback() || ipv4.is_link_local();
                            crate::app::hex_to_color(if is_private { private } else { public }).unwrap_or(Color::WHITE)
                        }
                    };
                    for i in start..end { if i < custom_colors.len() { custom_colors[i] = Some(color); } }
                }
            }
            std::net::IpAddr::V6(ipv6) => {
                if let Some(ip_cfg) = &config.ipv6 {
                    let color = match ip_cfg {
                        crate::config::client::IpCustomization::Unified(c) => crate::app::hex_to_color(c).unwrap_or(Color::WHITE),
                        crate::config::client::IpCustomization::Split { public, private } => {
                            let is_private = ipv6.is_loopback() || (ipv6.segments()[0] & 0xfe00) == 0xfc00 || (ipv6.segments()[0] & 0xffc0) == 0xfe80;
                            crate::app::hex_to_color(if is_private { private } else { public }).unwrap_or(Color::WHITE)
                        }
                    };
                    for i in start..end { if i < custom_colors.len() { custom_colors[i] = Some(color); } }
                }
            }
        }
    }
}


// ─── Ponto de Entrada ─────────────────────────────────────────────────────────

/// Inicia o `TerminalApp` como aplicação Iced independente.
pub fn run_terminal(init: TerminalInit) -> iced::Result {
    let title = match &init {
        TerminalInit::SavedHost(name) => name.clone(),
        TerminalInit::QuickSsh { address, .. } => address.clone(),
    };
    
    TerminalApp::run(iced::Settings {
        fonts: vec![],
        flags:  init,
        window: iced::window::Settings {
            size:     iced::Size::new(DEFAULT_WIN_W, DEFAULT_WIN_H),
            min_size: Some(iced::Size::new(900.0, 400.0)),
            exit_on_close_request: false,
            icon: crate::ui::icons::load_window_icon(),
            ..Default::default()
        },
        ..iced::Settings::default()
    })
}

struct PaletteStyle;
impl iced::widget::container::StyleSheet for PaletteStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
            border: iced::Border {
                color: Color::from_rgb(0.2, 0.2, 0.2),
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.8),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 20.0,
            },
        }
    }
}
