use heartbeat::interactive_config;
use std::io::Result;

fn main() -> Result<()> {
    interactive_config()?;
    Ok(())
}
