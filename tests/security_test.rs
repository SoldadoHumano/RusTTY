#[cfg(windows)]
#[test]
fn test_restricted_dacl_denies_vm_read_and_all_access() {
    use windows::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentProcessId, OpenProcess,
        PROCESS_VM_READ, PROCESS_ALL_ACCESS, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_ACCESS_RIGHTS,
    };
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW,
        SetSecurityInfo, SDDL_REVISION_1, SE_KERNEL_OBJECT,
    };
    use windows::Win32::Security::{
        PSECURITY_DESCRIPTOR, DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl,
    };
    use windows::core::PCWSTR;

    unsafe {
        let proc = GetCurrentProcess();
        let pid = GetCurrentProcessId();

        // 1. Antes da DACL, OpenProcess(PROCESS_VM_READ) deve funcionar (mesmo usuário)
        let before = OpenProcess(PROCESS_VM_READ, false, pid);
        assert!(before.is_ok(), "Antes da DACL, OpenProcess(PROCESS_VM_READ) deveria funcionar");
        let _ = windows::Win32::Foundation::CloseHandle(before.unwrap());

        // 2. Aplica a DACL restritiva com SDDL
        // D:P = Protected DACL (SE_DACL_PROTECTED)
        // Concede apenas PROCESS_TERMINATE (0x1), PROCESS_QUERY_LIMITED_INFORMATION (0x1000) e SYNCHRONIZE (0x100000)
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
        assert!(res_sddl.is_ok(), "Falha ao converter SDDL: {:?}", res_sddl);

        let mut dacl_present = windows::Win32::Foundation::BOOL(0);
        let mut p_dacl = std::ptr::null_mut();
        let mut dacl_defaulted = windows::Win32::Foundation::BOOL(0);
        let res_dacl = GetSecurityDescriptorDacl(
            p_sd,
            &mut dacl_present,
            &mut p_dacl,
            &mut dacl_defaulted,
        );
        assert!(res_dacl.is_ok(), "Falha ao extrair DACL: {:?}", res_dacl);

        let res_set = SetSecurityInfo(
            proc,
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(p_dacl),
            None,
        );
        assert!(res_set.is_ok(), "Falha em SetSecurityInfo: {:?}", res_set);

        #[link(name = "kernel32")]
        extern "system" {
            fn LocalFree(hmem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        LocalFree(p_sd.0);

        // 3. APÓS A DACL: OpenProcess com PROCESS_VM_READ (Cheat Engine) DEVE FALHAR com Access is denied (0x80070005)
        let after_vm_read = OpenProcess(PROCESS_VM_READ, false, pid);
        assert!(after_vm_read.is_err(), "OpenProcess(PROCESS_VM_READ) DEVE falhar após a DACL restritiva!");

        // 4. APÓS A DACL: OpenProcess com PROCESS_ALL_ACCESS DEVE FALHAR
        let after_all_access = OpenProcess(PROCESS_ALL_ACCESS, false, pid);
        assert!(after_all_access.is_err(), "OpenProcess(PROCESS_ALL_ACCESS) DEVE falhar após a DACL restritiva!");

        // 5. Direitos essenciais do sistema (SYNCHRONIZE | QUERY_LIMITED) continuam funcionando
        let allowed_rights = PROCESS_ACCESS_RIGHTS(0x00100000 | PROCESS_QUERY_LIMITED_INFORMATION.0);
        let allowed_handle = OpenProcess(allowed_rights, false, pid);
        assert!(allowed_handle.is_ok(), "Direitos essenciais (SYNCHRONIZE | QUERY_LIMITED) devem funcionar");
        let _ = windows::Win32::Foundation::CloseHandle(allowed_handle.unwrap());
    }
}

#[cfg(windows)]
#[test]
fn test_mitigation_policies_windows_8_1() {
    use windows::Win32::System::Threading::{
        SetProcessMitigationPolicy, ProcessStrictHandleCheckPolicy,
    };

    unsafe {
        // Strict handle check policy (Windows 8.1+)
        let strict_flags: u32 = 0x1 | 0x2;
        let res_strict = SetProcessMitigationPolicy(
            ProcessStrictHandleCheckPolicy,
            &strict_flags as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<u32>(),
        );
        assert!(res_strict.is_ok(), "ProcessStrictHandleCheckPolicy deve ser aceito");
    }
}

#[cfg(windows)]
#[test]
fn test_thread_hide_from_debugger() {
    use windows::Win32::System::Threading::GetCurrentThread;

    type NtSetInformationThreadFn = unsafe extern "system" fn(
        thread_handle: windows::Win32::Foundation::HANDLE,
        thread_information_class: u32,
        thread_information: *mut std::ffi::c_void,
        thread_information_length: u32,
    ) -> i32;

    unsafe {
        use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
        use windows::core::PCWSTR;

        let ntdll_name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        let h_ntdll = GetModuleHandleW(PCWSTR(ntdll_name.as_ptr())).unwrap();
        let proc = GetProcAddress(h_ntdll, windows::core::s!("NtSetInformationThread"));
        assert!(proc.is_some(), "NtSetInformationThread deve existir em ntdll.dll");

        let nt_set_info: NtSetInformationThreadFn = std::mem::transmute(proc.unwrap());
        // 0x11 = ThreadHideFromDebugger
        let status = nt_set_info(GetCurrentThread(), 0x11, std::ptr::null_mut(), 0);
        assert_eq!(status, 0, "ThreadHideFromDebugger deve retornar STATUS_SUCCESS (0)");
    }
}

#[cfg(windows)]
#[test]
fn test_direct2d_directwrite_compatibility_under_hardened_process() {
    use windows::Win32::Graphics::Direct2D::{
        D2D1CreateFactory, ID2D1Factory, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    };
    use windows::Win32::Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED,
    };

    let d2d = unsafe {
        D2D1CreateFactory::<ID2D1Factory>(
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
            None,
        )
    };
    assert!(d2d.is_ok(), "Direct2D deve inicializar normalmente sob processo endurecido");

    let dwrite = unsafe {
        DWriteCreateFactory::<IDWriteFactory>(
            DWRITE_FACTORY_TYPE_SHARED,
        )
    };
    assert!(dwrite.is_ok(), "DirectWrite deve inicializar normalmente sob processo endurecido");
}
