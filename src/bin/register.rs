use heartbeat::{
    is_stdout_tty, read_yes_no,
    tasks::{generate_xml, register_task_xml, Error},
};
fn main() -> Result<(), Error> {
    let xml = generate_xml()?;
    println!("{xml}");
    if is_stdout_tty() {
        eprintln!();
        let should_register = read_yes_no("Would you like to register the task now?", Some(true))?;
        eprintln!("{should_register}");
        if should_register {
            register_task_xml(&xml)?;
        }
    }
    Ok(())
}
