pub mod crypto;
pub mod client;
pub mod protected_mem;

pub use client::{ClientConfig, load_client_config, save_client_config};

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub use protected_mem::ProtectedMemory;

/// Tipo de autenticação suportada para conexão SSH.
///
/// Serializado como tagged union no JSON da config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AuthType {
    /// Autenticação por senha simples (armazenada criptografada em memória e no disco).
    Password(ProtectedMemory),
    /// Autenticação por chave privada SSH.
    Key {
        /// Caminho absoluto para o arquivo de chave privada (ex: ~/.ssh/id_ed25519).
        path: String,
        /// Passphrase para descriptografar a chave (None se a chave não for criptografada).
        passphrase: Option<ProtectedMemory>,
    },
    /// Sem autenticação (não recomendado; apenas para testes locais).
    None,
}

fn default_true() -> bool { true }

/// Um perfil de host salvo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostProfile {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthType,
    #[serde(default = "default_true")]
    pub enable_icmp: bool,
}

/// Representa um nó na árvore de pastas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigNode {
    Folder {
        name: String,
        children: Vec<ConfigNode>,
    },
    Host(HostProfile),
}

/// Configuração global da aplicação que será salva
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub root_nodes: Vec<ConfigNode>,
    // Futuro: Configurações de UI (fonte, cores do terminal)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            root_nodes: Vec::new(),
        }
    }
}

/// Retorna o caminho absoluto do arquivo de configuração (.rtty)
pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ByVitor");
    path.push("RusTTY");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("config.rtty");
    path
}

#[derive(Serialize)]
struct ObfuscatedConfig<'a> {
    #[serde(flatten)]
    config: &'a AppConfig,
    _noise: String,
}

/// Salva a configuração criptografando e escrevendo no disco
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    use rand::RngCore;
    
    // Gera entre 512 e 4096 bytes de lixo aleatório para ofuscar o tamanho real do JSON
    let noise_len = (rand::rngs::OsRng.next_u32() % 3584) + 512;
    let mut noise_bytes = vec![0u8; noise_len as usize];
    rand::rngs::OsRng.fill_bytes(&mut noise_bytes);
    
    let obf_config = ObfuscatedConfig {
        config,
        _noise: hex::encode(noise_bytes),
    };

    let json_bytes = serde_json::to_vec(&obf_config).map_err(|e| e.to_string())?;
    let encrypted_data = crypto::encrypt_data(&json_bytes)?;
    
    let path = get_config_path();
    fs::write(path, encrypted_data).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Lê a configuração descriptografando do disco.
///
/// # Segurança
/// O buffer de plaintext (retornado por `decrypt_data`) é encapsulado em
/// `Zeroizing<Vec<u8>>` e zerizado automaticamente ao final desta função,
/// após a deserialização JSON — o conteúdo decriptado não persiste na heap.
pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    let encrypted_data = match fs::read(&path) {
        Ok(data) => data,
        Err(_) => return AppConfig::default(),
    };

    match crypto::decrypt_data(&encrypted_data) {
        Ok(plaintext_zeroing) => {
            // plaintext_zeroing é Zeroizing<Vec<u8>>; após from_slice, os bytes
            // serão zerizados ao sair deste escopo.
            let config = serde_json::from_slice(&plaintext_zeroing).unwrap_or_default();
            config // plaintext_zeroing zerado aqui (drop)
        }
        Err(e) => {
            eprintln!("Erro ao carregar configurações: {}", e);
            AppConfig::default()
        }
    }
}
