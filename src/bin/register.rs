// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use heartbeat::{
    is_stdout_tty, read_yes_no,
    tasks::{generate_xml, register_task_xml},
};
fn main() -> std::io::Result<()> {
    let xml = generate_xml();
    println!("{xml}");
    if is_stdout_tty() {
        eprintln!();
        let should_register = read_yes_no("Would you like to register the task now?", Some(true))?;
        if should_register {
            let (out, err) = register_task_xml(&xml);
            println!("{out}");
            eprintln!("{err}");
        }
    }
    Ok(())
}
