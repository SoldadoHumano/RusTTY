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

/// Gera um UUID v4 como identificador único textual para nós e perfis
pub fn generate_node_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Um perfil de host salvo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostProfile {
    #[serde(default = "generate_node_id")]
    pub id: String,
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
        #[serde(default = "generate_node_id")]
        id: String,
        name: String,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        color: Option<String>,
        children: Vec<ConfigNode>,
    },
    Host(HostProfile),
}

impl ConfigNode {
    pub fn id(&self) -> &str {
        match self {
            ConfigNode::Folder { id, .. } => id,
            ConfigNode::Host(h) => &h.id,
        }
    }

    #[allow(dead_code)]
    pub fn is_folder(&self) -> bool {
        matches!(self, ConfigNode::Folder { .. })
    }
}

/// Configuração global da aplicação que será salva
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub root_nodes: Vec<ConfigNode>,
    #[serde(default)]
    pub bridges: Vec<BridgeProfile>,
    // Futuro: Configurações de UI (fonte, cores do terminal)
}

impl AppConfig {
    /// Adiciona uma nova pasta respeitando estritamente o limite de profundidade (máximo 2 níveis: pasta principal e subpasta).
    pub fn add_folder(
        &mut self,
        parent_id: Option<&str>,
        name: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> Result<String, String> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err("O nome da pasta não pode ser vazio.".to_string());
        }

        let new_id = generate_node_id();
        let new_folder = ConfigNode::Folder {
            id: new_id.clone(),
            name: trimmed_name.to_string(),
            icon,
            color,
            children: Vec::new(),
        };

        match parent_id {
            None => {
                // Pasta principal na raiz (Nível 1) - Permitido
                self.root_nodes.push(new_folder);
                Ok(new_id)
            }
            Some(pid) => {
                // Procura a pasta pai
                // 1. Verifica se a pasta pai está na raiz (Nível 1) -> Se sim, a nova pasta será Nível 2 (Subpasta) - Permitido
                for node in &mut self.root_nodes {
                    if let ConfigNode::Folder { id, children, .. } = node {
                        if id == pid {
                            children.push(new_folder);
                            return Ok(new_id);
                        }

                        // 2. Verifica se a pasta pai já é uma subpasta (Nível 2)
                        for subnode in children.iter() {
                            if let ConfigNode::Folder { id: sub_id, .. } = subnode {
                                if sub_id == pid {
                                    return Err(
                                        "Limite atingido: é permitida apenas a criação de uma pasta principal e de uma subpasta apenas."
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
                Err("Pasta pai não encontrada.".to_string())
            }
        }
    }

    /// Atualiza nome, ícone e cor de uma pasta existente.
    pub fn update_folder(
        &mut self,
        folder_id: &str,
        name: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> Result<(), String> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err("O nome da pasta não pode ser vazio.".to_string());
        }

        fn update_in_nodes(
            nodes: &mut [ConfigNode],
            target_id: &str,
            new_name: &str,
            new_icon: Option<String>,
            new_color: Option<String>,
        ) -> bool {
            for node in nodes {
                if let ConfigNode::Folder { id, name, icon, color, children } = node {
                    if id == target_id {
                        *name = new_name.to_string();
                        *icon = new_icon;
                        *color = new_color;
                        return true;
                    }
                    if update_in_nodes(children, target_id, new_name, new_icon.clone(), new_color.clone()) {
                        return true;
                    }
                }
            }
            false
        }

        if update_in_nodes(&mut self.root_nodes, folder_id, trimmed_name, icon, color) {
            Ok(())
        } else {
            Err("Pasta não encontrada.".to_string())
        }
    }

    /// Exclui uma pasta. Se `keep_contents` for true, os filhos são movidos para o nível pai ou raiz.
    pub fn delete_folder(&mut self, folder_id: &str, keep_contents: bool) -> Result<(), String> {
        // Caso 1: A pasta está na raiz
        if let Some(pos) = self.root_nodes.iter().position(|n| match n {
            ConfigNode::Folder { id, .. } => id == folder_id,
            _ => false,
        }) {
            let removed = self.root_nodes.remove(pos);
            if keep_contents {
                if let ConfigNode::Folder { children, .. } = removed {
                    for (i, child) in children.into_iter().enumerate() {
                        self.root_nodes.insert(pos + i, child);
                    }
                }
            }
            return Ok(());
        }

        // Caso 2: A pasta é uma subpasta dentro de uma pasta na raiz
        for node in &mut self.root_nodes {
            if let ConfigNode::Folder { children, .. } = node {
                if let Some(pos) = children.iter().position(|n| match n {
                    ConfigNode::Folder { id, .. } => id == folder_id,
                    _ => false,
                }) {
                    let removed = children.remove(pos);
                    if keep_contents {
                        if let ConfigNode::Folder { children: sub_children, .. } = removed {
                            for (i, child) in sub_children.into_iter().enumerate() {
                                children.insert(pos + i, child);
                            }
                        }
                    }
                    return Ok(());
                }
            }
        }

        Err("Pasta não encontrada.".to_string())
    }

    /// Move um host existente para um destino (raiz se `target_folder_id` for None, ou para dentro de uma pasta/subpasta).
    pub fn move_host(
        &mut self,
        host_id: &str,
        target_folder_id: Option<&str>,
        target_index: Option<usize>,
    ) -> Result<(), String> {
        fn extract_host_from_nodes(nodes: &mut Vec<ConfigNode>, target_id: &str) -> Option<HostProfile> {
            if let Some(pos) = nodes.iter().position(|n| match n {
                ConfigNode::Host(h) => h.id == target_id,
                _ => false,
            }) {
                if let ConfigNode::Host(h) = nodes.remove(pos) {
                    return Some(h);
                }
            }

            for node in nodes {
                if let ConfigNode::Folder { children, .. } = node {
                    if let Some(h) = extract_host_from_nodes(children, target_id) {
                        return Some(h);
                    }
                }
            }
            None
        }

        let host = match extract_host_from_nodes(&mut self.root_nodes, host_id) {
            Some(h) => h,
            None => return Err("Host não encontrado.".to_string()),
        };

        // 2. Insere no destino pretendido
        match target_folder_id {
            None => {
                // Mover para a raiz
                let idx = target_index.unwrap_or(self.root_nodes.len()).min(self.root_nodes.len());
                self.root_nodes.insert(idx, ConfigNode::Host(host));
                Ok(())
            }
            Some(tfid) => {
                fn insert_into_folder(
                    nodes: &mut [ConfigNode],
                    target_id: &str,
                    host: HostProfile,
                    target_index: Option<usize>,
                ) -> Result<(), HostProfile> {
                    let mut cur_host = host;
                    for node in nodes {
                        if let ConfigNode::Folder { id, children, .. } = node {
                            if id == target_id {
                                let idx = target_index.unwrap_or(children.len()).min(children.len());
                                children.insert(idx, ConfigNode::Host(cur_host));
                                return Ok(());
                            }
                            match insert_into_folder(children, target_id, cur_host, target_index) {
                                Ok(()) => return Ok(()),
                                Err(h) => { cur_host = h; }
                            }
                        }
                    }
                    Err(cur_host)
                }

                match insert_into_folder(&mut self.root_nodes, tfid, host, target_index) {
                    Ok(()) => Ok(()),
                    Err(returned_host) => {
                        self.root_nodes.push(ConfigNode::Host(returned_host));
                        Err("Pasta de destino não encontrada.".to_string())
                    }
                }
            }
        }
    }

    /// Reordena nós dentro da raiz ou dentro de uma pasta específica.
    pub fn reorder_nodes(&mut self, parent_id: Option<&str>, order_ids: &[String]) -> Result<(), String> {
        let nodes_to_reorder = match parent_id {
            None => &mut self.root_nodes,
            Some(pid) => {
                fn find_folder_children<'a>(
                    nodes: &'a mut [ConfigNode],
                    target_id: &str,
                ) -> Option<&'a mut Vec<ConfigNode>> {
                    for node in nodes {
                        if let ConfigNode::Folder { id, children, .. } = node {
                            if id == target_id {
                                return Some(children);
                            }
                            if let Some(res) = find_folder_children(children, target_id) {
                                return Some(res);
                            }
                        }
                    }
                    None
                }
                match find_folder_children(&mut self.root_nodes, pid) {
                    Some(c) => c,
                    None => return Err("Pasta não encontrada para reordenação.".to_string()),
                }
            }
        };

        let mut reordered = Vec::with_capacity(nodes_to_reorder.len());
        let mut original_nodes = std::mem::take(nodes_to_reorder);

        for order_id in order_ids {
            if let Some(pos) = original_nodes.iter().position(|n| n.id() == order_id) {
                reordered.push(original_nodes.remove(pos));
            }
        }
        // Anexa qualquer nó que porventura não estivesse na lista ordenada
        reordered.extend(original_nodes);
        *nodes_to_reorder = reordered;
        Ok(())
    }

    /// Remove um host por ID em qualquer nível da árvore
    pub fn delete_host_by_id(&mut self, host_id: &str) -> Result<(), String> {
        fn remove_host(nodes: &mut Vec<ConfigNode>, target_id: &str) -> bool {
            if let Some(pos) = nodes.iter().position(|n| match n {
                ConfigNode::Host(h) => h.id == target_id,
                _ => false,
            }) {
                nodes.remove(pos);
                return true;
            }
            for node in nodes {
                if let ConfigNode::Folder { children, .. } = node {
                    if remove_host(children, target_id) {
                        return true;
                    }
                }
            }
            false
        }

        if remove_host(&mut self.root_nodes, host_id) {
            Ok(())
        } else {
            Err("Host não encontrado para exclusão.".to_string())
        }
    }

    /// Busca um host por ID recursivamente
    #[allow(dead_code)]
    pub fn find_host(&self, host_id: &str) -> Option<&HostProfile> {
        fn search<'a>(nodes: &'a [ConfigNode], target_id: &str) -> Option<&'a HostProfile> {
            for node in nodes {
                match node {
                    ConfigNode::Host(h) if h.id == target_id => return Some(h),
                    ConfigNode::Folder { children, .. } => {
                        if let Some(h) = search(children, target_id) {
                            return Some(h);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        search(&self.root_nodes, host_id)
    }

    /// Busca um host por ID de forma mutável recursivamente
    pub fn find_host_mut(&mut self, host_id: &str) -> Option<&mut HostProfile> {
        fn search_mut<'a>(nodes: &'a mut [ConfigNode], target_id: &str) -> Option<&'a mut HostProfile> {
            for node in nodes {
                match node {
                    ConfigNode::Host(h) if h.id == target_id => return Some(h),
                    ConfigNode::Folder { children, .. } => {
                        if let Some(h) = search_mut(children, target_id) {
                            return Some(h);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        search_mut(&mut self.root_nodes, host_id)
    }

    /// Coleta todos os hosts recursivamente com seu caminho de pastas (ex: `["Infra", "AWS"]`)
    pub fn get_all_hosts_with_path(&self) -> Vec<(&HostProfile, Vec<String>)> {
        let mut results = Vec::new();

        fn traverse<'a>(
            nodes: &'a [ConfigNode],
            current_path: Vec<String>,
            results: &mut Vec<(&'a HostProfile, Vec<String>)>,
        ) {
            for node in nodes {
                match node {
                    ConfigNode::Host(h) => {
                        results.push((h, current_path.clone()));
                    }
                    ConfigNode::Folder { name, children, .. } => {
                        let mut next_path = current_path.clone();
                        next_path.push(name.clone());
                        traverse(children, next_path, results);
                    }
                }
            }
        }

        traverse(&self.root_nodes, Vec::new(), &mut results);
        results
    }
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
            id: generate_node_id(),
            name: "Host A".into(), address: "1.1.1.1".into(), port: 22, username: "u1".into(),
            auth: AuthType::None, enable_icmp: true, bridge_id: None, legacy_ssh: false,
            icon: Some("terminal".into()),
        };
        let host_b = HostProfile {
            id: generate_node_id(),
            name: "Host B".into(), address: "2.2.2.2".into(), port: 22, username: "u2".into(),
            auth: AuthType::None, enable_icmp: true, bridge_id: None, legacy_ssh: false,
            icon: Some("server".into()),
        };
        let host_c = HostProfile {
            id: generate_node_id(),
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

    #[test]
    fn test_ignore_security_warnings_config_and_schema() {
        use crate::config::client::{ClientConfig, get_settings_schema};

        // 1. Padrão deve ser false (seguro por padrão - Fail Secure)
        let default_cfg = ClientConfig::default();
        assert!(!default_cfg.ignore_security_warnings);

        // 2. Compatibilidade retroativa com JSON que não possui o campo
        let json_without_field = r#"{"max_scrollback_lines": 5000, "performance_mode": false}"#;
        let parsed_cfg: ClientConfig = serde_json::from_str(json_without_field).unwrap();
        assert!(!parsed_cfg.ignore_security_warnings);

        // 3. Serialização e desserialização com campo ativo
        let mut active_cfg = ClientConfig::default();
        active_cfg.ignore_security_warnings = true;
        let serialized = serde_json::to_string(&active_cfg).unwrap();
        let reloaded: ClientConfig = serde_json::from_str(&serialized).unwrap();
        assert!(reloaded.ignore_security_warnings);

        // 4. Schema contém o campo registrado na categoria "Segurança"
        let schema = get_settings_schema();
        let sec_setting = schema.iter().find(|s| s.key == "ignore_security_warnings");
        assert!(sec_setting.is_some(), "ignore_security_warnings deve estar presente no schema de settings");
        let sec_setting = sec_setting.unwrap();
        assert_eq!(sec_setting.category, "Segurança");
        assert_eq!(sec_setting.setting_type, "boolean");
    }

    #[test]
    fn test_folder_depth_limit_enforced() {
        let mut config = AppConfig::default();

        // 1. Cria pasta principal na raiz (Nível 1) -> Deve ter sucesso
        let root_folder_id = config
            .add_folder(None, "Produção".to_string(), Some("server".to_string()), Some("#FF7300".to_string()))
            .expect("Deve permitir criar pasta principal");
        assert_eq!(config.root_nodes.len(), 1);

        // 2. Cria subpasta dentro da pasta principal (Nível 2) -> Deve ter sucesso
        let subfolder_id = config
            .add_folder(Some(&root_folder_id), "Web Servers".to_string(), Some("globe".to_string()), None)
            .expect("Deve permitir criar subpasta dentro da pasta principal");

        // 3. Tenta criar uma pasta dentro da subpasta (Nível 3) -> DEVE FALHAR (limite estrito de 2 níveis)
        let result = config.add_folder(
            Some(&subfolder_id),
            "Nível Proibido".to_string(),
            None,
            None,
        );
        assert!(result.is_err(), "Não deve permitir criar mais de uma subpasta em cadeia");
        assert!(result.unwrap_err().contains("Limite atingido"));
    }

    #[test]
    fn test_move_host_lifecycle() {
        let mut config = AppConfig::default();

        // Cria host na raiz
        let host_id = "test-host-123".to_string();
        let host = HostProfile {
            id: host_id.clone(),
            name: "Servidor DB".into(),
            address: "192.168.1.50".into(),
            port: 22,
            username: "admin".into(),
            auth: AuthType::None,
            enable_icmp: true,
            bridge_id: None,
            legacy_ssh: false,
            icon: Some("hard-drive".into()),
        };
        config.root_nodes.push(ConfigNode::Host(host));

        // Cria estrutura de pastas: Raiz -> Pasta A -> Subpasta B
        let folder_a = config.add_folder(None, "Infra".into(), None, None).unwrap();
        let folder_b = config.add_folder(Some(&folder_a), "Bancos".into(), None, None).unwrap();

        // 1. Move host da raiz para a Pasta A
        config.move_host(&host_id, Some(&folder_a), None).expect("Deve mover para pasta principal");
        assert_eq!(config.root_nodes.len(), 1); // Apenas Pasta A na raiz

        // 2. Move host da Pasta A para a Subpasta B
        config.move_host(&host_id, Some(&folder_b), None).expect("Deve mover para subpasta");

        // 3. Verifica busca recursiva do host
        let found = config.find_host(&host_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Servidor DB");

        // 4. Move host de volta para a raiz
        config.move_host(&host_id, None, None).expect("Deve mover de volta para a raiz");
        assert_eq!(config.root_nodes.len(), 2); // Pasta A + Host na raiz
    }

    #[test]
    fn test_delete_folder_preserve_hosts() {
        let mut config = AppConfig::default();

        let folder_id = config.add_folder(None, "Clusters".into(), None, None).unwrap();
        let host = HostProfile {
            id: "k8s-node".into(),
            name: "K8s Worker".into(),
            address: "10.0.0.1".into(),
            port: 22,
            username: "root".into(),
            auth: AuthType::None,
            enable_icmp: true,
            bridge_id: None,
            legacy_ssh: false,
            icon: None,
        };
        config.move_host("k8s-node", Some(&folder_id), None).unwrap_err(); // ainda não estava na config
        config.root_nodes.push(ConfigNode::Host(host));
        config.move_host("k8s-node", Some(&folder_id), None).unwrap();

        // Exclui a pasta mantendo os hosts
        config.delete_folder(&folder_id, true).expect("Deve excluir pasta preservando hosts");
        assert_eq!(config.root_nodes.len(), 1);
        assert!(matches!(&config.root_nodes[0], ConfigNode::Host(h) if h.id == "k8s-node"));
    }

    #[test]
    fn test_reorder_folders_and_subfolders() {
        let mut config = AppConfig::default();
        let f1 = config.add_folder(None, "Pasta 1".into(), None, None).unwrap();
        let f2 = config.add_folder(None, "Pasta 2".into(), None, None).unwrap();
        let f3 = config.add_folder(None, "Pasta 3".into(), None, None).unwrap();

        // Reordena na raiz: f3, f1, f2
        config.reorder_nodes(None, &[f3.clone(), f1.clone(), f2.clone()]).unwrap();
        assert_eq!(config.root_nodes[0].id(), f3);
        assert_eq!(config.root_nodes[1].id(), f1);
        assert_eq!(config.root_nodes[2].id(), f2);

        // Adiciona subpastas em f3
        let sub1 = config.add_folder(Some(&f3), "Sub 1".into(), None, None).unwrap();
        let sub2 = config.add_folder(Some(&f3), "Sub 2".into(), None, None).unwrap();

        // Reordena subpastas: sub2, sub1
        config.reorder_nodes(Some(&f3), &[sub2.clone(), sub1.clone()]).unwrap();
        if let ConfigNode::Folder { children, .. } = &config.root_nodes[0] {
            assert_eq!(children[0].id(), sub2);
            assert_eq!(children[1].id(), sub1);
        } else {
            panic!("Esperado pasta f3 na posição 0");
        }
    }
}

