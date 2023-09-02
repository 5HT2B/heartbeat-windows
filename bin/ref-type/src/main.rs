// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use clap::{Arg, Command};
use regex::Regex;
use std::{env, fs::OpenOptions, io::Write};

fn main() {
    let cmd = Command::new("ref-type").arg(Arg::new("reference").long("reference"));
    let args = cmd.get_matches();
    let regex = Regex::new("^refs/tags/[[:digit:]]+[.][[:digit:]]+[.][[:digit:]]+$")
        .expect("Failed to compile release regex");
    let reference = args
        .get_one::<String>("reference")
        .expect("missing required argument `--reference`");
    let value = if regex.is_match(reference) {
        "release"
    } else {
        "other"
    };
    eprintln!("ref: {reference}");
    eprintln!("value: {value}");
    if cfg!(test) {
        println!("::set-output name=value::{value}");
        return;
    }
    env::var("GITHUB_OUTPUT")
        .map(|path| {
            OpenOptions::new()
                .append(true)
                .open(path)
                .expect("GITHUB_OUTPUT is not a valid file")
        })
        .map_or_else(
            |_| println!("::set-output name=value::{value}"),
            |mut f| writeln!(f, "value={value}").expect("Failed to write to GITHUB_OUTPUT"),
        );
}
