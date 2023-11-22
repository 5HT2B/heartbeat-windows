fn main() {
    let version = heartbeat_sys::build::long_version(env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=HB_VERSION={version}");
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return;
    }
    // only search system32 for DLLs.
    //
    // this applies to DLLs loaded at load time. however, this setting is ignored
    // before Windows 10 RS1 (aka 1601), https://learn.microsoft.com/en-us/cpp/build/reference/dependentloadflag?view=msvc-170
    println!("cargo:rustc-link-arg-bin=heartbeat-client=/DEPENDENTLOADFLAG:0x800");
    println!("cargo:rustc-link-arg-bin=heartbeat-task=/DEPENDENTLOADFLAG:0x800");
    #[rustfmt::skip] // i now see why this feature is unstable.
    // delay load
    //
    // delay load `bcrypt.dll` which isn't a "known DLL"*.
    // known DLLs are always loaded from the system directory whereas other DLLs are loaded from the application
    // directory. by delay loading the latter, we can ensure that they are instead loaded from the system directory.
    // *: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order#factors-that-affect-searching
    //
    // this will work on all supported Windows versions, but it relies on us using `SetDefaultDllDirectories` before any
    // libraries are loaded.
    // see also: src/main.rs, src/bin/task.rs
    println!("cargo:rustc-link-arg-bin=heartbeat-client=/delayload:bcrypt.dll");
    println!("cargo:rustc-link-arg-bin=heartbeat-task=/delayload:bcrypt.dll");
    // when using delayload, it is necessary to also link delayimp.lib
    // https://learn.microsoft.com/en-us/cpp/build/reference/dependentloadflag?view=msvc-170
    println!("cargo:rustc-link-arg-bin=heartbeat-client=delayimp.lib");
    println!("cargo:rustc-link-arg-bin=heartbeat-task=delayimp.lib");
    // turn linker warnings into errors
    //
    // rust hides linker warnings, meaning mistakes may go unnoticed.
    // turning them into errors forces them to be displayed (and the build to fail).
    // if we do want to ignore specific warnings, then `/IGNORE:` should be used.
    println!("cargo:rustc-link-arg-bin=heartbeat-client=/WX");
    println!("cargo:rustc-link-arg-bin=heartbeat-task=/WX");
}
