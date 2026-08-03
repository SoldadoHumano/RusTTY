use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Realiza até 3 tentativas de ping no IP/Host fornecido.
/// Retorna `true` se ao menos uma tentativa for bem-sucedida, `false` caso contrário.
pub async fn check_icmp(address: &str) -> bool {
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    for _ in 0..3 {
        let addr = address.to_string();
        let output = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new("ping");
            
            #[cfg(target_os = "windows")]
            {
                cmd.arg("-n")
                   .arg("1")
                   .arg("-w")
                   .arg("1000")
                   .arg(&addr)
                   .creation_flags(CREATE_NO_WINDOW);
            }

            #[cfg(not(target_os = "windows"))]
            {
                cmd.arg("-c")
                   .arg("1")
                   .arg("-W")
                   .arg("1")
                   .arg(&addr);
            }

            cmd.output()
        })
        .await;

        match output {
            Ok(Ok(cmd_out)) => {
                // No Windows, falhas como "Destination host unreachable" ou "TTL expired" 
                // podem retornar código de saída 0. Precisamos validar a saída de fato.
                let stdout = String::from_utf8_lossy(&cmd_out.stdout).to_lowercase();
                
                if cmd_out.status.success() {
                    let success = stdout.lines().any(|line| {
                        let is_reply = line.contains("reply from") 
                            || line.contains("resposta de") 
                            || line.contains("respuesta desde") 
                            || line.contains("bytes from");
                            
                        let has_metric = line.contains("ttl=") 
                            || line.contains("time=") || line.contains("time<") 
                            || line.contains("tempo=") || line.contains("tempo<") 
                            || line.contains("tiempo=") || line.contains("tiempo<");
                            
                        let is_error = line.contains("unreachable") 
                            || line.contains("inacessível") 
                            || line.contains("inaccesible") 
                            || line.contains("expired") 
                            || line.contains("esgotado") 
                            || line.contains("expirado");
                        
                        is_reply && has_metric && !is_error
                    });

                    if success {
                        return true;
                    }
                }
            }
            _ => {}
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    
    false
}
