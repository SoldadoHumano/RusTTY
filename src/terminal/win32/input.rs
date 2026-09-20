//! Processador de entradas (teclado, mouse e clipboard) em Win32 API puro.
//!
//! Converte eventos Win32 nativos para sequências VT100/xterm e gerencia seleção e clipboard.

use std::ptr;
use std::time::Instant;
use windows::Win32::Foundation::{HWND, HGLOBAL};
use windows::Win32::System::DataExchange::{
    OpenClipboard, CloseClipboard, EmptyClipboard, GetClipboardData, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_SHIFT, VK_MENU,
    VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT,
    VK_HOME, VK_END, VK_PRIOR, VK_NEXT,
    VK_INSERT, VK_DELETE, VK_BACK, VK_ESCAPE, VK_TAB, VK_RETURN,
    VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6,
    VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
};

const CF_UNICODETEXT: u32 = 13;

use crate::terminal::TerminalGrid;

/// Estado da seleção e interação do mouse no terminal Win32.
#[derive(Default)]
pub struct InputState {
    pub dragging: bool,
    pub has_dragged: bool,
    pub start_pos: Option<(usize, usize)>,
    pub last_click_time: Option<Instant>,
    pub click_count: u8,
    pub sel_anchor: Option<(usize, usize)>,
    pub sel_cursor: Option<(usize, usize)>,
}

impl InputState {
    pub fn has_selection(&self) -> bool {
        match (self.sel_anchor, self.sel_cursor) {
            (Some(a), Some(b)) => a != b,
            _ => false,
        }
    }

    pub fn clear_selection(&mut self) {
        self.sel_anchor = None;
        self.sel_cursor = None;
    }
}

/// Verifica se uma tecla modificadora está atualmente pressionada.
#[inline]
pub fn is_key_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
}

#[inline]
pub fn is_ctrl_down() -> bool {
    is_key_down(VK_CONTROL)
}

#[inline]
pub fn is_shift_down() -> bool {
    is_key_down(VK_SHIFT)
}

#[inline]
#[allow(dead_code)]
pub fn is_alt_down() -> bool {
    is_key_down(VK_MENU)
}

// ─── Clipboard Win32 Nativo ──────────────────────────────────────────────────

/// Lê texto UTF-8 da Área de Transferência nativa do Windows.
pub fn get_clipboard_text(hwnd: HWND) -> Option<String> {
    unsafe {
        if OpenClipboard(hwnd).is_err() {
            return None;
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_err() {
            let _ = CloseClipboard();
            return None;
        }
        let handle = handle.unwrap();

        let hglobal = HGLOBAL(handle.0);
        let ptr = GlobalLock(hglobal) as *const u16;
        if ptr.is_null() {
            let _ = CloseClipboard();
            return None;
        }

        // Calcula tamanho da string terminada em nulo
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(ptr, len);
        let text = String::from_utf16_lossy(slice);

        let _ = GlobalUnlock(hglobal);
        let _ = CloseClipboard();

        Some(text)
    }
}

/// Define texto UTF-8 na Área de Transferência nativa do Windows.
pub fn set_clipboard_text(hwnd: HWND, text: &str) -> bool {
    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes_len = utf16.len() * std::mem::size_of::<u16>();

    unsafe {
        let hglobal = match GlobalAlloc(GMEM_MOVEABLE, bytes_len) {
            Ok(h) => h,
            Err(_) => return false,
        };

        let ptr = GlobalLock(hglobal) as *mut u16;
        if ptr.is_null() {
            return false;
        }

        ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
        let _ = GlobalUnlock(hglobal);

        if OpenClipboard(hwnd).is_err() {
            return false;
        }

        let _ = EmptyClipboard();
        let set_res = SetClipboardData(CF_UNICODETEXT, windows::Win32::Foundation::HANDLE(hglobal.0));
        let _ = CloseClipboard();

        set_res.is_ok()
    }
}

// ─── Tratamento de Teclado ───────────────────────────────────────────────────

/// Mapeia tecla virtual (WM_KEYDOWN) para sequência de escape VT100/xterm, se aplicável.
pub fn keydown_to_escape_sequence(vk: VIRTUAL_KEY) -> Option<&'static [u8]> {
    match vk {
        VK_UP => Some(b"\x1b[A"),
        VK_DOWN => Some(b"\x1b[B"),
        VK_RIGHT => Some(b"\x1b[C"),
        VK_LEFT => Some(b"\x1b[D"),
        VK_HOME => Some(b"\x1b[H"),
        VK_END => Some(b"\x1b[F"),
        VK_PRIOR => Some(b"\x1b[5~"), // Page Up
        VK_NEXT => Some(b"\x1b[6~"),  // Page Down
        VK_INSERT => Some(b"\x1b[2~"),
        VK_DELETE => Some(b"\x1b[3~"),
        VK_BACK => Some(b"\x7f"),     // Backspace padrão VT100/xterm
        VK_ESCAPE => Some(b"\x1b"),
        VK_TAB => {
            if is_shift_down() {
                Some(b"\x1b[Z") // Backtab
            } else {
                Some(b"\t")
            }
        }
        VK_RETURN => Some(b"\r"),
        VK_F1 => Some(b"\x1bOP"),
        VK_F2 => Some(b"\x1bOQ"),
        VK_F3 => Some(b"\x1bOR"),
        VK_F4 => Some(b"\x1bOS"),
        VK_F5 => Some(b"\x1b[15~"),
        VK_F6 => Some(b"\x1b[17~"),
        VK_F7 => Some(b"\x1b[18~"),
        VK_F8 => Some(b"\x1b[19~"),
        VK_F9 => Some(b"\x1b[20~"),
        VK_F10 => Some(b"\x1b[21~"),
        VK_F11 => Some(b"\x1b[23~"),
        VK_F12 => Some(b"\x1b[24~"),
        _ => None,
    }
}

// ─── Seleção de Palavras e Linhas ─────────────────────────────────────────────

pub fn select_word_at(grid: &TerminalGrid, row: usize, col: usize, state: &mut InputState) {
    if let Some(line) = grid.get_line(row) {
        let is_word_char = |ch: char| {
            ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == ':'
        };

        let mut start_c = col;
        let mut end_c = col;

        if col < line.len() && is_word_char(line[col].ch) {
            while start_c > 0 && is_word_char(line[start_c - 1].ch) {
                start_c -= 1;
            }
            while end_c + 1 < line.len() && is_word_char(line[end_c + 1].ch) {
                end_c += 1;
            }
        }

        state.sel_anchor = Some((row, start_c));
        state.sel_cursor = Some((row, end_c));
    }
}

pub fn select_line_at(grid: &TerminalGrid, row: usize, state: &mut InputState) {
    let cols = grid.cols;
    state.sel_anchor = Some((row, 0));
    state.sel_cursor = Some((row, cols.saturating_sub(1)));
}

// ─── Formatação de Mouse SGR ─────────────────────────────────────────────────

/// Formata sequência de evento de mouse no padrão SGR 1006.
#[inline]
pub fn format_sgr_mouse(code: u32, col: usize, row: usize, release: bool) -> Vec<u8> {
    let suffix = if release { 'm' } else { 'M' };
    format!("\x1b[<{};{};{}{}", code, col + 1, row + 1, suffix).into_bytes()
}
