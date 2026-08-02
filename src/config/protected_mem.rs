use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[cfg(windows)]
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectMemory, CryptUnprotectMemory, CRYPTPROTECTMEMORY_SAME_PROCESS,
};

/// Armazena dados criptografados em memória usando a DPAPI do Windows.
#[derive(Clone)]
pub struct ProtectedMemory {
    encrypted_buffer: Vec<u8>,
    original_len: usize,
}

impl ProtectedMemory {
    /// Cria uma nova instância a partir de texto plano e criptografa a memória imediatamente.
    pub fn new(plaintext: &str) -> Result<Self, String> {
        let original_len = plaintext.len();
        
        // O Windows exige que o tamanho do buffer seja múltiplo de 16 bytes.
        let padding = 16 - (original_len % 16);
        let padded_len = if padding == 16 && original_len > 0 { original_len } else { original_len + padding };
        let padded_len = std::cmp::max(padded_len, 16); // Mínimo 16 bytes
        
        let mut buffer = vec![0u8; padded_len];
        buffer[..original_len].copy_from_slice(plaintext.as_bytes());

        #[cfg(windows)]
        {
            let success = unsafe {
                CryptProtectMemory(
                    buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    buffer.len() as u32,
                    CRYPTPROTECTMEMORY_SAME_PROCESS,
                )
            };
            if success == 0 {
                return Err("Falha ao proteger memória com DPAPI".to_string());
            }
        }
        
        Ok(Self {
            encrypted_buffer: buffer,
            original_len,
        })
    }

    /// Descriptografa temporariamente e retorna um SecretString.
    /// O SecretString cuidará de realizar o zeroing automático no final do escopo de uso.
    pub fn unprotect(&self) -> Result<SecretString, String> {
        if self.original_len == 0 {
            return Ok(SecretString::new(String::new()));
        }

        let mut temp_buffer = self.encrypted_buffer.clone();

        #[cfg(windows)]
        {
            let success = unsafe {
                CryptUnprotectMemory(
                    temp_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    temp_buffer.len() as u32,
                    CRYPTPROTECTMEMORY_SAME_PROCESS,
                )
            };
            if success == 0 {
                return Err("Falha ao desproteger memória com DPAPI".to_string());
            }
        }
        
        // Extrair apenas o texto real (sem o padding)
        let plaintext_slice = &temp_buffer[..self.original_len];
        let plaintext = String::from_utf8(plaintext_slice.to_vec())
            .map_err(|_| "Falha ao decodificar UTF-8 no buffer protegido".to_string())?;
        
        // Zeroiza o buffer temporário
        use zeroize::Zeroize;
        temp_buffer.zeroize();
        
        Ok(SecretString::new(plaintext))
    }
}

// Para evitar que a senha apareça em logs acidentalmente
impl fmt::Debug for ProtectedMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[PROTECTED_MEMORY]")
    }
}

impl Serialize for ProtectedMemory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Durante a serialização, precisamos expor o dado por um breve instante para o JSON.
        match self.unprotect() {
            Ok(secret) => serializer.serialize_str(secret.expose_secret()),
            Err(_) => Err(serde::ser::Error::custom("Falha ao descriptografar para serialização")),
        }
    }
}

impl<'de> Deserialize<'de> for ProtectedMemory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // A struct SecretString customizada do secrecy implementa um deserialize
        // que imediatamente zera as alocações temporárias na heap quando dropado.
        let secret = SecretString::deserialize(deserializer)?;
        ProtectedMemory::new(secret.expose_secret())
            .map_err(|e| serde::de::Error::custom(e))
    }
}
