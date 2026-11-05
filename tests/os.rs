//! `zprf os` is a preview and must stay one: the module that builds it may not run a program, touch
//! a file or read the environment, so nothing it prints can be applied from `zprf`.

use std::path::Path;

#[test]
fn the_os_preview_cannot_apply_anything() {
    let source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/os.rs")).unwrap();
    for forbidden in [
        "process",
        "Command",
        "fs::",
        "std::{",
        "std::env",
        "env::",
        "std::net",
        "File::",
        "OpenOptions",
        "extern",
        "unsafe",
    ] {
        assert!(
            !source.contains(forbidden),
            "src/os.rs mentions {forbidden}; the preview must never run or write anything"
        );
    }
}

#[test]
fn os_needs_print_and_has_no_apply_option() {
    let bin = env!("CARGO_BIN_EXE_zprf");
    let root = env!("CARGO_MANIFEST_DIR");
    let without = std::process::Command::new(bin)
        .current_dir(root)
        .args(["os", "profiles/zacess-dark.toml"])
        .output()
        .unwrap();
    assert_eq!(without.status.code(), Some(2));
    for flag in ["--apply", "--write", "--yes"] {
        let out = std::process::Command::new(bin)
            .current_dir(root)
            .args(["os", "profiles/zacess-dark.toml", "--print", flag])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{flag} was accepted");
    }
    let help = std::process::Command::new(bin)
        .args(["os", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("never"), "{help}");
    for platform in ["macos", "gnome", "windows"] {
        let out = std::process::Command::new(bin)
            .current_dir(root)
            .args([
                "os",
                "profiles/zacess-dark.toml",
                "--print",
                "--platform",
                platform,
            ])
            .output()
            .unwrap();
        assert!(out.status.success(), "{platform}");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.starts_with("# system settings for "), "{text}");
        assert!(text.contains("preview only"), "{text}");
    }
}
