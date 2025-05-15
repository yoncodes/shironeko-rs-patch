use std::ffi::CString;

use super::{MhyContext, MhyModule, ModuleType};
use crate::util::{import, read_csharp_string};
use anyhow::Result;
use ilhook::x64::Registers;

const WEB_REQUEST_UTILS_MAKE_INITIAL_URL: usize = 0x2FFE190;
const BROWSER_LOAD_URL: usize = 0x3012810;
const SET_REQUEST_HEADER: usize = 0xD9AC00;

pub struct Http;

impl MhyModule for MhyContext<Http> {
    unsafe fn init(&mut self) -> Result<()> {
        self.interceptor.attach(
            self.assembly_base + WEB_REQUEST_UTILS_MAKE_INITIAL_URL,
            on_make_initial_url,
        )?;

        self.interceptor
            .attach(self.assembly_base + BROWSER_LOAD_URL, on_browser_load_url)?;

        self.interceptor.attach(
            self.assembly_base + SET_REQUEST_HEADER,
            on_set_request_header,
        )?;

        Ok(())
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> super::ModuleType {
        ModuleType::Http
    }
}

import!(il2cpp_string_new(cstr: *const u8) -> usize = 0x41A4E0);

unsafe extern "win64" fn on_make_initial_url(reg: *mut Registers, _: usize) {
    let url = read_csharp_string((*reg).rcx as usize);

    const HOST_10800: &[&str] = &[
        "cat-cn-v2.fantanggame.com",
        "cat-cn-veteran.fantanggame.com",
        "operational-event.fantanggame.com",
    ];

    const HOST_10443: &[&str] = &["passport-v2-pc.fantanggame.com"];

    let mut new_url = if HOST_10800.iter().any(|host| url.contains(host)) {
        "http://127.0.0.1:10800".to_string()
    } else if HOST_10443.iter().any(|host| url.contains(host)) {
        "http://127.0.0.1:10443".to_string()
    } else {
        return;
    };

    url.split('/').skip(3).for_each(|s| {
        new_url.push('/');
        new_url.push_str(s);
    });

    if !url.contains("/query_cur_region") {
        println!("Redirect: {url} -> {new_url}");

        let cstr = CString::new(new_url.as_str()).unwrap();
        let new_ptr = il2cpp_string_new(cstr.as_ptr() as *const u8);

        (*reg).rcx = new_ptr as u64;
    }
}

unsafe extern "win64" fn on_browser_load_url(reg: *mut Registers, _: usize) {
    let url = read_csharp_string((*reg).rdx as usize);

    if url == "about:blank" {
        return;
    }

    // Build new_url but don't use it yet
    let mut new_url = String::from("https://127.0.0.1:10443");
    url.split('/').skip(3).for_each(|s| {
        new_url.push('/');
        new_url.push_str(s);
    });

    println!("Browser::LoadURL: {url}"); // → we no longer show the rewritten version

    // Commented out: override the actual URL
    /*
    let cstr = CString::new(new_url.as_str()).unwrap();
    let new_ptr = il2cpp_string_new(cstr.as_ptr() as *const u8);
    (*reg).rdx = new_ptr as u64;
    */
}

unsafe extern "win64" fn on_set_request_header(reg: *mut Registers, _: usize) {
    let key = read_csharp_string((*reg).rdx as usize);
    let value = read_csharp_string((*reg).r8 as usize);

    if key.is_empty() || value.is_empty() {
        return;
    }

    if key == "Host" {
        println!("[SetRequestHeader] Rewriting Host: {value} → 127.0.0.1");
        let new_ptr = il2cpp_string_new(CString::new("127.0.0.1").unwrap().as_ptr() as *const u8);
        (*reg).r8 = new_ptr as u64;
    } else {
        println!("[SetRequestHeader] {key}: {value}");
    }
}
