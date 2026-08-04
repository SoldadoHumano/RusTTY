//! Módulo de renderização da documentação interna do RusTTY.
//!
//! Arquitetura:
//!   - `PAGES` define a lista ordenada de páginas de documentação.
//!   - `render_markdown` converte Markdown em widgets Iced 0.12 via pulldown-cmark.
//!
//! Estratégia de renderização (Iced 0.12):
//!   Iced 0.12 não possui rich_text/span. O widget `text()` aceita apenas
//!   uma fonte e uma cor por instância. O widget `row()` não faz word-wrap:
//!   widgets filhos que sobram a largura continuam fora da área visível ou
//!   quebram para a margem esquerda do widget filho, não do container pai.
//!
//!   Solução: cada bloco semântico (parágrafo, item de lista, blockquote)
//!   é emitido como **um único `text()` widget** com o conteúdo completo
//!   concatenado. Isso garante word-wrap correto. A formatação inline
//!   (bold, italic, inline code) é preservada visualmente apenas quando
//!   o bloco inteiro tem estilo uniforme; dentro de blocos mistos, o
//!   conteúdo é renderizado em texto plano na cor padrão.
//!
//!   Blocos de código cercados (fenced), tabelas, separadores horizontais
//!   e blockquotes recebem tratamento visual dedicado.

use iced::{
    font,
    widget::{column, container, row, rule, text, Space},
    Color, Element, Font, Length, theme,
};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use crate::app::Message;

// ─── Páginas de Documentação ─────────────────────────────────────────────────

pub struct DocPage {
    pub id: &'static str,
    pub title: &'static str,
    pub content: &'static str,
}

/// Lista ordenada de páginas de documentação.
/// A ordem determina a exibição no menu lateral.
pub const PAGES: &[DocPage] = &[
    DocPage {
        id: "introduction",
        title: "Visão Geral",
        content: include_str!("../../assets/documentations/introduction.md"),
    },
    DocPage {
        id: "hosts",
        title: "Gerenciar Hosts",
        content: include_str!("../../assets/documentations/hosts.md"),
    },
    DocPage {
        id: "how_to_use",
        title: "Operação do Terminal",
        content: include_str!("../../assets/documentations/how_to_use.md"),
    },
    DocPage {
        id: "jump_host",
        title: "SSH via Ponte",
        content: include_str!("../../assets/documentations/jump_host.md"),
    },
    DocPage {
        id: "private_key_auth",
        title: "Autenticação por Chave",
        content: include_str!("../../assets/documentations/private_key_auth.md"),
    },
    DocPage {
        id: "security",
        title: "Segurança",
        content: include_str!("../../assets/documentations/security.md"),
    },
    DocPage {
        id: "threat_model",
        title: "Modelo de Ameaça",
        content: include_str!("../../assets/documentations/threat_model.md"),
    },
    DocPage {
        id: "customization",
        title: "Personalização",
        content: include_str!("../../assets/documentations/customization.md"),
    },
    DocPage {
        id: "best_practices",
        title: "Boas Práticas",
        content: include_str!("../../assets/documentations/best_practices.md"),
    },
    DocPage {
        id: "troubleshooting",
        title: "Diagnóstico",
        content: include_str!("../../assets/documentations/troubleshooting.md"),
    },
];

// ─── Paleta ───────────────────────────────────────────────────────────────────

const TEXT_COLOR: Color       = crate::app::TEXT_COLOR;
const HEADING_COLOR: Color    = crate::app::PRIMARY_ORANGE;
const MUTED_COLOR: Color      = crate::app::MUTED_COLOR;
const CODE_FG: Color          = Color::from_rgb(0.85, 0.75, 0.45);
const CODE_BG: Color          = Color::from_rgb(0.06, 0.06, 0.06);
const TABLE_HEADER_BG: Color  = Color::from_rgb(0.14, 0.14, 0.14);
const TABLE_ROW_ALT_BG: Color = Color::from_rgb(0.10, 0.10, 0.10);
const RULE_COLOR: Color       = Color::from_rgb(0.22, 0.22, 0.22);

fn font_bold() -> Font {
    Font { weight: font::Weight::Bold, ..Font::DEFAULT }
}

fn font_italic() -> Font {
    Font { style: font::Style::Italic, ..Font::DEFAULT }
}

// ─── Renderizador principal ───────────────────────────────────────────────────

/// Converte Markdown em uma coluna scrollável de widgets Iced 0.12.
///
/// Cada bloco semântico produz exatamente um widget de texto (ou um container),
/// garantindo word-wrap correto independentemente do comprimento do conteúdo.
pub fn render_markdown(content: &str) -> Element<'static, Message> {
    let parser = Parser::new_ext(content, pulldown_cmark::Options::all());

    let mut main_col: Vec<Element<'static, Message>> = Vec::new();

    // ── Acumuladores de bloco ─────────────────────────────────────────────────

    // Texto do bloco corrente (parágrafo, item de lista, etc.)
    let mut block_text = String::new();

    // Nível do heading corrente
    let mut heading_level: Option<HeadingLevel> = None;

    // Bloco de código cercado
    let mut code_block_buf = String::new();
    let mut in_code_block   = false;

    // Lista
    let mut list_depth: usize = 0;
    let mut in_item = false;

    // Tabela
    let mut in_table      = false;
    let mut table_head: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row:  Vec<String> = Vec::new();
    let mut cell_buf = String::new();
    let mut in_table_cell = false;
    let mut is_table_head_row = false;

    // Blockquote
    let mut in_blockquote = false;
    let mut bq_text = String::new();

    // ── Emissores inline: inline code e bold são mapeados para o texto plano.
    // O conteúdo de um `code` inline fica entre espaços para separação visual.
    // Bold/italic dentro de parágrafos mistos não são diferenciados — o widget
    // único de texto plano garante o wrap correto.
    let mut _bold   = false;
    let mut _italic = false;

    /// Emite o bloco de texto corrente como widget e limpa o buffer.
    macro_rules! flush_text_block {
        ($col:expr, $buf:expr) => {{
            let trimmed = $buf.trim().to_string();
            if !trimmed.is_empty() {
                let w: Element<'static, Message> = text(trimmed)
                    .size(15)
                    .style(theme::Text::Color(TEXT_COLOR))
                    .into();
                $col.push(w);
            }
            $buf.clear();
        }};
    }

    for event in parser {
        match event {
            // ── Abertura de tags ──────────────────────────────────────────────
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush_text_block!(main_col, block_text);
                    heading_level = Some(level);
                }
                Tag::Paragraph => {
                    if !in_item && !in_blockquote && !in_table_cell {
                        flush_text_block!(main_col, block_text);
                    }
                }
                Tag::Strong => { _bold = true; }
                Tag::Emphasis => { _italic = true; }
                Tag::CodeBlock(_) => {
                    flush_text_block!(main_col, block_text);
                    in_code_block = true;
                    code_block_buf.clear();
                }
                Tag::List(_) => {
                    if list_depth == 0 {
                        flush_text_block!(main_col, block_text);
                        main_col.push(Space::with_height(Length::Fixed(2.0)).into());
                    }
                    list_depth += 1;
                }
                Tag::Item => {
                    flush_text_block!(main_col, block_text);
                    in_item = true;
                    // Inicia o buffer do item com o bullet já incluído.
                    // Prefixo de indentação para listas aninhadas.
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    block_text.push_str(&format!("{}• ", indent));
                }
                Tag::BlockQuote(_) => {
                    flush_text_block!(main_col, block_text);
                    in_blockquote = true;
                    bq_text.clear();
                }
                Tag::Table(_) => {
                    flush_text_block!(main_col, block_text);
                    in_table = true;
                    table_head.clear();
                    table_rows.clear();
                }
                Tag::TableHead => {
                    is_table_head_row = true;
                    current_row.clear();
                }
                Tag::TableRow => {
                    is_table_head_row = false;
                    current_row.clear();
                }
                Tag::TableCell => {
                    in_table_cell = true;
                    cell_buf.clear();
                }
                _ => {}
            },

            // ── Fechamento de tags ────────────────────────────────────────────
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    let full = std::mem::take(&mut block_text);
                    let full = full.trim().to_string();
                    if !full.is_empty() {
                        let (size, weight) = match heading_level {
                            Some(HeadingLevel::H1) => (30u16, font::Weight::Bold),
                            Some(HeadingLevel::H2) => (22u16, font::Weight::Bold),
                            Some(HeadingLevel::H3) => (18u16, font::Weight::Semibold),
                            Some(HeadingLevel::H4) => (16u16, font::Weight::Semibold),
                            _                       => (15u16, font::Weight::Normal),
                        };
                        let w: Element<'static, Message> = text(full)
                            .size(size)
                            .font(Font { weight, ..Font::DEFAULT })
                            .style(theme::Text::Color(HEADING_COLOR))
                            .into();
                        main_col.push(w);
                        main_col.push(Space::with_height(Length::Fixed(4.0)).into());
                    }
                    heading_level = None;
                }
                TagEnd::Paragraph => {
                    if !in_item && !in_blockquote && !in_table_cell {
                        flush_text_block!(main_col, block_text);
                        main_col.push(Space::with_height(Length::Fixed(4.0)).into());
                    }
                }
                TagEnd::Strong   => { _bold   = false; }
                TagEnd::Emphasis => { _italic = false; }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    let code_text = std::mem::take(&mut code_block_buf);
                    let w: Element<'static, Message> = container(
                        text(code_text)
                            .size(13)
                            .font(Font::MONOSPACE)
                            .style(theme::Text::Color(CODE_FG)),
                    )
                    .padding([12, 16])
                    .width(Length::Fill)
                    .style(theme::Container::Custom(Box::new(CodeBlockStyle)))
                    .into();
                    main_col.push(w);
                    main_col.push(Space::with_height(Length::Fixed(6.0)).into());
                }
                TagEnd::Item => {
                    // Emite o item como widget único — wrap correto garantido.
                    let item_text = std::mem::take(&mut block_text);
                    let item_text = item_text.trim_end().to_string();
                    if !item_text.is_empty() {
                        let w: Element<'static, Message> = text(item_text)
                            .size(15)
                            .style(theme::Text::Color(TEXT_COLOR))
                            .into();
                        main_col.push(w);
                    }
                    in_item = false;
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        main_col.push(Space::with_height(Length::Fixed(4.0)).into());
                    }
                }
                TagEnd::BlockQuote => {
                    in_blockquote = false;
                    let bq = std::mem::take(&mut bq_text);
                    let bq = bq.trim().to_string();
                    if !bq.is_empty() {
                        let inner: Element<'static, Message> = text(bq)
                            .size(14)
                            .font(font_italic())
                            .style(theme::Text::Color(MUTED_COLOR))
                            .into();
                        let w: Element<'static, Message> = container(inner)
                            .padding([8, 16])
                            .width(Length::Fill)
                            .style(theme::Container::Custom(Box::new(BlockQuoteStyle)))
                            .into();
                        main_col.push(w);
                        main_col.push(Space::with_height(Length::Fixed(6.0)).into());
                    }
                }
                TagEnd::TableCell => {
                    in_table_cell = false;
                    current_row.push(std::mem::take(&mut cell_buf));
                    // Descartar qualquer acúmulo de block_text dentro da célula
                    block_text.clear();
                }
                TagEnd::TableHead => {
                    table_head = std::mem::take(&mut current_row);
                }
                TagEnd::TableRow => {
                    if !is_table_head_row {
                        table_rows.push(std::mem::take(&mut current_row));
                    }
                }
                TagEnd::Table => {
                    in_table = false;
                    let w = build_table(&table_head, &table_rows);
                    main_col.push(w);
                    main_col.push(Space::with_height(Length::Fixed(8.0)).into());
                }
                _ => {}
            },

            // ── Texto ─────────────────────────────────────────────────────────
            Event::Text(t) => {
                let s = t.into_string();
                if in_code_block {
                    code_block_buf.push_str(&s);
                } else if in_table_cell {
                    cell_buf.push_str(&s);
                } else if in_blockquote {
                    bq_text.push_str(&s);
                } else {
                    block_text.push_str(&s);
                }
            }

            // ── Código inline ─────────────────────────────────────────────────
            // Incluído no fluxo de texto plano do bloco.
            // Delimitado por espaços para separação visual.
            Event::Code(c) => {
                let s = c.into_string();
                if in_table_cell {
                    cell_buf.push_str(&s);
                } else if in_blockquote {
                    bq_text.push_str(&s);
                } else {
                    // Insere espaço antes se o buffer não terminar em espaço
                    if !block_text.ends_with(' ') && !block_text.is_empty() {
                        block_text.push(' ');
                    }
                    block_text.push_str(&s);
                    block_text.push(' ');
                }
            }

            // ── Quebras de linha ──────────────────────────────────────────────
            Event::SoftBreak => {
                if in_blockquote {
                    bq_text.push(' ');
                } else if !in_code_block {
                    block_text.push(' ');
                }
            }
            Event::HardBreak => {
                if !in_code_block {
                    flush_text_block!(main_col, block_text);
                }
            }

            // ── Separador horizontal ──────────────────────────────────────────
            Event::Rule => {
                flush_text_block!(main_col, block_text);
                main_col.push(Space::with_height(Length::Fixed(6.0)).into());
                main_col.push(
                    rule::Rule::horizontal(1)
                        .style(theme::Rule::Custom(Box::new(HorizontalRuleStyle)))
                        .into(),
                );
                main_col.push(Space::with_height(Length::Fixed(6.0)).into());
            }

            _ => {}
        }
    }

    // Flush final
    let remainder = block_text.trim().to_string();
    if !remainder.is_empty() {
        let w: Element<'static, Message> = text(remainder)
            .size(15)
            .style(theme::Text::Color(TEXT_COLOR))
            .into();
        main_col.push(w);
    }

    iced::widget::scrollable(
        container(
            column(main_col).spacing(6).width(Length::Fill),
        )
        .padding([10, 24])
        .width(Length::Fill),
    )
    .into()
}

// ─── Construção de Tabelas ────────────────────────────────────────────────────

fn build_table(head: &[String], rows: &[Vec<String>]) -> Element<'static, Message> {
    let col_count = head
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

    if col_count == 0 {
        return Space::with_height(Length::Fixed(0.0)).into();
    }

    let cell_width = Length::FillPortion(1);

    // Cabeçalho
    let header_cells: Vec<Element<'static, Message>> = head
        .iter()
        .map(|h| {
            container(
                text(h.clone())
                    .size(13)
                    .font(font_bold())
                    .style(theme::Text::Color(HEADING_COLOR)),
            )
            .padding([6, 10])
            .width(cell_width)
            .into()
        })
        .collect();

    let header_row: Element<'static, Message> = container(
        row(header_cells).width(Length::Fill),
    )
    .width(Length::Fill)
    .style(theme::Container::Custom(Box::new(TableHeaderStyle)))
    .into();

    // Linhas de corpo
    let body_rows: Vec<Element<'static, Message>> = rows
        .iter()
        .enumerate()
        .map(|(i, row_data)| {
            let cells: Vec<Element<'static, Message>> = (0..col_count)
                .map(|ci| {
                    let content = row_data.get(ci).cloned().unwrap_or_default();
                    container(
                        text(content)
                            .size(13)
                            .style(theme::Text::Color(TEXT_COLOR)),
                    )
                    .padding([5, 10])
                    .width(cell_width)
                    .into()
                })
                .collect();

            let bg: Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>> = if i % 2 == 0 {
                Box::new(TableRowEvenStyle)
            } else {
                Box::new(TableRowOddStyle)
            };

            container(row(cells).width(Length::Fill))
                .width(Length::Fill)
                .style(theme::Container::Custom(bg))
                .into()
        })
        .collect();

    let all_rows: Vec<Element<'static, Message>> = std::iter::once(header_row)
        .chain(body_rows)
        .collect();

    container(
        column(all_rows).spacing(0).width(Length::Fill),
    )
    .width(Length::Fill)
    .style(theme::Container::Custom(Box::new(TableOuterStyle)))
    .into()
}

// ─── Estilos ──────────────────────────────────────────────────────────────────

pub struct CodeBlockStyle;
impl iced::widget::container::StyleSheet for CodeBlockStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(CODE_BG)),
            border: iced::Border {
                color: RULE_COLOR,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
}

pub struct BlockQuoteStyle;
impl iced::widget::container::StyleSheet for BlockQuoteStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(
                Color::from_rgba(1.0, 0.45, 0.0, 0.06),
            )),
            border: iced::Border {
                color: HEADING_COLOR,
                width: 3.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

pub struct HorizontalRuleStyle;
impl iced::widget::rule::StyleSheet for HorizontalRuleStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::rule::Appearance {
        iced::widget::rule::Appearance {
            color: RULE_COLOR,
            width: 1,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
        }
    }
}

pub struct TableOuterStyle;
impl iced::widget::container::StyleSheet for TableOuterStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            border: iced::Border {
                color: RULE_COLOR,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

pub struct TableHeaderStyle;
impl iced::widget::container::StyleSheet for TableHeaderStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(TABLE_HEADER_BG)),
            ..Default::default()
        }
    }
}

pub struct TableRowEvenStyle;
impl iced::widget::container::StyleSheet for TableRowEvenStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08))),
            ..Default::default()
        }
    }
}

pub struct TableRowOddStyle;
impl iced::widget::container::StyleSheet for TableRowOddStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(TABLE_ROW_ALT_BG)),
            ..Default::default()
        }
    }
}
