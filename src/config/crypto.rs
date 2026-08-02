//! Módulo de criptografia AES-256-GCM para proteção do config em disco.
//!
//! # Segurança
//! - A chave é gerada de forma 100% aleatória na primeira execução.
//! - A chave é armazenada de forma segura no Windows Credential Manager (DPAPI) via `keyring`.
//! - O buffer de plaintext é encapsulado em `Zeroizing<Vec<u8>>` e zerizado na memória
//!   automaticamente ao sair do escopo.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key,
};
use keyring::Entry;
use rand::RngCore;
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "RusTTY_SecureStore";
const KEYRING_USER: &str = "RusTTY_MasterKey";

/// Recupera a chave mestra do OS Keychain (Credential Manager).
/// Se a chave não existir, gera uma nova chave de 32 bytes de alta entropia
/// e salva no Keychain de forma segura.
fn get_or_create_master_key() -> Key<Aes256Gcm> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER).expect("Falha ao inicializar Keyring");

    let key_bytes = match entry.get_password() {
        Ok(hex_key) => {
            // Decodifica a chave em hex de volta para bytes
            hex::decode(&hex_key).expect("Chave corrompida no cofre do sistema")
        }
        Err(keyring::Error::NoEntry) => {
            // Primeira execução: não existe chave no cofre. Vamos gerar uma.
            let mut new_key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut new_key);
            
            let hex_key = hex::encode(new_key);
            entry.set_password(&hex_key).expect("Falha ao salvar a chave no cofre do sistema. Verifique as permissões do Windows Credential Manager.");
            
            new_key.to_vec()
        }
        Err(e) => {
            panic!("Erro crítico ao acessar o cofre de credenciais do sistema: {:?}", e);
        }
    };

    // Copia os bytes decodificados para o formato da chave AES e os protege com Zeroizing
    let mut key_material = Zeroizing::new([0u8; 32]);
    key_material.copy_from_slice(&key_bytes[0..32]);

    *Key::<Aes256Gcm>::from_slice(key_material.as_ref())
}

/// Criptografa dados em texto plano.
///
/// # Formato de saída
/// `[Nonce (12 bytes)] || [Ciphertext (N + 16 bytes tag)]`
pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key = get_or_create_master_key();
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Erro de criptografia: {:?}", e))?;

    let mut final_data = nonce.to_vec();
    final_data.extend(ciphertext);
    Ok(final_data)
}

/// Descriptografa dados (formato `Nonce || Ciphertext`).
///
/// # Retorno
/// O plaintext é encapsulado em `Zeroizing<Vec<u8>>` para garantir que
/// os bytes sejam zerorizados ao sair do escopo do chamador.
///
/// # Erros
/// Falha se a máquina for diferente da que criptografou (chave diverge),
/// ou se os dados estiverem corrompidos (autenticação GCM falha).
pub fn decrypt_data(encrypted_data: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
    if encrypted_data.len() < 12 {
        return Err("Dados corrompidos ou inválidos.".to_string());
    }

    let key = get_or_create_master_key();
    let cipher = Aes256Gcm::new(&key);

    let nonce = Nonce::from_slice(&encrypted_data[0..12]);
    let ciphertext = &encrypted_data[12..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| {
            "Falha na descriptografia! A chave mestra não foi encontrada ou o arquivo está corrompido."
                .to_string()
        })?;

    // Encapsula em Zeroizing — memória zerizada ao sair do escopo do chamador
    Ok(Zeroizing::new(plaintext))
}
