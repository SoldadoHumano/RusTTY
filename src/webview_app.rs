//! Módulo do Webview — interface HTML/CSS/JS servida via wry.
//!
//! Arquitetura:
//!   - Webview é servido via custom protocol `rustty://`
//!   - IPC bidirecional: JS → Rust via `window.ipc.postMessage(JSON)`
//!                        Rust → JS via `webview.evaluate_script("window.__rustCallback(...)")`
//!   - Todas as operações sensíveis (spawn terminal, CRUD config, ICMP) são executadas no Rust
//!   - Credenciais NUNCA são enviadas de volta ao frontend

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{WebView, WebViewBuilder, http::Response};
use serde_json::{json, Value};

use crate::config::{
    load_config, save_config, AppConfig, AuthType, ConfigNode, HostProfile,
    client::{load_client_config, save_client_config, ClientConfig},
};

// ─── Documentação embarcada ──────────────────────────────────────────────────

#[allow(dead_code)]
struct DocPage {
    id: &'static str,
    title: &'static str,
    content: &'static str,
}

const DOC_PAGES: &[DocPage] = &[
    DocPage { id: "introduction",     title: "Visão Geral",             content: include_str!("../assets/documentations/introduction.md") },
    DocPage { id: "hosts",            title: "Gerenciar Hosts",         content: include_str!("../assets/documentations/hosts.md") },
    DocPage { id: "how_to_use",       title: "Operação do Terminal",    content: include_str!("../assets/documentations/how_to_use.md") },
    DocPage { id: "jump_host",        title: "SSH via Ponte",           content: include_str!("../assets/documentations/jump_host.md") },
    DocPage { id: "private_key_auth", title: "Autenticação por Chave",  content: include_str!("../assets/documentations/private_key_auth.md") },
    DocPage { id: "security",         title: "Segurança",               content: include_str!("../assets/documentations/security.md") },
    DocPage { id: "threat_model",     title: "Modelo de Ameaça",        content: include_str!("../assets/documentations/threat_model.md") },
    DocPage { id: "customization",    title: "Personalização",          content: include_str!("../assets/documentations/customization.md") },
    DocPage { id: "best_practices",   title: "Boas Práticas",           content: include_str!("../assets/documentations/best_practices.md") },
    DocPage { id: "troubleshooting",  title: "Diagnóstico",             content: include_str!("../assets/documentations/troubleshooting.md") },
];

// ─── Estado compartilhado entre IPC handler e event loop ─────────────────────

struct WebviewState {
    config: AppConfig,
    client_config: ClientConfig,
    active_terminals: HashMap<String, std::process::Child>,
}

impl WebviewState {
    fn new() -> Self {
        Self {
            config: load_config(),
            client_config: load_client_config(),
            active_terminals: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn reload_config(&mut self) {
        self.config = load_config();
    }

    #[allow(dead_code)]
    fn reload_client_config(&mut self) {
        self.client_config = load_client_config();
    }
}

// ─── Serialização segura de config para o frontend ──────────────────────────

fn serialize_hosts(config: &AppConfig) -> Vec<Value> {
    config.root_nodes.iter().filter_map(|node| {
        if let ConfigNode::Host(h) = node {
            Some(json!({
                "name": h.name,
                "address": h.address,
                "port": h.port,
                "username": h.username,
                "has_password": !matches!(h.auth, AuthType::None),
                "enable_icmp": h.enable_icmp,
                "bridge_id": h.bridge_id.map(|id| id.to_string()),
                "legacy_ssh": h.legacy_ssh,
                "allow_domain": h.address.parse::<std::net::IpAddr>().is_err(),
            }))
        } else {
            None
        }
    }).collect()
}

fn serialize_bridges(config: &AppConfig) -> Vec<Value> {
    config.bridges.iter().map(|b| {
        json!({
            "id": b.id.to_string(),
            "name": b.name,
            "address": b.address,
            "port": b.port,
            "username": b.username,
            "has_password": !matches!(b.auth, AuthType::None),
            "allow_domain": b.address.parse::<std::net::IpAddr>().is_err(),
        })
    }).collect()
}

fn serialize_client_config(cc: &ClientConfig) -> Value {
    let cd = &cc.customization_data;
    let ipv4 = cd.ipv4.as_ref().map(serialize_ip_customization);
    let ipv6 = cd.ipv6.as_ref().map(serialize_ip_customization);
    let keywords: Vec<Value> = cd.keywords.iter().map(|kw| {
        json!({ "keyword": kw.keyword, "color": kw.color, "case_insensitive": kw.case_insensitive })
    }).collect();

    json!({
        "max_scrollback_lines": cc.max_scrollback_lines,
        "performance_mode": cc.performance_mode,
        "global_icmp": cc.global_icmp,
        "scroll_lines": cc.scroll_lines,
        "command_palette_key": cc.command_palette_key.to_string(),
        "enable_customization": cc.enable_customization,
        "allow_multiple_access_to_same_host": cc.allow_multiple_access_to_same_host,
        "enable_auto_update": cc.enable_auto_update,
        "terminal_font_size": cc.terminal_font_size,
        "debug_mode": cc.debug_mode,
        "antialiasing": cc.antialiasing,
        "experimental_webview_ui": cc.experimental_webview_ui,
        "customization_data": {
            "ipv4": ipv4,
            "ipv6": ipv6,
            "keywords": keywords,
        }
    })
}

fn serialize_ip_customization(ip: &crate::config::client::IpCustomization) -> Value {
    match ip {
        crate::config::client::IpCustomization::Unified(c) => json!({ "Unified": c }),
        crate::config::client::IpCustomization::Split { public, private } => {
            json!({ "Split": { "public": public, "private": private } })
        }
    }
}

fn serialize_doc_pages() -> Vec<Value> {
    DOC_PAGES.iter().map(|p| json!({ "id": p.id, "title": p.title })).collect()
}

// ─── Validação de endereço (réplica da lógica de app.rs) ────────────────────

#[allow(dead_code)]
fn validate_address(address: &str, allow_domain: bool) -> Result<(), String> {
    let addr = address.trim();
    if addr.is_empty() {
        return Err("Endereço não pode ser vazio.".to_string());
    }
    if addr.parse::<std::net::IpAddr>().is_ok() {
        return Ok(());
    }
    if !allow_domain {
        return Err(
            "Somente IPs numéricos são aceitos. Marque \"Permitir domínio\" para usar nomes DNS."
                .to_string(),
        );
    }
    if addr.len() > 253 {
        return Err("Domínio excede o limite de 253 caracteres.".to_string());
    }
    let labels: Vec<&str> = addr.split('.').collect();
    if labels.len() < 2 {
        return Err("Domínio inválido — deve ter pelo menos um ponto.".to_string());
    }
    for label in &labels {
        if label.is_empty() || label.len() > 63 {
            return Err(format!("Label de domínio inválida: \"{}\"", label));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(format!("Label \"{}\" não pode começar ou terminar com hífen.", label));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(format!("Label \"{}\" contém caracteres inválidos.", label));
        }
    }
    Ok(())
}

// ─── Callback helper: envia mensagem do Rust → Frontend ─────────────────────

#[allow(dead_code)]
fn send_to_frontend(webview: &WebView, msg: &Value) {
    let json_str = serde_json::to_string(msg).unwrap_or_default();
    let escaped = json_str.replace('\\', "\\\\").replace('\'', "\\'");
    let script = format!("window.__rustCallback('{}')", escaped);
    let _ = webview.evaluate_script(&script);
}

// ─── IPC Message Handler ─────────────────────────────────────────────────────

#[allow(dead_code)]
fn handle_ipc_message(
    msg_type: &str,
    parsed: &Value,
    state: &Arc<Mutex<WebviewState>>,
    webview: &WebView,
    proxy: &winit::event_loop::EventLoopProxy<String>,
) {
    match msg_type {
        // ── Config Data ──────────────────────────────────────────────
        "get_config" => {
            let st = state.lock().unwrap();
            send_to_frontend(webview, &json!({
                "type": "config_data",
                "hosts": serialize_hosts(&st.config),
                "bridges": serialize_bridges(&st.config),
            }));
        }

        "get_client_config" => {
            let st = state.lock().unwrap();
            send_to_frontend(webview, &json!({
                "type": "client_config_data",
                "data": serialize_client_config(&st.client_config),
                "doc_pages": serialize_doc_pages(),
                "settings_schema": crate::config::client::get_settings_schema(),
            }));
        }

        // ── Documentation ────────────────────────────────────────────
        "get_doc_page" => {
            if let Some(page_id) = parsed.get("page_id").and_then(|v| v.as_str()) {
                let content = DOC_PAGES.iter()
                    .find(|p| p.id == page_id)
                    .map(|p| p.content)
                    .unwrap_or("Página não encontrada.");
                send_to_frontend(webview, &json!({
                    "type": "doc_page_content",
                    "page_id": page_id,
                    "content": content,
                }));
            }
        }

        // ── CRUD: Hosts ──────────────────────────────────────────────
        "save_host" => {
            let data = match parsed.get("data") {
                Some(d) => d,
                None => { send_error(webview, "Dados do host não fornecidos."); return; }
            };

            let name = data.get("name").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let address = data.get("address").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let port = data.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
            let username = data.get("username").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let password = data.get("password").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let allow_domain = data.get("allow_domain").and_then(|v| v.as_bool()).unwrap_or(false);
            let enable_icmp = data.get("enable_icmp").and_then(|v| v.as_bool()).unwrap_or(true);
            let legacy_ssh = data.get("legacy_ssh").and_then(|v| v.as_bool()).unwrap_or(false);
            let bridge_id_str = data.get("bridge_id").and_then(|v| v.as_str());
            let bridge_id = bridge_id_str.and_then(|s| uuid::Uuid::parse_str(s).ok());
            let edit_index = parsed.get("edit_index").and_then(|v| v.as_u64()).map(|v| v as usize);

            // Validação
            if name.is_empty() {
                send_error(webview, "O nome/apelido do host é obrigatório."); return;
            }
            if address.is_empty() {
                send_error(webview, "O endereço é obrigatório."); return;
            }
            if let Err(e) = validate_address(&address, allow_domain) {
                send_error(webview, &e); return;
            }
            if username.is_empty() {
                send_error(webview, "O nome de usuário é obrigatório."); return;
            }
            if port == 0 {
                send_error(webview, "Porta inválida (1–65535)."); return;
            }

            let auth = if password.is_empty() {
                AuthType::None
            } else {
                AuthType::Password(
                    crate::config::ProtectedMemory::new(&password)
                        .unwrap_or_else(|_| crate::config::ProtectedMemory::new("").unwrap())
                )
            };

            let profile = HostProfile {
                name, address, port, username, auth,
                enable_icmp,
                bridge_id,
                legacy_ssh,
            };

            let mut st = state.lock().unwrap();
            if let Some(idx) = edit_index {
                if idx < st.config.root_nodes.len() {
                    // Ao editar, preserva a senha original se nenhuma nova foi fornecida
                    if password.is_empty() {
                        if let Some(ConfigNode::Host(existing)) = st.config.root_nodes.get(idx) {
                            let mut profile = profile;
                            profile.auth = existing.auth.clone();
                            st.config.root_nodes[idx] = ConfigNode::Host(profile);
                        } else {
                            st.config.root_nodes[idx] = ConfigNode::Host(profile);
                        }
                    } else {
                        st.config.root_nodes[idx] = ConfigNode::Host(profile);
                    }
                }
            } else {
                st.config.root_nodes.push(ConfigNode::Host(profile));
            }

            match save_config(&st.config) {
                Ok(()) => send_success(webview, "Host salvo com sucesso."),
                Err(e) => send_error(webview, &format!("Erro ao salvar: {}", e)),
            }
        }

        "delete_host" => {
            let index = match parsed.get("index").and_then(|v| v.as_u64()) {
                Some(i) => i as usize,
                None => { send_error(webview, "Índice inválido."); return; }
            };
            let mut st = state.lock().unwrap();
            if index < st.config.root_nodes.len() {
                st.config.root_nodes.remove(index);
                match save_config(&st.config) {
                    Ok(()) => send_success(webview, "Host removido."),
                    Err(e) => send_error(webview, &format!("Erro ao salvar: {}", e)),
                }
            }
        }

        // ── CRUD: Bridges ────────────────────────────────────────────
        "save_bridge" => {
            let data = match parsed.get("data") {
                Some(d) => d,
                None => { send_error(webview, "Dados da ponte não fornecidos."); return; }
            };

            let name = data.get("name").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let address = data.get("address").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let port = data.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
            let username = data.get("username").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let password = data.get("password").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let allow_domain = data.get("allow_domain").and_then(|v| v.as_bool()).unwrap_or(false);
            let edit_index = parsed.get("edit_index").and_then(|v| v.as_u64()).map(|v| v as usize);

            if name.is_empty() {
                send_error(webview, "O nome da ponte é obrigatório."); return;
            }
            if address.is_empty() {
                send_error(webview, "O endereço é obrigatório."); return;
            }
            if let Err(e) = validate_address(&address, allow_domain) {
                send_error(webview, &e); return;
            }
            if username.is_empty() {
                send_error(webview, "O nome de usuário é obrigatório."); return;
            }
            if port == 0 {
                send_error(webview, "Porta inválida (1–65535)."); return;
            }

            let mut st = state.lock().unwrap();

            let id = if let Some(idx) = edit_index {
                st.config.bridges.get(idx).map(|b| b.id).unwrap_or_else(uuid::Uuid::new_v4)
            } else {
                uuid::Uuid::new_v4()
            };

            let auth = if password.is_empty() {
                if let Some(idx) = edit_index {
                    st.config.bridges.get(idx).map(|b| b.auth.clone()).unwrap_or(AuthType::None)
                } else {
                    AuthType::None
                }
            } else {
                AuthType::Password(
                    crate::config::ProtectedMemory::new(&password)
                        .unwrap_or_else(|_| crate::config::ProtectedMemory::new("").unwrap())
                )
            };

            let profile = crate::config::BridgeProfile {
                id, name, address, port, username, auth,
            };

            if let Some(idx) = edit_index {
                if idx < st.config.bridges.len() {
                    st.config.bridges[idx] = profile;
                }
            } else {
                st.config.bridges.push(profile);
            }

            match save_config(&st.config) {
                Ok(()) => send_success(webview, "Ponte salva com sucesso."),
                Err(e) => send_error(webview, &format!("Erro ao salvar: {}", e)),
            }
        }

        "delete_bridge" => {
            let index = match parsed.get("index").and_then(|v| v.as_u64()) {
                Some(i) => i as usize,
                None => { send_error(webview, "Índice inválido."); return; }
            };
            let mut st = state.lock().unwrap();
            if index < st.config.bridges.len() {
                let deleted_id = st.config.bridges[index].id;
                st.config.bridges.remove(index);

                // Remove referências de hosts que usavam esta ponte
                for node in &mut st.config.root_nodes {
                    if let ConfigNode::Host(ref mut host) = node {
                        if host.bridge_id == Some(deleted_id) {
                            host.bridge_id = None;
                            host.enable_icmp = true;
                        }
                    }
                }

                match save_config(&st.config) {
                    Ok(()) => send_success(webview, "Ponte removida."),
                    Err(e) => send_error(webview, &format!("Erro ao salvar: {}", e)),
                }
            }
        }

        // ── Terminal Spawn ───────────────────────────────────────────
        "open_terminal" => {
            let host_name = parsed.get("host_name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let mut st = state.lock().unwrap();

            if !st.client_config.allow_multiple_access_to_same_host {
                if let Some(child) = st.active_terminals.get_mut(&host_name) {
                    if let Ok(None) = child.try_wait() {
                        send_to_frontend(webview, &json!({
                            "type": "terminal_error",
                            "error": "Uma sessão para este host já está aberta."
                        }));
                        return;
                    } else {
                        st.active_terminals.remove(&host_name);
                    }
                }
            }

            match std::env::current_exe() {
                Ok(exe_path) => {
                    match std::process::Command::new(&exe_path)
                        .args(["--terminal", &host_name])
                        .spawn()
                    {
                        Ok(child) => { st.active_terminals.insert(host_name, child); }
                        Err(e) => {
                            send_to_frontend(webview, &json!({
                                "type": "terminal_error",
                                "error": format!("Falha ao abrir terminal: {}", e)
                            }));
                        }
                    }
                }
                Err(e) => {
                    send_to_frontend(webview, &json!({
                        "type": "terminal_error",
                        "error": format!("Executável não encontrado: {}", e)
                    }));
                }
            }
        }

        "connect_bridge" => {
            let index = parsed.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let mut st = state.lock().unwrap();

            if let Some(bridge) = st.config.bridges.get(index) {
                let bridge_name = bridge.name.clone();
                let bridge_id = bridge.id.to_string();

                if !st.client_config.allow_multiple_access_to_same_host {
                    if let Some(child) = st.active_terminals.get_mut(&bridge_name) {
                        if let Ok(None) = child.try_wait() {
                            send_to_frontend(webview, &json!({
                                "type": "terminal_error",
                                "error": "Uma sessão para esta ponte já está aberta."
                            }));
                            return;
                        } else {
                            st.active_terminals.remove(&bridge_name);
                        }
                    }
                }

                match std::env::current_exe() {
                    Ok(exe_path) => {
                        match std::process::Command::new(&exe_path)
                            .args(["--bridge-terminal", &bridge_id])
                            .spawn()
                        {
                            Ok(child) => { st.active_terminals.insert(bridge_name, child); }
                            Err(e) => {
                                send_to_frontend(webview, &json!({
                                    "type": "terminal_error",
                                    "error": format!("Falha ao abrir terminal: {}", e)
                                }));
                            }
                        }
                    }
                    Err(e) => {
                        send_to_frontend(webview, &json!({
                            "type": "terminal_error",
                            "error": format!("Executável não encontrado: {}", e)
                        }));
                    }
                }
            }
        }

        "quick_connect" => {
            let data = match parsed.get("data") {
                Some(d) => d,
                None => { send_error(webview, "Dados não fornecidos."); return; }
            };
            let address = data.get("address").and_then(|v| v.as_str()).unwrap_or_default();
            let port = data.get("port").and_then(|v| v.as_u64()).unwrap_or(22).to_string();
            let username = data.get("username").and_then(|v| v.as_str()).unwrap_or_default();
            let password = data.get("password").and_then(|v| v.as_str()).unwrap_or("none");

            match std::env::current_exe() {
                Ok(exe_path) => {
                    if let Err(e) = std::process::Command::new(&exe_path)
                        .args(["--quick-ssh", address, &port, username, password])
                        .spawn()
                    {
                        send_to_frontend(webview, &json!({
                            "type": "terminal_error",
                            "error": format!("Falha ao abrir terminal: {}", e)
                        }));
                    }
                }
                Err(e) => {
                    send_to_frontend(webview, &json!({
                        "type": "terminal_error",
                        "error": format!("Executável não encontrado: {}", e)
                    }));
                }
            }
        }

        // ── Settings ─────────────────────────────────────────────────
        "save_setting" => {
            let key = parsed.get("key").and_then(|v| v.as_str()).unwrap_or_default();
            let value = parsed.get("value");

            let mut st = state.lock().unwrap();
            let cc = &mut st.client_config;

            match key {
                "max_scrollback_lines" => {
                    if let Some(v) = value.and_then(|v| v.as_str()).and_then(|s| s.parse::<usize>().ok()) {
                        cc.max_scrollback_lines = v.min(100_000);
                    }
                }
                "scroll_lines" => {
                    if let Some(v) = value.and_then(|v| v.as_str()).and_then(|s| s.parse::<usize>().ok()) {
                        cc.scroll_lines = v.clamp(1, 16);
                    }
                }
                "terminal_font_size" => {
                    if let Some(v) = value.and_then(|v| v.as_str()).and_then(|s| s.parse::<u8>().ok()) {
                        cc.terminal_font_size = v.clamp(1, 22);
                    }
                }
                "command_palette_key" => {
                    if let Some(ch) = value.and_then(|v| v.as_str()).and_then(|s| s.chars().next()) {
                        cc.command_palette_key = ch;
                    }
                }
                "performance_mode" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) {
                        cc.performance_mode = v;
                        crate::config::client::PERFORMANCE_MODE.store(v, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                "global_icmp" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) { cc.global_icmp = v; }
                }
                "enable_customization" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) { cc.enable_customization = v; }
                }
                "allow_multiple_access_to_same_host" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) { cc.allow_multiple_access_to_same_host = v; }
                }
                "enable_auto_update" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) { cc.enable_auto_update = v; }
                }
                "debug_mode" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) {
                        cc.debug_mode = v;
                        crate::config::client::DEBUG_MODE.store(v, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                "antialiasing" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) { cc.antialiasing = v; }
                }
                "experimental_webview_ui" => {
                    if let Some(v) = value.and_then(|v| v.as_bool()) { cc.experimental_webview_ui = v; }
                }
                _ => {}
            }

            if let Err(e) = crate::config::client::save_client_config(cc) {
                send_error(webview, &format!("Erro ao salvar configuração: {}", e));
                return;
            }

            // Envia a config atualizada de volta com o schema completo para manter
            // o frontend sincronizado sem necessidade de nova requisição
            send_to_frontend(webview, &json!({
                "type": "client_config_data",
                "data": serialize_client_config(cc),
                "doc_pages": serialize_doc_pages(),
                "settings_schema": crate::config::client::get_settings_schema(),
            }));
        }

        // ── Customization ────────────────────────────────────────────
        "save_customization" => {
            let data = match parsed.get("data") {
                Some(d) => d,
                None => return,
            };
            let cust_type = data.get("type").and_then(|v| v.as_str()).unwrap_or_default();
            let mut st = state.lock().unwrap();

            match cust_type {
                "keyword" => {
                    let keyword = data.get("keyword").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let color = data.get("color").and_then(|v| v.as_str()).unwrap_or("#FF7300").to_string();
                    let case_insensitive = data.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false);
                    let edit_index = data.get("edit_index").and_then(|v| v.as_u64()).map(|v| v as usize);

                    if keyword.is_empty() { return; }

                    let kw = crate::config::client::KeywordTheme {
                        id: uuid::Uuid::new_v4(),
                        keyword, color, case_insensitive,
                    };

                    if let Some(idx) = edit_index {
                        if idx < st.client_config.customization_data.keywords.len() {
                            st.client_config.customization_data.keywords[idx] = kw;
                        }
                    } else {
                        st.client_config.customization_data.keywords.push(kw);
                    }
                }
                "ip" => {
                    let target = data.get("target").and_then(|v| v.as_str()).unwrap_or("ipv4");
                    let split = data.get("split").and_then(|v| v.as_bool()).unwrap_or(false);

                    let ip_data = if split {
                        crate::config::client::IpCustomization::Split {
                            public: data.get("public_color").and_then(|v| v.as_str()).unwrap_or("#34C759").to_string(),
                            private: data.get("private_color").and_then(|v| v.as_str()).unwrap_or("#FF453A").to_string(),
                        }
                    } else {
                        crate::config::client::IpCustomization::Unified(
                            data.get("unified_color").and_then(|v| v.as_str()).unwrap_or("#FF7300").to_string()
                        )
                    };

                    match target {
                        "ipv4" => st.client_config.customization_data.ipv4 = Some(ip_data),
                        "ipv6" => st.client_config.customization_data.ipv6 = Some(ip_data),
                        _ => {}
                    }
                }
                _ => {}
            }

            if let Err(e) = save_client_config(&st.client_config) {
                send_error(webview, &format!("Erro ao salvar personalização: {}", e));
                return;
            }
            // Envia config atualizada com settings_schema para não quebrar a tela de Settings
            send_to_frontend(webview, &json!({
                "type": "client_config_data",
                "data": serialize_client_config(&st.client_config),
                "doc_pages": serialize_doc_pages(),
                "settings_schema": crate::config::client::get_settings_schema(),
            }));
            send_success(webview, "Personalização salva com sucesso.");
        }

        "delete_keyword" => {
            let index = parsed.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let mut st = state.lock().unwrap();
            if index < st.client_config.customization_data.keywords.len() {
                st.client_config.customization_data.keywords.remove(index);
                if let Err(e) = save_client_config(&st.client_config) {
                    send_error(webview, &format!("Erro ao salvar: {}", e));
                    return;
                }
                // Inclui settings_schema para não quebrar a tela de Settings após voltar
                send_to_frontend(webview, &json!({
                    "type": "client_config_data",
                    "data": serialize_client_config(&st.client_config),
                    "doc_pages": serialize_doc_pages(),
                    "settings_schema": crate::config::client::get_settings_schema(),
                }));
            }
        }

        // ── ICMP ─────────────────────────────────────────────────────
        "icmp_check" => {
            let st = state.lock().unwrap();
            if !st.client_config.global_icmp { return; }

            let hosts_for_icmp: Vec<(usize, String, bool)> = st.config.root_nodes.iter()
                .enumerate()
                .filter_map(|(idx, node)| {
                    if let ConfigNode::Host(h) = node {
                        if h.enable_icmp {
                            return Some((idx, h.address.clone(), true));
                        }
                    }
                    None
                })
                .collect();
            drop(st);

            let proxy = proxy.clone();
            std::thread::spawn(move || {
                let mut results: HashMap<String, bool> = HashMap::new();
                for (idx, ip, _) in hosts_for_icmp {
                    let mut cmd = std::process::Command::new("ping");
                    
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        const CREATE_NO_WINDOW: u32 = 0x08000000;
                        cmd.creation_flags(CREATE_NO_WINDOW);
                        cmd.args(["-n", "1", "-w", "1000", &ip]);
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        cmd.args(["-c", "1", "-W", "1", &ip]);
                    }

                    let result = cmd.stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    results.insert(idx.to_string(), result);
                }
                let _ = proxy.send_event(json!({
                    "type": "internal_icmp_results",
                    "data": results,
                }).to_string());
            });
        }
        
        "internal_icmp_results" => {
            send_to_frontend(webview, &json!({
                "type": "icmp_results",
                "data": parsed.get("data").unwrap_or(&json!({})),
            }));
        }

        // ── Open URL ─────────────────────────────────────────────────
        "open_url" => {
            if let Some(url) = parsed.get("url").and_then(|v| v.as_str()) {
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", url])
                    .spawn();
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(url).spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(url).spawn();
            }
        }

        _ => {}
    }
}

#[allow(dead_code)]
fn send_success(webview: &WebView, message: &str) {
    send_to_frontend(webview, &json!({
        "type": "operation_result",
        "success": true,
        "message": message,
    }));
}

#[allow(dead_code)]
fn send_error(webview: &WebView, error: &str) {
    send_to_frontend(webview, &json!({
        "type": "operation_result",
        "success": false,
        "error": error,
    }));
}

// ─── Entry Point ─────────────────────────────────────────────────────────────

pub fn run() -> iced::Result {
    let event_loop = EventLoopBuilder::<String>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    
    let window = WindowBuilder::new()
        .with_title("RusTTY")
        .with_inner_size(winit::dpi::LogicalSize::new(900.0, 640.0))
        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 420.0))
        .build(&event_loop)
        .unwrap();

    // ── Assets embarcados ────────────────────────────────────────────────────
    let html_content = include_str!("../webview/index.html");
    let css_content  = include_str!("../webview/style.css");
    let js_content   = include_str!("../webview/app.js");
    let icon_content = include_bytes!("../assets/images/iconv2.png");

    // ── Estado compartilhado ─────────────────────────────────────────────────
    let shared_state = Arc::new(Mutex::new(WebviewState::new()));
    let ipc_state = Arc::clone(&shared_state);

    // ── Webview ──────────────────────────────────────────────────────────────
    let webview = WebViewBuilder::new(&window)
        .with_custom_protocol("rustty".into(), move |request| {
            let path = request.uri().path();
            match path {
                "/style.css" => {
                    Response::builder()
                        .header("Content-Type", "text/css; charset=utf-8")
                        .body(css_content.as_bytes().to_vec().into())
                        .unwrap()
                }
                "/app.js" => {
                    Response::builder()
                        .header("Content-Type", "application/javascript; charset=utf-8")
                        .body(js_content.as_bytes().to_vec().into())
                        .unwrap()
                }
                "/assets/images/iconv2.png" => {
                    Response::builder()
                        .header("Content-Type", "image/png")
                        .body(icon_content.to_vec().into())
                        .unwrap()
                }
                _ => {
                    Response::builder()
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(html_content.as_bytes().to_vec().into())
                        .unwrap()
                }
            }
        })
        .with_url("rustty://localhost")
        .with_ipc_handler({
            let proxy_clone = proxy.clone();
            move |msg| {
                let body = msg.body();
                // Encaminha a mensagem IPC do JS diretamente para a thread principal (Event Loop)
                let _ = proxy_clone.send_event(body.to_string());
            }
        })
        .build()
        .unwrap();


    // ── Verificar atualização ────────────────────────────────────────────────
    {
        let st = shared_state.lock().unwrap();
        if crate::update::check_and_clear_update_flag() {
            let update_msg = json!({
                "type": "update_notification",
                "message": "RusTTY foi atualizado com sucesso!"
            });
            let script = format!(
                "setTimeout(function() {{ window.__rustCallback('{}'); }}, 2000);",
                serde_json::to_string(&update_msg).unwrap_or_default().replace('\\', "\\\\").replace('\'', "\\'"),
            );
            let _ = webview.evaluate_script(&script);
        }
        if st.client_config.enable_auto_update {
            std::thread::spawn(move || {
                let _ = crate::update::check_and_apply_update();
            });
        }
    }

    // ── Event Loop ───────────────────────────────────────────────────────────
    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);
        match event {
            Event::UserEvent(msg_body) => {
                // Aqui estamos na thread principal e TEMOS acesso ao `webview` real
                if let Ok(parsed) = serde_json::from_str::<Value>(&msg_body) {
                    if let Some(msg_type) = parsed.get("type").and_then(|t| t.as_str()) {
                        handle_ipc_message(msg_type, &parsed, &ipc_state, &webview, &proxy);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            _ => {}
        }
    });

    Ok(())
}
