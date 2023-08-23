use std::{env::current_exe, path::PathBuf};

// feature list
// - Win32_Security_Authentication_Identity
// - Win32_Foundation
// - Win32_Security_Authorization
// - Win32_Security
// - Win32_System_Com
// - Win32_system_TaskScheduler
// - Win32_System_Threading
// - Win32_System_Variant
// - Win32_System_Ole

use windows::{
    core::{Result, BSTR, PWSTR},
    Win32::{
        Foundation::HANDLE,
        Security::{
            Authentication::Identity::{GetUserNameExW, NameSamCompatible},
            Authorization::ConvertSidToStringSidW,
            GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
            TaskScheduler::{ITaskService, TaskScheduler},
            Threading::{GetCurrentProcess, OpenProcessToken},
            Variant::VARIANT,
        },
    },
};

const XML_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Date>2023-08-21T01:39:18.2128635</Date>
    <Author>{username}</Author>
    <URI>\Heartbeat Task</URI>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Repetition>
        <Interval>PT1M</Interval>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
      <Enabled>true</Enabled>
      <UserId>{username}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{sid}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>StopExisting</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <DisallowStartOnRemoteAppSession>false</DisallowStartOnRemoteAppSession>
    <UseUnifiedSchedulingEngine>true</UseUnifiedSchedulingEngine>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT72H</ExecutionTimeLimit>
    <Priority>7</Priority>
    <RestartOnFailure>
      <Interval>PT1M</Interval>
      <Count>50</Count>
    </RestartOnFailure>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
    </Exec>
  </Actions>
</Task>
"#;

#[derive(Debug)]
pub enum Error {
    Win32(windows::core::Error),
    Io(std::io::Error),
}

impl From<windows::core::Error> for Error {
    fn from(e: windows::core::Error) -> Self {
        Self::Win32(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn generate_xml() -> std::result::Result<String, Error> {
    let username = match get_current_user_name() {
        None => return Err(windows::core::Error::from_win32().into()),
        Some(username) => username,
    };
    let sid = get_current_user_sid()?;
    Ok(format_task(&username, &sid))
}

fn get_current_user_sid() -> Result<String> {
    unsafe {
        let current_process_handle = GetCurrentProcess();
        let mut token_handle = HANDLE::default();
        OpenProcessToken(
            current_process_handle,
            TOKEN_QUERY,
            std::ptr::addr_of_mut!(token_handle).cast::<HANDLE>(),
        )?;
        let mut token_user_length: u32 = 0;
        let _ = GetTokenInformation(
            token_handle,
            TokenUser,
            None,
            0,
            std::ptr::addr_of_mut!(token_user_length),
        );
        let mut buf: Vec<u16> = Vec::with_capacity(token_user_length as usize);
        let token_user = buf.as_mut_ptr().cast::<TOKEN_USER>();
        GetTokenInformation(
            token_handle,
            TokenUser,
            Some(token_user.cast()),
            token_user_length,
            std::ptr::addr_of_mut!(token_user_length),
        )?;
        buf.clear();
        let sid = (*token_user).User.Sid;
        let mut ret = PWSTR::from_raw(buf.as_mut_ptr());
        ConvertSidToStringSidW(sid, std::ptr::addr_of_mut!(ret))?;
        Ok(String::from_utf16_lossy(ret.as_wide()))
    }
}

fn get_current_user_name() -> Option<String> {
    let mut length = 0u32;
    unsafe {
        let _ = GetUserNameExW(
            NameSamCompatible,
            PWSTR::null(),
            std::ptr::addr_of_mut!(length),
        );
        let buf = PWSTR::from_raw(Vec::with_capacity(length as usize).as_mut_ptr());
        let success =
            GetUserNameExW(NameSamCompatible, buf, std::ptr::addr_of_mut!(length)).as_bool();
        if success {
            Some(String::from_utf16_lossy(buf.as_wide()))
        } else {
            None
        }
    }
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
    XML_TEMPLATE
        .replace("{username}", username)
        .replace("{sid}", sid)
        .replace(
            "{exe}",
            get_task_scheduler_bin_path().unwrap().to_str().unwrap(),
        )
}

fn get_task_service() -> Result<ITaskService> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
        let task_service = CoCreateInstance::<_, ITaskService>(&TaskScheduler, None, CLSCTX_ALL)?;
        task_service.Connect(
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
        )?;
        Ok(task_service)
    }
}

pub fn register_task_xml(xml: &str) -> Result<()> {
    unsafe {
        let task_service = get_task_service()?;
        let task_definition = task_service.NewTask(0)?;
        task_definition.SetXmlText(&BSTR::from_wide(
            xml.as_bytes()
                .to_vec()
                .iter()
                .map(|&c| u16::from(c))
                .collect::<Vec<_>>()
                .as_slice(),
        )?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_my_sid() {
        let sid = get_current_user_sid().unwrap();
        println!("{sid}");
    }
    #[test]
    fn get_my_username() {
        let username = get_current_user_name().unwrap();
        println!("{username}");
    }
}
