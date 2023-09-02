// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    path::PathBuf,
    process::Command,
};

fn executable_path(name: &str) -> PathBuf {
    let mut path = current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    let exe = String::from(name) + EXE_SUFFIX;
    path.push(exe);
    path
}

fn stdout(reference: &str) -> String {
    let output = Command::new(executable_path("ref-type"))
        .args(["--reference", reference])
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn junk_is_other() {
    assert_eq!(stdout("refs/tags/asdf"), "value=other\n");
}

#[test]
fn valid_version_is_release() {
    assert_eq!(stdout("refs/tags/0.0.0"), "value=release\n");
}

#[test]
fn valid_version_with_trailing_characters_is_other() {
    assert_eq!(stdout("refs/tags/0.0.0-rc.1"), "value=other\n");
}

#[test]
fn valid_version_with_lots_of_digits_is_release() {
    assert_eq!(
        stdout("refs/tags/01232132.098327498374.43268473849734"),
        "value=release\n"
    );
}
