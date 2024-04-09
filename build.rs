fn main() {
    #[cfg(target_os = "linux")]
    println!(
        "cargo:rustc-link-search={}",
        std::env::var("NIX_LD_LIBRARY_PATH").unwrap_or_default()
    );
}
