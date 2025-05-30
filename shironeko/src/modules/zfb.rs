use anyhow::Result;
use ilhook::x64::Registers;
use std::ffi::CStr;

use super::{MhyContext, MhyModule, ModuleType};

pub struct Zfb;

impl MhyModule for MhyContext<Zfb> {
    unsafe fn init(&mut self) -> Result<()> {
        if let Ok(addr) = self.get_export("ZFProxyWeb.dll", "zfb_goToURL") {
            self.interceptor.attach(addr, on_zfb_go_to_url)?;
            println!("[*] zfb_goToURL hook installed");
        } else {
            println!("[!] Failed to locate zfb_goToURL export");
        }
        Ok(())
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> ModuleType {
        ModuleType::Zfb
    }
}

unsafe extern "win64" fn on_zfb_go_to_url(reg: *mut Registers, _: usize) {
    let browser_id = (*reg).rcx as i32;
    let url_ptr = (*reg).rdx as *const u8;
    if url_ptr.is_null() {
        return;
    }

    if let Ok(url) = CStr::from_ptr(url_ptr as _).to_str() {
        println!("[zfb_goToURL] browserId={browser_id}, url={url}");
    }
}
