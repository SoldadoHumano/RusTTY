//! Gerenciamento de Janela Win32 Nativa e Loop de Mensagens para o Terminal RusTTY.
//!
//! Cria a janela do terminal com barra de título em modo escuro nativo do Windows (DWM),
//! despacha mensagens de I/O de forma assíncrona para o SSH e renderiza com Direct2D + DirectWrite.

use std::collections::VecDeque;
use std::ptr;
use std::sync::{Arc, Mutex};
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, RECT, BOOL};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, UpdateWindow, PAINTSTRUCT, HBRUSH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{SetCapture, ReleaseCapture, VIRTUAL_KEY};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
    GetMessageW, KillTimer, PostQuitMessage, RegisterClassExW, SetTimer, SetWindowLongPtrW,
    GetWindowLongPtrW, SetWindowTextW, TranslateMessage, LoadCursorW, ShowWindow,
    CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_IBEAM, SW_SHOW,
    MSG, WINDOW_EX_STYLE, WNDCLASSEXW, WS_OVERLAPPEDWINDOW, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
    WM_APP, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_ENTERSIZEMOVE, WM_ERASEBKGND, WM_EXITSIZEMOVE,
    WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_PAINT, WM_RBUTTONDOWN, WM_SETFOCUS, WM_SIZE, WM_TIMER,
    LoadIconW, SendMessageW, WM_SETICON, ICON_BIG, ICON_SMALL, HICON,
    LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE, LR_SHARED, LR_LOADFROMFILE,
    GetSystemMetrics, SM_CXSMICON, SM_CYSMICON,
};

const VK_C: VIRTUAL_KEY = VIRTUAL_KEY(0x43);
const VK_V: VIRTUAL_KEY = VIRTUAL_KEY(0x56);

const TIMER_CURSOR_BLINK: usize = 1;
const TIMER_RESIZE_DEBOUNCE: usize = 2;
const RESIZE_DEBOUNCE_MS: u32 = 40;

use crate::config::client::ClientConfig;
use crate::net::{NetworkCommand, NetworkEvent};
use crate::terminal::{TerminalState, MouseMode};
use crate::terminal::win32::input::{
    InputState, is_ctrl_down, is_shift_down, get_clipboard_text, set_clipboard_text,
    keydown_to_escape_sequence, select_word_at, select_line_at, format_sgr_mouse,
};
use crate::terminal::win32::renderer::{TerminalRenderer, MARGIN_X, MARGIN_Y};

/// Mensagem Win32 customizada para notificar eventos de rede do SSH de forma assíncrona.
pub const WM_APP_NETWORK_EVENT: u32 = WM_APP + 1;

/// Estado do terminal associado ao ciclo de vida da janela Win32.
pub struct WindowState {
    pub hwnd: HWND,
    pub terminal: TerminalState,
    pub renderer: TerminalRenderer,
    pub input: InputState,
    pub scroll_offset: usize,
    pub scroll_lines: usize,
    pub cursor_visible: bool,
    pub cmd_sender: tokio::sync::mpsc::Sender<NetworkCommand>,
    pub network_events: Arc<Mutex<VecDeque<NetworkEvent>>>,
    pub config: ClientConfig,
    pub host_name: String,
    pub status_tag: &'static str,
    pub rt_handle: tokio::runtime::Handle,
    pub pending_resize: Option<(u16, u16)>,
    pub last_sent_pty_size: (u16, u16),
}

impl WindowState {
    pub fn update_title(&self) {
        let title_text = format!("RusTTY — {} [{}]", self.host_name, self.status_tag);
        let htitle = HSTRING::from(title_text);
        unsafe {
            let _ = SetWindowTextW(self.hwnd, &htitle);
        }
    }

    pub fn send_bytes(&self, bytes: Vec<u8>) {
        let sender = self.cmd_sender.clone();
        self.rt_handle.spawn(async move {
            let _ = sender.send(NetworkCommand::SendData(bytes)).await;
        });
    }

    pub fn send_resize(&self, cols: u16, rows: u16) {
        let sender = self.cmd_sender.clone();
        self.rt_handle.spawn(async move {
            let _ = sender.send(NetworkCommand::ResizePty { cols, rows }).await;
        });
    }

    pub fn flush_pending_resize(&mut self) {
        if let Some((cols, rows)) = self.pending_resize.take() {
            if (cols, rows) != self.last_sent_pty_size {
                self.last_sent_pty_size = (cols, rows);
                crate::debug_log!(
                    "DEBUG",
                    "Win32 Terminal: Enviando ResizePty consolidado após debounce: {}x{}",
                    cols, rows
                );
                self.send_resize(cols, rows);
            }
        }
    }

    pub fn disconnect_and_close(&self) {
        let sender = self.cmd_sender.clone();
        self.rt_handle.spawn(async move {
            let _ = sender.send(NetworkCommand::Disconnect).await;
        });
    }
}

/// Registra a classe de janela e cria a janela principal do terminal Win32.
pub fn create_terminal_window(
    host_name: String,
    config: ClientConfig,
    cmd_sender: tokio::sync::mpsc::Sender<NetworkCommand>,
    network_events: Arc<Mutex<VecDeque<NetworkEvent>>>,
    rt_handle: tokio::runtime::Handle,
) -> windows::core::Result<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("RusTTY_Win32_Terminal");

        let sm_cx = GetSystemMetrics(SM_CXSMICON);
        let sm_cy = GetSystemMetrics(SM_CYSMICON);
        let res_id = PCWSTR(1 as _);

        let big_res = LoadImageW(
            instance,
            res_id,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        ).ok().map(|h| HICON(h.0)).or_else(|| {
            LoadIconW(instance, res_id).ok()
        });

        let sm_res = LoadImageW(
            instance,
            res_id,
            IMAGE_ICON,
            sm_cx,
            sm_cy,
            LR_SHARED,
        ).ok().map(|h| HICON(h.0));

        let (hicon_big, hicon_sm) = if big_res.is_none() || big_res.as_ref().map_or(true, |i| i.is_invalid()) {
            let ico_path_buf = if std::path::Path::new("assets/images/iconv2.ico").exists() {
                Some(std::path::PathBuf::from("assets/images/iconv2.ico"))
            } else if let Ok(mut exe) = std::env::current_exe() {
                exe.pop();
                let candidate = exe.join("assets/images/iconv2.ico");
                if candidate.exists() {
                    Some(candidate)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(path) = ico_path_buf {
                let path_hstring = HSTRING::from(path.to_string_lossy().as_ref());
                let ico_file = PCWSTR(path_hstring.as_ptr());
                let f_big = LoadImageW(
                    None,
                    ico_file,
                    IMAGE_ICON,
                    0,
                    0,
                    LR_LOADFROMFILE | LR_DEFAULTSIZE,
                ).ok().map(|h| HICON(h.0));

                let f_sm = LoadImageW(
                    None,
                    ico_file,
                    IMAGE_ICON,
                    sm_cx,
                    sm_cy,
                    LR_LOADFROMFILE,
                ).ok().map(|h| HICON(h.0));

                let big = f_big.unwrap_or_else(|| HICON(ptr::null_mut()));
                let sm = f_sm.unwrap_or(big);
                (big, sm)
            } else {
                (HICON(ptr::null_mut()), HICON(ptr::null_mut()))
            }
        } else {
            let big = big_res.unwrap_or_else(|| HICON(ptr::null_mut()));
            let sm = sm_res.unwrap_or(big);
            (big, sm)
        };

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: hicon_big,
            hCursor: LoadCursorW(None, IDC_IBEAM)?,
            hbrBackground: HBRUSH(ptr::null_mut()), // NULL brush para evitar flicker
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: hicon_sm,
        };

        let _ = RegisterClassExW(&wc);

        let initial_title = format!("RusTTY — {} [⟳]", host_name);
        let htitle = HSTRING::from(initial_title);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            &htitle,
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            960,
            540,
            None,
            None,
            instance,
            None,
        )?;

        // Aplica os ícones explicitamente à janela (barra de título e barra de tarefas / Alt+Tab)
        if !hicon_big.is_invalid() {
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_BIG as _),
                LPARAM(hicon_big.0 as _),
            );
        }
        if !hicon_sm.is_invalid() {
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_SMALL as _),
                LPARAM(hicon_sm.0 as _),
            );
        }

        // Ativa modo escuro nativo do Windows na barra de título
        let dark_mode: BOOL = true.into();
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const _ as *const _,
            std::mem::size_of::<BOOL>() as u32,
        );

        // Cria o motor gráfico Direct2D + DirectWrite
        let font_size = config.terminal_font_size as f32;
        let renderer = TerminalRenderer::new(hwnd, font_size)?;

        let cell_w = renderer.cell_w;
        let cell_h = renderer.cell_h;

        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let win_w = (rc.right - rc.left).max(100) as f32;
        let win_h = (rc.bottom - rc.top).max(100) as f32;

        let usable_w = (win_w - 2.0 * MARGIN_X).max(cell_w);
        let usable_h = (win_h - 2.0 * MARGIN_Y).max(cell_h);
        let init_cols = (usable_w / cell_w).floor() as usize;
        let init_rows = (usable_h / cell_h).floor() as usize;
        let pty_cols = init_cols.max(10);
        let pty_rows = init_rows.max(5);

        let mut terminal = TerminalState::new(pty_rows, pty_cols, config.max_scrollback_lines);
        terminal.process_bytes(b"Conectando...\r\n");

        let state = Box::new(WindowState {
            hwnd,
            terminal,
            renderer,
            input: InputState::default(),
            scroll_offset: 0,
            scroll_lines: config.scroll_lines,
            cursor_visible: true,
            cmd_sender,
            network_events,
            config,
            host_name,
            status_tag: "⟳",
            rt_handle: rt_handle.clone(),
            pending_resize: None,
            last_sent_pty_size: (pty_cols as u16, pty_rows as u16),
        });

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        // Timer para blink do cursor (600ms)
        let _ = SetTimer(hwnd, TIMER_CURSOR_BLINK, 600, None);

        // Notifica o PTY inicial
        let sender = state_from_hwnd(hwnd).cmd_sender.clone();
        rt_handle.spawn(async move {
            let _ = sender.send(NetworkCommand::ResizePty {
                cols: pty_cols as u16,
                rows: pty_rows as u16,
            }).await;
        });

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        Ok(hwnd)
    }
}

/// Executa o loop clássico de mensagens Win32.
pub fn run_message_loop() {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[inline]
unsafe fn state_from_hwnd<'a>(hwnd: HWND) -> &'a mut WindowState {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
    &mut *ptr
}

/// Procedimento de Janela (Window Procedure) principal.
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let raw_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if raw_ptr == 0 {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let state = &mut *(raw_ptr as *mut WindowState);

    match msg {
        WM_APP_NETWORK_EVENT => {
            // Descarrega todos os eventos de rede pendentes na fila
            let mut events = Vec::new();
            if let Ok(mut queue) = state.network_events.lock() {
                while let Some(ev) = queue.pop_front() {
                    events.push(ev);
                }
            }

            let mut needs_repaint = false;
            for event in events {
                match event {
                    NetworkEvent::Connected(msg) => {
                        crate::debug_log!("INFO", "Win32 Terminal: Conectado com sucesso: {}", msg);
                        state.status_tag = "●";
                        state.terminal.reset();
                        state.update_title();
                        needs_repaint = true;
                    }
                    NetworkEvent::DataReceived(bytes) => {
                        state.terminal.process_bytes(&bytes);
                        state.scroll_offset = 0;
                        needs_repaint = true;
                    }
                    NetworkEvent::Disconnected(msg) => {
                        crate::debug_log!("WARN", "Win32 Terminal: Desconectado: {}", msg);
                        state.status_tag = "○";
                        let note = format!("\r\n\r\n── {} ──\r\n", msg);
                        state.terminal.process_bytes(note.as_bytes());
                        state.update_title();
                        needs_repaint = true;
                    }
                    NetworkEvent::Error(msg) => {
                        crate::debug_log!("ERROR", "Win32 Terminal: Erro: {}", msg);
                        state.status_tag = "✕";
                        let err = format!("\r\n\x1b[31mERRO: {}\x1b[0m\r\n", msg);
                        state.terminal.process_bytes(err.as_bytes());
                        state.update_title();
                        needs_repaint = true;
                    }
                }
            }

            if needs_repaint {
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _hdc = BeginPaint(hwnd, &mut ps);

            state.renderer.render(
                &state.terminal.grid,
                &state.input,
                state.scroll_offset,
                state.cursor_visible,
                &state.config,
            );

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_SIZE => {
            let width = (lparam.0 & 0xffff) as u32;
            let height = ((lparam.0 >> 16) & 0xffff) as u32;

            state.renderer.resize(width, height);

            let cell_w = state.renderer.cell_w;
            let cell_h = state.renderer.cell_h;
            let usable_w = (width as f32 - 2.0 * MARGIN_X).max(cell_w);
            let usable_h = (height as f32 - 2.0 * MARGIN_Y).max(cell_h);
            let new_cols = (usable_w / cell_w).floor() as usize;
            let new_rows = (usable_h / cell_h).floor() as usize;

            if new_cols >= 10 && new_rows >= 5 {
                if new_cols != state.terminal.grid.cols || new_rows != state.terminal.grid.rows {
                    state.terminal.resize(new_rows, new_cols);
                    state.scroll_offset = state.scroll_offset.min(state.terminal.grid.scrollback.len());
                    let _ = InvalidateRect(hwnd, None, false);
                }

                let pty_cols = new_cols as u16;
                let pty_rows = new_rows as u16;
                if (pty_cols, pty_rows) != state.last_sent_pty_size {
                    state.pending_resize = Some((pty_cols, pty_rows));
                    // Reinicia timer de debounce no Win32 para coalescer múltiplos eventos de resize rápidos
                    let _ = SetTimer(hwnd, TIMER_RESIZE_DEBOUNCE, RESIZE_DEBOUNCE_MS, None);
                } else {
                    state.pending_resize = None;
                    let _ = KillTimer(hwnd, TIMER_RESIZE_DEBOUNCE);
                }
            }
            LRESULT(0)
        }

        WM_ENTERSIZEMOVE => LRESULT(0),

        WM_EXITSIZEMOVE => {
            let _ = KillTimer(hwnd, TIMER_RESIZE_DEBOUNCE);
            state.flush_pending_resize();
            LRESULT(0)
        }

        WM_KEYDOWN => {
            state.cursor_visible = true;
            let vk = VIRTUAL_KEY(wparam.0 as u16);

            // Atalhos de Área de Transferência (Ctrl+C, Ctrl+V, Ctrl+Shift+C, Ctrl+Shift+V)
            if is_ctrl_down() {
                if vk == VK_C {
                    if state.input.has_selection() {
                        let text = state.terminal.grid.selected_text(
                            state.input.sel_anchor.unwrap(),
                            state.input.sel_cursor.unwrap(),
                        );
                        state.input.clear_selection();
                        if !text.is_empty() {
                            set_clipboard_text(hwnd, &text);
                        }
                        let _ = InvalidateRect(hwnd, None, false);
                        return LRESULT(0); // Não envia SIGINT (\x03) quando há seleção
                    } else if is_shift_down() {
                        return LRESULT(0);
                    }
                    // Sem seleção e sem Shift: deixa cair para enviar \x03
                } else if vk == VK_V {
                    paste_clipboard_to_terminal(state);
                    let _ = InvalidateRect(hwnd, None, false);
                    return LRESULT(0);
                }
            }

            // Teclas de navegação / nomeadas
            if let Some(seq) = keydown_to_escape_sequence(vk) {
                state.input.clear_selection();
                state.scroll_offset = 0;
                state.send_bytes(seq.to_vec());
                let _ = InvalidateRect(hwnd, None, false);
                return LRESULT(0);
            }

            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_CHAR => {
            state.cursor_visible = true;
            let ch = wparam.0 as u16;

            // Ignora se for tecla de controle já tratada no WM_KEYDOWN
            // (Backspace=8, Tab=9, Enter=13, Escape=27)
            if ch == 8 || ch == 9 || ch == 13 || ch == 27 {
                return LRESULT(0);
            }

            if let Some(c) = char::from_u32(ch as u32) {
                state.input.clear_selection();
                state.scroll_offset = 0;
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                state.send_bytes(s.as_bytes().to_vec());
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        WM_ERASEBKGND => LRESULT(1),

        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
            let x = (lparam.0 & 0xffff) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xffff) as i16 as f32;

            let col = (((x - MARGIN_X).max(0.0) / state.renderer.cell_w) as usize)
                .min(state.terminal.grid.cols.saturating_sub(1));
            let row = (((y - MARGIN_Y).max(0.0) / state.renderer.cell_h) as usize)
                .min(state.terminal.grid.rows.saturating_sub(1));

            // Protocolo de mouse do terminal (ex: htop, vim)
            if state.terminal.grid.mouse_mode != MouseMode::None && !is_shift_down() {
                state.input.dragging = true;
                let bytes = format_sgr_mouse(0, col, row, false);
                state.send_bytes(bytes);
                return LRESULT(0);
            }

            let abs_top = if state.terminal.grid.is_alt_screen {
                0
            } else {
                state.terminal.grid.scrollback.len().saturating_sub(state.scroll_offset)
            };
            let abs_row = abs_top + row;

            state.input.dragging = true;
            state.input.has_dragged = false;
            state.input.start_pos = Some((abs_row, col));

            let now = std::time::Instant::now();
            if let Some(last) = state.input.last_click_time {
                if now.duration_since(last).as_millis() < 500 {
                    state.input.click_count += 1;
                } else {
                    state.input.click_count = 1;
                }
            } else {
                state.input.click_count = 1;
            }
            state.input.last_click_time = Some(now);

            match state.input.click_count {
                1 => {
                    // Reseta seleção prévia e define âncora do novo clique
                    state.input.sel_anchor = Some((abs_row, col));
                    state.input.sel_cursor = Some((abs_row, col));
                }
                2 => select_word_at(&state.terminal.grid, abs_row, col, &mut state.input),
                n if n >= 3 => {
                    state.input.click_count = 3;
                    select_line_at(&state.terminal.grid, abs_row, &mut state.input);
                }
                _ => {}
            }

            let _ = SetCapture(hwnd);
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xffff) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xffff) as i16 as f32;

            let col = (((x - MARGIN_X).max(0.0) / state.renderer.cell_w) as usize)
                .min(state.terminal.grid.cols.saturating_sub(1));
            let row = (((y - MARGIN_Y).max(0.0) / state.renderer.cell_h) as usize)
                .min(state.terminal.grid.rows.saturating_sub(1));

            if state.terminal.grid.mouse_mode != MouseMode::None && !is_shift_down() {
                if state.terminal.grid.mouse_mode == MouseMode::AnyEvent
                    || (state.terminal.grid.mouse_mode == MouseMode::ButtonEvent && state.input.dragging)
                {
                    let code = if state.input.dragging { 32 } else { 35 };
                    let bytes = format_sgr_mouse(code, col, row, false);
                    state.send_bytes(bytes);
                }
                return LRESULT(0);
            }

            if state.input.dragging {
                state.input.has_dragged = true;
                let abs_top = if state.terminal.grid.is_alt_screen {
                    0
                } else {
                    state.terminal.grid.scrollback.len().saturating_sub(state.scroll_offset)
                };
                let abs_row = abs_top + row;

                if state.input.sel_anchor.is_none() {
                    state.input.sel_anchor = state.input.start_pos;
                }
                state.input.sel_cursor = Some((abs_row, col));
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xffff) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xffff) as i16 as f32;

            let col = (((x - MARGIN_X).max(0.0) / state.renderer.cell_w) as usize)
                .min(state.terminal.grid.cols.saturating_sub(1));
            let row = (((y - MARGIN_Y).max(0.0) / state.renderer.cell_h) as usize)
                .min(state.terminal.grid.rows.saturating_sub(1));

            if state.terminal.grid.mouse_mode != MouseMode::None && !is_shift_down() {
                state.input.dragging = false;
                let bytes = format_sgr_mouse(0, col, row, true);
                state.send_bytes(bytes);
                return LRESULT(0);
            }

            state.input.dragging = false;
            let _ = ReleaseCapture();

            if !state.input.has_dragged && state.input.click_count <= 1 {
                // Clique simples sem arrastar: limpa seleção
                state.input.clear_selection();
            }

            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_RBUTTONDOWN => {
            // Botão direito: copia se tiver seleção, senão cola
            if state.input.has_selection() {
                let text = state.terminal.grid.selected_text(
                    state.input.sel_anchor.unwrap(),
                    state.input.sel_cursor.unwrap(),
                );
                state.input.clear_selection();
                if !text.is_empty() {
                    set_clipboard_text(hwnd, &text);
                }
            } else {
                paste_clipboard_to_terminal(state);
            }
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) as i16) as f32 / 120.0;
            if state.terminal.grid.is_alt_screen {
                let count = (delta.abs() as usize).max(1);
                let seq = if delta > 0.0 { b"\x1b[A" } else { b"\x1b[B" };
                let mut bytes = Vec::new();
                for _ in 0..count {
                    bytes.extend_from_slice(seq);
                }
                state.send_bytes(bytes);
            } else {
                let max_offset = state.terminal.grid.scrollback.len();
                let jump = (delta.abs() as usize).max(1) * state.scroll_lines;
                if delta > 0.0 {
                    state.scroll_offset = state.scroll_offset.saturating_add(jump).min(max_offset);
                } else {
                    state.scroll_offset = state.scroll_offset.saturating_sub(jump);
                }
            }
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_TIMER => {
            match wparam.0 {
                TIMER_CURSOR_BLINK => {
                    state.cursor_visible = !state.cursor_visible;
                    let _ = InvalidateRect(hwnd, None, false);
                }
                TIMER_RESIZE_DEBOUNCE => {
                    let _ = KillTimer(hwnd, TIMER_RESIZE_DEBOUNCE);
                    state.flush_pending_resize();
                }
                _ => {}
            }
            LRESULT(0)
        }

        WM_SETFOCUS => {
            state.renderer.has_focus = true;
            state.cursor_visible = true;
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_KILLFOCUS => {
            state.renderer.has_focus = false;
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_CLOSE => {
            let _ = KillTimer(hwnd, TIMER_CURSOR_BLINK);
            let _ = KillTimer(hwnd, TIMER_RESIZE_DEBOUNCE);
            state.disconnect_and_close();
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }

        WM_DESTROY => {
            let raw_ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            if raw_ptr != 0 {
                let _ = Box::from_raw(raw_ptr as *mut WindowState);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn paste_clipboard_to_terminal(state: &mut WindowState) {
    if let Some(text) = get_clipboard_text(state.hwnd) {
        let normalized = text.replace("\r\n", "\r").replace('\n', "\r");
        let mut bytes = Vec::new();
        if state.terminal.grid.bracketed_paste {
            bytes.extend_from_slice(b"\x1b[200~");
            bytes.extend_from_slice(normalized.as_bytes());
            bytes.extend_from_slice(b"\x1b[201~");
        } else {
            bytes.extend_from_slice(normalized.as_bytes());
        }
        state.send_bytes(bytes);
    }
}
