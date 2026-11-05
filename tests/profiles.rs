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
fn there_are_eight_starter_profiles() {
    assert_eq!(toml_files("profiles").len(), 8);
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
