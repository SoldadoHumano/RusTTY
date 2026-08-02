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
            Ok(Ok(cmd_out)) if cmd_out.status.success() => {
                return true;
            }
            _ => {}
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    
    false
}
