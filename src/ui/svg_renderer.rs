use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use iced::widget::image;
use iced::Color;
use usvg::{Options, TreeParsing};
use resvg::tiny_skia::{Pixmap, Transform};

use crate::ui::icons::LucideIcon;

// Cache global para as texturas renderizadas, evitado re-rasterização a cada frame.
static SVG_CACHE: OnceLock<Mutex<HashMap<CacheKey, image::Handle>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    icon: LucideIcon,
    size: u16,
    color_rgba: [u8; 4],
}

/// Escala de Supersampling (1x). 
/// Como o resvg usa anti-aliasing analítico puro, renderizar diretamente no tamanho alvo (1x)
/// produz o resultado mais suave e limpo sem sofrer do aliasing (pixel skipping) do downscale bilinear da GPU.
const SUPERSAMPLING_SCALE: u16 = 1;

/// Renderiza o SVG internamente via resvg em alta definição e retorna como uma imagem para o Iced.
pub fn render_supersampled_svg(icon: LucideIcon, target_size: u16, color: Color) -> image::Handle {
    let rgba = color.into_rgba8();
    
    let key = CacheKey {
        icon,
        size: target_size,
        color_rgba: rgba,
    };
    
    let cache_mutex = SVG_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    
    // Tenta pegar do cache
    if let Ok(mut map) = cache_mutex.lock() {
        if let Some(handle) = map.get(&key) {
            return handle.clone();
        }
        
        // Se não tiver, gera a imagem
        let handle = generate_svg_image(icon, target_size, rgba);
        map.insert(key, handle.clone());
        return handle;
    }
    
    // Fallback caso a trava falhe (muito raro)
    generate_svg_image(icon, target_size, rgba)
}

fn generate_svg_image(icon: LucideIcon, target_size: u16, rgba: [u8; 4]) -> image::Handle {
    let internal_size = target_size * SUPERSAMPLING_SCALE;
    let w = internal_size as u32;
    let h = internal_size as u32;
    
    // 1. Obtém a string do SVG e troca a cor via regex ou replace simples
    let svg_bytes = icon.raw_bytes();
    let mut svg_str = String::from_utf8_lossy(svg_bytes).into_owned();
    
    // Formata rgba para hex (#RRGGBB) - Ignoramos Alpha para a cor sólida do traço
    let hex_color = format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    svg_str = svg_str.replace("currentColor", &hex_color);
    
    // Aumenta a espessura nativa do traço (stroke-width) para garantir legibilidade 
    // quando o ícone é reduzido para tamanhos pequenos como 16x16 ou 18x18.
    // Lucide usa 2 por padrão, vamos engordar um pouquinho para não sumirem no anti-aliasing.
    svg_str = svg_str.replace("stroke-width=\"2\"", "stroke-width=\"2.5\"");
    svg_str = svg_str.replace("stroke-width=\"1.5\"", "stroke-width=\"2\"");
    svg_str = svg_str.replace("stroke-width=\"1\"", "stroke-width=\"1.5\"");
    
    // 2. Parse via usvg
    let opt = Options::default();
    let usvg_tree = usvg::Tree::from_data(svg_str.as_bytes(), &opt).unwrap();
    let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);
    
    // 3. Renderiza no tamanho supersampled
    let mut pixmap = Pixmap::new(w, h).unwrap();
    
    // O Lucide tem viewbox 24x24 nativo, mas pegamos o viewBox real do tree se existir.
    let svg_w = usvg_tree.size.width();
    let svg_h = usvg_tree.size.height();
    
    let scale_x = w as f32 / svg_w;
    let scale_y = h as f32 / svg_h;
    
    let transform = Transform::from_scale(scale_x, scale_y);
    resvg_tree.render(transform, &mut pixmap.as_mut());
    
    // 4. Retorna a imagem. O Pixmap já contém dados no formato RGBA.
    let rgba_data = pixmap.take(); // Retorna Vec<u8>
    
    image::Handle::from_pixels(w, h, rgba_data)
}
