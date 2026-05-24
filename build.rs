fn main() {
    println!("cargo::rustc-check-cfg=cfg(debug_draw_do_delay)");
    println!("cargo:rerun-if-changed=src/constants.rs");

    #[cfg(windows)]
    println!("cargo:rustc-link-lib=advapi32");

    let constants =
        std::fs::read_to_string("src/constants.rs").expect("failed to read src/constants.rs");
    if compile_time_bool_is_true(&constants, "DEBUG_SHOULD_DRAW_DO_DELAY") {
        println!("cargo:rustc-cfg=debug_draw_do_delay");
    }
}

fn compile_time_bool_is_true(source: &str, name: &str) -> bool {
    let prefix = format!("pub const {name}");
    source.lines().any(|line| {
        let line = line.split("//").next().unwrap_or("").trim();
        line.starts_with(&prefix) && line.contains("= true")
    })
}
