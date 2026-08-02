use iced::{
    widget::{column, container, text, Space, scrollable},
    Element, Length, theme, Color, Font,
};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, HeadingLevel};
use crate::app::Message;

pub struct DocPage {
    pub id: &'static str,
    pub title: &'static str,
    pub content: &'static str,
}

pub const PAGES: &[DocPage] = &[
    DocPage {
        id: "introduction",
        title: "Visão Geral",
        content: include_str!("../../assets/documentations/introduction.md"),
    },
    DocPage {
        id: "how_to_use",
        title: "Como Usar",
        content: include_str!("../../assets/documentations/how_to_use.md"),
    },
    DocPage {
        id: "hosts",
        title: "Gerenciar Hosts",
        content: include_str!("../../assets/documentations/hosts.md"),
    },
    DocPage {
        id: "security",
        title: "Segurança",
        content: include_str!("../../assets/documentations/security.md"),
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
];

pub fn render_markdown<'a>(content: &'a str) -> Element<'a, Message> {
    let parser = Parser::new(content);
    let mut main_col = column![].spacing(12);

    let mut current_text = String::new();
    let mut in_code_block = false;
    let mut current_heading_level: Option<HeadingLevel> = None;
    let mut in_list = false;
    let mut list_index = 0;
    
    // Simplification for Iced 0.12: We accumulate text per block, and output a styled text block when the block ends.
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        current_heading_level = Some(level);
                    }
                    Tag::CodeBlock(_) => {
                        in_code_block = true;
                    }
                    Tag::List(_) => {
                        in_list = true;
                        list_index = 0;
                    }
                    Tag::Item => {
                        if in_list {
                            list_index += 1;
                            current_text.push_str(" • ");
                        }
                    }
                    Tag::Strong => {
                        current_text.push_str("**");
                    }
                    Tag::Emphasis => {
                        current_text.push_str("_");
                    }
                    Tag::Paragraph => {}
                    _ => {}
                }
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading(_) => {
                        let size = match current_heading_level {
                            Some(HeadingLevel::H1) => 32,
                            Some(HeadingLevel::H2) => 26,
                            Some(HeadingLevel::H3) => 22,
                            Some(HeadingLevel::H4) => 18,
                            _ => 16,
                        };
                        let heading = text(current_text.trim().to_string())
                            .size(size)
                            .style(theme::Text::Color(crate::app::PRIMARY_ORANGE));
                        main_col = main_col.push(heading);
                        main_col = main_col.push(Space::with_height(Length::Fixed(4.0)));
                        current_text.clear();
                        current_heading_level = None;
                    }
                    TagEnd::Paragraph => {
                        if !current_text.trim().is_empty() {
                            let p = text(current_text.trim().to_string())
                                .size(15)
                                .style(theme::Text::Color(crate::app::TEXT_COLOR));
                            main_col = main_col.push(p);
                        }
                        current_text.clear();
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        let code_content = current_text.trim_end().to_string();
                        let code_box = container(
                            text(code_content)
                                .size(14)
                                .font(Font::MONOSPACE)
                                .style(theme::Text::Color(crate::app::TEXT_COLOR))
                        )
                        .padding(12)
                        .width(Length::Fill)
                        .style(theme::Container::Custom(Box::new(CodeBlockStyle)));
                        
                        main_col = main_col.push(code_box);
                        current_text.clear();
                    }
                    TagEnd::Item => {
                        if !current_text.trim().is_empty() {
                            let p = text(current_text.clone())
                                .size(15)
                                .style(theme::Text::Color(crate::app::TEXT_COLOR));
                            main_col = main_col.push(p);
                        }
                        current_text.clear();
                    }
                    TagEnd::List(_) => {
                        in_list = false;
                    }
                    TagEnd::Strong => {
                        current_text.push_str("**");
                    }
                    TagEnd::Emphasis => {
                        current_text.push_str("_");
                    }
                    _ => {}
                }
            }
            Event::Text(t) => {
                current_text.push_str(&t);
            }
            Event::Code(c) => {
                current_text.push_str(" \"");
                current_text.push_str(&c);
                current_text.push_str("\" ");
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }
    
    // Flush if any remaining
    if !current_text.trim().is_empty() {
        let p = text(current_text.trim().to_string())
            .size(15)
            .style(theme::Text::Color(crate::app::TEXT_COLOR));
        main_col = main_col.push(p);
    }

    let doc_content = container(main_col)
        .padding([10, 20])
        .width(Length::Fill)
        .center_x();

    scrollable(doc_content).into()
}

pub struct CodeBlockStyle;
impl iced::widget::container::StyleSheet for CodeBlockStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
            border: iced::Border {
                color: Color::from_rgb(0.2, 0.2, 0.2),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
}
