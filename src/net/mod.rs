//! Módulo de rede — tipos de eventos e comandos compartilhados entre protocolos.
//!
//! `dead_code` é esperado: o backend SSH/Telnet será integrado à UI na próxima fase.
#![allow(dead_code)]

pub mod ssh;
pub mod telnet;
pub mod serial;
pub mod icmp;

// Re-exporta o tipo de autenticação SSH
pub use ssh::SshAuth;

/// Eventos enviados DA thread de rede PARA a interface gráfica (Iced).
///
/// Invariante: todos os `String` são mensagens legíveis para exibir na UI.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Conexão estabelecida com sucesso.
    Connected(String),
    /// Bytes recebidos do host remoto (stdout do pty, dados raw do telnet, etc).
    DataReceived(Vec<u8>),
    /// Conexão encerrada normalmente.
    Disconnected(String),
    /// Erro não-fatal ou fatal que encerrou a conexão.
    Error(String),
}

/// Comandos enviados DA interface (Iced) PARA a thread de rede.
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    /// Inicia uma nova conexão SSH.
    ConnectSsh {
        host: String,
        port: u16,
        user: String,
        auth: SshAuth,
    },
    /// Envia bytes brutos para o host remoto (input de teclado, etc).
    SendData(Vec<u8>),
    /// Notifica o host sobre redimensionamento do terminal.
    ResizePty { cols: u16, rows: u16 },
    /// Solicita desconexão limpa.
    Disconnect,
}

/// Trait base para futuras implementações de protocolo (Telnet, Serial).
pub trait ProtocolConnection {
    // Extensão futura: métodos de status, reconexão, etc.
}
