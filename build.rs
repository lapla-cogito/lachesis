fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os != "linux" && target_os != "macos" {
        panic!("unsupported OS: {}", target_os);
    }

    let (asm_file, o_file, lib_file, lib_name) = match target_arch.as_str() {
        "x86_64" => (
            "asm/context_x86_64.S",
            "asm/context_x86_64.o",
            "asm/libcontext_x86_64.a",
            "context_x86_64",
        ),
        "aarch64" => (
            "asm/context_aarch64.S",
            "asm/context_aarch64.o",
            "asm/libcontext_aarch64.a",
            "context_aarch64",
        ),
        _ => panic!("unsupported architecture: {}", target_arch),
    };

    if !std::path::Path::new(asm_file).exists() {
        panic!("assembly file {} not found", asm_file);
    }

    // clean up old files
    let _ = std::fs::remove_file(o_file);
    let _ = std::fs::remove_file(lib_file);

    // compile assembly
    let mut build = cc::Build::new();
    build.file(asm_file).flag("-fPIC");

    if target_arch == "x86_64" {
        build.flag("-ggdb");
    }

    build.compile("context_temp");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let obj_path = std::path::Path::new(&out_dir)
        .join("context_temp")
        .with_extension("o");

    // if cc crate produced a different filename, search for it
    let actual_obj = if obj_path.exists() {
        obj_path
    } else {
        // search for any .o file in OUT_DIR that was just created
        std::fs::read_dir(&out_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.extension().and_then(|s| s.to_str()) == Some("o")
                    && p.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.contains("context"))
                        .unwrap_or(false)
            })
            .expect("Could not find compiled object file")
    };

    // create static library
    let mut builder = ar::Builder::new(std::fs::File::create(lib_file).unwrap());

    let obj_data = std::fs::read(&actual_obj).unwrap();
    let mut header = ar::Header::new(
        std::path::Path::new(o_file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .as_bytes()
            .to_vec(),
        obj_data.len() as u64,
    );
    header.set_mode(0o644);

    builder
        .append(&header, obj_data.as_slice())
        .expect("Failed to add object file to archive");

    drop(builder); // ensure the archive is written

    println!("cargo:rustc-link-search=native=asm");
    println!("cargo:rustc-link-lib=static={}", lib_name);
    println!("cargo:rerun-if-changed={}", asm_file);
    println!("cargo:rerun-if-changed=build.rs");
}
