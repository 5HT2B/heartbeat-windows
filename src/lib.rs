// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # heartbeat
//! A [heartbeat](https://github.com/lmaotrigine/heartbeat) client for Windows.
//!
//! This library contains common code used in all three binaries.
//!
//! It doesn't have a `#![forbid(unsafe_code)]` because we call into the Windows API directly.

#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs
)]

#[cfg(not(target_os = "windows"))]
compile_error!("This crate only supports Windows targets");

#[cfg(feature = "serde")]
use serde::Deserialize;

#[cfg(feature = "task_runner")]
pub mod ping;
pub mod tasks;

/// Prints a prompt to stderr and reads "y" or "n" from stdin.
///
/// Input has to be y/n, case-sensitive, yes/no won't work.
///
/// # Errors
///
/// This function will return an error if reading from stdin fails.
pub fn read_yes_no(prompt: &str, default: Option<bool>) -> std::io::Result<bool> {
    let mut buf = String::new();
    let to_append = match default {
        None => "",
        Some(true) => " [Y/n]",
        Some(false) => " [y/N]",
    };
    let prompt = format!("{prompt}{to_append}: ");
    loop {
        eprint!("{prompt}");
        std::io::stdin().read_line(&mut buf)?;
        buf = buf.trim().to_lowercase();
        if buf.is_empty() {
            match default {
                None => {
                    eprintln!("{prompt}");
                    buf.clear();
                    continue;
                }
                Some(default) => return Ok(default),
            }
        } else if buf.is_empty() || !matches!(buf.as_str(), "y" | "n") {
            buf.clear();
            eprintln!("Invalid input, please try again");
            eprint!("{prompt}");
            continue;
        }
        break;
    }
    Ok(buf == "y")
}

/// Settings for the client. This is read from `heartbeat.ini`.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
pub struct Settings {
    /// Settings for the client. This is read from `heartbeat.ini`.
    pub heartbeat: SettingsInner,
}

/// Settings for the client. This is read from `heartbeat.ini`.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
pub struct SettingsInner {
    /// The base URL of the server.
    pub base_url: String,
    /// The authorization token to use.
    pub auth_token: String,
    /// The `Device` header to use.
    pub device: String,
}

/// Interactively prompts the user for the server URL and authorization token.
///
/// # Errors
///
/// This function will return an error if reading from stdin fails.
#[cfg(feature = "config")]
pub fn interactive_config() -> std::io::Result<()> {
    let mut buf = String::new();
    let mut prompt = "Server base URL: ";
    loop {
        eprint!("{prompt}");
        std::io::stdin().read_line(&mut buf)?;
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
        std::io::stdin().read_line(&mut buf)?;
        if buf.trim().is_empty() {
            buf.clear();
            continue;
        }
        break;
    }
    let auth_token = buf.trim().to_string();
    buf.clear();
    prompt = "Device name: ";
    loop {
        eprint!("{prompt}");
        std::io::stdin().read_line(&mut buf)?;
        if buf.trim().is_empty() {
            buf.clear();
            continue;
        }
        break;
    }
    let device_name = buf.trim();
    if is_stdout_tty() {
        eprintln!("\n");
    }
    println!(
        r#"[heartbeat]
base_url = "{base_url}"
auth_token = "{auth_token}"
device = "{device_name}"
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
    Ok(())
}

/// Returns the path to `%APPDATA%\heartbeat`.
///
/// # Panics
///
/// This function will panic if `%APPDATA%` doesn't exist.
#[cfg(feature = "dirs")]
#[must_use]
pub fn app_data() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("No data directory found")
        .join("heartbeat")
}

#[link(name = "kernel32")]
extern "system" {
    // ref: https://docs.microsoft.com/en-us/windows/console/getconsolemode
    fn GetConsoleMode(h_console: *mut std::ffi::c_void, lp_mode: *mut u32) -> bool;
    // ref: https://docs.microsoft.com/en-us/windows/console/getstdhandle
    fn GetStdHandle(n_std_handle: u32) -> *mut std::ffi::c_void;
}

const STDOUT: u32 = 4_294_967_285; // (DWORD)-11

/// Returns whether stdout is a TTY.
#[must_use]
pub fn is_stdout_tty() -> bool {
    let handle = unsafe { GetStdHandle(STDOUT) };
    let mut mode = 0;
    unsafe {
        GetConsoleMode(handle, &mut mode);
    }
    mode != 0
}
