pub mod crypto;
pub mod client;
pub mod protected_mem;

#[allow(unused_imports)]
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

impl PartialEq for AuthType {
    fn eq(&self, _other: &Self) -> bool {
        // Implementação simplificada pois não usaremos PartialEq de forma estrita em AuthType,
        // apenas para permitir o derive em BridgeProfile
        false 
    }
}
impl Eq for AuthType {}

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
    #[serde(default)]
    pub bridge_id: Option<uuid::Uuid>,
    /// Habilita algoritmos de key exchange legados (diffie-hellman-group1-sha1,
    /// diffie-hellman-group14-sha1) para compatibilidade com servidores SSH antigos.
    ///
    /// # Segurança
    /// DH-group1-SHA1 é criptograficamente fraco. Use apenas quando necessário.
    #[serde(default)]
    pub legacy_ssh: bool,
    #[serde(default)]
    pub icon: Option<String>,
}

/// Um perfil de ponte (bridge) salva
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BridgeProfile {
    pub id: uuid::Uuid,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthType,
}

impl std::fmt::Display for BridgeProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
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
    #[serde(default)]
    pub bridges: Vec<BridgeProfile>,
    // Futuro: Configurações de UI (fonte, cores do terminal)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            root_nodes: Vec::new(),
            bridges: Vec::new(),
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
    fs::write(&path, encrypted_data).map_err(|e| e.to_string())?;
    crate::debug_log!("INFO", "Cofre de hosts e pontes salvo e criptografado (AES-256-GCM) em '{}'", path.display());
    
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
        crate::debug_log!("INFO", "Arquivo de cofre de hosts '{}' não existe ainda. Inicializando vazio.", path.display());
        return AppConfig::default();
    }

    let encrypted_data = match fs::read(&path) {
        Ok(data) => data,
        Err(e) => {
            crate::debug_log!("ERROR", "Falha ao ler cofre de hosts '{}': {}", path.display(), e);
            return AppConfig::default();
        }
    };

    match crypto::decrypt_data(&encrypted_data) {
        Ok(plaintext_zeroing) => {
            // plaintext_zeroing é Zeroizing<Vec<u8>>; após from_slice, os bytes
            // serão zerizados ao sair deste escopo.
            let config: AppConfig = serde_json::from_slice(&plaintext_zeroing).unwrap_or_default();
            crate::debug_log!("INFO", "Cofre de hosts descriptografado com sucesso ({} hosts, {} pontes)", config.root_nodes.len(), config.bridges.len());
            config // plaintext_zeroing zerado aqui (drop)
        }
        Err(e) => {
            crate::debug_log!("ERROR", "Erro ao descriptografar cofre de hosts: {}", e);
            AppConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_profile_icon_backward_compatibility() {
        // Testando JSON legado sem a propriedade "icon"
        let legacy_json = r#"{
            "name": "Legado",
            "address": "192.168.1.100",
            "port": 22,
            "username": "root",
            "auth": { "type": "None" },
            "enable_icmp": true,
            "bridge_id": null,
            "legacy_ssh": false
        }"#;

        let host: HostProfile = serde_json::from_str(legacy_json).expect("Deve desserializar JSON legado");
        assert_eq!(host.name, "Legado");
        assert_eq!(host.icon, None, "Host legado deve ter icon como None");

        // Serialização e re-desserialização preservando None
        let serialized = serde_json::to_string(&host).expect("Deve serializar host");
        let re_parsed: HostProfile = serde_json::from_str(&serialized).expect("Deve desserializar");
        assert_eq!(re_parsed.icon, None);
    }

    #[test]
    fn test_host_profile_icon_custom_field() {
        let json_with_icon = r#"{
            "name": "Servidor IA",
            "address": "10.0.0.50",
            "port": 2222,
            "username": "developer",
            "auth": { "type": "None" },
            "enable_icmp": false,
            "bridge_id": null,
            "legacy_ssh": true,
            "icon": "gpu"
        }"#;

        let host: HostProfile = serde_json::from_str(json_with_icon).expect("Deve desserializar host com icon");
        assert_eq!(host.name, "Servidor IA");
        assert_eq!(host.icon, Some("gpu".to_string()));

        let serialized = serde_json::to_string(&host).expect("Deve serializar host com icon");
        assert!(serialized.contains(r#""icon":"gpu""#));
    }

    #[test]
    fn test_reorder_hosts_and_deletion_lifecycle() {
        let mut config = AppConfig::default();

        let host_a = HostProfile {
            name: "Host A".into(), address: "1.1.1.1".into(), port: 22, username: "u1".into(),
            auth: AuthType::None, enable_icmp: true, bridge_id: None, legacy_ssh: false,
            icon: Some("terminal".into()),
        };
        let host_b = HostProfile {
            name: "Host B".into(), address: "2.2.2.2".into(), port: 22, username: "u2".into(),
            auth: AuthType::None, enable_icmp: true, bridge_id: None, legacy_ssh: false,
            icon: Some("server".into()),
        };
        let host_c = HostProfile {
            name: "Host C".into(), address: "3.3.3.3".into(), port: 22, username: "u3".into(),
            auth: AuthType::None, enable_icmp: true, bridge_id: None, legacy_ssh: false,
            icon: Some("gpu".into()),
        };

        config.root_nodes.push(ConfigNode::Host(host_a));
        config.root_nodes.push(ConfigNode::Host(host_b));
        config.root_nodes.push(ConfigNode::Host(host_c));

        assert_eq!(config.root_nodes.len(), 3);

        // 1. Reordenar: mover Host A (índice 0) para o fim (índice 2)
        let from = 0;
        let to = 2;
        let moved = config.root_nodes.remove(from);
        config.root_nodes.insert(to, moved);

        let names: Vec<String> = config.root_nodes.iter().map(|n| match n {
            ConfigNode::Host(h) => h.name.clone(),
            _ => String::new(),
        }).collect();

        assert_eq!(names, vec!["Host B", "Host C", "Host A"]);

        // 2. Deletar host do meio (Host C, agora no índice 1)
        config.root_nodes.remove(1);

        let names_after_delete: Vec<String> = config.root_nodes.iter().map(|n| match n {
            ConfigNode::Host(h) => h.name.clone(),
            _ => String::new(),
        }).collect();

        assert_eq!(names_after_delete, vec!["Host B", "Host A"]);
        assert_eq!(config.root_nodes.len(), 2);
    }
}
