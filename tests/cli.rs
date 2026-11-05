//! Runs the built binary the way a person would, capturing its output. Nothing here writes outside
//! Cargo's own temporary directory and nothing is sent to a real terminal.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zprf"));
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

fn run(args: &[&str]) -> Output {
    bin().args(args).output().unwrap()
}

fn text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn check_passes_every_shipped_profile() {
    let mut args = vec!["check"];
    let files: Vec<String> =
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("profiles"))
            .unwrap()
            .map(|e| e.unwrap().path().display().to_string())
            .collect();
    args.extend(files.iter().map(String::as_str));
    let out = run(&args);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("8 of 8 profiles pass"));
}

#[test]
fn check_fails_loudly_naming_the_pair() {
    let dir = scratch("cli-check-fail");
    let file = dir.join("low.toml");
    std::fs::write(
        &file,
        "spec_version = 1\nname = \"Low\"\n[palette]\nbackground = \"#ffffff\"\n\
         [palette.syntax]\ncomment = \"#999999\"\n",
    )
    .unwrap();
    let out = run(&["check", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    let stdout = text(&out);
    assert!(stdout.contains("FAIL"), "{stdout}");
    assert!(
        stdout.contains(
            "syntax.comment #999999 on background #ffffff: 2.84:1, below needs.contrast.text of 7:1"
        ),
        "{stdout}"
    );
}

#[test]
fn a_broken_file_exits_2_with_the_reason() {
    let dir = scratch("cli-broken");
    let file = dir.join("broken.toml");
    std::fs::write(
        &file,
        "spec_version = 1\nname = \"x\"\n[palette]\nbackground = \"red\"\n",
    )
    .unwrap();
    let out = run(&["check", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("#rrggbb"), "{stderr}");
}

#[test]
fn no_derive_refuses_an_incomplete_profile() {
    let out = run(&["check", "--no-derive", "spec/examples/minimal.toml"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn derive_marks_what_it_filled() {
    let out = run(&["derive", "spec/examples/minimal.toml"]);
    assert!(out.status.success());
    let stdout = text(&out);
    assert!(
        stdout.contains("background = \"#1b1612\"\n"),
        "a set colour is not marked"
    );
    assert!(
        stdout.contains("foreground = \"#ffffff\" # derived"),
        "{stdout}"
    );
}

#[test]
fn render_refuses_a_failing_profile_and_writes_nothing() {
    let dir = scratch("cli-render-refuse");
    let file = dir.join("low.toml");
    std::fs::write(
        &file,
        "spec_version = 1\nname = \"Low\"\n[palette]\nbackground = \"#000000\"\nforeground = \"#444444\"\n",
    )
    .unwrap();
    let out_dir = dir.join("out");
    let out = run(&[
        "render",
        file.to_str().unwrap(),
        "--target",
        "all",
        "--out",
        out_dir.to_str().unwrap(),
    ]);
    assert_ne!(out.status.code(), Some(0));
    assert!(!out_dir.exists());
}

#[test]
fn apply_live_sends_osc_4_10_11_and_12_to_stdout() {
    let out = run(&["apply", "profiles/zacess-deep-contrast-dark.toml", "--live"]);
    assert!(out.status.success());
    let raw = String::from_utf8(out.stdout).unwrap();
    assert!(raw.starts_with("\x1b]4;0;rgb:00/00/00\x1b\\"));
    assert!(raw.contains("\x1b]4;15;rgb:ff/ff/ff\x1b\\"));
    assert!(raw.contains("\x1b]10;rgb:ff/ff/ff\x1b\\"));
    assert!(raw.contains("\x1b]11;rgb:00/00/00\x1b\\"));
    assert!(raw.contains("\x1b]12;rgb:ff/ff/00\x1b\\"));
    assert_eq!(raw.matches("\x1b]4;").count(), 16);
}

#[test]
fn apply_escaped_is_printable() {
    let out = run(&["apply", "profiles/zacess-light.toml", "--live", "--escaped"]);
    let printed = text(&out);
    assert!(!printed.contains('\x1b'));
    assert!(printed.contains("\\e]11;rgb:ff/ff/ff\\e\\\\"), "{printed}");
}

#[test]
fn apply_needs_live() {
    let out = run(&["apply", "profiles/zacess-light.toml"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn list_targets_names_all_seven() {
    let out = run(&["list-targets"]);
    let stdout = text(&out);
    for name in [
        "ghostty",
        "kitty",
        "wezterm",
        "alacritty",
        "tmux",
        "neovim",
        "vscode",
    ] {
        assert!(
            stdout.lines().any(|l| l.starts_with(name)),
            "{name} missing:\n{stdout}"
        );
    }
}

#[test]
fn a_bad_file_does_not_stop_the_rest() {
    let dir = scratch("cli-multi");
    let bad = dir.join("bad.toml");
    std::fs::write(
        &bad,
        "spec_version = 1\nname = \"x\"\n[palette]\nbackground = \"red\"\n",
    )
    .unwrap();
    let bad = bad.display().to_string();
    let out = run(&[
        "check",
        &bad,
        "profiles/zacess-light.toml",
        "profiles/zacess-dark.toml",
    ]);
    assert_eq!(out.status.code(), Some(2));
    let stdout = text(&out);
    assert!(stdout.contains(&format!("ERROR {bad}")), "{stdout}");
    assert!(
        stdout.contains("pass profiles/zacess-dark.toml"),
        "{stdout}"
    );
    assert!(stdout.contains("2 of 3 profiles pass"), "{stdout}");
}

#[test]
fn an_accent_too_dim_to_read_is_refused_before_render() {
    let dir = scratch("cli-accent");
    let file = dir.join("accent.toml");
    std::fs::write(
        &file,
        "spec_version = 1\nname = \"Dim accent\"\n[palette]\nbackground = \"#000000\"\naccent = \"#262626\"\n",
    )
    .unwrap();
    let out = run(&["check", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out).contains("FAIL accent #262626 on background #000000: 1.38:1"));
    let out_dir = dir.join("out");
    let out = run(&[
        "render",
        file.to_str().unwrap(),
        "--target",
        "neovim",
        "--out",
        out_dir.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!out_dir.exists());
}

#[test]
fn escaped_alone_prints_without_live() {
    let out = run(&["apply", "profiles/zacess-light.toml", "--escaped"]);
    assert!(out.status.success());
    assert!(!text(&out).contains('\x1b'));
}
