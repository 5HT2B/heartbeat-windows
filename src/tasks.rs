// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Task XML generation and registration.

use anyhow::{Context, Result};
use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    io,
    path::PathBuf,
    process::Command,
    ptr,
};

use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{Authorization::ConvertSidToStringSidW, LookupAccountNameW},
    System::WindowsProgramming::GetUserNameW,
};

#[must_use]
/// Formats the included XML template with the current user's username and SID.
///
/// # Panics
///
/// This function dies if it can't get the current user's name or security identifier,
/// or if the `heartbeat-task` executable isn't present alongside the current executable.
pub fn generate_xml() -> String {
    let user = get_current_user().expect("something went wrong lol");
    format_task(&user.username, &user.sid)
}

/// A Windows user. Well, the important bits that we care about.
#[derive(Debug)]
pub struct User {
    username: String,
    sid: String,
}

/// Gets the current logged in user's fully qualified username and SID.
///
/// These values are then plugged into the task XML template so
/// it's evident who created it.
///
/// # Errors
///
/// Any errors raised here relate to the Windows API and system calls
/// thereof. Any I/O errors are propagated as-is, without wrapping,
/// since these shouldn't fail on a supported Windows version on
/// modern hardware.
pub fn get_current_user() -> io::Result<User> {
    let name = username()?;
    let name_buf = name.encode_utf16().collect::<Vec<_>>();
    let mut sid_buf = Vec::<u16>::new();
    let mut ref_name_buf = Vec::new();
    let mut capacity = 0;
    let mut ref_name_cap = 0;
    loop {
        unsafe {
            LookupAccountNameW(
                ptr::null(),
                name_buf.as_ptr(),
                sid_buf.as_mut_ptr().cast(),
                &mut capacity,
                ref_name_buf.as_mut_ptr(),
                &mut ref_name_cap,
                &mut 0,
            );
        }
        if capacity == 0 || ref_name_cap == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut flag = false;
        for (l, buf) in [(capacity, &mut sid_buf), (ref_name_cap, &mut ref_name_buf)] {
            let length = l as usize;
            if let Some(mut additional) = length.checked_sub(buf.capacity()) {
                debug_assert_ne!(0, additional);
                capacity += 2;
                additional += 2;
                buf.reserve(additional);
                flag = true;
            }
        }
        if flag {
            continue;
        }
        unsafe {
            sid_buf.set_len(wcslen(sid_buf.as_ptr()));
            ref_name_buf.set_len(wcslen(ref_name_buf.as_ptr()));
        }
        break;
    }
    let psid = sid_buf.as_mut_ptr().cast();
    let mut raw_string_sid = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(psid, &mut raw_string_sid) } == 0 || raw_string_sid.is_null() {
        return Err(io::Error::last_os_error());
    }
    let len = unsafe { wcslen(raw_string_sid) };
    let sid_string = unsafe {
        String::from_utf16(std::slice::from_raw_parts(raw_string_sid, len)).map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "SID should be valid UTF-8, but isn't. This is weird.",
            )
        })?
    };
    // I'm C programmer enough to catch this mem leak but that's about it.
    unsafe { LocalFree(raw_string_sid.cast()) };
    let username = format!(
        "{}\\{name}",
        String::from_utf16(&ref_name_buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 detected in reference domain name."))?,
    );
    Ok(User {
        username,
        sid: sid_string,
    })
}

fn username() -> io::Result<String> {
    let mut name_buf = Vec::new();
    let mut capacity = 0;
    loop {
        unsafe {
            GetUserNameW(name_buf.as_mut_ptr(), &mut capacity);
        }
        if capacity == 0 {
            return Err(io::Error::last_os_error());
        }
        let length = capacity as usize;
        if let Some(mut additional) = length.checked_sub(name_buf.capacity()) {
            debug_assert_ne!(0, additional);
            capacity += 2;
            additional += 2;
            name_buf.reserve(additional);
            continue;
        }
        unsafe {
            name_buf.set_len(wcslen(name_buf.as_ptr()));
        }
        break;
    }
    String::from_utf16(&name_buf).map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "username should be valid UTF-8, but isn't. How did you manage that?.",
        )
    })
}

// this is just the wide version of `strlen`.
extern "C" {
    fn wcslen(buf: *const u16) -> usize;
}

// this is a bit of a cluster fuck. it's written this way so
// that relative paths get resolved properly during tests.
// it's still a step up from hardcoding it as
// $HEARTBEAT_HOME/bin/heartbeat-task.exe
fn get_task_scheduler_bin_path() -> std::io::Result<PathBuf> {
    let mut path = current_exe()?;
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    let exe = String::from("heartbeat-task") + EXE_SUFFIX;
    path.push(&exe);
    // check if it's actually there.
    if std::fs::metadata(&path)?.is_file() {
        Ok(path)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, format!("{exe} is not a file.")))
    }
}

fn format_task(username: &str, sid: &str) -> String {
    include_str!("task.xml")
        .replace("{username}", username)
        .replace("{sid}", sid)
        .replace("{exe}", get_task_scheduler_bin_path().unwrap().to_str().unwrap())
}

/// Registers the task denoted by the `xml` string with the Task Scheduler.
///
/// This first writes the XML to a file, and then calls `schtasks.exe` to register the task.
///
/// I could have used the Windows API, but I really do not want to deal with COM and OLE.
///
/// # Returns
///
/// A tuple of `(stdout, stderr)`.
///
/// # Errors
///
/// This function returns an error if `schtasks.exe` fails or returns
/// invalid UTF-8. In the latter case you have bigger problems.
pub fn register_task_xml(xml: &str) -> Result<(String, String)> {
    std::fs::write("heartbeat.xml", xml)?;
    let output = Command::new("schtasks")
        .args(["/create", "/xml", "heartbeat.xml", "/tn", "\"Heartbeat\"", "/f"])
        .output()
        .context("failed to run schtasks.exe")?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let _ = std::fs::remove_file("heartbeat.xml");
    Ok((stdout, stderr))
}
