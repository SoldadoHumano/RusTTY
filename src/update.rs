use std::fs;
use std::path::PathBuf;

pub fn check_and_apply_update() -> Result<bool, String> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("SoldadoHumano")
        .repo_name("RusTTY")
        .bin_name("rustty.exe")
        // Como o asset upado manualmente se chama apenas "rustty.exe",
        // forçamos o target a buscar exatamente esse nome, ignorando arquitetura padrão.
        .target("rustty.exe")
        .show_download_progress(false)
        .current_version(self_update::cargo_crate_version!())
        .build()
        .map_err(|e| e.to_string())?
        .update()
        .map_err(|e| e.to_string())?;
    
    Ok(status.updated())
}

pub fn get_update_flag_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ByVitor");
    path.push("RusTTY");
    std::fs::create_dir_all(&path).unwrap_or_default();
    path.push(".updated");
    path
}

pub fn mark_updated() {
    let _ = fs::write(get_update_flag_path(), "true");
}

pub fn check_and_clear_update_flag() -> bool {
    let path = get_update_flag_path();
    if path.exists() {
        let _ = fs::remove_file(path);
        true
    } else {
        false
    }
}
