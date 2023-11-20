// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Pinging the server.

use curl::easy::{Easy, List};
use windows_sys::Win32::{
    System::{
        StationsAndDesktops::{CloseDesktop, OpenDesktopW, OpenInputDesktop, SwitchDesktop, DESKTOP_SWITCHDESKTOP},
        SystemInformation::GetTickCount,
    },
    UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
};

/// Pings the server at `server_url` with the given `authorization` header.
///
/// # Errors
///
/// This function will return an error if there is a network error, the server is unreachable,
/// or [`curl`] otherwise returns an error.
pub fn ping(server_url: &str, authorization: &str) -> Result<(), curl::Error> {
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
    list.append(&format!("Authorization: {authorization}"))?;
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
    let response = String::from_utf8_lossy(&buf);
    tracing::info!("{response}");
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn get_idle_time() -> u32 {
    let mut last_input_info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        GetLastInputInfo(&mut last_input_info);
    }
    let current_time = unsafe { GetTickCount() };
    current_time - last_input_info.dwTime
}

fn is_locked() -> bool {
    let mut locked = false;
    unsafe {
        let mut hwnd = OpenInputDesktop(0, 0, DESKTOP_SWITCHDESKTOP);
        if hwnd == 0 {
            hwnd = OpenDesktopW(
                "Default\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
                0,
                0,
                DESKTOP_SWITCHDESKTOP,
            );
        }
        if hwnd != 0 {
            if SwitchDesktop(hwnd) == 0 {
                locked = true;
            }
            CloseDesktop(hwnd);
        }
    }
    locked
}
