use std::{collections::HashMap, net::Ipv4Addr};

use anyhow::Result;
use ilhook::x64::Registers;
use std::sync::LazyLock;
use windows::Win32::Networking::WinSock::{AF_INET, SOCKADDR_IN};

use super::{MhyContext, MhyModule, ModuleType};
use crate::util::get_port_map;

pub struct Socket;

impl MhyModule for MhyContext<Socket> {
    unsafe fn init(&mut self) -> Result<()> {
        let addr = self.get_export("Ws2_32.dll", "connect")?;
        self.interceptor.attach(addr, on_connect)?;
        println!("[*] Socket hook attached to connect()");
        Ok(())
    }

    unsafe fn de_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_module_type(&self) -> ModuleType {
        ModuleType::Socket
    }
}

unsafe extern "win64" fn on_connect(reg: *mut Registers, _: usize) {
    let sockaddr_ptr = (*reg).rdx as *mut SOCKADDR_IN;

    if sockaddr_ptr.is_null() {
        return;
    }

    let sockaddr = &mut *sockaddr_ptr;

    if sockaddr.sin_family.0 != AF_INET.0 {
        return;
    }

    let ip = Ipv4Addr::from(u32::from_be(sockaddr.sin_addr.S_un.S_addr));
    let port = u16::from_be(sockaddr.sin_port);

    let key = format!("{ip}:{port}");
    println!("[connect] IP: {ip}, Port: {port}");

    if ip == Ipv4Addr::new(43, 132, 55, 55) && port == 443 {
        println!("↪ Overriding {ip}:{port} → 127.0.0.1:10443 (game.local fallback)");
        sockaddr.sin_addr.S_un.S_addr = u32::from(Ipv4Addr::LOCALHOST).to_be();
        sockaddr.sin_port = 10443u16.to_be();
        return;
    }

    if let Some((redir_ip, redir_port)) = get_target_map().get(&key) {
        println!("Redirecting {key} → {redir_ip}:{redir_port}");

        sockaddr.sin_addr.S_un.S_addr = u32::from(*redir_ip).to_be();
        sockaddr.sin_port = redir_port.to_be();
    } else if let Some(new_port) = get_port_map().get(&port) {
        println!("Redirecting {ip}:{port} → 127.0.0.1:{new_port}");

        sockaddr.sin_addr.S_un.S_addr = u32::from(Ipv4Addr::LOCALHOST).to_be();
        sockaddr.sin_port = new_port.to_be();
    }
}

fn get_target_map() -> &'static HashMap<String, (Ipv4Addr, u16)> {
    static TARGET_MAP: LazyLock<HashMap<String, (Ipv4Addr, u16)>> = LazyLock::new(|| {
        HashMap::from([
            ("8.153.14.167:22008".into(), (Ipv4Addr::LOCALHOST, 22008)),
            ("8.153.12.82:22000".into(), (Ipv4Addr::LOCALHOST, 22000)),
        ])
    });
    &TARGET_MAP
}
