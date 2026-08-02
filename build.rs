use image::ImageFormat;
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=assets/images/rustty_icon.png");
    println!("cargo:rerun-if-changed=build.rs");

    // Only apply the icon on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let png_path = "assets/images/rustty_icon.png";
        let ico_path = "assets/images/rustty_icon.ico";

        // Converter PNG para ICO usando a crate image
        if let Ok(img) = image::open(png_path) {
            let resized = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
            let mut file = File::create(ico_path).expect("Falha ao criar o arquivo .ico");
            resized.write_to(&mut file, ImageFormat::Ico)
                .expect("Falha ao escrever o .ico");

            // Embutir o .ico no executável do Windows
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico_path);
            if let Err(e) = res.compile() {
                eprintln!("Aviso: Falha ao embutir o ícone no .exe (winres): {}", e);
            }
        } else {
            eprintln!("Aviso: Ícone PNG não encontrado em {}", png_path);
        }
    }
}
