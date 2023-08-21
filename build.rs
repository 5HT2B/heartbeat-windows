fn main() {
    println!("cargo:rerun-if-env-changed=HEARTBEAT_BUILD_FOR_TASK_SCHEDULER");
    if let Ok(s) = std::env::var("HEARTBEAT_BUILD_FOR_TASK_SCHEDULER") {
        if !["", "0"].contains(&s.trim()) {
            println!("cargo:rustc-cfg=task_scheduler");
        }
    }
}
