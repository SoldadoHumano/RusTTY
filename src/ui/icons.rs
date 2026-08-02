use iced::widget::image;
use iced::widget::Image;
use iced::widget::svg;
use iced::Element;
use iced::Length;
use iced::Theme;
use crate::ui::svg_renderer;
use crate::config::client::PERFORMANCE_MODE;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LucideIcon {
    Home,
    Settings,
    Monitor,
    Folder,
    FolderOpen,
    Terminal,
    Server,
    Key,
    Lock,
    Wifi,
    Eye,
    EyeOff,
    ArrowLeft,
    Check,
    X,
    User,
    AtSign,
    Globe,
    Network,
    Save,
    Shield,
    Plug,
    Trash2,
    LogOut,
    AlertTriangle,
    WifiOff,
    Edit,
    GlobeLock,
    StarPlus,
    ServerPlus,
    Cpu,
    Gpu,
    MemoryStick,
    HardDrive,
    Undo2,
    Paintbrush,
    HeartPlus,
    BookOpenCheck,
}

impl LucideIcon {
    pub fn raw_bytes(self) -> &'static [u8] {
        match self {
            LucideIcon::Home       => include_bytes!("../../assets/icons/house.svg").as_slice(),
            LucideIcon::Settings   => include_bytes!("../../assets/icons/settings.svg").as_slice(),
            LucideIcon::Monitor    => include_bytes!("../../assets/icons/monitor.svg").as_slice(),
            LucideIcon::Folder     => include_bytes!("../../assets/icons/folder.svg").as_slice(),
            LucideIcon::FolderOpen => include_bytes!("../../assets/icons/folder-open.svg").as_slice(),
            LucideIcon::Terminal   => include_bytes!("../../assets/icons/terminal.svg").as_slice(),
            LucideIcon::Server     => include_bytes!("../../assets/icons/server.svg").as_slice(),
            LucideIcon::Key        => include_bytes!("../../assets/icons/key-round.svg").as_slice(),
            LucideIcon::Lock       => include_bytes!("../../assets/icons/lock.svg").as_slice(),
            LucideIcon::Wifi       => include_bytes!("../../assets/icons/globe.svg").as_slice(),
            LucideIcon::Eye        => include_bytes!("../../assets/icons/eye.svg").as_slice(),
            LucideIcon::EyeOff     => include_bytes!("../../assets/icons/eye-off.svg").as_slice(),
            LucideIcon::ArrowLeft  => include_bytes!("../../assets/icons/log-out.svg").as_slice(),
            LucideIcon::Check      => include_bytes!("../../assets/icons/check.svg").as_slice(),
            LucideIcon::X          => include_bytes!("../../assets/icons/x.svg").as_slice(),
            LucideIcon::User       => include_bytes!("../../assets/icons/user.svg").as_slice(),
            LucideIcon::AtSign     => include_bytes!("../../assets/icons/info.svg").as_slice(),
            LucideIcon::Globe      => include_bytes!("../../assets/icons/globe.svg").as_slice(),
            LucideIcon::Network    => include_bytes!("../../assets/icons/network.svg").as_slice(),
            LucideIcon::Save       => include_bytes!("../../assets/icons/save.svg").as_slice(),
            LucideIcon::Shield     => include_bytes!("../../assets/icons/shield.svg").as_slice(),
            LucideIcon::Plug       => include_bytes!("../../assets/icons/cable.svg").as_slice(),
            LucideIcon::Trash2     => include_bytes!("../../assets/icons/trash-2.svg").as_slice(),
            LucideIcon::LogOut     => include_bytes!("../../assets/icons/log-out.svg").as_slice(),
            LucideIcon::AlertTriangle => include_bytes!("../../assets/icons/triangle-alert.svg").as_slice(),
            LucideIcon::WifiOff    => include_bytes!("../../assets/icons/globe-off.svg").as_slice(),
            LucideIcon::Edit       => include_bytes!("../../assets/icons/pencil.svg").as_slice(),
            LucideIcon::GlobeLock  => include_bytes!("../../assets/icons/globe-lock.svg").as_slice(),
            LucideIcon::StarPlus   => include_bytes!("../../assets/icons/star-plus.svg").as_slice(),
            LucideIcon::ServerPlus => include_bytes!("../../assets/icons/server-plus.svg").as_slice(),
            LucideIcon::Cpu        => include_bytes!("../../assets/icons/cpu.svg").as_slice(),
            LucideIcon::Gpu        => include_bytes!("../../assets/icons/gpu.svg").as_slice(),
            LucideIcon::MemoryStick=> include_bytes!("../../assets/icons/memory-stick.svg").as_slice(),
            LucideIcon::HardDrive  => include_bytes!("../../assets/icons/hard-drive.svg").as_slice(),
            LucideIcon::Undo2      => include_bytes!("../../assets/icons/undo-2.svg").as_slice(),
            LucideIcon::Paintbrush => include_bytes!("../../assets/icons/paintbrush.svg").as_slice(),
            LucideIcon::HeartPlus  => include_bytes!("../../assets/icons/heart-plus.svg").as_slice(),
            LucideIcon::BookOpenCheck => include_bytes!("../../assets/icons/book-open-check.svg").as_slice(),
        }
    }
    
    pub fn as_handle(self) -> svg::Handle {
        svg::Handle::from_memory(self.raw_bytes().to_vec())
    }
}

// Para simplificar e evitar injetar o Theme via closure no Image (o que complicaria a vida no `app.rs`),
// estamos fixando a cor do texto do tema escuro como branco. Você pode parametrizar isso se futuramente adicionar Light mode.
const ICON_COLOR: iced::Color = iced::Color::from_rgb(0.9, 0.9, 0.9);

pub fn icon<'a, Message: 'a>(icon: LucideIcon) -> Element<'a, Message> {
    icon_colored(icon, ICON_COLOR)
}

fn color_success(_theme: &iced::Theme) -> iced::widget::svg::Appearance {
    iced::widget::svg::Appearance { color: Some(crate::app::SUCCESS_COLOR) }
}
fn color_error(_theme: &iced::Theme) -> iced::widget::svg::Appearance {
    iced::widget::svg::Appearance { color: Some(crate::app::ERROR_COLOR) }
}
fn color_icon(_theme: &iced::Theme) -> iced::widget::svg::Appearance {
    iced::widget::svg::Appearance { color: Some(ICON_COLOR) }
}

pub fn icon_colored<'a, Message: 'a>(icon: LucideIcon, color: iced::Color) -> Element<'a, Message> {
    let size = 18;
    
    if PERFORMANCE_MODE.load(Ordering::Relaxed) {
        let style_fn = if color == crate::app::SUCCESS_COLOR {
            color_success as fn(&iced::Theme) -> iced::widget::svg::Appearance
        } else if color == crate::app::ERROR_COLOR {
            color_error as fn(&iced::Theme) -> iced::widget::svg::Appearance
        } else {
            color_icon as fn(&iced::Theme) -> iced::widget::svg::Appearance
        };
        return iced::widget::svg(icon.as_handle())
            .width(Length::Fixed(size as f32))
            .height(Length::Fixed(size as f32))
            .content_fit(iced::ContentFit::Fill)
            .style(iced::theme::Svg::custom_fn(style_fn))
            .into();
    }
    
    let handle = svg_renderer::render_supersampled_svg(icon, size, color);
    
    iced::widget::image(handle)
        .width(Length::Fixed(size as f32))
        .height(Length::Fixed(size as f32))
        .content_fit(iced::ContentFit::Fill)
        .into()
}

pub fn icon_sized<'a, Message: 'a>(icon: LucideIcon, size: u16) -> Element<'a, Message> {
    if PERFORMANCE_MODE.load(Ordering::Relaxed) {
        return iced::widget::svg(icon.as_handle())
            .width(Length::Fixed(size as f32))
            .height(Length::Fixed(size as f32))
            .content_fit(iced::ContentFit::Fill)
            .style(iced::theme::Svg::custom_fn(|theme| {
                iced::widget::svg::Appearance {
                    color: Some(theme.palette().text),
                }
            }))
            .into();
    }
    
    let handle = svg_renderer::render_supersampled_svg(icon, size, ICON_COLOR);
    
    iced::widget::image(handle)
        .width(Length::Fixed(size as f32))
        .height(Length::Fixed(size as f32))
        .content_fit(iced::ContentFit::Fill)
        .into()
}

pub fn load_window_icon() -> Option<iced::window::icon::Icon> {
    let icon_bytes = include_bytes!("../../assets/images/rustty_icon.png");
    let img = ::image::load_from_memory(icon_bytes).ok()?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    iced::window::icon::from_rgba(rgba.into_raw(), width, height).ok()
}
