use std::ffi::CString;

use super::{MhyContext, MhyModule, ModuleType};
use crate::util::import;
use anyhow::Result;
use ilhook::x64::Registers;

// RVA for il2cpp_string_new_utf16
const IL2CPP_STRING_NEW_UTF16: usize = 0x41A6A0;

pub struct Network;

impl MhyModule for MhyContext<Network> {
    unsafe fn init(&mut self) -> Result<()> {
        self.interceptor.attach(
            self.assembly_base + IL2CPP_STRING_NEW_UTF16,
            on_il2cpp_string_new_utf16,
        )
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> ModuleType {
        ModuleType::Network
    }
}

// il2cpp_string_new (UTF-8 → System.String*)
import!(il2cpp_string_new(cstr: *const u8) -> usize = 0x41A4E0);

static TARGET_IPS: &[&str] = &[
    "43.132.55.55",
    "101.132.135.131",
    "101.132.132.81",
    "163.181.60.219",
    "163.181.60.220",
    "163.181.60.221",
    "163.181.60.222",
    "163.181.60.223",
    "163.181.60.224",
    "163.181.60.225",
    "47.246.22.201",
    "47.246.22.203",
    "47.246.22.204",
    "47.246.22.205",
    "47.246.22.206",
    "47.246.22.207",
    "47.246.22.208",
];

static REPLACEMENT: &str = "127.0.0.1";

unsafe extern "win64" fn on_il2cpp_string_new_utf16(reg: *mut Registers, _: usize) {
    let utf16_ptr = (*reg).rcx as *const u16;
    let utf16_len = (*reg).rdx as u32;

    if utf16_ptr.is_null() || utf16_len == 0 || utf16_len > 4096 {
        return;
    }

    let slice = std::slice::from_raw_parts(utf16_ptr, utf16_len as usize);
    let original = String::from_utf16_lossy(slice);

    for ip in TARGET_IPS {
        if original.contains(ip) {
            let replaced = original.replace(ip, REPLACEMENT);
            println!("[IPReplace] \"{original}\" → \"{replaced}\"");

            let cstr = CString::new(replaced).unwrap();
            let new_ptr = il2cpp_string_new(cstr.as_ptr() as *const u8);

            (*reg).rax = new_ptr as u64;
            return;
        }
    }
}
