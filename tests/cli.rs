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

/// Kills the watcher when the test ends, even when an assertion fails first, so no test can leave
/// one running.
struct Watcher(std::process::Child);

impl Drop for Watcher {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for(what: &str, mut done: impl FnMut() -> bool) {
    let start = std::time::Instant::now();
    while !done() {
        assert!(
            start.elapsed() < std::time::Duration::from_secs(20),
            "timed out waiting for {what}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn watch_follows_the_appearance_and_keeps_the_last_good_files() {
    let dir = scratch("cli-watch");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("profiles");
    for v in ["dark", "light"] {
        let name = format!("zacess-deep-contrast-{v}.toml");
        std::fs::copy(root.join(&name), dir.join(&name)).unwrap();
    }
    let appearance = dir.join("appearance");
    std::fs::write(&appearance, "dark").unwrap();
    let out = dir.join("out");
    let child = bin()
        .args([
            "watch",
            dir.join("zacess-deep-contrast-dark.toml").to_str().unwrap(),
            "--target",
            "all",
            "--out",
            out.to_str().unwrap(),
            "--interval-ms",
            "50",
            "--appearance-from",
            appearance.to_str().unwrap(),
            // a ceiling of its own, so the watcher stops even if this test is killed outright
            "--stop-after-ms",
            "60000",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let _guard = Watcher(child);
    // both variants are written under one id, so an app can point at a single file
    let ghostty = out.join("ghostty/zacess-deep-contrast");
    let read = || std::fs::read_to_string(&ghostty).unwrap_or_default();
    wait_for("the dark render", || {
        read().contains("background = #000000")
    });

    std::fs::write(&appearance, "light").unwrap();
    wait_for("the light render", || {
        read().contains("background = #ffffff")
    });

    // a change that breaks the light profile's own needs leaves the last good files alone
    let light = dir.join("zacess-deep-contrast-light.toml");
    let good = std::fs::read_to_string(&light).unwrap();
    std::fs::write(
        &light,
        good.replace("foreground = \"#000000\"", "foreground = \"#eeeeee\""),
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(read().contains("foreground = #000000"), "{}", read());

    // fixing it renders again; nothing is ever written outside --out
    std::fs::write(&light, good.replace("size_pt = 14", "size_pt = 20")).unwrap();
    wait_for("the fixed render", || read().contains("font-size = 20"));
    let mut outside: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    outside.sort();
    assert_eq!(
        outside,
        [
            "appearance",
            "out",
            "zacess-deep-contrast-dark.toml",
            "zacess-deep-contrast-light.toml"
        ]
    );
}

#[test]
fn watch_stops_on_its_limit_and_refuses_a_missing_variant() {
    let dir = scratch("cli-watch-limit");
    let out = run(&[
        "watch",
        "profiles/zacess-cream.toml",
        "--target",
        "kitty",
        "--out",
        dir.to_str().unwrap(),
        "--stop-after-ms",
        "200",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(dir.join("kitty/zacess-cream.conf").is_file());

    let out = run(&[
        "watch",
        "profiles/zacess-cream.toml",
        "--target",
        "kitty",
        "--out",
        dir.to_str().unwrap(),
        "--variant",
        "profiles/no-such-profile.toml",
        "--stop-after-ms",
        "200",
    ]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn render_and_watch_never_overwrite_the_profile_they_read() {
    let dir = scratch("cli-own-input");
    // a profile saved where its own kitty output would go
    std::fs::create_dir_all(dir.join("kitty")).unwrap();
    let input = dir.join("kitty/mine.conf");
    let text = "spec_version = 1\nname = \"Mine\"\nid = \"mine\"\n";
    std::fs::write(&input, text).unwrap();
    let out = run(&[
        "render",
        input.to_str().unwrap(),
        "--target",
        "kitty",
        "--out",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("is the profile being read"));
    assert_eq!(std::fs::read_to_string(&input).unwrap(), text);

    let out = run(&[
        "watch",
        input.to_str().unwrap(),
        "--target",
        "kitty",
        "--out",
        dir.to_str().unwrap(),
        "--stop-after-ms",
        "400",
        "--interval-ms",
        "50",
    ]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("is the profile being read"));
    assert_eq!(std::fs::read_to_string(&input).unwrap(), text);
}

#[test]
fn watch_renders_the_matching_variant_when_the_other_is_broken() {
    let dir = scratch("cli-watch-broken-variant");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("profiles");
    std::fs::copy(
        root.join("zacess-deep-contrast-dark.toml"),
        dir.join("zacess-deep-contrast-dark.toml"),
    )
    .unwrap();
    std::fs::write(dir.join("zacess-deep-contrast-light.toml"), "not a profile").unwrap();
    std::fs::write(dir.join("appearance"), "dark").unwrap();
    let out_dir = dir.join("out");
    let out = run(&[
        "watch",
        dir.join("zacess-deep-contrast-dark.toml").to_str().unwrap(),
        "--target",
        "ghostty",
        "--out",
        out_dir.to_str().unwrap(),
        "--appearance-from",
        dir.join("appearance").to_str().unwrap(),
        "--stop-after-ms",
        "400",
        "--interval-ms",
        "50",
    ]);
    let rendered = std::fs::read_to_string(out_dir.join("ghostty/zacess-deep-contrast")).unwrap();
    assert!(rendered.contains("background = #000000"));
    assert!(String::from_utf8_lossy(&out.stderr).contains("zacess-deep-contrast-light.toml"));
    // the write goes through a temporary file that never stays behind
    let left: Vec<_> = std::fs::read_dir(out_dir.join("ghostty"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["zacess-deep-contrast"]);
}

#[test]
fn watch_tries_a_failed_write_again_without_the_profile_changing() {
    let dir = scratch("cli-watch-retry");
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    // a file where the target's folder belongs makes every write fail until it is gone
    std::fs::write(out_dir.join("kitty"), "in the way").unwrap();
    let child = bin()
        .args([
            "watch",
            "profiles/zacess-cream.toml",
            "--target",
            "kitty",
            "--out",
            out_dir.to_str().unwrap(),
            "--interval-ms",
            "50",
            "--stop-after-ms",
            "60000",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let _guard = Watcher(child);
    std::thread::sleep(std::time::Duration::from_millis(400));
    std::fs::remove_file(out_dir.join("kitty")).unwrap();
    wait_for("the retried write", || {
        out_dir.join("kitty/zacess-cream.conf").is_file()
    });
}

#[test]
fn verbose_check_names_each_floor_once() {
    let dir = scratch("cli-verbose-floor");
    let file = dir.join("low.toml");
    std::fs::write(
        &file,
        "spec_version = 1\nname = \"Low\"\n[palette]\nbackground = \"#ffffff\"\n\
         [palette.syntax]\ncomment = \"#999999\"\n",
    )
    .unwrap();
    let out = run(&["check", "--verbose", file.to_str().unwrap()]);
    let stdout = text(&out);
    for line in stdout
        .lines()
        .filter(|l| l.contains("syntax.comment #999999 on background"))
    {
        assert_eq!(line.matches("7:1").count(), 1, "{line}");
    }
    assert!(stdout.contains("(floor 7:1)"), "{stdout}");
}
