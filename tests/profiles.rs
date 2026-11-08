//! Every shipped profile must be complete and pass its own checks, so a starter profile can never
//! hand someone a palette that breaks the needs it states.

use std::path::{Path, PathBuf};

use zacess_profile::{Profile, check, derive};

fn toml_files(dir: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", root.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    files
}

#[test]
fn there_are_ten_starter_profiles() {
    assert_eq!(toml_files("profiles").len(), 10);
}

#[test]
fn every_shipped_profile_is_complete_and_passes() {
    for path in toml_files("profiles") {
        let profile = Profile::load(&path).unwrap();
        let derived = derive(&profile).unwrap();
        assert!(
            derived.filled.is_empty(),
            "{} leaves colours to derive: {:?}",
            path.display(),
            derived.filled
        );
        let report = check(&profile).unwrap();
        let fails: Vec<String> = report.failures().iter().map(|p| p.to_string()).collect();
        assert!(
            fails.is_empty(),
            "{}:\n{}",
            path.display(),
            fails.join("\n")
        );
        // every pair class is exercised, so a profile cannot pass by checking less
        assert!(report.pairs.len() >= 100, "{}", path.display());
    }
}

#[test]
fn only_the_warm_pair_lowers_the_text_floor() {
    // the themes give Warm Dark and Warm Light a per-theme 4.5:1 text floor; every other starter
    // profile must keep 7:1, so the gentler pair can never be used to weaken the rest
    let warm = ["zacess-warm-dark", "zacess-warm-light"];
    for path in toml_files("profiles") {
        let profile = Profile::load(&path).unwrap();
        let c = &profile.needs.contrast;
        let floor = if warm.contains(&profile.slug().as_str()) {
            4.5
        } else {
            7.0
        };
        assert_eq!(c.text, floor, "{} text floor", path.display());
        assert_eq!(c.terminal, floor, "{} terminal floor", path.display());
        assert_eq!(c.non_text, 3.0, "{} non-text floor", path.display());
        assert_eq!(c.overlay, 4.5, "{} overlay floor", path.display());
        assert_eq!(c.colour_vision_distance, 0.04, "{}", path.display());
        assert_eq!(c.tone_difference, 0.05, "{}", path.display());
    }
}

#[test]
fn every_shipped_profile_has_a_unique_id_matching_its_file() {
    let mut seen = std::collections::HashSet::new();
    for path in toml_files("profiles") {
        let profile = Profile::load(&path).unwrap();
        let id = profile.slug();
        assert_eq!(path.file_stem().unwrap().to_str().unwrap(), id);
        assert!(seen.insert(id.clone()), "duplicate id {id}");
    }
}

#[test]
fn every_example_derives_and_passes() {
    for path in toml_files("spec/examples") {
        let derived = derive(&Profile::load(&path).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        assert!(report.failures().is_empty(), "{}", path.display());
        // a derived profile written out must read back as the same profile
        let text = derived.profile.to_toml().unwrap();
        assert_eq!(
            Profile::parse(&text).unwrap(),
            derived.profile,
            "{}",
            path.display()
        );
    }
}

#[test]
fn derivation_is_stable() {
    // deriving a derived profile fills nothing, so derive can be run again safely
    for path in toml_files("spec/examples") {
        let once = derive(&Profile::load(&path).unwrap()).unwrap();
        let twice = derive(&once.profile).unwrap();
        assert!(twice.filled.is_empty(), "{}", path.display());
    }
}
