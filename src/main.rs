#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_in_result)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use heartbeat::{
    interactive_config,
    tasks::{generate_xml, register_task_xml},
};
use heartbeat_sys::{
    cli::common::question_bool,
    heartbeat_home,
    process::{process, with, OSProcess, ProcessLike},
    utils::utils::ensure_dir_exists,
    verbose,
};
use windows_sys::Win32::System::LibraryLoader::{SetDefaultDllDirectories, LOAD_LIBRARY_SEARCH_SYSTEM32};

/// The Heartbeat Client for Windows
#[derive(Debug, Parser)]
#[clap(version = env!("HB_VERSION"))]
struct Cli {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Configure the client parameters.
    Config,
    /// Generate and optionally register the ping task using Task Scheduler.
    Task,
}

fn main() -> Result<()> {
    pre_run();
    let p = OSProcess::new();
    with(p.into(), _main)?;
    Ok(())
}

fn _main() -> Result<()> {
    let args = Cli::parse();
    ensure_dir_exists(&heartbeat_home()?.join("logs"), |path| {
        if process().var("HEARTBEAT_DEBUG").is_ok() {
            verbose!("Creating logging directory {path:?}");
        }
    })?;
    match args.subcommand {
        Command::Config => {
            interactive_config()?;
        }
        Command::Task => {
            let xml = generate_xml();
            writeln!(process().stdout().lock(), "{xml}")?;
            writeln!(process().stderr().lock())?;
            let res = question_bool("Would you like to register the task now?", true)?;
            if res {
                let (out, err) = register_task_xml(&xml)?;
                writeln!(process().stdout().lock(), "{out}")?;
                writeln!(process().stderr().lock(), "{err}")?;
            }
        }
    }
    Ok(())
}

fn pre_run() {
    let result = unsafe { SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32) };
    assert_ne!(result, 0);
}
