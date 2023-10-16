// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Pining the server.

use curl::easy::{Easy, List};

/// Errors that can occur while pinging the server.
#[derive(Debug)]
pub enum Error {
    /// `curl` returned an error.
    Curl(curl::Error),
    /// Technically Infallible, but if the server sends invalid UTF-8 we return this.
    Utf8(std::string::FromUtf8Error),
}

impl From<curl::Error> for Error {
    fn from(e: curl::Error) -> Self {
        Self::Curl(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8(e)
    }
}

/// Pings the server at `server_url` with the given `authorization` header.
///
/// # Errors
///
/// This function will return an error if there is a network error, the server is unreachable,
/// or `curl` otherwise returns an error, or the server returns invalid UTF-8.
pub fn ping(server_url: &str, authorization: &str, device: &str) -> Result<(), Error> {
    if is_locked() {
        return Ok(());
    }
    if get_idle_time() > 120_000 {
        return Ok(());
    }
    tracing::info!("Running heartbeat");
    let mut easy = Easy::new();
    easy.url(&format!("{server_url}/api/beat"))?;
    easy.post(true)?;
    let mut list = List::new();
    list.append(&format!("Auth: {authorization}"))?;
    list.append(&format!("Device: {device}"))?;
    easy.http_headers(list)?;
    let mut buf = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    let response = String::from_utf8(buf)?;
    tracing::info!("{response}");
    Ok(())
}

#[link(name = "kernel32")]
extern "system" {
    // ref: https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount
    fn GetTickCount() -> u32;
}

// ref: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-lastinputinfo
#[repr(C)]
struct LastInputInfo {
    cb_size: u32,
    dw_time: u32,
}

#[link(name = "user32")]
extern "system" {
    // ref: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getlastinputinfo
    fn GetLastInputInfo(last_input_info: *mut LastInputInfo) -> bool;
    // ref: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-opendesktopa
    fn OpenDesktopA(
        lpsz_desktop: *const std::ffi::c_char,
        dw_flags: u32,
        f_inherit: bool,
        dw_desired_access: u32,
    ) -> *mut std::ffi::c_void;
    // ref: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openinputdesktop
    fn OpenInputDesktop(
        dw_flags: u32,
        f_inherit: bool,
        dw_desired_access: u32,
    ) -> *mut std::ffi::c_void;
    // ref: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-switchdesktop
    fn SwitchDesktop(h_desktop: *mut std::ffi::c_void) -> bool;
    // ref: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-closedesktop
    fn CloseDesktop(h_desktop: *mut std::ffi::c_void) -> bool;
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
    // ref: https://learn.microsoft.com/en-us/windows/win32/winstation/desktop-security-and-access-rights
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
