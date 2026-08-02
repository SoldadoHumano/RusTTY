use iced::{
    alignment, widget::{container, column, row, text, text_input, toggler, Space, scrollable},
    Alignment, Element, Length, theme,
};
use crate::app::Message;
use crate::ui::icons::{icon_sized, LucideIcon};

// Cores padrão usadas no app
const PRIMARY_ORANGE: iced::Color = iced::Color::from_rgb(1.0, 0.45, 0.0);
const MUTED: iced::Color = iced::Color::from_rgb(0.5, 0.5, 0.5);

/// Renderiza a aba de configurações (estilo agrupado).
pub fn view<'a>(
    scrollback_input: &'a str,
    scroll_lines_input: &'a str,
    command_palette_key_input: &'a str,
    performance_mode: bool,
    global_icmp: bool,
    enable_customization: bool,
) -> Element<'a, Message> {
    // Título
    let title = row![
        icon_sized::<Message>(LucideIcon::Settings, 28),
        text("  Configurações")
            .size(32)
            .style(theme::Text::Color(PRIMARY_ORANGE)),
    ].align_items(Alignment::Center);

        // Bloco 1: Configurações do Cliente
    let rustty_group_title = text("CONFIGURAÇÕES DO RUSTTY")
        .size(12)
        .style(theme::Text::Color(MUTED));

    let group_01= container(
        column![
            row![
                column![
                    text("Modo Performance").size(16),
                    text("Utilize esta opção apenas se o seu computador for extremamente ruim, afetará a qualidade geral do RusTTY")
                        .size(12)
                        .style(theme::Text::Color(MUTED)),
                ].width(Length::Fill),
                toggler(
                    "".to_string(),
                    performance_mode,
                    Message::SettingsPerformanceModeToggled,
                ).width(Length::Shrink)
            ].align_items(Alignment::Center),
            Space::with_height(16),
            row![
                column![
                    text("Habilitar personalização").size(16),
                    text("Habilita ou desabilita os recursos visuais de personalização e temas.")
                        .size(12)
                        .style(theme::Text::Color(MUTED)),
                ].width(Length::Fill),
                toggler(
                    "".to_string(),
                    enable_customization,
                    Message::SettingsCustomizationToggled,
                ).width(Length::Shrink)
            ].align_items(Alignment::Center),
        ]
        .spacing(8)
    )
    .padding(20);

    // Bloco 2: Comportamento do Terminal
    let terminal_group_title = text("COMPORTAMENTO DO TERMINAL")
        .size(12)
        .style(theme::Text::Color(MUTED));

    let scrollback_row = row![
        text("Limite do Histórico (linhas)").size(16),
        Space::with_width(Length::Fill),
        text_input("Ex: 4000", scrollback_input)
            .on_input(Message::SettingsMaxScrollbackChanged)
            .width(Length::Fixed(120.0))
            .padding(8),
    ]
    .align_items(Alignment::Center);

    let scroll_lines_row = row![
        text("Linhas por Scroll").size(16),
        Space::with_width(Length::Fill),
        text_input("Ex: 1", scroll_lines_input)
            .on_input(Message::SettingsScrollLinesChanged)
            .width(Length::Fixed(120.0))
            .padding(8),
    ]
    .align_items(Alignment::Center);

    let palette_key_row = row![
        text("Tecla do Command Palette (Ctrl + Tecla)").size(16),
        Space::with_width(Length::Fill),
        text_input("Ex: .", command_palette_key_input)
            .on_input(Message::SettingsCommandPaletteKeyChanged)
            .width(Length::Fixed(120.0))
            .padding(8),
    ]
    .align_items(Alignment::Center);

    let group_2 = container(
        column![
            scrollback_row,
            text("Define o máximo de linhas que podem ser retidas na memória. 0 desativa o scrollback.")
                .size(12)
                .style(theme::Text::Color(MUTED)),
            Space::with_height(16),
            scroll_lines_row,
            text("Define a quantidade de linhas puladas a cada rotação do scroll do mouse no terminal.")
                .size(12)
                .style(theme::Text::Color(MUTED)),
            Space::with_height(16),
            palette_key_row,
            text("Define a tecla usada junto com o Ctrl para abrir a barra de comandos local.")
                .size(12)
                .style(theme::Text::Color(MUTED)),
        ]
        .spacing(8)
    )
    .padding(20);

    // Bloco 3: Configurações de Hosts
    let hosts_group_title = text("CONFIGURAÇÕES DE HOSTS")
        .size(12)
        .style(theme::Text::Color(MUTED));

    let group_3 = container(
        column![
            row![
                column![
                    text("Habilitar ICMP Global").size(16),
                    text("Realiza testes de ping (ICMP) em background para os hosts habilitados.")
                        .size(12)
                        .style(theme::Text::Color(MUTED)),
                ].width(Length::Fill),
                toggler(
                    "".to_string(),
                    global_icmp,
                    Message::SettingsGlobalIcmpToggled,
                ).width(Length::Shrink)
            ].align_items(Alignment::Center),
        ]
        .spacing(8)
    )
    .padding(20);

    let content = column![
        title,
        Space::with_height(Length::Fixed(30.0)),
        terminal_group_title,
        group_2,
        Space::with_height(Length::Fixed(20.0)),
        rustty_group_title,
        group_01,
        Space::with_height(Length::Fixed(20.0)),
        hosts_group_title,
        group_3,
        Space::with_height(Length::Fixed(20.0)),
        text("SOBRE O CLIENTE")
            .size(12)
            .style(theme::Text::Color(MUTED)),
        container(
            column![
                row![
                    text("Nome do Cliente").size(16),
                    Space::with_width(Length::Fill),
                    text("RusTTY Beta").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
                row![
                    text("Versão do Cliente").size(16),
                    Space::with_width(Length::Fill),
                    text("Beta v0.22").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
                row![
                    text("Data da Versão").size(16),
                    Space::with_width(Length::Fill),
                    text("01/08/2026").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
                row![
                    text("Licença").size(16),
                    Space::with_width(Length::Fill),
                    text("GNU Affero General Public License v3").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
                row![
                    text("Desenvolvedor").size(16),
                    Space::with_width(Length::Fill),
                    text("Vitor").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
                row![
                    text("Co-desenvolvedor").size(16),
                    Space::with_width(Length::Fill),
                    text("N/A").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
                row![
                    text("Website").size(16),
                    Space::with_width(Length::Fill),
                    text("byvitor.com.br").size(16).style(theme::Text::Color(MUTED)),
                ].align_items(Alignment::Center),
            ]
            .spacing(12)
        )
        .padding(20)
        .width(Length::Fill),
        Space::with_height(Length::Fixed(20.0)),
        text("APOIADORES DO PROJETO")
            .size(12)
            .style(theme::Text::Color(MUTED)),
        container(
            column![
                text("Pessoas, empresas e organizações que apoiam o desenvolvimento do RusTTY:")
                    .size(14)
                    .style(theme::Text::Color(MUTED)),
                Space::with_height(Length::Fixed(8.0)),
                row![
                    text("DeepCraft Network").size(16),
                    Space::with_width(Length::Fixed(40.0)),
                ].align_items(Alignment::Center),
                row![
                    text("\n\nSe o RusTTY foi útil para você, para a sua empresa ou organização, considere apoiar o projeto!").size(16),
                    Space::with_width(Length::Fixed(40.0)),
                ].align_items(Alignment::Center),
            ]
            .spacing(8)
        )
        .padding(20)
        .width(Length::Fill),
    ]
    .spacing(12)
    .max_width(650.0);

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