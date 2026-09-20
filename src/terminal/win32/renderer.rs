//! Renderizador Direct2D + DirectWrite de altíssima performance para o RusTTY.
//!
//! Minimiza ciclos de drawcall através de:
//! - Clear de fundo em 1 operação de GPU
//! - Batching de retângulos de background contíguos
//! - Batching de spans de texto contíguos com atributos idênticos em uma única chamada `DrawText`
//! - Reutilização de um único `ID2D1SolidColorBrush` mutável via `SetColor`
//! - Renderização nítida via ClearType e fonte "JetBrains Mono"
//! - Métricas de célula via DirectWrite (GetLineMetrics) para alinhamento subpixel perfeito
//! - Margens de respiração visual (MARGIN_X, MARGIN_Y) para enquadramento nativo

use std::net::IpAddr;
use std::str::FromStr;
use windows::core::{w, Interface, Result};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1RenderTarget, ID2D1SolidColorBrush,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_FEATURE_LEVEL_DEFAULT,
    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE,
    D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE, D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_COLOR_F, D2D_RECT_F, D2D_SIZE_U, D2D1_PIXEL_FORMAT, D2D1_ALPHA_MODE_PREMULTIPLIED,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteFontCollection,
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, DWRITE_MEASURING_MODE_NATURAL,
    DWRITE_WORD_WRAPPING_NO_WRAP,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

use crate::config::client::{ClientConfig, IpCustomization};
use crate::terminal::{TerminalGrid, Cell, CellColor};
use crate::terminal::win32::input::InputState;

/// Margem horizontal em pixels para afastar o texto da borda da janela
pub const MARGIN_X: f32 = 8.0;
/// Margem vertical em pixels para afastar o texto do topo da janela
pub const MARGIN_Y: f32 = 4.0;
/// Offset de subpixel horizontal (em pixels) para dar mais corpo/peso (embolden) sutil às letras da fonte
pub const FONT_EMBOLDEN_OFFSET: f32 = 0.05;

/// Converte `CellColor` para cor Direct2D `D2D1_COLOR_F`.
#[inline]
pub fn cell_color_to_d2d(c: CellColor, alpha: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: c.r as f32 / 255.0,
        g: c.g as f32 / 255.0,
        b: c.b as f32 / 255.0,
        a: alpha,
    }
}

/// Palavra-chave pré-compilada para correspondência de alta performance (zero alocações no frame)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedKeyword {
    pub target: String,
    pub color: CellColor,
    pub case_insensitive: bool,
}

/// Cores de IPs pré-parseadas para evitar conversão repetitiva de hexadecimais a cada frame
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CachedIpColors {
    pub ipv4_unified: Option<CellColor>,
    pub ipv4_public: Option<CellColor>,
    pub ipv4_private: Option<CellColor>,
    pub ipv6_unified: Option<CellColor>,
    pub ipv6_public: Option<CellColor>,
    pub ipv6_private: Option<CellColor>,
}

/// Motor de renderização gráfico do terminal Win32.
pub struct TerminalRenderer {
    #[allow(dead_code)]
    d2d_factory: ID2D1Factory,
    #[allow(dead_code)]
    dwrite_factory: IDWriteFactory,
    render_target: ID2D1RenderTarget,
    hwnd_target: ID2D1HwndRenderTarget,
    text_format: IDWriteTextFormat,
    text_format_bold: IDWriteTextFormat,
    brush: ID2D1SolidColorBrush,
    /// Largura de uma célula monospace em pixels.
    pub cell_w: f32,
    /// Altura de uma célula monospace em pixels.
    pub cell_h: f32,
    /// Deslocamento vertical para centralizar o glifo perfeitamente dentro da célula.
    pub glyph_offset_y: f32,
    #[allow(dead_code)]
    pub font_size: f32,
    /// Se a janela do terminal tem foco ativo (bloco sólido vs contorno oco).
    pub has_focus: bool,

    // ── Buffers de Scratch Reutilizáveis (Zero alocações no loop de renderização) ──
    custom_colors: Vec<Option<CellColor>>,
    row_str: String,
    row_str_lower: String,
    span_utf16: Vec<u16>,
    cached_keywords: Vec<CachedKeyword>,
    cached_ip_colors: CachedIpColors,
}

impl TerminalRenderer {
    /// Cria uma nova instância do renderizador para o HWND especificado.
    pub fn new(hwnd: HWND, font_size: f32) -> Result<Self> {
        unsafe {
            // 1. Cria Factory Direct2D
            let d2d_factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

            // 2. Cria Factory DirectWrite
            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            // 3. Cria TextFormat para "JetBrains Mono" (com verificação de coleção de fontes)
            let (text_format, text_format_bold) = Self::create_text_formats(&dwrite_factory, font_size)?;

            // 4. Mede dimensões exatas de célula (monospace pixel-perfect)
            let (cell_w, cell_h, glyph_offset_y) = Self::measure_cell_metrics(&dwrite_factory, &text_format, font_size)?;

            // 5. Cria RenderTarget Direct2D associado à janela
            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            let width = ((rc.right - rc.left).max(100)) as u32;
            let height = ((rc.bottom - rc.top).max(100)) as u32;

            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let hwnd_target = d2d_factory.CreateHwndRenderTarget(&rt_props, &hwnd_props)?;
            hwnd_target.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
            let render_target: ID2D1RenderTarget = hwnd_target.cast()?;

            // 6. Cria brush único reutilizável
            let default_color = cell_color_to_d2d(CellColor::DEFAULT_FG, 1.0);
            let brush = render_target.CreateSolidColorBrush(&default_color, None)?;

            Ok(Self {
                d2d_factory,
                dwrite_factory,
                render_target,
                hwnd_target,
                text_format,
                text_format_bold,
                brush,
                cell_w,
                cell_h,
                glyph_offset_y,
                font_size,
                has_focus: true,
                custom_colors: vec![None; 256],
                row_str: String::with_capacity(256),
                row_str_lower: String::with_capacity(256),
                span_utf16: Vec::with_capacity(256),
                cached_keywords: Vec::new(),
                cached_ip_colors: CachedIpColors::default(),
            })
        }
    }

    fn create_text_formats(
        dwrite: &IDWriteFactory,
        font_size: f32,
    ) -> Result<(IDWriteTextFormat, IDWriteTextFormat)> {
        unsafe {
            // Garante que o arquivo TTF de JetBrains Mono foi registrado no Windows
            crate::terminal::win32::font::ensure_jetbrains_mono_registered();

            // Verifica se "JetBrains Mono" está presente na coleção de fontes do sistema
            let mut font_collection: Option<IDWriteFontCollection> = None;
            let _ = dwrite.GetSystemFontCollection(&mut font_collection, false);
            let jb_available = font_collection.and_then(|col| {
                let mut index = 0u32;
                let mut exists = windows::Win32::Foundation::BOOL(0);
                if col.FindFamilyName(w!("JetBrains Mono"), &mut index, &mut exists).is_ok() {
                    Some(exists.as_bool())
                } else {
                    None
                }
            }).unwrap_or(false);

            let chosen_font = if jb_available {
                crate::debug_log!("INFO", "DirectWrite: Aplicando fonte 'JetBrains Mono' ({:.1}pt)", font_size);
                w!("JetBrains Mono")
            } else {
                crate::debug_log!("WARN", "DirectWrite: 'JetBrains Mono' não indexada, usando 'Cascadia Code'");
                w!("Cascadia Code")
            };

            let try_regular = dwrite.CreateTextFormat(
                chosen_font,
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                font_size,
                w!("en-us"),
            );

            let try_bold = dwrite.CreateTextFormat(
                chosen_font,
                None,
                DWRITE_FONT_WEIGHT_BOLD,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                font_size,
                w!("en-us"),
            );

            let (regular, bold) = match (try_regular, try_bold) {
                (Ok(r), Ok(b)) => (r, b),
                _ => {
                    // Fallback para Consolas
                    crate::debug_log!("WARN", "DirectWrite: Fallback para 'Consolas'");
                    let r = dwrite.CreateTextFormat(
                        w!("Consolas"),
                        None,
                        DWRITE_FONT_WEIGHT_REGULAR,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        font_size,
                        w!("en-us"),
                    )?;
                    let b = dwrite.CreateTextFormat(
                        w!("Consolas"),
                        None,
                        DWRITE_FONT_WEIGHT_BOLD,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        font_size,
                        w!("en-us"),
                    )?;
                    (r, b)
                }
            };

            // Desativa quebra de linha (CRUCIAL para terminais — evita que caracteres pulem de linha)
            let _ = regular.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
            let _ = bold.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);

            Ok((regular, bold))
        }
    }

    fn measure_cell_metrics(
        dwrite: &IDWriteFactory,
        text_format: &IDWriteTextFormat,
        _font_size: f32,
    ) -> Result<(f32, f32, f32)> {
        unsafe {
            // Mede um glifo representativo 'M'
            let sample_u16: Vec<u16> = "M".encode_utf16().collect();
            let layout = dwrite.CreateTextLayout(&sample_u16, text_format, 1000.0, 1000.0)?;
            let mut metrics = std::mem::zeroed();
            layout.GetMetrics(&mut metrics)?;

            let cell_w = metrics.width.max(4.0);

            let mut line_metric = windows::Win32::Graphics::DirectWrite::DWRITE_LINE_METRICS::default();
            let mut line_count = 1u32;
            let res = layout.GetLineMetrics(Some(std::slice::from_mut(&mut line_metric)), &mut line_count);

            let (cell_h, glyph_offset_y) = if res.is_ok() && line_metric.height > 0.0 {
                // Adiciona 2px de respiração vertical entre linhas
                let h = (line_metric.height.ceil() + 2.0).max(8.0);
                let offset = ((h - line_metric.height) / 2.0).max(0.0);
                (h, offset)
            } else {
                let h = metrics.height.ceil().max(8.0);
                (h, 0.0)
            };

            crate::debug_log!(
                "INFO",
                "DirectWrite: métricas de célula -> largura={:.2}px, altura={:.2}px, offset_y={:.2}px",
                cell_w, cell_h, glyph_offset_y
            );

            Ok((cell_w, cell_h, glyph_offset_y))
        }
    }

    /// Redimensiona o render target Direct2D quando a janela do Win32 muda de tamanho.
    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            let size = D2D_SIZE_U { width, height };
            let _ = self.hwnd_target.Resize(&size);
        }
    }

    fn sync_customization_if_needed(&mut self, config: &ClientConfig) {
        let kws = &config.customization_data.keywords;
        let kws_synced = self.cached_keywords.len() == kws.len()
            && self.cached_keywords.iter().zip(kws.iter()).all(|(c, k)| {
                c.case_insensitive == k.case_insensitive
                    && (if k.case_insensitive {
                        c.target == k.keyword.to_lowercase()
                    } else {
                        c.target == k.keyword
                    })
            });

        if !kws_synced {
            self.cached_keywords.clear();
            for kw in kws {
                if let Ok(color) = parse_hex_color(&kw.color) {
                    let target = if kw.case_insensitive {
                        kw.keyword.to_lowercase()
                    } else {
                        kw.keyword.clone()
                    };
                    self.cached_keywords.push(CachedKeyword {
                        target,
                        color,
                        case_insensitive: kw.case_insensitive,
                    });
                }
            }
        }

        // Pré-cacheia cores de IPv4 e IPv6 para eliminar conversões hex em tempo real
        self.cached_ip_colors = CachedIpColors {
            ipv4_unified: config.customization_data.ipv4.as_ref().and_then(|c| match c {
                IpCustomization::Unified(h) => parse_hex_color(h).ok(),
                _ => None,
            }),
            ipv4_public: config.customization_data.ipv4.as_ref().and_then(|c| match c {
                IpCustomization::Split { public, .. } => parse_hex_color(public).ok(),
                _ => None,
            }),
            ipv4_private: config.customization_data.ipv4.as_ref().and_then(|c| match c {
                IpCustomization::Split { private, .. } => parse_hex_color(private).ok(),
                _ => None,
            }),
            ipv6_unified: config.customization_data.ipv6.as_ref().and_then(|c| match c {
                IpCustomization::Unified(h) => parse_hex_color(h).ok(),
                _ => None,
            }),
            ipv6_public: config.customization_data.ipv6.as_ref().and_then(|c| match c {
                IpCustomization::Split { public, .. } => parse_hex_color(public).ok(),
                _ => None,
            }),
            ipv6_private: config.customization_data.ipv6.as_ref().and_then(|c| match c {
                IpCustomization::Split { private, .. } => parse_hex_color(private).ok(),
                _ => None,
            }),
        };
    }

    /// Executa o frame completo de renderização da grade VTE com mínimo absoluto de draw calls.
    pub fn render(
        &mut self,
        grid: &TerminalGrid,
        input: &InputState,
        scroll_offset: usize,
        cursor_visible: bool,
        config: &ClientConfig,
    ) {
        unsafe {
            if config.enable_customization {
                self.sync_customization_if_needed(config);
            }

            self.render_target.BeginDraw();

            // 1. Limpa a tela inteira em 1 único comando com a cor padrão do terminal
            let bg_color = cell_color_to_d2d(CellColor::DEFAULT_BG, 1.0);
            self.render_target.Clear(Some(&bg_color));

            let cw = self.cell_w;
            let ch = self.cell_h;
            let glyph_offset_y = self.glyph_offset_y;
            let rows = grid.rows;
            let cols = grid.cols;

            if self.custom_colors.len() < cols {
                self.custom_colors.resize(cols.max(256), None);
            }

            let abs_top = if grid.is_alt_screen {
                0
            } else {
                grid.scrollback.len().saturating_sub(scroll_offset)
            };

            let has_sel = input.has_selection();
            let sel_anchor = input.sel_anchor;
            let sel_cursor = input.sel_cursor;

            // ── PASSAGEM 1: Backgrounds Batched ─────────────────────────────────
            for row in 0..rows {
                let abs_row = abs_top + row;
                let line_cells = match grid.get_line(abs_row) {
                    Some(l) => l.as_slice(),
                    None => continue,
                };

                let y = MARGIN_Y + row as f32 * ch;
                let mut col = 0;
                while col < cols {
                    let in_sel = has_sel && grid.in_selection(
                        abs_row, col,
                        sel_anchor.unwrap(),
                        sel_cursor.unwrap(),
                    );

                    let cell_bg = if col < line_cells.len() {
                        line_cells[col].effective_bg()
                    } else {
                        CellColor::DEFAULT_BG
                    };

                    if in_sel {
                        // Início do span de seleção
                        let span_start = col;
                        col += 1;
                        while col < cols {
                            let next_in_sel = grid.in_selection(
                                abs_row, col,
                                sel_anchor.unwrap(),
                                sel_cursor.unwrap(),
                            );
                            if !next_in_sel { break; }
                            col += 1;
                        }
                        let span_end = col;

                        let rect = D2D_RECT_F {
                            left: MARGIN_X + span_start as f32 * cw,
                            top: y,
                            right: MARGIN_X + span_end as f32 * cw,
                            bottom: y + ch,
                        };
                        // Cor azul semitransparente de seleção nativa
                        self.brush.SetColor(&D2D1_COLOR_F { r: 0.18, g: 0.38, b: 0.72, a: 0.60 });
                        self.render_target.FillRectangle(&rect, &self.brush);
                    } else if cell_bg != CellColor::DEFAULT_BG {
                        // Span de célula com fundo customizado
                        let span_start = col;
                        let target_bg = cell_bg;
                        col += 1;
                        while col < cols {
                            let next_in_sel = has_sel && grid.in_selection(
                                abs_row, col,
                                sel_anchor.unwrap(),
                                sel_cursor.unwrap(),
                            );
                            if next_in_sel { break; }

                            let next_bg = if col < line_cells.len() {
                                line_cells[col].effective_bg()
                            } else {
                                CellColor::DEFAULT_BG
                            };
                            if next_bg != target_bg { break; }
                            col += 1;
                        }
                        let span_end = col;

                        let rect = D2D_RECT_F {
                            left: MARGIN_X + span_start as f32 * cw,
                            top: y,
                            right: MARGIN_X + span_end as f32 * cw,
                            bottom: y + ch,
                        };
                        self.brush.SetColor(&cell_color_to_d2d(target_bg, 1.0));
                        self.render_target.FillRectangle(&rect, &self.brush);
                    } else {
                        col += 1;
                    }
                }
            }

            // ── PASSAGEM 2: Texto Batched Otimizado (Merge de Espaços Contíguos) ─
            for row in 0..rows {
                let abs_row = abs_top + row;
                let line_cells = match grid.get_line(abs_row) {
                    Some(l) => l.as_slice(),
                    None => continue,
                };

                // Encontra última coluna visível desta linha (ignora espaços vazios no final)
                let last_content = line_cells.iter().rposition(|c| {
                    (c.ch != ' ' && c.ch != '\0') || c.underline
                });

                let line_end_col = match last_content {
                    Some(idx) => (idx + 1).min(cols),
                    None => {
                        if !has_sel {
                            continue; // Linha vazia sem seleção: skip imediato!
                        }
                        0
                    }
                };

                let y = MARGIN_Y + row as f32 * ch;

                if config.enable_customization && line_end_col > 0 {
                    compute_line_highlights(
                        &line_cells[..line_end_col],
                        line_end_col,
                        config,
                        &self.cached_keywords,
                        &self.cached_ip_colors,
                        &mut self.row_str,
                        &mut self.row_str_lower,
                        &mut self.custom_colors[..cols],
                    );
                } else {
                    self.custom_colors[..cols].fill(None);
                }

                let mut col = 0;
                while col < line_end_col {
                    let cell = &line_cells[col];

                    // Pula espaços iniciais vazios não sublinhados
                    if (cell.ch == ' ' || cell.ch == '\0') && !cell.underline {
                        col += 1;
                        continue;
                    }

                    let in_sel = has_sel && grid.in_selection(
                        abs_row, col,
                        sel_anchor.unwrap(),
                        sel_cursor.unwrap(),
                    );

                    // Determina estilo do span atual
                    let span_start = col;
                    let is_bold = cell.bold;
                    let is_underline = cell.underline;
                    let eff_fg = if in_sel {
                        CellColor::WHITE
                    } else if let Some(custom) = self.custom_colors[col] {
                        custom
                    } else {
                        cell.effective_fg()
                    };

                    self.span_utf16.clear();
                    let char_code = if cell.ch == '\0' { ' ' } else { cell.ch };
                    let mut ch_buf = [0u16; 2];
                    self.span_utf16.extend_from_slice(char_code.encode_utf16(&mut ch_buf));

                    col += 1;
                    while col < line_end_col {
                        let next_cell = &line_cells[col];

                        let next_in_sel = has_sel && grid.in_selection(
                            abs_row, col,
                            sel_anchor.unwrap(),
                            sel_cursor.unwrap(),
                        );

                        let next_fg = if next_in_sel {
                            CellColor::WHITE
                        } else if let Some(custom) = self.custom_colors[col] {
                            custom
                        } else {
                            next_cell.effective_fg()
                        };

                        // Agrega caracteres incluindo espaços contíguos enquanto os atributos forem estritamente idênticos
                        if next_cell.bold != is_bold
                            || next_cell.underline != is_underline
                            || next_fg != eff_fg
                            || next_in_sel != in_sel
                        {
                            break;
                        }

                        let next_ch = if next_cell.ch == '\0' { ' ' } else { next_cell.ch };
                        self.span_utf16.extend_from_slice(next_ch.encode_utf16(&mut ch_buf));
                        col += 1;
                    }
                    let span_end = col;

                    if !self.span_utf16.is_empty() {
                        let rect = D2D_RECT_F {
                            left: MARGIN_X + span_start as f32 * cw,
                            top: y + glyph_offset_y,
                            right: (MARGIN_X + span_end as f32 * cw) + 4.0,
                            bottom: y + ch + glyph_offset_y,
                        };

                        self.brush.SetColor(&cell_color_to_d2d(eff_fg, 1.0));
                        let fmt = if is_bold { &self.text_format_bold } else { &self.text_format };

                        self.render_target.DrawText(
                            &self.span_utf16,
                            fmt,
                            &rect,
                            &self.brush,
                            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );

                        // Aplica emboldening sutil se configurado
                        if FONT_EMBOLDEN_OFFSET > 0.0 {
                            let bold_rect = D2D_RECT_F {
                                left: rect.left + FONT_EMBOLDEN_OFFSET,
                                top: rect.top,
                                right: rect.right + FONT_EMBOLDEN_OFFSET,
                                bottom: rect.bottom,
                            };
                            self.render_target.DrawText(
                                &self.span_utf16,
                                fmt,
                                &bold_rect,
                                &self.brush,
                                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                                DWRITE_MEASURING_MODE_NATURAL,
                            );
                        }

                        if is_underline {
                            let line_rect = D2D_RECT_F {
                                left: MARGIN_X + span_start as f32 * cw,
                                top: y + ch - 2.0,
                                right: MARGIN_X + span_end as f32 * cw,
                                bottom: y + ch - 1.0,
                            };
                            self.render_target.FillRectangle(&line_rect, &self.brush);
                        }
                    }
                }
            }

            // ── PASSAGEM 3: Cursor do Terminal ──────────────────────────────────
            let cursor_row = grid.cursor_row;
            let cursor_col = grid.cursor_col;

            let abs_cursor_row = if grid.is_alt_screen {
                cursor_row
            } else {
                grid.scrollback.len() + cursor_row
            };

            if grid.cursor_visible && abs_cursor_row >= abs_top {
                let cursor_screen_row = abs_cursor_row - abs_top;

                if cursor_screen_row < rows && cursor_col < cols {
                    let cx = MARGIN_X + cursor_col as f32 * cw;
                    let cy = MARGIN_Y + cursor_screen_row as f32 * ch;

                    let cur_rect = D2D_RECT_F {
                        left: cx,
                        top: cy,
                        right: cx + cw,
                        bottom: cy + ch,
                    };

                    if self.has_focus {
                        // Com foco: cursor bloco sólido piscante
                        if cursor_visible {
                            self.brush.SetColor(&D2D1_COLOR_F { r: 0.94, g: 0.94, b: 0.94, a: 0.95 });
                            self.render_target.FillRectangle(&cur_rect, &self.brush);

                            // Caractere invertido dentro do cursor (zero alocações)
                            if cursor_row < grid.cells.len() && cursor_col < grid.cells[cursor_row].len() {
                                let cur_cell = &grid.cells[cursor_row][cursor_col];
                                if cur_cell.ch != ' ' && cur_cell.ch != '\0' {
                                    let mut cur_buf = [0u16; 2];
                                    let cur_utf16 = cur_cell.ch.encode_utf16(&mut cur_buf);
                                    self.brush.SetColor(&cell_color_to_d2d(CellColor::DEFAULT_BG, 1.0));
                                    let cur_text_rect = D2D_RECT_F {
                                        left: cx,
                                        top: cy + glyph_offset_y,
                                        right: cx + cw + 4.0,
                                        bottom: cy + ch + glyph_offset_y,
                                    };
                                    self.render_target.DrawText(
                                        cur_utf16,
                                        &self.text_format,
                                        &cur_text_rect,
                                        &self.brush,
                                        D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                                        DWRITE_MEASURING_MODE_NATURAL,
                                    );
                                    if FONT_EMBOLDEN_OFFSET > 0.0 {
                                        let cur_bold_rect = D2D_RECT_F {
                                            left: cur_text_rect.left + FONT_EMBOLDEN_OFFSET,
                                            top: cur_text_rect.top,
                                            right: cur_text_rect.right + FONT_EMBOLDEN_OFFSET,
                                            bottom: cur_text_rect.bottom,
                                        };
                                        self.render_target.DrawText(
                                            cur_utf16,
                                            &self.text_format,
                                            &cur_bold_rect,
                                            &self.brush,
                                            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                                            DWRITE_MEASURING_MODE_NATURAL,
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        // Sem foco: cursor oco (hollow box), sempre visível
                        let b = 1.5f32;
                        self.brush.SetColor(&D2D1_COLOR_F { r: 0.85, g: 0.85, b: 0.85, a: 0.75 });
                        // Topo
                        self.render_target.FillRectangle(
                            &D2D_RECT_F { left: cx, top: cy, right: cx + cw, bottom: cy + b },
                            &self.brush,
                        );
                        // Baixo
                        self.render_target.FillRectangle(
                            &D2D_RECT_F { left: cx, top: cy + ch - b, right: cx + cw, bottom: cy + ch },
                            &self.brush,
                        );
                        // Esquerda
                        self.render_target.FillRectangle(
                            &D2D_RECT_F { left: cx, top: cy, right: cx + b, bottom: cy + ch },
                            &self.brush,
                        );
                        // Direita
                        self.render_target.FillRectangle(
                            &D2D_RECT_F { left: cx + cw - b, top: cy, right: cx + cw, bottom: cy + ch },
                            &self.brush,
                        );
                    }
                }
            }

            let _ = self.render_target.EndDraw(None, None);
        }
    }
}


// ─── Highlighting Helper Otimizado (Zero Heap Allocations) ────────────────────

#[inline]
pub(crate) fn is_keyword_token_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

pub(crate) fn compute_line_highlights(
    line_cells: &[Cell],
    cols: usize,
    config: &ClientConfig,
    cached_keywords: &[CachedKeyword],
    cached_ip_colors: &CachedIpColors,
    row_str: &mut String,
    row_str_lower: &mut String,
    custom_colors: &mut [Option<CellColor>],
) {
    row_str.clear();
    row_str_lower.clear();
    custom_colors[..cols].fill(None);

    let mut has_non_ascii = false;
    let mut has_any_non_space = false;
    for col in 0..cols {
        let ch = if col < line_cells.len() {
            if line_cells[col].ch == '\0' { ' ' } else { line_cells[col].ch }
        } else {
            ' '
        };
        row_str.push(ch);
        if ch != ' ' {
            has_any_non_space = true;
        }
        if !ch.is_ascii() {
            has_non_ascii = true;
        }
    }

    if !has_any_non_space {
        return;
    }

    // Palavras-chave pré-compiladas (zero regex, zero parsing hex)
    let mut has_lower = false;
    for kw in cached_keywords {
        let search_in = if kw.case_insensitive {
            if !has_lower {
                row_str_lower.clear();
                row_str_lower.push_str(&row_str.to_lowercase());
                has_lower = true;
            }
            row_str_lower.as_str()
        } else {
            row_str.as_str()
        };

        let target = &kw.target;
        let mut start_idx = 0;
        while let Some(idx) = search_in[start_idx..].find(target) {
            let match_start = start_idx + idx;
            let match_end = match_start + target.len();

            let is_start_boundary = match_start == 0
                || !is_keyword_token_char(search_in[..match_start].chars().last().unwrap());
            let is_end_boundary = match_end == search_in.len()
                || !is_keyword_token_char(search_in[match_end..].chars().next().unwrap());

            if is_start_boundary && is_end_boundary {
                let char_start = if has_non_ascii {
                    search_in[..match_start].chars().count()
                } else {
                    match_start
                };
                let char_count = if has_non_ascii {
                    target.chars().count()
                } else {
                    target.len()
                };
                for i in char_start..(char_start + char_count) {
                    if i < cols {
                        custom_colors[i] = Some(kw.color);
                    }
                }
            }
            start_idx = match_start + target.len();
        }
    }

    // IPs (tokenização direta sobre fatias de string sem alocações)
    let check_ipv4 = config.customization_data.ipv4.is_some();
    let check_ipv6 = config.customization_data.ipv6.is_some();
    if check_ipv4 || check_ipv6 {
        let mut token_start = None;
        let mut char_idx = 0;
        for (byte_offset, c) in row_str.char_indices() {
            let is_ip_char = c.is_ascii_alphanumeric() || c == '.' || c == ':';
            if is_ip_char {
                if token_start.is_none() {
                    token_start = Some((byte_offset, char_idx));
                }
            } else if let Some((start_byte, start_char)) = token_start {
                let token = &row_str[start_byte..byte_offset];
                apply_ip_color(token, start_char, char_idx, custom_colors, cached_ip_colors);
                token_start = None;
            }
            char_idx += 1;
        }
        if let Some((start_byte, start_char)) = token_start {
            let token = &row_str[start_byte..];
            apply_ip_color(token, start_char, char_idx, custom_colors, cached_ip_colors);
        }
    }
}

pub(crate) fn parse_hex_color(hex: &str) -> std::result::Result<CellColor, ()> {
    let clean = hex.trim_start_matches('#');
    if clean.len() != 6 { return Err(()); }
    let r = u8::from_str_radix(&clean[0..2], 16).map_err(|_| ())?;
    let g = u8::from_str_radix(&clean[2..4], 16).map_err(|_| ())?;
    let b = u8::from_str_radix(&clean[4..6], 16).map_err(|_| ())?;
    Ok(CellColor::rgb(r, g, b))
}

#[inline]
fn paint_ip(
    ip: IpAddr,
    start: usize,
    end: usize,
    custom_colors: &mut [Option<CellColor>],
    cached: &CachedIpColors,
) {
    let color = match ip {
        IpAddr::V4(ipv4) => {
            if let Some(c) = cached.ipv4_unified {
                Some(c)
            } else if cached.ipv4_public.is_some() || cached.ipv4_private.is_some() {
                let octets = ipv4.octets();
                let is_private = octets[0] == 10
                    || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                    || (octets[0] == 192 && octets[1] == 168)
                    || ipv4.is_loopback()
                    || ipv4.is_link_local();
                if is_private { cached.ipv4_private } else { cached.ipv4_public }
            } else {
                None
            }
        }
        IpAddr::V6(ipv6) => {
            if let Some(c) = cached.ipv6_unified {
                Some(c)
            } else if cached.ipv6_public.is_some() || cached.ipv6_private.is_some() {
                let is_private = ipv6.is_loopback()
                    || (ipv6.segments()[0] & 0xfe00) == 0xfc00
                    || (ipv6.segments()[0] & 0xffc0) == 0xfe80;
                if is_private { cached.ipv6_private } else { cached.ipv6_public }
            } else {
                None
            }
        }
    };

    if let Some(c) = color {
        for i in start..end {
            if i < custom_colors.len() {
                custom_colors[i] = Some(c);
            }
        }
    }
}

fn apply_ip_color(
    word: &str,
    start: usize,
    end: usize,
    custom_colors: &mut [Option<CellColor>],
    cached: &CachedIpColors,
) {
    // 1. Tenta correspondência direta sem modificações (caso padrão rápido sem overhead)
    if let Ok(ip) = IpAddr::from_str(word) {
        paint_ip(ip, start, end, custom_colors, cached);
        return;
    }

    // 2. Se falhar, trata pontuação terminal/inicial absorvida pelo tokenizador (ex: "192.168.56.18.", "192.168.56.18...", "192.168.56.18:", "...192.168.56.18")
    let trimmed_end = word.trim_end_matches(|c| c == '.' || c == ':');
    let trailing_count = word.len() - trimmed_end.len();
    let trimmed = trimmed_end.trim_start_matches('.');
    let leading_count = trimmed_end.len() - trimmed.len();

    if !trimmed.is_empty() {
        if let Ok(ip) = IpAddr::from_str(trimmed) {
            let actual_start = start + leading_count;
            let actual_end = end - trailing_count;
            paint_ip(ip, actual_start, actual_end, custom_colors, cached);
        }
    }
}


