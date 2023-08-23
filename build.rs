fn main() {
    if !cfg!(target_os = "windows") {
        panic!("This crate only supports Windows");
    }
}
