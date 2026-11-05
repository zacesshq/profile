//! One snapshot per target per profile, plus a parse check on the formats that have a parser here.
//!
//! Run `UPDATE_SNAPSHOTS=1 cargo test --test targets` to rewrite the snapshots after an intended
//! template change, then review the diff before committing it.

use std::path::{Path, PathBuf};

use zacess_profile::{Profile, derive, render};

const SNAPSHOT_PROFILES: &[&str] = &[
    "profiles/zacess-deep-contrast-dark.toml",
    "spec/examples/minimal.toml",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn complete(path: &str) -> Profile {
    derive(&Profile::load(&root().join(path)).unwrap())
        .unwrap()
        .profile
}

#[test]
fn every_target_matches_its_snapshot() {
    let update = std::env::var_os("UPDATE_SNAPSHOTS").is_some();
    let mut stale = Vec::new();
    for path in SNAPSHOT_PROFILES {
        let profile = complete(path);
        for target in render::TARGETS {
            let text = render::render(&profile, target).unwrap();
            let snap = root()
                .join("tests/snapshots")
                .join(render::relative_path(&profile, target));
            if update {
                std::fs::create_dir_all(snap.parent().unwrap()).unwrap();
                std::fs::write(&snap, &text).unwrap();
                continue;
            }
            match std::fs::read_to_string(&snap) {
                Ok(expected) if expected == text => {}
                _ => stale.push(snap.display().to_string()),
            }
        }
    }
    assert!(
        stale.is_empty(),
        "these renders differ from their snapshots (UPDATE_SNAPSHOTS=1 rewrites them):\n{}",
        stale.join("\n")
    );
}

#[test]
fn every_target_renders_every_profile() {
    let dir = root().join("profiles");
    for entry in std::fs::read_dir(dir).unwrap() {
        let profile = Profile::load(&entry.unwrap().path()).unwrap();
        for target in render::TARGETS {
            let text = render::render(&profile, target).unwrap();
            // plain JSON has no comments, so only the other targets can name their profile
            if target.name != "vscode" {
                assert!(
                    text.contains(&profile.name),
                    "{} {}",
                    target.name,
                    profile.name
                );
            }
            // every colour is written as hex, never as an empty value; an empty fontStyle is
            // deliberate, since it clears any style the base theme gives that token
            let without_styles = text.replace("\"fontStyle\": \"\"", "");
            assert!(
                !without_styles.contains("\"\""),
                "{} {}",
                target.name,
                profile.name
            );
        }
    }
}

#[test]
fn vscode_output_is_valid_json_with_every_token() {
    let profile = complete(SNAPSHOT_PROFILES[1]);
    let text = render::render(&profile, render::target("vscode").unwrap()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    let rules = json["editor.tokenColorCustomizations"]["textMateRules"]
        .as_array()
        .unwrap();
    assert_eq!(rules.len(), 18);
    let comment = rules.iter().find(|r| r["name"] == "comment").unwrap();
    assert_eq!(comment["settings"]["fontStyle"], "italic");
    assert_eq!(json["workbench.reduceMotion"], "on");
    assert_eq!(json["editor.fontSize"], 24);
}

#[test]
fn alacritty_output_is_valid_toml() {
    let profile = complete(SNAPSHOT_PROFILES[0]);
    let text = render::render(&profile, render::target("alacritty").unwrap()).unwrap();
    let value: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(
        value["colors"]["primary"]["background"].as_str(),
        Some("#000000")
    );
    assert_eq!(value["cursor"]["style"]["blinking"].as_str(), Some("Never"));
}

#[test]
fn render_writes_only_inside_the_out_directory() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("render-only-inside");
    let _ = std::fs::remove_dir_all(&out);
    let profile = complete(SNAPSHOT_PROFILES[0]);
    for target in render::TARGETS {
        let written = render::write(&profile, target, &out).unwrap();
        assert!(written.starts_with(&out), "{}", written.display());
        assert!(written.is_file());
    }
}

#[test]
fn an_unknown_target_lists_the_real_ones() {
    let err = render::target("nano").unwrap_err().to_string();
    assert!(
        err.contains("ghostty, kitty, wezterm, alacritty, tmux, neovim and vscode"),
        "{err}"
    );
}
