//! Módulo de terminal em Win32 API pura + DirectWrite de alta performance para o RusTTY.
//!
//! Não utiliza nenhuma biblioteca gráfica pesada (como Iced ou Skia), alcançando
//! taxa de quadros máxima, latência de renderização mínima e consumo ínfimo de CPU e memória.

pub mod font;
pub mod input;
pub mod renderer;
pub mod window;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::config::{load_config, AuthType, ConfigNode, client::load_client_config};
use crate::net::{NetworkCommand, NetworkEvent, SshAuth};
use crate::net::ssh::start_ssh_session;
use crate::terminal::TerminalInit;
use crate::terminal::win32::window::WM_APP_NETWORK_EVENT;

fn find_host_in_nodes(nodes: &[ConfigNode], target: &str) -> Option<crate::config::HostProfile> {
    for node in nodes {
        match node {
            ConfigNode::Host(h) if h.name == target => return Some(h.clone()),
            ConfigNode::Folder { children, .. } => {
                if let Some(h) = find_host_in_nodes(children, target) {
                    return Some(h);
                }
            }
            _ => {}
        }
    }
    None
}

/// Ponto de entrada para execução da janela de terminal nativa Win32.
pub fn run_win32_terminal(init: TerminalInit) -> Result<(), Box<dyn std::error::Error>> {
    crate::debug_log!("INFO", "Iniciando Terminal Nativo Win32 DirectWrite (Experimental)");

    // 0. Garante que a fonte JetBrains Mono esteja registrada no sistema
    font::ensure_jetbrains_mono_registered();

    // 1. Inicializa COM para Direct2D/DirectWrite
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }

    // 2. Carrega configurações do cliente
    let config = load_config();
    let client_config = load_client_config();

    // 3. Localiza o perfil do host ou usa Quick Connect / Bridge
    let (host_name_display, host) = match init {
        TerminalInit::SavedHost(name) => {
            let h = find_host_in_nodes(&config.root_nodes, &name);
            (name, h)
        }
        TerminalInit::Bridge(id_str) => {
            if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                let h = config.bridges.iter().find(|b| b.id == id).map(|b| {
                    crate::config::HostProfile {
                        id: b.id.to_string(),
                        name: b.name.clone(),
                        address: b.address.clone(),
                        port: b.port,
                        username: b.username.clone(),
                        auth: b.auth.clone(),
                        enable_icmp: false,
                        bridge_id: None,
                        legacy_ssh: false,
                        icon: None,
                    }
                });
                ("Ponte".to_string(), h)
            } else {
                ("Ponte".to_string(), None)
            }
        }
        TerminalInit::QuickSsh { address, port, user, pass } => {
            let auth = if pass == "none" {
                AuthType::None
            } else {
                AuthType::Password(
                    crate::config::ProtectedMemory::new(&pass).unwrap_or_else(|_| crate::config::ProtectedMemory::new("").unwrap())
                )
            };

            let h = crate::config::HostProfile {
                id: "quick-connect".to_string(),
                name: "Conexão Rápida".to_string(),
                address: address.clone(),
                port,
                username: user,
                auth,
                enable_icmp: false,
                bridge_id: None,
                legacy_ssh: false,
                icon: None,
            };
            (address, Some(h))
        }
    };

    let host = match host {
        Some(h) => h,
        None => {
            crate::debug_log!("ERROR", "Win32 Terminal: Host '{}' não encontrado", host_name_display);
            return Err(format!("Host '{}' não encontrado", host_name_display).into());
        }
    };

    // 4. Extrai credenciais SSH
    let ssh_auth = match &host.auth {
        AuthType::Password(p) => match p.unprotect() {
            Ok(secret) => SshAuth::Password(secret),
            Err(e) => return Err(format!("Erro ao desproteger senha SSH: {}", e).into()),
        },
        AuthType::Key { path, passphrase } => SshAuth::PrivateKey {
            path: path.clone(),
            passphrase: passphrase.as_ref().and_then(|p| p.unprotect().ok()),
        },
        AuthType::None => SshAuth::Password(secrecy::SecretString::new(String::new())),
    };

    let bridge_info = if let Some(bridge_id) = host.bridge_id {
        config.bridges.iter().find(|b| b.id == bridge_id).map(|bridge| {
            let bridge_auth = match &bridge.auth {
                AuthType::Password(p) => match p.unprotect() {
                    Ok(secret) => SshAuth::Password(secret),
                    Err(_) => SshAuth::Password(secrecy::SecretString::new(String::new())),
                },
                AuthType::Key { path, passphrase } => SshAuth::PrivateKey {
                    path: path.clone(),
                    passphrase: passphrase.as_ref().and_then(|p| p.unprotect().ok()),
                },
                AuthType::None => SshAuth::Password(secrecy::SecretString::new(String::new())),
            };
            Box::new((bridge.address.clone(), bridge.port, bridge.username.clone(), bridge_auth))
        })
    } else {
        None
    };

    // 5. Inicializa runtime Tokio para I/O assíncrono de rede
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _rt_guard = rt.enter();
    let rt_handle = rt.handle().clone();

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<NetworkCommand>(64);
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<NetworkEvent>(256);

    let network_events = Arc::new(Mutex::new(VecDeque::<NetworkEvent>::new()));

    // 6. Cria a janela do terminal Win32
    let hwnd = window::create_terminal_window(
        host_name_display,
        client_config,
        cmd_tx,
        Arc::clone(&network_events),
        rt_handle,
    )?;

    // 7. Spawna a sessão SSH e o despachante de eventos para o loop Win32
    let host_addr = host.address.clone();
    let host_port = host.port;
    let host_user = host.username.clone();
    let host_legacy = host.legacy_ssh;

    rt.spawn(async move {
        start_ssh_session(
            host_addr,
            host_port,
            host_user,
            ssh_auth,
            80,
            24,
            event_tx,
            cmd_rx,
            bridge_info,
            host_legacy,
        ).await;
    });

    // Tarefa de despache de eventos SSH -> Win32 via PostMessageW
    let hwnd_raw = hwnd.0 as usize;
    let events_queue = Arc::clone(&network_events);
    rt.spawn(async move {
        while let Some(ev) = event_rx.recv().await {
            if let Ok(mut q) = events_queue.lock() {
                q.push_back(ev);
            }
            unsafe {
                let target_hwnd = HWND(hwnd_raw as _);
                let _ = PostMessageW(target_hwnd, WM_APP_NETWORK_EVENT, WPARAM(0), LPARAM(0));
            }
        }
    });

    // 8. Executa o loop de mensagens da janela
    window::run_message_loop();

    // 9. Finaliza COM
    unsafe {
        CoUninitialize();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_BACK, VK_ESCAPE, VK_F1,
    };
    use crate::terminal::win32::input::{
        keydown_to_escape_sequence, format_sgr_mouse, select_word_at, select_line_at, InputState,
    };
    use crate::terminal::win32::renderer::cell_color_to_d2d;
    use crate::terminal::{TerminalGrid, CellColor};

    #[test]
    fn test_keydown_escape_sequences() {
        assert_eq!(keydown_to_escape_sequence(VK_UP), Some(&b"\x1b[A"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_DOWN), Some(&b"\x1b[B"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_LEFT), Some(&b"\x1b[D"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_RIGHT), Some(&b"\x1b[C"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_HOME), Some(&b"\x1b[H"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_END), Some(&b"\x1b[F"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_BACK), Some(&b"\x7f"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_ESCAPE), Some(&b"\x1b"[..]));
        assert_eq!(keydown_to_escape_sequence(VK_F1), Some(&b"\x1bOP"[..]));
    }

    #[test]
    fn test_sgr_mouse_formatting() {
        let press = format_sgr_mouse(0, 9, 19, false);
        assert_eq!(press, b"\x1b[<0;10;20M");

        let release = format_sgr_mouse(0, 9, 19, true);
        assert_eq!(release, b"\x1b[<0;10;20m");

        let scroll_up = format_sgr_mouse(64, 4, 8, false);
        assert_eq!(scroll_up, b"\x1b[<64;5;9M");
    }

    #[test]
    fn test_word_and_line_selection() {
        let mut grid = TerminalGrid::new(5, 20, 100);
        let text = "hello_world 123.456";
        for (i, c) in text.chars().enumerate() {
            grid.cells[0][i].ch = c;
        }

        let mut input = InputState::default();
        select_word_at(&grid, 0, 3, &mut input);
        assert_eq!(input.sel_anchor, Some((0, 0)));
        assert_eq!(input.sel_cursor, Some((0, 10))); // "hello_world"

        select_line_at(&grid, 0, &mut input);
        assert_eq!(input.sel_anchor, Some((0, 0)));
        assert_eq!(input.sel_cursor, Some((0, 19)));
    }

    #[test]
    fn test_cell_color_to_d2d() {
        let color = CellColor::rgb(255, 128, 0);
        let d2d = cell_color_to_d2d(color, 0.5);
        assert!((d2d.r - 1.0).abs() < 0.001);
        assert!((d2d.g - (128.0 / 255.0)).abs() < 0.001);
        assert_eq!(d2d.b, 0.0);
        assert_eq!(d2d.a, 0.5);
    }

    #[test]
    fn test_jetbrains_mono_font_loaded() {
        unsafe {
            use windows::Win32::Graphics::DirectWrite::{DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED};
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let mut collection: Option<IDWriteFontCollection> = None;
            dwrite.GetSystemFontCollection(&mut collection, false).unwrap();
            let collection = collection.unwrap();
            let mut index = 0u32;
            let mut exists = windows::Win32::Foundation::BOOL(0);
            collection.FindFamilyName(windows::core::w!("JetBrains Mono"), &mut index, &mut exists).unwrap();
            assert!(exists.as_bool(), "JetBrains Mono must be in system font collection!");
        }
    }

    #[test]
    fn test_jetbrains_mono_metrics() {
        unsafe {
            use windows::Win32::Graphics::DirectWrite::*;
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let format = dwrite.CreateTextFormat(
                windows::core::w!("JetBrains Mono"),
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                14.0,
                windows::core::w!("en-us"),
            ).unwrap();

            // Measure single 'M'
            let s_m: Vec<u16> = "M".encode_utf16().collect();
            let layout_m = dwrite.CreateTextLayout(&s_m, &format, 1000.0, 1000.0).unwrap();
            let mut m_metrics = std::mem::zeroed();
            layout_m.GetMetrics(&mut m_metrics).unwrap();

            // Measure single 'i'
            let s_i: Vec<u16> = "i".encode_utf16().collect();
            let layout_i = dwrite.CreateTextLayout(&s_i, &format, 1000.0, 1000.0).unwrap();
            let mut i_metrics = std::mem::zeroed();
            layout_i.GetMetrics(&mut i_metrics).unwrap();

            println!("M width: {}, i width: {}", m_metrics.width, i_metrics.width);
            assert!((m_metrics.width - i_metrics.width).abs() < 0.001, "JetBrains Mono must be strictly monospace!");

            // Check line metrics
            let mut line_metric = DWRITE_LINE_METRICS::default();
            let mut line_count = 1u32;
            layout_m.GetLineMetrics(Some(std::slice::from_mut(&mut line_metric)), &mut line_count).unwrap();
            println!("Line height: {}, baseline: {}", line_metric.height, line_metric.baseline);
            assert!(line_metric.height > 10.0);
        }
    }

    #[test]
    fn test_selection_lifecycle() {
        let mut input = InputState::default();
        assert!(!input.has_selection());

        // Single click sets anchor and cursor to same cell
        input.sel_anchor = Some((5, 10));
        input.sel_cursor = Some((5, 10));
        assert!(!input.has_selection(), "Identical anchor and cursor must not count as active selection");

        // Dragging expands selection
        input.sel_cursor = Some((5, 15));
        assert!(input.has_selection(), "Different anchor and cursor must count as active selection");

        // Subsequent single click elsewhere resets selection to a point
        input.sel_anchor = Some((8, 2));
        input.sel_cursor = Some((8, 2));
        assert!(!input.has_selection(), "New click must immediately clear previous active selection");

        // Clear selection
        input.clear_selection();
        assert_eq!(input.sel_anchor, None);
        assert_eq!(input.sel_cursor, None);
        assert!(!input.has_selection());
    }

    #[test]
    fn test_margin_coordinate_mapping() {
        let margin_x = crate::terminal::win32::renderer::MARGIN_X;
        let margin_y = crate::terminal::win32::renderer::MARGIN_Y;
        let cell_w = 8.4f32;
        let cell_h = 18.0f32;
        let cols = 80usize;
        let rows = 24usize;

        // Inside the left margin: should map to column 0, row 0
        let x = 4.0f32;
        let y = 2.0f32;
        let col = (((x - margin_x).max(0.0) / cell_w) as usize).min(cols.saturating_sub(1));
        let row = (((y - margin_y).max(0.0) / cell_h) as usize).min(rows.saturating_sub(1));
        assert_eq!(col, 0);
        assert_eq!(row, 0);

        // At exact start of text (x = margin_x): should map to column 0
        let x = margin_x;
        let y = margin_y;
        let col = (((x - margin_x).max(0.0) / cell_w) as usize).min(cols.saturating_sub(1));
        let row = (((y - margin_y).max(0.0) / cell_h) as usize).min(rows.saturating_sub(1));
        assert_eq!(col, 0);
        assert_eq!(row, 0);

        // At column 1 (x = margin_x + cell_w + 1.0)
        let x = margin_x + cell_w + 1.0;
        let y = margin_y + cell_h + 1.0;
        let col = (((x - margin_x).max(0.0) / cell_w) as usize).min(cols.saturating_sub(1));
        let row = (((y - margin_y).max(0.0) / cell_h) as usize).min(rows.saturating_sub(1));
        assert_eq!(col, 1);
        assert_eq!(row, 1);
    }

    #[test]
    fn test_keyword_boundary_standalone_up() {
        use crate::terminal::win32::renderer::is_keyword_token_char;

        let text = "<LOOPBACK,UP,LOWER_UP>";
        let target = "UP";

        // Finding instances of "UP"
        let mut matches = Vec::new();
        let mut start_idx = 0;
        while let Some(idx) = text[start_idx..].find(target) {
            let match_start = start_idx + idx;
            let match_end = match_start + target.len();

            let is_start_boundary = match_start == 0 || !is_keyword_token_char(text[..match_start].chars().last().unwrap());
            let is_end_boundary = match_end == text.len() || !is_keyword_token_char(text[match_end..].chars().next().unwrap());

            if is_start_boundary && is_end_boundary {
                matches.push((match_start, match_end));
            }
            start_idx = match_start + target.len();
        }

        // Must ONLY match the standalone "UP", never "UP" inside "LOWER_UP"
        assert_eq!(matches.len(), 1, "Only standalone UP must match");
        let (s, e) = matches[0];
        assert_eq!(&text[s..e], "UP");
        assert_eq!(s, 10); // inside ",UP,"
    }

    #[test]
    fn test_natural_measuring_mode_span_split() {
        unsafe {
            use windows::Win32::Graphics::DirectWrite::*;
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let format = dwrite.CreateTextFormat(
                windows::core::w!("JetBrains Mono"),
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                14.0,
                windows::core::w!("en-us"),
            ).unwrap();

            // Combined string "LOWER_UP"
            let s_full: Vec<u16> = "LOWER_UP".encode_utf16().collect();
            let layout_full = dwrite.CreateTextLayout(&s_full, &format, 1000.0, 1000.0).unwrap();
            let mut m_full = std::mem::zeroed();
            layout_full.GetMetrics(&mut m_full).unwrap();

            // Split string "LOWER_" and "UP"
            let s_part1: Vec<u16> = "LOWER_".encode_utf16().collect();
            let layout_part1 = dwrite.CreateTextLayout(&s_part1, &format, 1000.0, 1000.0).unwrap();
            let mut m_part1 = std::mem::zeroed();
            layout_part1.GetMetrics(&mut m_part1).unwrap();

            let s_part2: Vec<u16> = "UP".encode_utf16().collect();
            let layout_part2 = dwrite.CreateTextLayout(&s_part2, &format, 1000.0, 1000.0).unwrap();
            let mut m_part2 = std::mem::zeroed();
            layout_part2.GetMetrics(&mut m_part2).unwrap();

            println!("Full width: {}, Part1 + Part2: {}", m_full.width, m_part1.width + m_part2.width);
            assert!(
                (m_full.width - (m_part1.width + m_part2.width)).abs() < 0.0001,
                "Monospace natural advances must sum perfectly when split into separate spans!"
            );
        }
    }

    #[test]
    fn test_space_merging_drawcall_reduction() {
        use crate::terminal::{Cell, CellColor};
        // Linha com 10 palavras separadas por espaços simples e múltiplos
        let text = "Linux srv01 6.17.2-1-pve #1 SMP PREEMPT_DYNAMIC PMX 6.17.2-1 x86_64";
        let mut row_cells: Vec<Cell> = text.chars().map(|c| Cell {
            ch: c,
            fg: CellColor::WHITE,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }).collect();
        while row_cells.len() < 120 {
            row_cells.push(Cell::default());
        }

        // Lógica de batched spans com merge de espaços
        let last_content = row_cells.iter().rposition(|c| (c.ch != ' ' && c.ch != '\0') || c.underline);
        assert_eq!(last_content, Some(text.len() - 1));
        let line_end_col = last_content.unwrap() + 1;

        let mut spans = Vec::new();
        let mut col = 0;
        while col < line_end_col {
            let cell = &row_cells[col];
            if (cell.ch == ' ' || cell.ch == '\0') && !cell.underline {
                col += 1;
                continue;
            }

            let span_start = col;
            col += 1;
            while col < line_end_col {
                let next_cell = &row_cells[col];
                if next_cell.bold != cell.bold || next_cell.underline != cell.underline || next_cell.effective_fg() != cell.effective_fg() {
                    break;
                }
                col += 1;
            }
            let span_end = col;
            spans.push((span_start, span_end));
        }

        // Sem merge de espaços: haveria 10 spans (1 por palavra)
        // Com merge de espaços: há exatamente 1 ÚNICO span contínuo!
        assert_eq!(spans.len(), 1, "Frase com formatação uniforme deve ser agrupada em 1 único span!");
        assert_eq!(spans[0], (0, text.len()));
    }

    #[test]
    fn test_cached_keywords_and_ip_parsing() {
        use crate::terminal::win32::renderer::{CachedKeyword, parse_hex_color};
        let kw = CachedKeyword {
            target: "error".to_string(),
            color: parse_hex_color("#FF0000").unwrap(),
            case_insensitive: true,
        };

        assert_eq!(kw.target, "error");
        assert_eq!(kw.color.r, 255);
        assert_eq!(kw.color.g, 0);
        assert_eq!(kw.color.b, 0);
    }

    #[test]
    fn test_ip_highlighting_with_trailing_dot() {
        use crate::config::client::{ClientConfig, IpCustomization};
        use crate::terminal::win32::renderer::{compute_line_highlights, CachedIpColors, parse_hex_color};
        use crate::terminal::{Cell, CellColor};

        let mut config = ClientConfig::default();
        config.customization_data.ipv4 = Some(IpCustomization::Unified("#00FF00".to_string()));

        let cached_ip = CachedIpColors {
            ipv4_unified: parse_hex_color("#00FF00").ok(),
            ipv4_public: None,
            ipv4_private: None,
            ipv6_unified: None,
            ipv6_public: None,
            ipv6_private: None,
        };

        // 1. IP padrão sem ponto final ("192.168.65.18") -> aplica a todos os 13 caracteres
        let text1 = "192.168.65.18";
        let cells1: Vec<Cell> = text1.chars().map(|c| Cell {
            ch: c,
            fg: CellColor::WHITE,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }).collect();
        let mut row_str = String::new();
        let mut row_str_lower = String::new();
        let mut colors1 = vec![None; text1.len()];

        compute_line_highlights(&cells1, text1.len(), &config, &[], &cached_ip, &mut row_str, &mut row_str_lower, &mut colors1);
        for i in 0..13 {
            assert_eq!(colors1[i], Some(CellColor::rgb(0, 255, 0)), "Char {} de '{}' deve ser verde", i, text1);
        }

        // 2. IP com ponto final ("192.168.56.18.") -> aplica aos 13 caracteres do IP, e o ponto terminal NÃO é colorido
        let text2 = "192.168.56.18.";
        let cells2: Vec<Cell> = text2.chars().map(|c| Cell {
            ch: c,
            fg: CellColor::WHITE,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }).collect();
        let mut colors2 = vec![None; text2.len()];

        compute_line_highlights(&cells2, text2.len(), &config, &[], &cached_ip, &mut row_str, &mut row_str_lower, &mut colors2);
        for i in 0..13 {
            assert_eq!(colors2[i], Some(CellColor::rgb(0, 255, 0)), "Char {} de '{}' deve ser verde", i, text2);
        }
        assert_eq!(colors2[13], None, "Ponto terminal '.' de '192.168.56.18.' não deve ser colorido como parte do IP");

        // 3. IP com múltiplos pontos no fim ("192.168.56.18...")
        let text3 = "192.168.56.18...";
        let cells3: Vec<Cell> = text3.chars().map(|c| Cell {
            ch: c,
            fg: CellColor::WHITE,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }).collect();
        let mut colors3 = vec![None; text3.len()];
        compute_line_highlights(&cells3, text3.len(), &config, &[], &cached_ip, &mut row_str, &mut row_str_lower, &mut colors3);
        for i in 0..13 {
            assert_eq!(colors3[i], Some(CellColor::rgb(0, 255, 0)));
        }
        for i in 13..16 {
            assert_eq!(colors3[i], None, "Reticências terminais não devem ser coloridas");
        }

        // 4. IP com dois-pontos final ("192.168.56.18:")
        let text4 = "192.168.56.18:";
        let cells4: Vec<Cell> = text4.chars().map(|c| Cell {
            ch: c,
            fg: CellColor::WHITE,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }).collect();
        let mut colors4 = vec![None; text4.len()];
        compute_line_highlights(&cells4, text4.len(), &config, &[], &cached_ip, &mut row_str, &mut row_str_lower, &mut colors4);
        for i in 0..13 {
            assert_eq!(colors4[i], Some(CellColor::rgb(0, 255, 0)));
        }
        assert_eq!(colors4[13], None, "Dois-pontos terminal não deve ser colorido");

        // 5. String inválida como "192.168.56.18.1" (5 octetos) não deve colorir
        let text5 = "192.168.56.18.1";
        let cells5: Vec<Cell> = text5.chars().map(|c| Cell {
            ch: c,
            fg: CellColor::WHITE,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }).collect();
        let mut colors5 = vec![None; text5.len()];
        compute_line_highlights(&cells5, text5.len(), &config, &[], &cached_ip, &mut row_str, &mut row_str_lower, &mut colors5);
        assert!(colors5.iter().all(|c| c.is_none()), "5 octetos não devem ser coloridos como IP");
    }
}


