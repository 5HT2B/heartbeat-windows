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
