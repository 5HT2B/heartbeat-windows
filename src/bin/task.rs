// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(test), windows_subsystem = "windows")]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_in_result)]

use heartbeat::{ping::ping, Settings};
use heartbeat_sys::heartbeat_home;
use std::fs::read_to_string;

fn main() {
    pre_run();
    let appender = tracing_appender::rolling::never(heartbeat_home().unwrap().join("logs"), "heartbeat.log");
    tracing_subscriber::fmt().with_writer(appender).with_level(true).init();
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |i| {
        tracing::error!("PANIC!!! {i:?}");
        original(i);
    }));
    let settings: Settings =
        toml::from_str(&read_to_string(heartbeat_home().unwrap().join("config.toml")).unwrap()).unwrap();
    ping(&settings.client.base_url, &settings.client.auth_token).unwrap();
}

fn pre_run() {
    use windows_sys::Win32::System::LibraryLoader::{SetDefaultDllDirectories, LOAD_LIBRARY_SEARCH_SYSTEM32};
    let result = unsafe { SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32) };
    assert_ne!(result, 0);
}
