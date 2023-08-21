#![cfg_attr(task_scheduler, windows_subsystem = "windows")]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use curl::easy::{Easy, List};
use serde::Deserialize;
use std::{fs::read_to_string, path::PathBuf};

fn main() {
    #[cfg(not(windows))]
    panic!("This program is only intended to run on Windows");
    if !cfg!(task_scheduler) {
        interactive_config();
        return;
    }
    let appender = tracing_appender::rolling::never(app_data(), "heartbeat.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_writer(std::io::stdout)
        .with_level(true)
        .init();
    std::panic::set_hook(Box::new(|i| {
        tracing::error!("PANIC!!! {:?}", i);
    }));
    let settings: Settings =
        toml::from_str(&read_to_string(app_data().join("heartbeat.ini")).unwrap()).unwrap();
    ping(&settings.heartbeat.base_url, &settings.heartbeat.auth_token);
}

#[derive(Deserialize)]
struct Settings {
    heartbeat: SettingsInner,
}

#[derive(Deserialize)]
struct SettingsInner {
    base_url: String,
    auth_token: String,
}

fn interactive_config() {
    let mut buf = String::new();
    let mut prompt = "Server base URL: ";
    loop {
        eprint!("{prompt}");
        std::io::stdin().read_line(&mut buf).unwrap();
        if buf.trim().is_empty() {
            buf.clear();
            continue;
        }
        break;
    }
    let base_url = buf.trim().to_string();
    buf.clear();
    prompt = "Authorization token: ";
    loop {
        eprint!("{prompt}");
        std::io::stdin().read_line(&mut buf).unwrap();
        if buf.trim().is_empty() {
            buf.clear();
            continue;
        }
        break;
    }
    let auth_token = buf.trim();
    if is_stdout_tty() {
        eprintln!("\n");
    }
    println!(
        r#"[heartbeat]
base_url = "{base_url}"
auth_token = "{auth_token}"
"#
    );
    if is_stdout_tty() {
        eprintln!(
            r#"
    Copy the above into {}\heartbeat.ini, and replace the values with your own.
    "#,
            app_data().display()
        );
    }
}

fn app_data() -> PathBuf {
    dirs::data_dir()
        .expect("No data directory found")
        .join("heartbeat")
}

fn ping(server_url: &str, authorization: &str) {
    if is_locked() {
        return;
    }
    if get_idle_time() > 120_000 {
        return;
    }
    tracing::info!("Running heartbeat");
    let mut easy = Easy::new();
    easy.url(&format!("{server_url}/api/beat")).unwrap();
    easy.post(true).unwrap();
    let mut list = List::new();
    list.append(&format!("Authorization: {authorization}"))
        .unwrap();
    easy.http_headers(list).unwrap();
    let mut buf = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    let response = String::from_utf8(buf).unwrap();
    tracing::info!("{response}");
}

#[repr(C)]
struct LastInputInfo {
    cb_size: u32,
    dw_time: u32,
}

#[link(name = "user32")]
extern "system" {
    fn GetLastInputInfo(last_input_info: *mut LastInputInfo) -> bool;
    fn OpenDesktopA(
        lpsz_desktop: *const std::ffi::c_char,
        dw_flags: u32,
        f_inherit: bool,
        dw_desired_access: u32,
    ) -> *mut std::ffi::c_void;
    fn OpenInputDesktop(
        dw_flags: u32,
        f_inherit: bool,
        dw_desired_access: u32,
    ) -> *mut std::ffi::c_void;
    fn SwitchDesktop(h_desktop: *mut std::ffi::c_void) -> bool;
    fn CloseDesktop(h_desktop: *mut std::ffi::c_void) -> bool;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetTickCount() -> u32;
    fn GetConsoleMode(h_console: *mut std::ffi::c_void, lp_mode: *mut u32) -> bool;
    fn GetStdHandle(n_std_handle: u32) -> *mut std::ffi::c_void;
}

const STDOUT: u32 = 4_294_967_285; // (DWORD)-11

fn is_stdout_tty() -> bool {
    let handle = unsafe { GetStdHandle(STDOUT) };
    let mut mode = 0;
    unsafe {
        GetConsoleMode(handle, &mut mode);
    }
    mode != 0
}

fn get_idle_time() -> u32 {
    let mut last_input_info = LastInputInfo {
        cb_size: u32::try_from(std::mem::size_of::<LastInputInfo>()).unwrap(),
        dw_time: 0,
    };
    unsafe {
        GetLastInputInfo(&mut last_input_info);
    }
    let current_time = unsafe { GetTickCount() };
    current_time - last_input_info.dw_time
}

fn is_locked() -> bool {
    const DESKTOP_SWITCHDESKTOP: u32 = 0x0100;
    let mut locked = false;
    unsafe {
        let mut hwnd = OpenInputDesktop(0, false, DESKTOP_SWITCHDESKTOP);
        if hwnd.is_null() {
            // maybe already lcoked?
            hwnd = OpenDesktopA("Default\0".as_ptr().cast(), 0, false, DESKTOP_SWITCHDESKTOP);
        }
        if !hwnd.is_null() {
            if !SwitchDesktop(hwnd) {
                locked = true;
            }
            CloseDesktop(hwnd);
        }
    }
    locked
}
