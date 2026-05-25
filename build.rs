fn main() {
    println!("cargo::rustc-check-cfg=cfg(debug_draw_do_delay)");

    #[cfg(windows)]
    println!("cargo:rustc-link-lib=advapi32");

    if std::env::var("CARGO_FEATURE_DEBUG_SHOULD_DRAW_DO_DELAY").is_ok() {
        println!("cargo:rustc-cfg=debug_draw_do_delay");
    }
}
