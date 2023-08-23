use std::{env::current_exe, fs::read_to_string, path::PathBuf, process::Command};

#[must_use]
pub fn generate_xml() -> String {
    let username = get_current_user_name();
    let sid = get_current_user_sid();
    format_task(&username, &sid)
}

fn get_current_user_sid() -> String {
    let cmd = Command::new("powershell")
    .arg("-Command")
    .arg("Get-WmiObject -Class Win32_UserAccount | Where-Object { $_.Name -eq $env:USERNAME } | Select-Object -ExpandProperty SID").output().expect("failed to execute process");
    String::from_utf8(cmd.stdout).unwrap().trim().to_string()
}

fn get_current_user_name() -> String {
    let cmd = Command::new("powershell")
    .arg("-Command")
    .arg("Get-WmiObject -Class Win32_UserAccount | Where-Object { $_.Name -eq $env:USERNAME } | Select-Object -ExpandProperty Caption").output().expect("failed to execute process");
    String::from_utf8(cmd.stdout).unwrap().trim().to_string()
}

fn get_task_scheduler_bin_path() -> std::io::Result<PathBuf> {
    let this = current_exe()?;
    let expected = this.parent().unwrap().join("heartbeat-task.exe");
    if !expected.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Expected task scheduler binary at {expected:?}, but it was not found"),
        ));
    }
    Ok(expected)
}

fn format_task(username: &str, sid: &str) -> String {
    read_to_string("task.xml")
        .unwrap()
        .replace("{username}", username)
        .replace("{sid}", sid)
        .replace(
            "{exe}",
            get_task_scheduler_bin_path().unwrap().to_str().unwrap(),
        )
}

#[must_use]
pub fn register_task_xml(xml: &str) -> (String, String) {
    std::fs::write("heartbeat.xml", xml).unwrap();

    let output = Command::new("powershell")
        .arg("-Command")
        .arg("schtasks.exe /create /xml heartbeat.xml /tn \"Heartbeat\"")
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if stderr.is_empty() {
        let _ = std::fs::remove_file("heartbeat.xml");
    }
    (stdout, stderr)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_my_sid() {
        let sid = get_current_user_sid();
        println!("{sid}");
    }
    #[test]
    fn get_my_username() {
        let username = get_current_user_name();
        println!("{username}");
    }
    #[test]
    fn register() {
        let (out, err) = register_task_xml(&format_task(
            &get_current_user_name(),
            &get_current_user_sid(),
        ));
        println!("out={out}");
        println!("err={err}");
    }
}
