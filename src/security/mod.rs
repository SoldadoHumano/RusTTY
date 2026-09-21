//! Módulo de segurança, endurecimento de processo e proteção de memória no Windows.
//!
//! # Recursos de Segurança
//! - **DACL Restritiva**: Configura um descritor de segurança protegido via SDDL no processo atual,
//!   revogando direitos como `PROCESS_VM_READ` (leitura de memória), `PROCESS_VM_WRITE` (gravação),
//!   `PROCESS_CREATE_THREAD` (injeção remota de DLLs) e `PROCESS_ALL_ACCESS` para ferramentas
//!   de análise em modo usuário (como o Cheat Engine).
//! - **Políticas de Mitigação de Processo (`SetProcessMitigationPolicy`)**: Ativa proteções contra
//!   adulteração de handles e restrições de código nativas do Windows 8.1 em diante.
//! - **Ocultação de Threads (`ThreadHideFromDebugger`)**: Oculta as threads do processo de depuradores
//!   via chamada nativa em `ntdll.dll`.

#[cfg(windows)]
pub mod windows_security {
    use std::ffi::c_void;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SetSecurityInfo,
        SDDL_REVISION_1, SE_KERNEL_OBJECT,
    };
    use windows::Win32::Security::{
        GetSecurityDescriptorDacl, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    };
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    use windows::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentThread, SetProcessMitigationPolicy,
        ProcessStrictHandleCheckPolicy,
    };

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(hmem: *mut c_void) -> *mut c_void;
        fn SetErrorMode(u_mode: u32) -> u32;
    }

    type NtSetInformationThreadFn = unsafe extern "system" fn(
        thread_handle: windows::Win32::Foundation::HANDLE,
        thread_information_class: u32,
        thread_information: *mut c_void,
        thread_information_length: u32,
    ) -> i32;

    /// Aplica o conjunto completo de endurecimento de segurança do processo no Windows.
    ///
    /// Deve ser chamado no início de `main()` antes de qualquer operação ou alocação sensível.
    pub fn apply_process_security() {
        // Suprime popups intrusivos de erro crítico do Windows para drivers/subssistemas
        unsafe {
            const SEM_FAILCRITICALERRORS: u32 = 0x0001;
            SetErrorMode(SEM_FAILCRITICALERRORS);
        }

        // 1. Aplica políticas de mitigação do Windows 8.1+
        apply_process_mitigations();

        // 2. Aplica DACL restritiva para bloquear inspeção de memória por ferramentas externas
        apply_restricted_dacl();

        // 3. Oculta a thread principal contra depuradores
        hide_current_thread_from_debugger();
    }

    /// Configura uma DACL (Discretionary Access Control List) restritiva no processo atual.
    ///
    /// # Permissões Concedidas (Apenas Essenciais):
    /// - `0x00000001`: `PROCESS_TERMINATE` (permite que o sistema finalize o processo se necessário)
    /// - `0x00001000`: `PROCESS_QUERY_LIMITED_INFORMATION` (necessário para Task Manager, WER e WebView2)
    /// - `0x00100000`: `SYNCHRONIZE` (necessário para `WaitForSingleObject` entre processos pai/filho)
    ///
    /// # Permissões Negadas/Revogadas:
    /// - `PROCESS_VM_READ` (0x0010) — Cheat Engine / ReadProcessMemory falham com `Access is denied (5)`
    /// - `PROCESS_VM_WRITE` (0x0020) — WriteProcessMemory falha
    /// - `PROCESS_VM_OPERATION` (0x0008) — VirtualProtectEx / VirtualAllocEx falham
    /// - `PROCESS_CREATE_THREAD` (0x0002) — CreateRemoteThread falha (impede injeção de DLLs)
    /// - `PROCESS_DUP_HANDLE` (0x0040) — Impede duplicação indevida de handles
    /// - `PROCESS_ALL_ACCESS` (0x1FFFFF) — Falha sumariamente
    pub fn apply_restricted_dacl() {
        unsafe {
            let proc = GetCurrentProcess();

            // SDDL:
            // D: = DACL
            // P = Protegido contra herança (SE_DACL_PROTECTED)
            // (A;;0x101401;;;WD) = Allow Everyone (WD) direitos mínimos essenciais (0x101401: TERMINATE, QUERY_INFO, QUERY_LIMITED, SYNCHRONIZE)
            // (A;;0x101401;;;SY) = Allow Local SYSTEM (SY) direitos mínimos essenciais
            let sddl: Vec<u16> = "D:P(A;;0x101401;;;WD)(A;;0x101401;;;SY)"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut p_sd = PSECURITY_DESCRIPTOR::default();
            let res_sddl = ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(sddl.as_ptr()),
                SDDL_REVISION_1,
                &mut p_sd,
                None,
            );

            if let Err(e) = res_sddl {
                crate::debug_log!("ERROR", "Segurança Win32: Falha ao converter SDDL: {:?}", e);
                return;
            }

            if p_sd.0.is_null() {
                crate::debug_log!("ERROR", "Segurança Win32: Ponteiro de descritor de segurança nulo");
                return;
            }

            let mut dacl_present = BOOL(0);
            let mut p_dacl = std::ptr::null_mut();
            let mut dacl_defaulted = BOOL(0);

            let res_dacl = GetSecurityDescriptorDacl(
                p_sd,
                &mut dacl_present,
                &mut p_dacl,
                &mut dacl_defaulted,
            );

            if res_dacl.is_ok() && dacl_present.as_bool() && !p_dacl.is_null() {
                let res_set = SetSecurityInfo(
                    proc,
                    SE_KERNEL_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(p_dacl),
                    None,
                );

                if res_set.is_ok() {
                    crate::debug_log!(
                        "INFO",
                        "Segurança Win32: DACL restritiva aplicada com sucesso no processo (PID: {})",
                        std::process::id()
                    );
                } else {
                    crate::debug_log!(
                        "WARN",
                        "Segurança Win32: SetSecurityInfo retornou aviso: {:?}",
                        res_set
                    );
                }
            }

            // Libera a memória alocada pelo ConvertStringSecurityDescriptor
            LocalFree(p_sd.0);
        }
    }

    /// Aplica políticas de mitigação nativas do Windows (`SetProcessMitigationPolicy`).
    ///
    /// Compatível com Windows 8.1 em diante com tratamento de erro resiliente.
    pub fn apply_process_mitigations() {
        unsafe {
            // 1. ProcessStrictHandleCheckPolicy (Windows 8.1+)
            // bit 0 = RaiseExceptionOnInvalidHandleReference (1)
            // bit 1 = HandleExceptionsPermanentlyEnabled (2)
            let strict_handle_flags: u32 = 0x1 | 0x2;
            let res_handle = SetProcessMitigationPolicy(
                ProcessStrictHandleCheckPolicy,
                &strict_handle_flags as *const _ as *const c_void,
                std::mem::size_of::<u32>(),
            );
            if res_handle.is_ok() {
                crate::debug_log!("INFO", "Segurança Win32: ProcessStrictHandleCheckPolicy ativado");
            } else {
                crate::debug_log!("DEBUG", "Segurança Win32: ProcessStrictHandleCheckPolicy não suportado ou erro: {:?}", res_handle);
            }
        }
    }

    /// Oculta a thread atual contra anexação e inspeção de depuradores via `NtSetInformationThread`.
    ///
    /// Utiliza a classe nativa `ThreadHideFromDebugger (0x11)`.
    pub fn hide_current_thread_from_debugger() {
        unsafe {
            let ntdll_name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
            if let Ok(h_ntdll) = GetModuleHandleW(PCWSTR(ntdll_name.as_ptr())) {
                if let Some(proc) = GetProcAddress(h_ntdll, windows::core::s!("NtSetInformationThread")) {
                    let nt_set_info: NtSetInformationThreadFn = std::mem::transmute(proc);
                    // 0x11 = ThreadHideFromDebugger
                    let status = nt_set_info(GetCurrentThread(), 0x11, std::ptr::null_mut(), 0);
                    if status == 0 {
                        crate::debug_log!("INFO", "Segurança Win32: Thread ocultada de depuradores (ThreadHideFromDebugger)");
                    } else {
                        crate::debug_log!("DEBUG", "Segurança Win32: NtSetInformationThread status: 0x{:08X}", status);
                    }
                }
            }
        }
    }
}

/// Ponto de entrada multiplataforma para aplicação das proteções de segurança de processo.
pub fn apply_process_security() {
    #[cfg(windows)]
    windows_security::apply_process_security();
}

/// Oculta uma nova thread de depuradores (para ser chamada em workers ou threads secundárias).
#[allow(dead_code)]
pub fn hide_current_thread_from_debugger() {
    #[cfg(windows)]
    windows_security::hide_current_thread_from_debugger();
}
