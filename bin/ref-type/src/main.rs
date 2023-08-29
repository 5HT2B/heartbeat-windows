// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use regex::Regex;
use std::{env, fs::OpenOptions, io::Write};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Arguments {
    #[structopt(long)]
    reference: String,
}

fn main() {
    let args = Arguments::from_args();
    let regex = Regex::new("^refs/tags/[[:digit:]]+[.][[:digit:]]+[.][[:digit:]]+$")
        .expect("Failed to compile release regex");
    let value = if regex.is_match(&args.reference) {
        "release"
    } else {
        "other"
    };
    eprintln!("ref: {}", args.reference);
    eprintln!("value: {value}");
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
