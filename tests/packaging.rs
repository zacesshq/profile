//! The Homebrew formula and the winget manifests are written for one version, so a release that
//! bumps Cargo.toml without them would point every installer at the previous release.

use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn packaging_matches_the_crate_version() {
    let version = env!("CARGO_PKG_VERSION");
    let formula = read("packaging/homebrew/zacess-profile.rb");
    assert!(
        formula.contains(&format!("version \"{version}\"")),
        "the formula is not at {version}"
    );
    let urls: Vec<&str> = formula.lines().filter(|l| l.contains("url \"")).collect();
    assert_eq!(urls.len(), 4);
    for url in urls {
        assert!(url.contains(&format!("/v{version}/")), "{url}");
    }
    for file in [
        "packaging/winget/Zacess.Profile.yaml",
        "packaging/winget/Zacess.Profile.installer.yaml",
        "packaging/winget/Zacess.Profile.locale.en-US.yaml",
    ] {
        let text = read(file);
        assert!(
            text.contains(&format!("PackageVersion: {version}\n")),
            "{file} is not at {version}"
        );
        assert!(text.contains("ManifestVersion: 1.12.0"), "{file}");
    }
    // the extension manifests take their version from package.json
    let pkg: serde_json::Value = serde_json::from_str(&read("extension/package.json")).unwrap();
    assert_eq!(pkg["version"], version, "extension/package.json");
    let installer = read("packaging/winget/Zacess.Profile.installer.yaml");
    assert!(installer.contains(&format!(
        "zacess-profile-v{version}-x86_64-pc-windows-msvc\\zprf.exe"
    )));
}

#[test]
fn every_version_in_packaging_is_the_crate_version() {
    let version = env!("CARGO_PKG_VERSION");
    let tagged = regex::Regex::new(r"v(\d+\.\d+\.\d+)").unwrap();
    // a bare version too, apart from the winget manifest schema version
    let bare =
        regex::Regex::new(r#"(?:PackageVersion: |version "|Zacess Profile )(\d+\.\d+\.\d+)"#)
            .unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("packaging");
    let mut files = vec![dir.join("fill-checksums.sh"), dir.join("README.md")];
    for sub in ["homebrew", "winget"] {
        for entry in std::fs::read_dir(dir.join(sub)).unwrap() {
            files.push(entry.unwrap().path());
        }
    }
    let mut seen = 0;
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for re in [&tagged, &bare] {
            for cap in re.captures_iter(&text) {
                assert_eq!(&cap[1], version, "{} names {}", file.display(), &cap[0]);
                seen += 1;
            }
        }
    }
    // the four formula URLs, the Windows zip path, its URL and the release notes link at least
    assert!(seen >= 7, "only {seen} versions found");
}
