// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # heartbeat
//! A [heartbeat](https://github.com/lmaotrigine/heartbeat) client for Windows.
//!
//! This library contains common code used in the binaries.
//!
//! It doesn't have a `#![forbid(unsafe_code)]` because we call into the Windows
//! API directly.

#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    clippy::unwrap_in_result
)]

#[cfg(not(target_os = "windows"))]
compile_error!("This crate only supports Windows targets");

use anyhow::Result;
use heartbeat_sys::{cli::common::question_str, heartbeat_home};
#[cfg(feature = "serde")]
use serde::Deserialize;
use std::{fs::OpenOptions, io::Write};

#[cfg(feature = "task_runner")]
pub mod ping;
pub mod tasks;

/// Settings for the client. This is read from `$HEARTBEAT_HOME/config.toml`.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
pub struct Settings {
    /// Settings for the client. This is read from
    /// `$HEARTBEAT_HOME/config.toml`.
    pub client: SettingsInner,
}

/// Settings for the heartbeat client.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
pub struct SettingsInner {
    /// The base URL of the server.
    pub base_url: String,
    /// The authorization token to use.
    pub auth_token: String,
}

/// Interactively prompts the user for the server URL and authorization token.
///
/// # Errors
///
/// This function will return an error if reading from stdin fails.
pub fn interactive_config() -> Result<()> {
    let server_url = loop {
        let server_url = question_str("server base URL: ", "")?;
        if !server_url.is_empty() {
            break server_url;
        }
    };
    let auth_token = loop {
        let auth_token = question_str("Authorization token: ", "")?;
        if !auth_token.is_empty() {
            break auth_token;
        }
    };
    let mut opts = OpenOptions::new();
    let mut file = opts
        .write(true)
        .truncate(true)
        .create(true)
        .open(heartbeat_home()?.join("config.toml"))?;
    writeln!(
        &mut file,
        r#"[client]
base_url = "{server_url}"
auth_token = "{auth_token}""#
    )?;
    Ok(())
}
