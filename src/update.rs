use std::fs;
use std::path::PathBuf;

pub fn check_and_apply_update() -> Result<bool, String> {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("SoldadoHumano")
        .repo_name("RusTTY")
        .bin_name("rustty.exe")
        .target("rustty.exe")
        .current_version(self_update::cargo_crate_version!())
        .build()
        .map_err(|e| e.to_string())?;

    let latest_release = updater.get_latest_release().map_err(|e| e.to_string())?;
    let is_greater = self_update::version::bump_is_greater(
        updater.current_version().as_str(),
        &latest_release.version,
    )
    .map_err(|e| e.to_string())?;

    if !is_greater {
        return Ok(false);
    }

    let asset = latest_release.asset_for("rustty.exe", None)
        .ok_or_else(|| "Asset 'rustty.exe' não encontrado na nova release.".to_string())?;
    
    let temp_dir = std::env::temp_dir();
    let temp_exe = temp_dir.join(format!("rustty_update_{}.exe", uuid::Uuid::new_v4()));

    let mut response = reqwest::blocking::Client::builder()
        .user_agent("RusTTY-Updater")
        .build()
        .map_err(|e| e.to_string())?
        .get(&asset.download_url)
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Falha no download da nova versão: {}", response.status()));
    }

    let mut file = fs::File::create(&temp_exe).map_err(|e| e.to_string())?;
    std::io::copy(&mut response, &mut file).map_err(|e| e.to_string())?;
    drop(file);

    self_replace::self_replace(&temp_exe).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&temp_exe);

    mark_updated();
    
    Ok(true)
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
