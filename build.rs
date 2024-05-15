fn main() {
    #[cfg(target_os = "linux")]
    {
        let nix_ld_library_path = std::env::var("NIX_LD_LIBRARY_PATH");
        if let Ok(path) = nix_ld_library_path {
            println!("cargo:rustc-link-search={}", path);
        }
    }
}
