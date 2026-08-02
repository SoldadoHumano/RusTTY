use iced::{
    alignment, widget::{button, checkbox, column, container, row, scrollable, slider, text, text_input, Space},
    Alignment, Element, Length, theme, Color,
};
use crate::app::{Message, CustomizationState, CustomizationViewMode, PRIMARY_ORANGE, MUTED_COLOR, WARNING_BG, TEXT_COLOR, color_to_hex};
use crate::ui::icons::{icon, icon_sized, icon_colored, LucideIcon};
use crate::config::client::{CustomizationConfig, IpCustomization};

pub fn view<'a>(
    enable_customization: bool,
    state: &'a CustomizationState,
    config: &'a CustomizationConfig,
) -> Element<'a, Message> {
    if !enable_customization {
        let warning_box = container(
            column![
                icon_colored::<Message>(LucideIcon::AlertTriangle, PRIMARY_ORANGE),
                Space::with_height(16),
                text("Personalização Desativada")
                    .size(24)
                    .style(theme::Text::Color(TEXT_COLOR)),
                Space::with_height(8),
                text("Habilite a personalização nas Configurações para editar temas e aparências.")
                    .size(14)
                    .style(theme::Text::Color(MUTED_COLOR))
                    .horizontal_alignment(alignment::Horizontal::Center),
            ]
            .align_items(Alignment::Center)
        )
        .padding(40)
        .style(theme::Container::Custom(Box::new(WarningContainerStyle)));

        return container(warning_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
    }

    match &state.mode {
        CustomizationViewMode::List => view_list(state, config),
        CustomizationViewMode::EditKeyword(_) => view_keyword_form(state),
        CustomizationViewMode::EditIpv4 | CustomizationViewMode::EditIpv6 => view_ip_form(state),
    }
}

fn view_list<'a>(_state: &'a CustomizationState, config: &'a CustomizationConfig) -> Element<'a, Message> {
    let title = row![
        icon_sized::<Message>(LucideIcon::Paintbrush, 28),
        text("  Personalização")
            .size(32)
            .style(theme::Text::Color(PRIMARY_ORANGE)),
    ].align_items(Alignment::Center);

    let create_btn = button(
        row![
            icon_sized::<Message>(LucideIcon::HeartPlus, 18),
            text(" Nova personalização").size(16)
        ]
        .align_items(Alignment::Center)
    )
    .padding([8, 16])
    .style(theme::Button::Primary)
    .on_press(Message::CustomizationOpen(CustomizationViewMode::EditKeyword(None)));

    let defaults_title = text("PADRÕES DE SISTEMA")
        .size(12)
        .style(theme::Text::Color(MUTED_COLOR));

    // IPv4
    let ipv4_color_str = match &config.ipv4 {
        Some(IpCustomization::Unified(c)) => format!("Cor unificada: {}", c),
        Some(IpCustomization::Split { public, private }) => format!("Público: {} | Privado: {}", public, private),
        None => "Padrão do sistema".to_string(),
    };
    let ipv4_item = container(
        row![
            icon_sized::<Message>(LucideIcon::Network, 24),
            Space::with_width(12),
            column![
                text("Personalização de IPv4").size(16),
                text(&ipv4_color_str).size(12).style(theme::Text::Color(MUTED_COLOR)),
            ],
            Space::with_width(Length::Fill),
            button(icon_sized::<Message>(LucideIcon::Edit, 16))
                .style(theme::Button::Secondary)
                .padding(6)
                .on_press(Message::CustomizationOpen(CustomizationViewMode::EditIpv4))
        ]
        .align_items(Alignment::Center)
    )
    .padding(16)
    .style(theme::Container::Custom(Box::new(ItemContainerStyle)));

    // IPv6
    let ipv6_color_str = match &config.ipv6 {
        Some(IpCustomization::Unified(c)) => format!("Cor unificada: {}", c),
        Some(IpCustomization::Split { public, private }) => format!("Público: {} | Privado: {}", public, private),
        None => "Padrão do sistema".to_string(),
    };
    let ipv6_item = container(
        row![
            icon_sized::<Message>(LucideIcon::Globe, 24),
            Space::with_width(12),
            column![
                text("Personalização de IPv6").size(16),
                text(&ipv6_color_str).size(12).style(theme::Text::Color(MUTED_COLOR)),
            ],
            Space::with_width(Length::Fill),
            button(icon_sized::<Message>(LucideIcon::Edit, 16))
                .style(theme::Button::Secondary)
                .padding(6)
                .on_press(Message::CustomizationOpen(CustomizationViewMode::EditIpv6))
        ]
        .align_items(Alignment::Center)
    )
    .padding(16)
    .style(theme::Container::Custom(Box::new(ItemContainerStyle)));

    let keywords_title = text("PALAVRAS-CHAVE PERSONALIZADAS")
        .size(12)
        .style(theme::Text::Color(MUTED_COLOR));

    let mut kw_list = column![keywords_title].spacing(12);

    if config.keywords.is_empty() {
        kw_list = kw_list.push(
            text("Nenhuma palavra-chave personalizada.")
                .size(14)
                .style(theme::Text::Color(MUTED_COLOR))
        );
    } else {
        for (idx, kw) in config.keywords.iter().enumerate() {
            let color = crate::app::hex_to_color(&kw.color).unwrap_or(TEXT_COLOR);
            let case_text = if kw.case_insensitive { "(Case Insensitive)" } else { "(Case Sensitive)" };
            
            let item = container(
                row![
                    icon_colored::<Message>(LucideIcon::AtSign, color),
                    Space::with_width(12),
                    column![
                        text(&kw.keyword).size(16).style(theme::Text::Color(color)),
                        text(&format!("{} {}", kw.color, case_text)).size(12).style(theme::Text::Color(MUTED_COLOR)),
                    ],
                    Space::with_width(Length::Fill),
                    button(icon_sized::<Message>(LucideIcon::Edit, 16))
                        .style(theme::Button::Secondary)
                        .padding(6)
                        .on_press(Message::CustomizationOpen(CustomizationViewMode::EditKeyword(Some(idx)))),
                    Space::with_width(8),
                    button(icon_sized::<Message>(LucideIcon::Trash2, 16))
                        .style(theme::Button::Destructive)
                        .padding(6)
                        .on_press(Message::KwDelete(idx)),
                ]
                .align_items(Alignment::Center)
            )
            .padding(16)
            .style(theme::Container::Custom(Box::new(ItemContainerStyle)));
            
            kw_list = kw_list.push(item);
        }
    }

    let content = column![
        row![
            title,
            Space::with_width(Length::Fill),
            create_btn,
        ].align_items(Alignment::Center),
        Space::with_height(30),
        defaults_title,
        ipv4_item,
        ipv6_item,
        Space::with_height(20),
        kw_list,
    ]
    .spacing(12)
    .max_width(800.0);

    let inner = container(content)
        .width(Length::Fill)
        .padding(40)
        .center_x();

    scrollable(inner)
        .direction(scrollable::Direction::Vertical(
            scrollable::Properties::new().width(0).scroller_width(0),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ─── Componente de Seleção de Cor RGB ─────────────────────────────────────────

fn hex_color_picker<'a>(label: &str, color_hex: &str, on_change: fn(String) -> Message) -> Element<'a, Message> {
    let parsed = crate::app::hex_to_color(color_hex).unwrap_or(Color::TRANSPARENT);
    
    let preview = container(Space::with_width(40).height(40))
        .style(theme::Container::Custom(Box::new(ColorPreviewStyle(parsed))));

    let input = text_input("Ex: #FF0000", color_hex)
        .on_input(on_change)
        .padding(10)
        .width(Length::Fixed(150.0));

    column![
        text(label).size(14).style(theme::Text::Color(MUTED_COLOR)),
        row![
            preview,
            Space::with_width(16),
            input
        ].align_items(Alignment::Center)
    ].spacing(8).into()
}

struct ColorPreviewStyle(Color);
impl container::StyleSheet for ColorPreviewStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(self.0)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.2, 0.2, 0.2),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

// ─── Formulário de Palavra-Chave ──────────────────────────────────────────────

fn view_keyword_form<'a>(state: &'a CustomizationState) -> Element<'a, Message> {
    let header = row![
        button(
            row![
                icon::<Message>(LucideIcon::Undo2),
                text("  Voltar").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::CustomizationClose)
        .style(theme::Button::Text),
        Space::with_width(Length::Fixed(16.0)),
        icon_sized::<Message>(LucideIcon::Edit, 22),
        text(if state.mode == CustomizationViewMode::EditKeyword(None) {
            "  Nova Palavra-Chave"
        } else {
            "  Editar Palavra-Chave"
        })
        .size(24)
        .style(theme::Text::Color(PRIMARY_ORANGE)),
    ]
    .align_items(Alignment::Center);

    let kw_input = text_input("Palavra ou texto (ex: ERROR, down)", &state.kw_keyword)
        .on_input(Message::KwKeywordChanged)
        .padding(10);

    let case_check = checkbox("Case Insensitive (ignorar maiúsculas/minúsculas)", state.kw_case_insensitive)
        .on_toggle(Message::KwCaseInsensitiveToggled)
        .size(16)
        .text_size(14);

    let picker = hex_color_picker("Cor da Palavra-Chave (Hex)", &state.kw_color, Message::KwColorChanged);

    let save_btn = button(text("  Salvar Personalização  ").size(16))
        .on_press(Message::KwSave)
        .padding([10, 20])
        .style(theme::Button::Primary);

    let content = column![
        header,
        Space::with_height(24),
        text("Palavra-Chave").size(14).style(theme::Text::Color(MUTED_COLOR)),
        kw_input,
        Space::with_height(12),
        case_check,
        Space::with_height(24),
        picker,
        Space::with_height(32),
        save_btn,
    ].spacing(8).max_width(500.0);

    container(content)
        .width(Length::Fill)
        .padding(40)
        .center_x()
        .into()
}

// ─── Formulário de IPv4 / IPv6 ────────────────────────────────────────────────

fn view_ip_form<'a>(state: &'a CustomizationState) -> Element<'a, Message> {
    let is_v4 = state.mode == CustomizationViewMode::EditIpv4;
    let title_text = if is_v4 { "  Editar IPv4" } else { "  Editar IPv6" };

    let header = row![
        button(
            row![
                icon::<Message>(LucideIcon::Undo2),
                text("  Voltar").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::CustomizationClose)
        .style(theme::Button::Text),
        Space::with_width(Length::Fixed(16.0)),
        icon_sized::<Message>(if is_v4 { LucideIcon::Network } else { LucideIcon::Globe }, 22),
        text(title_text)
            .size(24)
            .style(theme::Text::Color(PRIMARY_ORANGE)),
    ]
    .align_items(Alignment::Center);

    let split_check = checkbox("Cores separadas para IPs públicos e locais", state.ip_split)
        .on_toggle(Message::IpSplitToggled)
        .size(16)
        .text_size(14);

    let mut form_col = column![header, Space::with_height(24), split_check, Space::with_height(16)];

    if state.ip_split {
        form_col = form_col.push(hex_color_picker("Cor para IP Público (Hex)", &state.ip_public_color, Message::IpPublicColorChanged));
        form_col = form_col.push(Space::with_height(16));
        form_col = form_col.push(hex_color_picker("Cor para IP Privado (Hex)", &state.ip_private_color, Message::IpPrivateColorChanged));
    } else {
        form_col = form_col.push(hex_color_picker("Cor Unificada para todos os IPs (Hex)", &state.ip_unified_color, Message::IpUnifiedColorChanged));
    }

    let save_btn = button(text("  Salvar Personalização  ").size(16))
        .on_press(Message::IpSave)
        .padding([10, 20])
        .style(theme::Button::Primary);

    form_col = form_col.push(Space::with_height(32)).push(save_btn);

    container(form_col.max_width(500.0))
        .width(Length::Fill)
        .padding(40)
        .center_x()
        .into()
}

// ─── Estilos de container ─────────────────────────────────────────────────────

struct WarningContainerStyle;
impl container::StyleSheet for WarningContainerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(WARNING_BG)),
            border: iced::Border {
                color: PRIMARY_ORANGE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ItemContainerStyle;
impl container::StyleSheet for ItemContainerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.15))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.2, 0.2, 0.2),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}
