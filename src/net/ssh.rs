//! Módulo SSH — implementação real com loop bidirecional.
//!
//! Arquitetura:
//!   - `start_ssh_session` roda numa task Tokio dedicada
//!   - Loop principal usa `tokio::select!` para:
//!       * Ler dados DO servidor SSH → envia via `ui_sender`
//!       * Escrever dados DA UI → `channel.data()`
//!   - Suporta autenticação por senha e por chave privada (ED25519/RSA)

use super::{NetworkCommand, NetworkEvent};
use russh::client::{Config, Handler, Session};
use russh::{ChannelId, ChannelMsg, Preferred, kex, cipher, mac};
use russh_keys::key::KeyPair;
use std::sync::Arc;
use tokio::sync::mpsc;
use async_trait::async_trait;
use zeroize::Zeroize;

// ─── Algoritmos Legacy ──────────────────────────────────────────────────────

/// Lista de KEX ampliada que inclui algoritmos legados para compatibilidade
/// com servidores SSH antigos (ex: equipamentos de rede, switches, firewalls).
///
/// # Segurança
/// Os algoritmos `diffie-hellman-group1-sha1` e `diffie-hellman-group14-sha1`
/// são criptograficamente fracos. Essa lista só é usada quando o host possui
/// `legacy_ssh: true` na configuração.
const LEGACY_KEX_ORDER: &[kex::Name] = &[
    kex::CURVE25519,
    kex::CURVE25519_PRE_RFC_8731,
    kex::DH_G16_SHA512,
    kex::DH_G14_SHA256,
    kex::DH_G14_SHA1,
    kex::DH_G1_SHA1,
    kex::EXTENSION_SUPPORT_AS_CLIENT,
    kex::EXTENSION_OPENSSH_STRICT_KEX_AS_CLIENT,
];

/// Constrói a `Config` SSH, opcionalmente habilitando algoritmos legacy.
fn build_ssh_config(legacy_ssh: bool) -> Arc<Config> {
    let mut config = Config::default();
    if legacy_ssh {
        config.preferred = Preferred {
            kex: LEGACY_KEX_ORDER,
            ..Preferred::DEFAULT
        };
        crate::debug_log!("WARN", "SSH Config: modo legacy habilitado (inclui DH-group1-SHA1, DH-group14-SHA1)");
    } else {
        crate::debug_log!("INFO", "SSH Config: modo padrão (algoritmos seguros)");
    }
    Arc::new(config)
}

// ─── Handler ────────────────────────────────────────────────────────────────

/// Handler de eventos do servidor SSH (apenas eventos não solicitados).
/// Dados normais do pty são recebidos via `channel.wait()` no loop principal.
pub struct SshClientHandler;

#[async_trait]
impl Handler for SshClientHandler {
    type Error = russh::Error;

    /// Verificação da chave do servidor.
    ///
    /// # Segurança
    /// Em produção, isso deve comparar contra `~/.ssh/known_hosts`.
    /// Por ora, aceita todas as chaves (equivalente a StrictHostKeyChecking=no).
    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    /// Dados recebidos fora do fluxo principal (ex: banner).
    /// Os dados normais do pty chegam via `channel.wait()`.
    async fn data(
        &mut self,
        _channel: ChannelId,
        _data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ─── Tipos de Autenticação ───────────────────────────────────────────────────

use secrecy::{ExposeSecret, SecretString};

/// Métodos de autenticação suportados pelo cliente SSH.
#[derive(Clone, Debug)]
pub enum SshAuth {
    /// Autenticação por senha simples.
    Password(SecretString),
    /// Autenticação por chave privada carregada de arquivo.
    /// `passphrase` é necessária apenas para chaves criptografadas.
    PrivateKey {
        path: String,
        passphrase: Option<SecretString>,
    },
}

// ─── Sessão Principal ────────────────────────────────────────────────────────

async fn authenticate_session(
    session: &mut russh::client::Handle<SshClientHandler>,
    user: &str,
    auth: &SshAuth,
) -> Result<bool, String> {
    match auth {
        SshAuth::Password(password) => {
            crate::debug_log!("INFO", "Autenticando usuário '{}' por senha", user);
            match session.authenticate_password(user, password.expose_secret()).await {
                Ok(true) => {
                    crate::debug_log!("INFO", "Autenticação por senha bem-sucedida para '{}'", user);
                    Ok(true)
                }
                Ok(false) => {
                    crate::debug_log!("ERROR", "Autenticação por senha falhou para '{}': credenciais inválidas", user);
                    Err("Autenticação SSH falhou: credenciais inválidas.".into())
                }
                Err(e) => {
                    crate::debug_log!("ERROR", "Erro de autenticação por senha para '{}': {}", user, e);
                    Err(format!("Erro de autenticação: {}", e))
                }
            }
        }
        SshAuth::PrivateKey { path, passphrase } => {
            crate::debug_log!("INFO", "Autenticando usuário '{}' por chave privada: {}", user, path);
            let pass_str = passphrase.as_ref().map(|p| p.expose_secret().to_string());
            let key_pair = match load_private_key(path, pass_str.as_deref()) {
                Ok(k) => k,
                Err(e) => {
                    crate::debug_log!("ERROR", "Falha ao carregar chave privada '{}': {}", path, e);
                    return Err(format!("Falha ao carregar chave privada '{}': {}", path, e));
                }
            };

            let key_arc = Arc::new(key_pair);
            match session.authenticate_publickey(user, key_arc).await {
                Ok(true) => {
                    crate::debug_log!("INFO", "Autenticação por chave bem-sucedida para '{}'", user);
                    Ok(true)
                }
                Ok(false) => {
                    crate::debug_log!("ERROR", "Autenticação por chave falhou para '{}'", user);
                    Err("Autenticação por chave falhou.".into())
                }
                Err(e) => {
                    crate::debug_log!("ERROR", "Erro de autenticação por chave para '{}': {}", user, e);
                    Err(format!("Erro de autenticação por chave: {}", e))
                }
            }
        }
    }
}

/// Inicia e gerencia uma sessão SSH completa numa task Tokio assíncrona.
pub async fn start_ssh_session(
    host: String,
    port: u16,
    user: String,
    auth: SshAuth,
    pty_cols: u16,
    pty_rows: u16,
    ui_sender: mpsc::Sender<NetworkEvent>,
    mut command_receiver: mpsc::Receiver<NetworkCommand>,
    bridge_info: Option<Box<(String, u16, String, SshAuth)>>,
    legacy_ssh: bool,
) {
    let config = build_ssh_config(legacy_ssh);
    let addr = if host.contains(':') && !host.starts_with('[') {
        format!("[{}]:{}", host, port)
    } else {
        format!("{}:{}", host, port)
    };

    crate::debug_log!("INFO", "Iniciando sessão SSH para {}@{} (PTY: {}x{}, legacy: {})",
        user, addr, pty_cols, pty_rows, legacy_ssh);

    let mut session = if let Some(bridge) = bridge_info {
        let (bridge_host, bridge_port, bridge_user, bridge_auth) = *bridge;
        let bridge_addr = if bridge_host.contains(':') && !bridge_host.starts_with('[') {
            format!("[{}]:{}", bridge_host, bridge_port)
        } else {
            format!("{}:{}", bridge_host, bridge_port)
        };
        crate::debug_log!("INFO", "Conectando via ponte: {}", bridge_addr);
        let _ = ui_sender.send(NetworkEvent::Disconnected(format!("Conectando via ponte: {}...", bridge_host))).await;

        let mut bridge_session = match russh::client::connect(config.clone(), &bridge_addr, SshClientHandler).await {
            Ok(s) => {
                crate::debug_log!("INFO", "Conexão TCP à ponte {} estabelecida", bridge_addr);
                s
            }
            Err(e) => {
                crate::debug_log!("ERROR", "Falha ao conectar na ponte {}: {}", bridge_addr, e);
                let _ = ui_sender.send(NetworkEvent::Error(format!("Falha ao conectar na ponte {}: {}", bridge_addr, e))).await;
                return;
            }
        };

        if let Err(e) = authenticate_session(&mut bridge_session, &bridge_user, &bridge_auth).await {
            let _ = ui_sender.send(NetworkEvent::Error(format!("Autenticação na ponte falhou: {}", e))).await;
            return;
        }

        let mut channel = match bridge_session.channel_open_direct_tcpip(host.clone(), port as u32, "localhost", 0).await {
            Ok(c) => c,
            Err(e) => {
                crate::debug_log!("ERROR", "Ponte falhou ao rotear para {}: {}", addr, e);
                let _ = ui_sender.send(NetworkEvent::Error(format!("Ponte falhou ao rotear para {}: {}", addr, e))).await;
                return;
            }
        };

        let stream = channel.into_stream();
        match russh::client::connect_stream(config.clone(), stream, SshClientHandler).await {
            Ok(s) => {
                crate::debug_log!("INFO", "Conexão SSH via ponte estabelecida para {}", addr);
                s
            }
            Err(e) => {
                crate::debug_log!("ERROR", "Falha ao conectar via ponte em {}: {}", addr, e);
                let _ = ui_sender.send(NetworkEvent::Error(format!("Falha ao conectar via ponte em {}: {}", addr, e))).await;
                return;
            }
        }
    } else {
        match russh::client::connect(config.clone(), &addr, SshClientHandler).await {
            Ok(s) => {
                crate::debug_log!("INFO", "Conexão TCP a {} estabelecida", addr);
                s
            }
            Err(e) => {
                crate::debug_log!("ERROR", "Falha ao conectar em {}: {}", addr, e);
                let _ = ui_sender.send(NetworkEvent::Error(format!("Falha ao conectar em {}: {}", addr, e))).await;
                return;
            }
        }
    };

    // ── 2. Autenticar no host final ───────────────────────────────────────────
    if let Err(e) = authenticate_session(&mut session, &user, &auth).await {
        let _ = ui_sender.send(NetworkEvent::Error(e)).await;
        return;
    }

    let _ = ui_sender
        .send(NetworkEvent::Connected(format!(
            "SSH: Conectado a {}@{}",
            user, addr
        )))
        .await;
    crate::debug_log!("INFO", "SSH: Autenticado e conectado a {}@{}", user, addr);

    // ── 3. Abrir canal e PTY ─────────────────────────────────────────────────
    let mut channel = match session.channel_open_session().await {
        Ok(c) => c,
        Err(e) => {
            let _ = ui_sender
                .send(NetworkEvent::Error(format!("Falha ao abrir canal SSH: {}", e)))
                .await;
            return;
        }
    };

    // Solicita terminal interativo xterm-256color com o tamanho real do canvas
    crate::debug_log!("INFO", "Solicitando PTY xterm-256color {}x{}", pty_cols, pty_rows);
    if let Err(e) = channel
        .request_pty(false, "xterm-256color", pty_cols as u32, pty_rows as u32, 0, 0, &[])
        .await
    {
        crate::debug_log!("ERROR", "Falha ao alocar PTY: {}", e);
        let _ = ui_sender
            .send(NetworkEvent::Error(format!("Falha ao alocar PTY: {}", e)))
            .await;
        return;
    }

    // Inicia shell interativa
    if let Err(e) = channel.request_shell(true).await {
        let _ = ui_sender
            .send(NetworkEvent::Error(format!("Falha ao iniciar shell: {}", e)))
            .await;
        return;
    }

    // ── 4. Loop Bidirecional ─────────────────────────────────────────────────
    //
    // `tokio::select!` processa concorrentemente:
    //   - Mensagens DO servidor SSH (stdout/stderr do pty) → ui_sender
    //   - Comandos DA UI (input do teclado) → channel.data()
    loop {
        tokio::select! {
            // Dados vindo DO servidor SSH
            msg = channel.wait() => {
                match msg {
                    Some(ChannelMsg::Data { ref data }) => {
                        let _ = ui_sender
                            .send(NetworkEvent::DataReceived(data.to_vec()))
                            .await;
                    }
                    Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                        // stderr do processo remoto
                        let _ = ui_sender
                            .send(NetworkEvent::DataReceived(data.to_vec()))
                            .await;
                    }
                    Some(ChannelMsg::ExitStatus { exit_status }) => {
                        let _ = ui_sender
                            .send(NetworkEvent::Disconnected(format!(
                                "SSH: Processo encerrado com código {}",
                                exit_status
                            )))
                            .await;
                        break;
                    }
                    Some(ChannelMsg::Eof) => {
                        let _ = ui_sender
                            .send(NetworkEvent::Disconnected(
                                "SSH: Servidor encerrou a conexão (EOF).".into(),
                            ))
                            .await;
                        break;
                    }
                    None => {
                        // Canal fechado inesperadamente
                        let _ = ui_sender
                            .send(NetworkEvent::Disconnected(
                                "SSH: Canal fechado.".into(),
                            ))
                            .await;
                        break;
                    }
                    _ => {} // Outros eventos de controle do protocolo
                }
            }

            // Comandos vindos DA UI
            cmd = command_receiver.recv() => {
                match cmd {
                    Some(NetworkCommand::SendData(data)) => {
                        if let Err(e) = channel.data(data.as_ref()).await {
                            let _ = ui_sender
                                .send(NetworkEvent::Error(format!(
                                    "Falha ao enviar dados: {}",
                                    e
                                )))
                                .await;
                            break;
                        }
                    }
                    Some(NetworkCommand::ResizePty { cols, rows }) => {
                        // Resize do terminal quando o usuário redimensiona a janela
                        let _ = channel
                            .window_change(cols as u32, rows as u32, 0, 0)
                            .await;
                    }
                    Some(NetworkCommand::Disconnect) | None => {
                        let _ = channel.eof().await;
                        let _ = ui_sender
                            .send(NetworkEvent::Disconnected("SSH: Desconectado.".into()))
                            .await;
                        break;
                    }
                    // ConnectSsh é ignorado durante sessão ativa
                    Some(NetworkCommand::ConnectSsh { .. }) => {}
                }
            }
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Carrega uma chave privada SSH do disco, descriptografando com passphrase se necessário.
///
/// Suporta formatos: OpenSSH (ED25519, RSA, ECDSA P-256/P-521).
fn load_private_key(path: &str, passphrase: Option<&str>) -> Result<KeyPair, String> {
    let key_str = std::fs::read_to_string(path)
        .map_err(|e| format!("Erro ao ler arquivo: {}", e))?;

    russh_keys::decode_secret_key(&key_str, passphrase)
        .map_err(|e| format!("Erro ao decodificar chave: {}", e))
}
