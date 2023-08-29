// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![windows_subsystem = "windows"]

use heartbeat::{app_data, ping::ping, Settings};
use std::fs::read_to_string;

fn main() {
    let appender = tracing_appender::rolling::never(app_data(), "heartbeat.log");
    tracing_subscriber::fmt()
        .with_writer(appender)
        .with_level(true)
        .init();
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |i| {
        tracing::error!("PANIC!!! {i:?}");
        original(i);
    }));
    let settings: Settings =
        toml::from_str(&read_to_string(app_data().join("heartbeat.ini")).unwrap()).unwrap();
    ping(&settings.heartbeat.base_url, &settings.heartbeat.auth_token).unwrap();
}
