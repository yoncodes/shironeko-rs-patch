use super::{MhyContext, MhyModule, ModuleType};
use anyhow::Result;
use ilhook::x64::Registers;

// RVA for: UnityEngine::Networking::CertificateHandler::ValidateCertificateNative
const VALIDATE_CERT_NATIVE_RVA: usize = 0x2FF6B60;

pub struct CertBypass;

impl MhyModule for MhyContext<CertBypass> {
    unsafe fn init(&mut self) -> Result<()> {
        let target = self.assembly_base + VALIDATE_CERT_NATIVE_RVA;
        self.interceptor
            .replace(target, CertBypass::bypass_cert_check)?;
        println!("[*] CertBypass hook installed at 0x{:X}", target);
        Ok(())
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> ModuleType {
        ModuleType::CertBypass
    }
}

impl CertBypass {
    /// Correct signature for a RetnRoutine
    unsafe extern "win64" fn bypass_cert_check(
        _reg: *mut Registers,
        _ret_addr: usize,
        _arg_count: usize,
    ) -> usize {
        println!("[CertBypass] ValidateCertificateNative called — bypassing");
        1 // return true
    }
}
