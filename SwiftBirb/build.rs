fn main() {
    #[cfg(target_os = "macos")]
    build_cocoa();

    #[cfg(not(target_os = "macos"))]
    panic!("This module is macOS-only");
}

#[cfg(target_os = "macos")]
fn build_cocoa() {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::{env, fs};

    let is_release = env::var("PROFILE") == Ok("release".to_string());
    let proj_path = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut xcode_args = Vec::new();
    xcode_args.push("-configuration");
    if is_release {
        xcode_args.push("Release");
    } else {
        xcode_args.push("Debug");
    }

    let output = Command::new("xcodebuild")
        .args(&xcode_args)
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("{}", stderr);
    assert!(output.status.success(), "xcodebuild failed");

    println!("cargo:rerun-if-changed={}/SwiftBirb", proj_path);
    for entry in
        fs::read_dir(format!("{}/SwiftBirb", proj_path)).expect("Failed to read ./SwiftBirb")
    {
        println!(
            "cargo:rerun-if-changed={}/SwiftBirb/{}",
            proj_path,
            entry.unwrap().path().to_str().unwrap()
        );
    }

    // libswift libraries
    println!("cargo:rustc-link-search=/usr/lib/swift");

    let lib_out_path = if is_release { "Release" } else { "Debug" };
    println!(
        "cargo:rustc-link-search={}/build/{}",
        proj_path, lib_out_path
    );
    println!("cargo:rustc-link-lib=SwiftBirb");

    // also generate bindings from the header
    let bindings = bindgen::Builder::default()
        .header("../birb-core.h")
        .whitelist_type("SBEvent")
        .whitelist_type("SBPatch")
        .whitelist_type("SBEventDispatcher")
        // unstable: .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true, })
        .prepend_enum_name(false)
        .generate()
        .expect("Failed to generate birb core bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("protocol.rs"))
        .expect("Failed to write bindings");
}
