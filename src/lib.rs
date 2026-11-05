//! Zacess Profile: one accessibility profile for every app.
//!
//! A profile states what a person needs (contrast floors, font, motion, cursor and spacing) and may
//! carry a palette. The library derives any colour the palette leaves out, checks every pair
//! against the floors and renders the result for terminals and editors.

pub mod check;
pub mod colour;
pub mod derive;
pub mod os;
pub mod osc;
pub mod profile;
pub mod render;
pub mod watch;

pub use check::{Report, check};
pub use derive::{Derived, derive};
pub use profile::Profile;

use std::path::Path;

use anyhow::{Context, Result, bail};

/// Loads, completes and checks a profile, refusing one that breaks its own needs.
pub fn checked(path: &Path) -> Result<Profile> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    checked_text(&text, path)
}

/// As `checked`, from text already read, with `path` used only to name the file in messages.
pub fn checked_text(text: &str, path: &Path) -> Result<Profile> {
    let profile = Profile::parse(text)
        .with_context(|| format!("{} is not a valid profile", path.display()))?;
    let derived =
        derive(&profile).with_context(|| format!("{} cannot be completed", path.display()))?;
    let report = check(&derived.profile)?;
    let fails = report.failures();
    if !fails.is_empty() {
        let lines: Vec<String> = fails.iter().map(|p| format!("  {p}")).collect();
        bail!(
            "{} breaks its own needs, so nothing was written:\n{}",
            path.display(),
            lines.join("\n")
        );
    }
    Ok(derived.profile)
}
