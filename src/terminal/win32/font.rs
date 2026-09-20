//! Registro e gerenciamento da fonte JetBrains Mono no Windows.
//!
//! Embutida diretamente no binário do RusTTY via `include_bytes!`, garantindo que
//! a fonte "JetBrains Mono" estará sempre disponível no DirectWrite em qualquer máquina,
//! sem requerer que o usuário a instale manualmente no sistema operacional.

use std::fs;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceExW, FONT_RESOURCE_CHARACTERISTICS};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, HWND_BROADCAST, WM_FONTCHANGE};

/// Bytes do arquivo TTF da fonte JetBrains Mono Regular.
pub static JETBRAINS_MONO_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// Bytes do arquivo TTF da fonte JetBrains Mono Bold.
pub static JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Bold.ttf");

/// Garante que os arquivos de fonte JetBrains Mono existam no disco do usuário
/// e estejam registrados no subsistema gráfico do Windows (GDI e DirectWrite).
pub fn ensure_jetbrains_mono_registered() {
    let fonts_dir = match get_user_fonts_dir() {
        Some(dir) => dir,
        None => return,
    };

    let reg_path = fonts_dir.join("JetBrainsMono-Regular.ttf");
    let bold_path = fonts_dir.join("JetBrainsMono-Bold.ttf");

    let reg_written = write_font_if_needed(&reg_path, JETBRAINS_MONO_REGULAR);
    let bold_written = write_font_if_needed(&bold_path, JETBRAINS_MONO_BOLD);

    // Registra na GDI com FR_PRIVATE para o processo atual
    register_font_file(&reg_path);
    register_font_file(&bold_path);

    // Registra nas chaves de registro do usuário para que o DirectWrite indexe nativamente
    register_in_user_registry(&reg_path, "JetBrains Mono Regular (TrueType)");
    register_in_user_registry(&bold_path, "JetBrains Mono Bold (TrueType)");

    if reg_written || bold_written {
        crate::debug_log!("INFO", "JetBrains Mono instalada no perfil do usuário: {:?}", fonts_dir);
        unsafe {
            let _ = PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, WPARAM(0), LPARAM(0));
        }
    }
}

fn get_user_fonts_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = dirs::data_local_dir() {
        let p = local_app_data.join("Microsoft").join("Windows").join("Fonts");
        let _ = fs::create_dir_all(&p);
        Some(p)
    } else {
        None
    }
}

fn write_font_if_needed(path: &PathBuf, data: &[u8]) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.len() as usize == data.len() {
            return false;
        }
    }
    fs::write(path, data).is_ok()
}

fn register_font_file(path: &PathBuf) {
    let wide_path: Vec<u16> = path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        // 0x10 = FR_PRIVATE (apenas este processo)
        let _ = AddFontResourceExW(PCWSTR(wide_path.as_ptr()), FONT_RESOURCE_CHARACTERISTICS(0x10), None);
    }
}

fn register_in_user_registry(font_path: &PathBuf, font_name: &str) {
    #[cfg(windows)]
    {
        use windows::Win32::System::Registry::{
            RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, REG_OPTION_NON_VOLATILE,
        };
        use windows::core::HSTRING;

        unsafe {
            let subkey = HSTRING::from("Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts");
            let mut hkey = HKEY::default();
            let res = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                &subkey,
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut hkey,
                None,
            );

            if res.is_ok() {
                let name_h = HSTRING::from(font_name);
                let path_str = font_path.to_string_lossy().to_string();
                let wide_val: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes = std::slice::from_raw_parts(wide_val.as_ptr() as *const u8, wide_val.len() * 2);

                let _ = RegSetValueExW(
                    hkey,
                    &name_h,
                    0,
                    REG_SZ,
                    Some(bytes),
                );
                let _ = windows::Win32::System::Registry::RegCloseKey(hkey);
            }
        }
    }
}
