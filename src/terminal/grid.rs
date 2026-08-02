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

// ─── Constantes de Layout ─────────────────────────────────────────────────────

/// Tamanho de fonte em pixels lógicos (Consolas 14 no Windows).
pub const FONT_SIZE: f32 = 14.0;
/// Largura de célula monospace em pixels (Consolas 14 ≈ 8.4 px).
pub const CELL_W: f32 = 8.4;
/// Altura de célula (incluindo espaçamento de linha).
pub const CELL_H: f32 = 19.0;

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
            if self.max_scrollback > 0 {
                self.scrollback.push(row);
            }
            self.cells.push(vec![Cell::default(); self.cols]);
        }
        if self.max_scrollback > 0 && self.scrollback.len() > self.max_scrollback {
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
        for col in c..self.cols {
            self.cells[r][col] = Cell::default();
        }
    }

    fn clear_to_start_of_line(&mut self) {
        let (r, c) = (self.cursor_row, self.cursor_col.min(self.cols - 1));
        for col in 0..=c {
            self.cells[r][col] = Cell::default();
        }
    }

    fn clear_line(&mut self, row: usize) {
        for col in 0..self.cols {
            self.cells[row][col] = Cell::default();
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

    // ── API Pública ───────────────────────────────────────────────────────────

    /// Redimensiona a grade, preservando o conteúdo visível.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        // Ajusta cada linha visível
        for row in &mut self.cells {
            row.resize(cols, Cell::default());
        }
        // Ajusta cada linha no scrollback
        for row in &mut self.scrollback {
            row.resize(cols, Cell::default());
        }
        // Ajusta número de linhas
        self.cells.resize(rows, vec![Cell::default(); cols]);
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
        let total_rows = self.scrollback.len() + self.rows;
        let max_r2 = total_rows.saturating_sub(1);
        let actual_r2 = r2.min(max_r2);

        for r in r1..=actual_r2 {
            let from = if r == r1 { c1 } else { 0 };
            let to   = if r == r2 { (c2 + 1).min(self.cols) } else { self.cols };

            let row_slice = if r < self.scrollback.len() {
                &self.scrollback[r]
            } else {
                let cell_r = r - self.scrollback.len();
                if cell_r < self.rows {
                    &self.cells[cell_r]
                } else {
                    continue;
                }
            };

            let line: String = row_slice[from..to]
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

    pub fn clear_all(&mut self) {
        self.grid.clear_all();
    }
}
