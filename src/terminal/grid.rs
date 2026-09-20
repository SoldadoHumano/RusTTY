//! Grade de terminal VTE para o RusTTY.
//!
//! Implementa um emulador de terminal completo compatível com xterm-256color:
//!   - VTE state machine via crate `vte` (CSI, OSC, ESC dispatch)
//!   - Célula grid com atributos SGR (cor, bold, underline, reverse video)
//!   - Paleta de 256 cores + RGB true color
//!   - Scroll automático e scrollback de até 1.000 linhas
//!   - Cursor, wrap automático, clear screen/line
//!
//! # Uso
//! ```rust
//! let mut state = TerminalState::new(24, 80);
//! state.process_bytes(b"Hello\x1b[31m World\x1b[0m\r\n");
//! ```

use vte::{Params, Perform};



// ─── Tipos de Cor ─────────────────────────────────────────────────────────────

/// Cor RGB de uma célula de terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl CellColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }

    // Paleta ANSI 16 cores (Windows Terminal / xterm padrão)
    pub const BLACK:          Self = Self::rgb(12,  12,  12);
    pub const DARK_RED:       Self = Self::rgb(197, 15,  31);
    pub const DARK_GREEN:     Self = Self::rgb(19,  161, 14);
    pub const DARK_YELLOW:    Self = Self::rgb(193, 156, 0);
    pub const DARK_BLUE:      Self = Self::rgb(0,   55,  218);
    pub const DARK_MAGENTA:   Self = Self::rgb(136, 23,  152);
    pub const DARK_CYAN:      Self = Self::rgb(58,  150, 221);
    pub const GRAY:           Self = Self::rgb(204, 204, 204);
    pub const DARK_GRAY:      Self = Self::rgb(118, 118, 118);
    pub const BRIGHT_RED:     Self = Self::rgb(231, 72,  86);
    pub const BRIGHT_GREEN:   Self = Self::rgb(22,  198, 12);
    pub const BRIGHT_YELLOW:  Self = Self::rgb(249, 241, 165);
    pub const BRIGHT_BLUE:    Self = Self::rgb(59,  120, 255);
    pub const BRIGHT_MAGENTA: Self = Self::rgb(180, 0,   158);
    pub const BRIGHT_CYAN:    Self = Self::rgb(97,  214, 214);
    pub const WHITE:          Self = Self::rgb(242, 242, 242);

    /// Cor de fundo padrão do terminal.
    pub const DEFAULT_BG: Self = Self::rgb(12, 12, 12);
    /// Cor de foreground padrão.
    pub const DEFAULT_FG: Self = Self::WHITE;
}

// ─── Atributos SGR ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAttrs {
    pub fg:        CellColor,
    pub bg:        CellColor,
    pub bold:      bool,
    pub underline: bool,
    pub reverse:   bool,
}

impl Default for CellAttrs {
    fn default() -> Self {
        Self {
            fg: CellColor::DEFAULT_FG,
            bg: CellColor::DEFAULT_BG,
            bold: false,
            underline: false,
            reverse: false,
        }
    }
}

// ─── Célula de Terminal ───────────────────────────────────────────────────────

/// Uma célula da grade do terminal com seu caractere e atributos visuais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch:        char,
    pub fg:        CellColor,
    pub bg:        CellColor,
    pub bold:      bool,
    pub underline: bool,
    pub reverse:   bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch:        ' ',
            fg:        CellColor::DEFAULT_FG,
            bg:        CellColor::DEFAULT_BG,
            bold:      false,
            underline: false,
            reverse:   false,
        }
    }
}

impl Cell {
    /// Retorna a cor efetiva de foreground (considerando reverse video).
    pub fn effective_fg(&self) -> CellColor {
        if self.reverse { self.bg } else { self.fg }
    }

    /// Retorna a cor efetiva de background (considerando reverse video).
    pub fn effective_bg(&self) -> CellColor {
        if self.reverse { self.fg } else { self.bg }
    }

    /// Célula vazia padrão.
    pub fn is_default_empty(&self) -> bool {
        self.ch == ' ' && self.bg == CellColor::DEFAULT_BG && !self.reverse
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMode {
    None,
    Normal,      // 1000
    ButtonEvent, // 1002
    AnyEvent,    // 1003
}

// ─── Grade do Terminal ────────────────────────────────────────────────────────

/// Estado completo da grade do terminal: células, cursor e atributos SGR.
///
/// Implementa `vte::Perform` para processar bytes SSH brutos.
pub struct TerminalGrid {
    /// Células do ecrã ativo (rows × cols)
    pub cells:        Vec<Vec<Cell>>,
    pub cursor_row:   usize,
    pub cursor_col:   usize,
    pub rows:         usize,
    pub cols:         usize,
    /// Scrollback (linhas que saíram do topo do ecrã)
    pub scrollback:   Vec<Vec<Cell>>,
    current_attrs:    CellAttrs,
    saved_cursor:     (usize, usize),
    saved_attrs:      CellAttrs,
    auto_wrap:        bool,
    /// Pendente de wrap na próxima impressão
    pending_wrap:     bool,
    pub max_scrollback: usize,

    // ── Buffer Alternativo (Vim / Htop / Btop) ──
    pub is_alt_screen: bool,
    pub alt_cells: Vec<Vec<Cell>>,
    saved_primary_cursor: (usize, usize),
    saved_primary_attrs: CellAttrs,

    // ── Mouse & Terminal Modes ──
    pub mouse_mode: MouseMode,
    pub sgr_mouse: bool,
    pub bracketed_paste: bool,
    pub cursor_visible: bool,
}

impl TerminalGrid {
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        Self {
            cells:        vec![vec![Cell::default(); cols]; rows],
            cursor_row:   0,
            cursor_col:   0,
            rows,
            cols,
            scrollback:   Vec::new(),
            current_attrs: CellAttrs::default(),
            saved_cursor: (0, 0),
            saved_attrs:  CellAttrs::default(),
            auto_wrap:    true,
            pending_wrap: false,
            max_scrollback,
            is_alt_screen: false,
            alt_cells: vec![vec![Cell::default(); cols]; rows],
            saved_primary_cursor: (0, 0),
            saved_primary_attrs: CellAttrs::default(),
            mouse_mode: MouseMode::None,
            sgr_mouse: false,
            bracketed_paste: false,
            cursor_visible: true,
        }
    }

    /// Retorna uma referência à linha correspondente ao índice absoluto `abs_row`.
    pub fn get_line(&self, abs_row: usize) -> Option<&Vec<Cell>> {
        if self.is_alt_screen {
            if abs_row < self.rows {
                Some(&self.cells[abs_row])
            } else {
                None
            }
        } else if abs_row < self.scrollback.len() {
            Some(&self.scrollback[abs_row])
        } else {
            let active_row = abs_row - self.scrollback.len();
            if active_row < self.rows {
                Some(&self.cells[active_row])
            } else {
                None
            }
        }
    }

    // ── Operações internas ───────────────────────────────────────────────────

    fn put_char(&mut self, c: char) {
        if self.pending_wrap {
            self.pending_wrap = false;
            self.cursor_col = 0;
            self.lf();
        }

        if self.cursor_col < self.cols {
            let row = self.cursor_row;
            let col = self.cursor_col;
            let a = self.current_attrs;
            self.cells[row][col] = Cell {
                ch:        c,
                fg:        a.fg,
                bg:        a.bg,
                bold:      a.bold,
                underline: a.underline,
                reverse:   a.reverse,
            };
            self.cursor_col += 1;

            // Wrap pendente quando alcança a última coluna
            if self.cursor_col == self.cols && self.auto_wrap {
                self.pending_wrap = true;
                self.cursor_col = self.cols - 1; // fica na última coluna
            }
        }
    }

    /// Line feed: avança uma linha, fazendo scroll se necessário.
    fn lf(&mut self) {
        if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        } else {
            self.scroll_up(1);
        }
    }

    fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            let row = self.cells.remove(0);
            if !self.is_alt_screen && self.max_scrollback > 0 {
                self.scrollback.push(row);
            }
            self.cells.push(vec![Cell::default(); self.cols]);
        }
        if !self.is_alt_screen && self.max_scrollback > 0 && self.scrollback.len() > self.max_scrollback {
            let excess = self.scrollback.len() - self.max_scrollback;
            self.scrollback.drain(0..excess);
        } else if self.max_scrollback == 0 {
            self.scrollback.clear();
        }
    }

    fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            self.cells.pop();
            self.cells.insert(0, vec![Cell::default(); self.cols]);
        }
    }

    fn clear_to_end_of_line(&mut self) {
        let (r, c) = (self.cursor_row, self.cursor_col);
        let max_c = self.cells[r].len().max(self.cols);
        for col in c..max_c {
            if col < self.cells[r].len() {
                self.cells[r][col] = Cell::default();
            }
        }
    }

    fn clear_to_start_of_line(&mut self) {
        let (r, c) = (self.cursor_row, self.cursor_col.min(self.cols - 1));
        for col in 0..=c {
            if col < self.cells[r].len() {
                self.cells[r][col] = Cell::default();
            }
        }
    }

    fn clear_line(&mut self, row: usize) {
        let max_c = self.cells[row].len().max(self.cols);
        for col in 0..max_c {
            if col < self.cells[row].len() {
                self.cells[row][col] = Cell::default();
            }
        }
    }

    fn clear_to_end_of_screen(&mut self) {
        self.clear_to_end_of_line();
        let r = self.cursor_row;
        for row in (r + 1)..self.rows {
            self.clear_line(row);
        }
    }

    fn clear_to_start_of_screen(&mut self) {
        let r = self.cursor_row;
        for row in 0..r {
            self.clear_line(row);
        }
        self.clear_to_start_of_line();
    }

    fn clear_all(&mut self) {
        for row in 0..self.rows {
            self.clear_line(row);
        }
    }

    fn clamp_cursor(&mut self) {
        self.cursor_row = self.cursor_row.min(self.rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(self.cols.saturating_sub(1));
    }

    // ── Buffer Alternativo ───────────────────────────────────────────────────

    pub fn enter_alt_screen(&mut self) {
        if !self.is_alt_screen {
            self.is_alt_screen = true;
            self.saved_primary_cursor = (self.cursor_row, self.cursor_col);
            self.saved_primary_attrs = self.current_attrs;
            if self.alt_cells.len() != self.rows || self.alt_cells.first().map_or(0, |r| r.len()) != self.cols {
                self.alt_cells = vec![vec![Cell::default(); self.cols]; self.rows];
            }
            std::mem::swap(&mut self.cells, &mut self.alt_cells);
            self.clear_all();
            self.cursor_row = 0;
            self.cursor_col = 0;
            crate::debug_log!("INFO", "TerminalGrid: ativando Buffer Alternativo (tela cheia/TUI)");
        }
    }

    pub fn leave_alt_screen(&mut self) {
        if self.is_alt_screen {
            self.is_alt_screen = false;
            std::mem::swap(&mut self.cells, &mut self.alt_cells);
            let (r, c) = self.saved_primary_cursor;
            self.cursor_row = r.min(self.rows.saturating_sub(1));
            self.cursor_col = c.min(self.cols.saturating_sub(1));
            self.current_attrs = self.saved_primary_attrs;
            crate::debug_log!("INFO", "TerminalGrid: desativando Buffer Alternativo (restaurado buffer primário)");
        }
    }

    // ── API Pública ───────────────────────────────────────────────────────────

    /// Redimensiona a grade, preservando o conteúdo visível e scrollback.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        crate::debug_log!("DEBUG", "TerminalGrid::resize: {}x{} -> {}x{} (scrollback: {} linhas)", self.cols, self.rows, cols, rows, self.scrollback.len());
        for row in &mut self.cells {
            if row.len() < cols {
                row.resize(cols, Cell::default());
            }
        }
        for row in &mut self.alt_cells {
            if row.len() < cols {
                row.resize(cols, Cell::default());
            }
        }
        for row in &mut self.scrollback {
            if row.len() < cols {
                row.resize(cols, Cell::default());
            }
        }

        if !self.is_alt_screen {
            if rows < self.rows {
                let last_content_row = self.cells
                    .iter()
                    .rposition(|row| !row.iter().all(|c| c.is_default_empty()));
                let active_bottom = match last_content_row {
                    Some(r) => r.max(self.cursor_row),
                    None => self.cursor_row,
                };

                let needed_rows = active_bottom + 1;
                if needed_rows > rows {
                    let overflow = needed_rows - rows;
                    for _ in 0..overflow {
                        if !self.cells.is_empty() {
                            let row = self.cells.remove(0);
                            if !row.iter().all(|c| c.is_default_empty()) && self.max_scrollback > 0 {
                                self.scrollback.push(row);
                            }
                        }
                    }
                    self.cursor_row = self.cursor_row.saturating_sub(overflow);
                }

                self.cells.truncate(rows);
                while self.cells.len() < rows {
                    self.cells.push(vec![Cell::default(); cols]);
                }
            } else if rows > self.rows {
                let delta = rows - self.rows;

                let last_content_row = self.cells
                    .iter()
                    .rposition(|row| !row.iter().all(|c| c.is_default_empty()));
                let active_bottom = match last_content_row {
                    Some(r) => r.max(self.cursor_row),
                    None => self.cursor_row,
                };

                let is_screen_filled = active_bottom >= self.rows.saturating_sub(1);

                let mut pulled = 0;
                if is_screen_filled {
                    while pulled < delta && !self.scrollback.is_empty() {
                        let row = self.scrollback.pop().unwrap();
                        self.cells.insert(0, row);
                        pulled += 1;
                    }
                    self.cursor_row = (self.cursor_row + pulled).min(rows.saturating_sub(1));
                }

                while self.cells.len() < rows {
                    self.cells.push(vec![Cell::default(); cols]);
                }
            }
        } else {
            self.cells.resize(rows, vec![Cell::default(); cols]);
        }
        self.alt_cells.resize(rows, vec![Cell::default(); cols]);

        if self.max_scrollback > 0 && self.scrollback.len() > self.max_scrollback {
            let excess = self.scrollback.len() - self.max_scrollback;
            self.scrollback.drain(0..excess);
        }

        self.rows = rows;
        self.cols = cols;
        self.clamp_cursor();
    }

    /// Retorna o texto selecionado entre dois pontos da grade.
    ///
    /// Os pontos são (row, col) e a função normaliza a ordem.
    pub fn selected_text(&self, a: (usize, usize), b: (usize, usize)) -> String {
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        let (r1, c1) = start;
        let (r2, c2) = end;

        let mut out = String::new();
        let total_rows = if self.is_alt_screen { self.rows } else { self.scrollback.len() + self.rows };
        let max_r2 = total_rows.saturating_sub(1);
        let actual_r2 = r2.min(max_r2);

        for r in r1..=actual_r2 {
            let from = if r == r1 { c1 } else { 0 };
            let to   = if r == r2 { (c2 + 1).min(self.cols) } else { self.cols };

            let row_slice = if self.is_alt_screen {
                if r < self.rows { &self.cells[r] } else { continue; }
            } else if r < self.scrollback.len() {
                &self.scrollback[r]
            } else {
                let cell_r = r - self.scrollback.len();
                if cell_r < self.rows {
                    &self.cells[cell_r]
                } else {
                    continue;
                }
            };

            let actual_to = to.min(row_slice.len());
            let actual_from = from.min(actual_to);
            let line: String = row_slice[actual_from..actual_to]
                .iter()
                .map(|c| c.ch)
                .collect::<String>()
                .trim_end()
                .to_string();
            out.push_str(&line);
            if r < actual_r2 {
                out.push('\n');
            }
        }
        out
    }

    /// Retorna `true` se a célula (row, col) está dentro da seleção.
    pub fn in_selection(&self, row: usize, col: usize, a: (usize, usize), b: (usize, usize)) -> bool {
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        let (r1, c1) = start;
        let (r2, c2) = end;

        if row < r1 || row > r2 { return false; }
        if row == r1 && col < c1 { return false; }
        if row == r2 && col > c2 { return false; }
        true
    }
}

// ─── VTE Perform ─────────────────────────────────────────────────────────────

impl Perform for TerminalGrid {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        self.pending_wrap = false;
        match byte {
            0x07 => {}  // BEL
            0x08 => {   // Backspace
                if self.cursor_col > 0 { self.cursor_col -= 1; }
            }
            0x09 => {   // HT (Tab)
                let next_tab = (self.cursor_col / 8 + 1) * 8;
                self.cursor_col = next_tab.min(self.cols - 1);
            }
            0x0A | 0x0B | 0x0C => self.lf(), // LF / VT / FF
            0x0D => { self.cursor_col = 0; }  // CR
            0x0E | 0x0F => {}  // SO / SI (charset)
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        // Coleta parâmetros: primeiro valor de cada grupo semicolon-separated
        let ps: Vec<usize> = params
            .iter()
            .map(|p| *p.first().unwrap_or(&0) as usize)
            .collect();

        let p1 = ps.first().copied().unwrap_or(0);
        let p2 = ps.get(1).copied().unwrap_or(0);

        self.pending_wrap = false;

        match action {
            // ── Movimento de cursor ──────────────────────────────────────────
            'A' => { let n = p1.max(1); self.cursor_row = self.cursor_row.saturating_sub(n); }
            'B' => { let n = p1.max(1); self.cursor_row = (self.cursor_row + n).min(self.rows - 1); }
            'C' => { let n = p1.max(1); self.cursor_col = (self.cursor_col + n).min(self.cols - 1); }
            'D' => { let n = p1.max(1); self.cursor_col = self.cursor_col.saturating_sub(n); }
            'E' => { let n = p1.max(1); self.cursor_row = (self.cursor_row + n).min(self.rows - 1); self.cursor_col = 0; }
            'F' => { let n = p1.max(1); self.cursor_row = self.cursor_row.saturating_sub(n); self.cursor_col = 0; }
            'G' => { self.cursor_col = p1.saturating_sub(1).min(self.cols - 1); }
            'H' | 'f' => {
                self.cursor_row = p1.saturating_sub(1).min(self.rows - 1);
                self.cursor_col = p2.saturating_sub(1).min(self.cols - 1);
            }
            'd' => { self.cursor_row = p1.saturating_sub(1).min(self.rows - 1); }

            // ── Erase ────────────────────────────────────────────────────────
            'J' => match p1 {
                0 => self.clear_to_end_of_screen(),
                1 => self.clear_to_start_of_screen(),
                2 | 3 => self.clear_all(),
                _ => {}
            },
            'K' => match p1 {
                0 => self.clear_to_end_of_line(),
                1 => self.clear_to_start_of_line(),
                2 => {
                    let r = self.cursor_row;
                    self.clear_line(r);
                }
                _ => {}
            },
            'X' => { // Erase N characters
                let n = p1.max(1);
                let r = self.cursor_row;
                let c = self.cursor_col;
                for col in c..(c + n).min(self.cols) {
                    self.cells[r][col] = Cell::default();
                }
            }

            // ── Inserir/Deletar linhas e chars ───────────────────────────────
            'L' => { // Insert Lines
                let n = p1.max(1);
                let r = self.cursor_row;
                for _ in 0..n {
                    self.cells.pop();
                    self.cells.insert(r, vec![Cell::default(); self.cols]);
                }
            }
            'M' => { // Delete Lines
                let n = p1.max(1);
                let r = self.cursor_row;
                for _ in 0..n {
                    if r < self.cells.len() {
                        self.cells.remove(r);
                        self.cells.push(vec![Cell::default(); self.cols]);
                    }
                }
            }
            'P' => { // Delete Chars
                let n = p1.max(1);
                let r = self.cursor_row;
                let c = self.cursor_col;
                for _ in 0..n {
                    if c < self.cells[r].len() {
                        self.cells[r].remove(c);
                        self.cells[r].push(Cell::default());
                    }
                }
            }
            '@' => { // Insert Chars
                let n = p1.max(1);
                let r = self.cursor_row;
                let c = self.cursor_col;
                for _ in 0..n {
                    self.cells[r].insert(c, Cell::default());
                    self.cells[r].truncate(self.cols);
                }
            }

            // ── Scroll ───────────────────────────────────────────────────────
            'S' => self.scroll_up(p1.max(1)),
            'T' => self.scroll_down(p1.max(1)),

            // ── SGR ──────────────────────────────────────────────────────────
            'm' => self.apply_sgr(&ps),

            // ── Cursor save/restore ──────────────────────────────────────────
            's' => {
                self.saved_cursor = (self.cursor_row, self.cursor_col);
                self.saved_attrs = self.current_attrs;
            }
            'u' => {
                let (r, c) = self.saved_cursor;
                self.cursor_row = r.min(self.rows - 1);
                self.cursor_col = c.min(self.cols - 1);
                self.current_attrs = self.saved_attrs;
            }

            // ── Set/Reset Mode (SM / RM / DECSET / DECRST) ───────────────────
            'h' => {
                let is_private = _intermediates.contains(&b'?');
                if is_private {
                    for &p in &ps {
                        match p {
                            1049 | 1047 | 47 => self.enter_alt_screen(),
                            1000 => self.mouse_mode = MouseMode::Normal,
                            1002 => self.mouse_mode = MouseMode::ButtonEvent,
                            1003 => self.mouse_mode = MouseMode::AnyEvent,
                            1006 => self.sgr_mouse = true,
                            2004 => self.bracketed_paste = true,
                            25 => self.cursor_visible = true,
                            _ => {}
                        }
                    }
                }
            }
            'l' => {
                let is_private = _intermediates.contains(&b'?');
                if is_private {
                    for &p in &ps {
                        match p {
                            1049 | 1047 | 47 => self.leave_alt_screen(),
                            1000 | 1002 | 1003 => self.mouse_mode = MouseMode::None,
                            1006 => self.sgr_mouse = false,
                            2004 => self.bracketed_paste = false,
                            25 => self.cursor_visible = false,
                            _ => {}
                        }
                    }
                }
            }

            _ => {} // Outros CSI não tratados
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        self.pending_wrap = false;
        match byte {
            b'7' => {
                self.saved_cursor = (self.cursor_row, self.cursor_col);
                self.saved_attrs = self.current_attrs;
            }
            b'8' => {
                let (r, c) = self.saved_cursor;
                self.cursor_row = r.min(self.rows - 1);
                self.cursor_col = c.min(self.cols - 1);
                self.current_attrs = self.saved_attrs;
            }
            b'M' => { // Reverse Index
                if self.cursor_row == 0 {
                    self.scroll_down(1);
                } else {
                    self.cursor_row -= 1;
                }
            }
            b'c' => { // Full reset
                *self = TerminalGrid::new(self.rows, self.cols, self.max_scrollback);
            }
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC: set title, hyperlinks, etc. — ignorado para o MVP
    }
    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
}

// ─── SGR (Select Graphic Rendition) ──────────────────────────────────────────

impl TerminalGrid {
    fn apply_sgr(&mut self, params: &[usize]) {
        if params.is_empty() {
            self.current_attrs = CellAttrs::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0  => { self.current_attrs = CellAttrs::default(); }
                1  => { self.current_attrs.bold = true; }
                4  => { self.current_attrs.underline = true; }
                7  => { self.current_attrs.reverse = true; }
                22 => { self.current_attrs.bold = false; }
                24 => { self.current_attrs.underline = false; }
                27 => { self.current_attrs.reverse = false; }

                // FG ANSI 8 cores
                30 => { self.current_attrs.fg = CellColor::BLACK; }
                31 => { self.current_attrs.fg = CellColor::DARK_RED; }
                32 => { self.current_attrs.fg = CellColor::DARK_GREEN; }
                33 => { self.current_attrs.fg = CellColor::DARK_YELLOW; }
                34 => { self.current_attrs.fg = CellColor::DARK_BLUE; }
                35 => { self.current_attrs.fg = CellColor::DARK_MAGENTA; }
                36 => { self.current_attrs.fg = CellColor::DARK_CYAN; }
                37 => { self.current_attrs.fg = CellColor::GRAY; }
                38 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        self.current_attrs.fg = color256(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        self.current_attrs.fg = CellColor::rgb(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                39 => { self.current_attrs.fg = CellColor::DEFAULT_FG; }

                // BG ANSI 8 cores
                40 => { self.current_attrs.bg = CellColor::BLACK; }
                41 => { self.current_attrs.bg = CellColor::DARK_RED; }
                42 => { self.current_attrs.bg = CellColor::DARK_GREEN; }
                43 => { self.current_attrs.bg = CellColor::DARK_YELLOW; }
                44 => { self.current_attrs.bg = CellColor::DARK_BLUE; }
                45 => { self.current_attrs.bg = CellColor::DARK_MAGENTA; }
                46 => { self.current_attrs.bg = CellColor::DARK_CYAN; }
                47 => { self.current_attrs.bg = CellColor::GRAY; }
                48 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        self.current_attrs.bg = color256(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        self.current_attrs.bg = CellColor::rgb(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                49 => { self.current_attrs.bg = CellColor::DEFAULT_BG; }

                // FG Bright (90–97)
                90 => { self.current_attrs.fg = CellColor::DARK_GRAY; }
                91 => { self.current_attrs.fg = CellColor::BRIGHT_RED; }
                92 => { self.current_attrs.fg = CellColor::BRIGHT_GREEN; }
                93 => { self.current_attrs.fg = CellColor::BRIGHT_YELLOW; }
                94 => { self.current_attrs.fg = CellColor::BRIGHT_BLUE; }
                95 => { self.current_attrs.fg = CellColor::BRIGHT_MAGENTA; }
                96 => { self.current_attrs.fg = CellColor::BRIGHT_CYAN; }
                97 => { self.current_attrs.fg = CellColor::WHITE; }

                // BG Bright (100–107)
                100 => { self.current_attrs.bg = CellColor::DARK_GRAY; }
                101 => { self.current_attrs.bg = CellColor::BRIGHT_RED; }
                102 => { self.current_attrs.bg = CellColor::BRIGHT_GREEN; }
                103 => { self.current_attrs.bg = CellColor::BRIGHT_YELLOW; }
                104 => { self.current_attrs.bg = CellColor::BRIGHT_BLUE; }
                105 => { self.current_attrs.bg = CellColor::BRIGHT_MAGENTA; }
                106 => { self.current_attrs.bg = CellColor::BRIGHT_CYAN; }
                107 => { self.current_attrs.bg = CellColor::WHITE; }

                _ => {}
            }
            i += 1;
        }
    }
}

/// Mapeia índice 256-color para CellColor.
fn color256(index: u8) -> CellColor {
    match index {
        0  => CellColor::BLACK,
        1  => CellColor::DARK_RED,
        2  => CellColor::DARK_GREEN,
        3  => CellColor::DARK_YELLOW,
        4  => CellColor::DARK_BLUE,
        5  => CellColor::DARK_MAGENTA,
        6  => CellColor::DARK_CYAN,
        7  => CellColor::GRAY,
        8  => CellColor::DARK_GRAY,
        9  => CellColor::BRIGHT_RED,
        10 => CellColor::BRIGHT_GREEN,
        11 => CellColor::BRIGHT_YELLOW,
        12 => CellColor::BRIGHT_BLUE,
        13 => CellColor::BRIGHT_MAGENTA,
        14 => CellColor::BRIGHT_CYAN,
        15 => CellColor::WHITE,
        16..=231 => {
            let i = index - 16;
            let r = (i / 36) % 6;
            let g = (i / 6) % 6;
            let b = i % 6;
            let v = |x: u8| if x == 0 { 0 } else { 55 + x * 40 };
            CellColor::rgb(v(r), v(g), v(b))
        }
        232..=255 => {
            let v = 8 + (index - 232) * 10;
            CellColor::rgb(v, v, v)
        }
    }
}

// ─── TerminalState ────────────────────────────────────────────────────────────

/// Wrapper que combina `TerminalGrid` (Perform) com o `vte::Parser`.
///
/// Necessário porque `vte::Parser::advance` precisa de `&mut self` (parser)
/// e `&mut Perform` (grid) simultaneamente — eles devem ser campos separados.
pub struct TerminalState {
    pub grid:  TerminalGrid,
    parser:    vte::Parser,
}

impl TerminalState {
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        Self {
            grid:   TerminalGrid::new(rows, cols, max_scrollback),
            parser: vte::Parser::new(),
        }
    }

    /// Processa bytes SSH brutos pela state machine VTE.
    pub fn process_bytes(&mut self, data: &[u8]) {
        for &byte in data {
            self.parser.advance(&mut self.grid, byte);
        }
    }

    /// Proxy para resize do grid.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.grid.resize(rows, cols);
    }

    #[allow(dead_code)]
    pub fn clear_all(&mut self) {
        self.grid.clear_all();
    }

    /// Reseta completamente o terminal (limpa tela, scrollback e reseta o cursor)
    pub fn reset(&mut self) {
        self.grid.clear_all();
        self.grid.scrollback.clear();
        self.grid.cursor_row = 0;
        self.grid.cursor_col = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_line_text(grid: &TerminalGrid, row: usize) -> String {
        grid.cells
            .get(row)
            .map(|r| r.iter().map(|c| c.ch).collect::<String>().trim_end().to_string())
            .unwrap_or_default()
    }

    #[test]
    fn test_resize_preserves_lines_and_scrollback() {
        let mut state = TerminalState::new(4, 20, 100);
        state.process_bytes(b"line 1\r\nline 2\r\nline 3\r\nline 4");

        // Check line 1 is in line 0
        assert_eq!(get_line_text(&state.grid, 0), "line 1");
        assert_eq!(get_line_text(&state.grid, 3), "line 4");

        // Shrink height from 4 to 2: top lines should enter scrollback, bottom lines remain
        state.resize(2, 20);
        assert_eq!(state.grid.rows, 2);
        assert_eq!(state.grid.scrollback.len(), 2);
        assert_eq!(get_line_text(&state.grid, 0), "line 3");
        assert_eq!(get_line_text(&state.grid, 1), "line 4");

        // Expand back to 4: lines should be pulled back from scrollback
        state.resize(4, 20);
        assert_eq!(state.grid.rows, 4);
        assert_eq!(state.grid.scrollback.len(), 0);
        assert_eq!(get_line_text(&state.grid, 0), "line 1");
        assert_eq!(get_line_text(&state.grid, 1), "line 2");
        assert_eq!(get_line_text(&state.grid, 2), "line 3");
        assert_eq!(get_line_text(&state.grid, 3), "line 4");
    }

    #[test]
    fn test_resize_prompt_not_pushed_down_by_empty_rows() {
        let mut state = TerminalState::new(25, 80, 100);
        // Simula banner de login SSH e prompt do shell nas linhas 0 e 1
        state.process_bytes(b"Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2\r\nroot@srv01:~# ");

        assert_eq!(get_line_text(&state.grid, 0), "Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2");
        assert_eq!(get_line_text(&state.grid, 1), "root@srv01:~#");
        assert_eq!(state.grid.cursor_row, 1);
        assert_eq!(state.grid.scrollback.len(), 0);

        // Encolhe horizontalmente e verticalmente (ex: de 25x80 para 10x60)
        state.resize(10, 60);
        assert_eq!(state.grid.scrollback.len(), 0, "Linhas ativas cabem na nova altura: scrollback deve permanecer vazio");
        assert_eq!(get_line_text(&state.grid, 0), "Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2");
        assert_eq!(get_line_text(&state.grid, 1), "root@srv01:~#");
        assert_eq!(state.grid.cursor_row, 1);

        // Expande para 30x100 (maior que o inicial)
        state.resize(30, 100);
        assert_eq!(state.grid.scrollback.len(), 0, "Scrollback deve continuar vazio");
        assert_eq!(get_line_text(&state.grid, 0), "Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2");
        assert_eq!(get_line_text(&state.grid, 1), "root@srv01:~#");
        assert_eq!(state.grid.cursor_row, 1, "Cursor e prompt NÃO podem ser deslocados para baixo!");

        // Linhas subsequentes (2..30) devem ser limpas, sem duplicação de prompts
        for r in 2..30 {
            assert_eq!(get_line_text(&state.grid, r), "", "Linha {} deve estar em branco", r);
        }

        // Múltiplos redimensionamentos rápidos para frente e para trás
        for _ in 0..10 {
            state.resize(8, 50);
            state.resize(25, 80);
        }
        assert_eq!(state.grid.scrollback.len(), 0);
        assert_eq!(get_line_text(&state.grid, 0), "Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2");
        assert_eq!(get_line_text(&state.grid, 1), "root@srv01:~#");
        assert_eq!(state.grid.cursor_row, 1);
    }

    #[test]
    fn test_resize_extreme_shrink_and_restore() {
        let mut state = TerminalState::new(25, 80, 100);
        state.process_bytes(b"Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2\r\nroot@srv01:~# ");

        // Encolhe até 1 única linha (situação extrema de esmagamento de janela)
        state.resize(1, 80);
        assert_eq!(state.grid.rows, 1);
        assert_eq!(state.grid.scrollback.len(), 1, "Apenas a linha 0 excedente deve ir pro scrollback");
        assert_eq!(get_line_text(&state.grid, 0), "root@srv01:~#");
        assert_eq!(state.grid.cursor_row, 0);

        // Expande de volta para 25 linhas
        state.resize(25, 80);
        assert_eq!(state.grid.rows, 25);
        assert_eq!(state.grid.scrollback.len(), 0, "Linha do scrollback deve retornar para linha 0");
        assert_eq!(get_line_text(&state.grid, 0), "Last login: Sat Sep 19 11:53:08 2026 from 192.168.18.2");
        assert_eq!(get_line_text(&state.grid, 1), "root@srv01:~#");
        assert_eq!(state.grid.cursor_row, 1);
        for r in 2..25 {
            assert_eq!(get_line_text(&state.grid, r), "", "Linha {} deve ser vazia", r);
        }
    }

    #[test]
    fn test_alt_screen_buffer_switching() {
        let mut state = TerminalState::new(5, 20, 100);
        state.process_bytes(b"shell command 1\r\nshell command 2\r\n");

        assert_eq!(get_line_text(&state.grid, 0), "shell command 1");
        assert_eq!(get_line_text(&state.grid, 1), "shell command 2");
        assert!(!state.grid.is_alt_screen);

        // Enter alternate screen (vim / htop): \x1b[?1049h
        state.process_bytes(b"\x1b[?1049h");
        assert!(state.grid.is_alt_screen);

        // In alt screen, write vim contents
        state.process_bytes(b"VIM EDITOR BUFFER\r\n~ line 2\r\n");
        assert_eq!(get_line_text(&state.grid, 0), "VIM EDITOR BUFFER");
        assert_eq!(get_line_text(&state.grid, 1), "~ line 2");

        // Exit vim / alt screen: \x1b[?1049l
        state.process_bytes(b"\x1b[?1049l");
        assert!(!state.grid.is_alt_screen);

        // Verify primary screen was restored completely
        assert_eq!(get_line_text(&state.grid, 0), "shell command 1");
        assert_eq!(get_line_text(&state.grid, 1), "shell command 2");
    }

    #[test]
    fn test_mouse_modes_and_bracketed_paste() {
        let mut state = TerminalState::new(5, 20, 100);
        assert_eq!(state.grid.mouse_mode, MouseMode::None);
        assert!(!state.grid.sgr_mouse);
        assert!(!state.grid.bracketed_paste);

        // Enable Normal mouse (1000), SGR mouse (1006), bracketed paste (2004)
        state.process_bytes(b"\x1b[?1000h\x1b[?1006h\x1b[?2004h");
        assert_eq!(state.grid.mouse_mode, MouseMode::Normal);
        assert!(state.grid.sgr_mouse);
        assert!(state.grid.bracketed_paste);

        // Switch to ButtonEvent mouse (1002)
        state.process_bytes(b"\x1b[?1002h");
        assert_eq!(state.grid.mouse_mode, MouseMode::ButtonEvent);

        // Switch to AnyEvent mouse (1003)
        state.process_bytes(b"\x1b[?1003h");
        assert_eq!(state.grid.mouse_mode, MouseMode::AnyEvent);

        // Disable mouse and bracketed paste
        state.process_bytes(b"\x1b[?1003l\x1b[?1006l\x1b[?2004l");
        assert_eq!(state.grid.mouse_mode, MouseMode::None);
        assert!(!state.grid.sgr_mouse);
        assert!(!state.grid.bracketed_paste);
    }

    #[test]
    fn test_multiline_paste_normalization() {
        let raw_windows_paste = "comando 1\r\ncomando 2\r\ncomando 3\r\n";
        let normalized = raw_windows_paste.replace("\r\n", "\r").replace('\n', "\r");
        assert_eq!(normalized, "comando 1\rcomando 2\rcomando 3\r");

        let raw_unix_paste = "comando 1\ncomando 2\ncomando 3";
        let normalized_unix = raw_unix_paste.replace("\r\n", "\r").replace('\n', "\r");
        assert_eq!(normalized_unix, "comando 1\rcomando 2\rcomando 3");
    }
}

