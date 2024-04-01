fn main() {
    println!(
        "cargo:rustc-link-search={}",
        std::env::var("NIX_LD_LIBRARY_PATH").unwrap_or_default()
    );
}
