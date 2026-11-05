//! `zprf watch`: re-checks and re-renders a profile whenever it changes, following the system's
//! light or dark appearance when the profile comes in both variants.
//!
//! The watcher polls rather than subscribing to file system events. Editors save in different
//! ways (in place, through a temporary file, by renaming) and a poll of the file's contents sees
//! every one of them the same way on every platform. The appearance has to be polled in any case,
//! since no platform offers a change event to a plain command line program.
//!
//! Nothing is written outside `--out` and an app's real config is never touched. A profile that
//! breaks its own needs is reported and the files from the last good version are left in place.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};

use crate::checked_text;
use crate::profile::{Appearance, Profile};
use crate::render::{self, Target};

/// How the watcher picks between two variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Follow {
    /// Follow the system appearance.
    System,
    /// Always use the variant with this appearance.
    Fixed(Appearance),
}

pub struct Options {
    pub profile: PathBuf,
    pub variant: Option<PathBuf>,
    pub targets: Vec<&'static Target>,
    pub out: PathBuf,
    pub follow: Follow,
    pub interval: Duration,
    /// A file holding `dark` or `light`, read in place of the system setting. Used by the tests,
    /// so they never depend on the machine they run on.
    pub appearance_from: Option<PathBuf>,
    /// Stop after this long. Used by the tests, so a watcher can never outlive them.
    pub stop_after: Option<Duration>,
}

/// The other variant of a profile, found by name: `name-dark.toml` beside `name-light.toml`.
pub fn sibling_variant(path: &Path) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_str()?;
    let other = match stem.strip_suffix("-dark") {
        Some(base) => format!("{base}-light"),
        None => format!("{}-dark", stem.strip_suffix("-light")?),
    };
    let ext = path
        .extension()
        .map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
    let candidate = path.with_file_name(format!("{other}{ext}"));
    candidate.is_file().then_some(candidate)
}

/// The id both variants are written under, so an app's config can point at one file whichever
/// variant is active: the profile id without its `-dark` or `-light` ending.
pub fn shared_id(id: &str) -> String {
    id.strip_suffix("-dark")
        .or_else(|| id.strip_suffix("-light"))
        .filter(|base| !base.is_empty())
        .unwrap_or(id)
        .to_string()
}

/// Reads `dark` or `light` from text, as written by the system tools below or by a test.
pub fn parse_appearance(text: &str) -> Option<Appearance> {
    match text.trim().to_ascii_lowercase().as_str() {
        "dark" => Some(Appearance::Dark),
        "light" => Some(Appearance::Light),
        _ => None,
    }
}

/// The freedesktop portal answers with a variant holding 0 (no preference), 1 (dark) or 2 (light).
pub fn parse_portal(text: &str) -> Option<Appearance> {
    let digit = text.chars().rev().find(char::is_ascii_digit)?;
    match digit {
        '1' => Some(Appearance::Dark),
        '2' => Some(Appearance::Light),
        _ => None,
    }
}

/// `reg query` prints the value as `0x0` (dark) or `0x1` (light) at the end of its line.
pub fn parse_reg(text: &str) -> Option<Appearance> {
    let line = text.lines().find(|l| l.contains("AppsUseLightTheme"))?;
    match line.split_whitespace().last()? {
        "0x0" => Some(Appearance::Dark),
        "0x1" => Some(Appearance::Light),
        _ => None,
    }
}

/// Reads the system appearance without changing anything. `None` when it cannot be read.
pub fn system_appearance() -> Option<Appearance> {
    let output = |program: &str, args: &[&str]| {
        Command::new(program).args(args).output().ok().map(|o| {
            (
                o.status.success(),
                String::from_utf8_lossy(&o.stdout).into_owned(),
            )
        })
    };
    if cfg!(target_os = "macos") {
        // the key exists only while dark mode is on, so a failed read means light
        let (ok, text) = output("defaults", &["read", "-g", "AppleInterfaceStyle"])?;
        Some(if ok && text.trim() == "Dark" {
            Appearance::Dark
        } else {
            Appearance::Light
        })
    } else if cfg!(target_os = "windows") {
        let (ok, text) = output(
            "reg",
            &[
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
            ],
        )?;
        ok.then(|| parse_reg(&text)).flatten()
    } else {
        // the desktop portal is shared by GNOME, KDE and the other freedesktop desktops
        let (ok, text) = output(
            "gdbus",
            &[
                "call",
                "--session",
                "--dest",
                "org.freedesktop.portal.Desktop",
                "--object-path",
                "/org/freedesktop/portal/desktop",
                "--method",
                "org.freedesktop.portal.Settings.Read",
                "org.freedesktop.appearance",
                "color-scheme",
            ],
        )?;
        ok.then(|| parse_portal(&text)).flatten()
    }
}

/// Writes one target, leaving a file alone when its contents would not change, so an app that
/// reloads on change is not woken for nothing. Returns whether the file was written.
fn write_if_changed(
    profile: &Profile,
    target: &Target,
    out: &Path,
    inputs: &[PathBuf],
) -> Result<bool> {
    let text = render::render(profile, target)?;
    let path = out.join(render::relative_path(profile, target));
    if std::fs::read_to_string(&path).is_ok_and(|old| old == text) {
        return Ok(false);
    }
    render::write_guarded(profile, target, out, inputs)?;
    Ok(true)
}

/// Why a build did not finish. A profile that fails waits for the file to change; a write that
/// fails is tried again on the next look, since the cause may be outside the profile.
enum Failure {
    Profile(anyhow::Error),
    Write(anyhow::Error),
}

struct Variants {
    /// The profile given first, then its other variant when there is one.
    paths: Vec<PathBuf>,
}

impl Variants {
    fn contents(&self) -> Vec<Option<String>> {
        self.paths
            .iter()
            .map(|p| std::fs::read_to_string(p).ok())
            .collect()
    }
}

/// Runs until interrupted or until `stop_after` when it is set.
pub fn run(opts: &Options) -> Result<()> {
    if opts.out.as_os_str().is_empty() {
        bail!("--out must name a directory");
    }
    let variant = match &opts.variant {
        Some(v) if !v.is_file() => bail!("the variant {} does not exist", v.display()),
        Some(v) => Some(v.clone()),
        None => sibling_variant(&opts.profile),
    };
    let variants = Variants {
        paths: std::iter::once(opts.profile.clone())
            .chain(variant)
            .collect(),
    };
    let two = variants.paths.len() == 2;
    if two {
        println!(
            "watching {} and {}; following the {}",
            variants.paths[0].display(),
            variants.paths[1].display(),
            match opts.follow {
                Follow::System => "system appearance".to_string(),
                Follow::Fixed(a) => format!("{a} variant only (--appearance {a})"),
            }
        );
    } else {
        println!("watching {}", variants.paths[0].display());
        if let Follow::Fixed(a) = opts.follow {
            eprintln!(
                "--appearance {a} has no effect: {} has no other variant to choose between",
                variants.paths[0].display()
            );
        }
    }
    println!(
        "writing {} into {}; press Ctrl+C to stop",
        opts.targets
            .iter()
            .map(|t| t.name)
            .collect::<Vec<_>>()
            .join(", "),
        opts.out.display()
    );

    let started = Instant::now();
    let mut last_inputs: Option<Vec<Option<String>>> = None;
    let mut last_appearance: Option<Option<Appearance>> = None;
    let mut warned_unknown = false;
    let mut last_message: Option<String> = None;
    loop {
        let inputs = variants.contents();
        let appearance = if two {
            match opts.follow {
                Follow::Fixed(a) => Some(a),
                Follow::System => match &opts.appearance_from {
                    Some(file) => std::fs::read_to_string(file)
                        .ok()
                        .and_then(|t| parse_appearance(&t)),
                    None => system_appearance(),
                },
            }
        } else {
            None
        };
        if two && appearance.is_none() && !warned_unknown {
            eprintln!(
                "the system appearance cannot be read here, so {} is used",
                variants.paths[0].display()
            );
            warned_unknown = true;
        }
        let changed =
            last_inputs.as_ref() != Some(&inputs) || last_appearance.as_ref() != Some(&appearance);
        if changed {
            // built from the text just read, so what is recorded as seen is what was checked
            let (message, retry) = match build(&variants, &inputs, two, appearance, opts) {
                Ok(lines) => (lines, false),
                Err(Failure::Profile(err)) => (
                    format!("error: {err:#}\nthe files from the last profile that passed are kept"),
                    false,
                ),
                Err(Failure::Write(err)) => (
                    format!("error: {err:#}\nthis is tried again until it works"),
                    true,
                ),
            };
            // a write that keeps failing is reported once, not on every look
            if last_message.as_ref() != Some(&message) {
                // the summary goes to standard output and every error to standard error
                let (summary, errors) = match message.find("error:") {
                    Some(at) => message.split_at(at),
                    None => (message.as_str(), ""),
                };
                if !summary.trim().is_empty() {
                    println!("{}", summary.trim_end());
                }
                if !errors.is_empty() {
                    eprintln!("{errors}");
                }
                last_message = Some(message);
            }
            if !retry {
                last_inputs = Some(inputs);
                last_appearance = Some(appearance);
            }
        }
        if opts
            .stop_after
            .is_some_and(|limit| started.elapsed() >= limit)
        {
            return Ok(());
        }
        std::thread::sleep(opts.interval);
    }
}

/// Checks each variant on its own and renders the one that matches the appearance. A broken other
/// variant is reported without holding the matching one back.
fn build(
    variants: &Variants,
    inputs: &[Option<String>],
    two: bool,
    appearance: Option<Appearance>,
    opts: &Options,
) -> std::result::Result<String, Failure> {
    let results: Vec<Result<Profile>> = variants
        .paths
        .iter()
        .zip(inputs)
        .map(|(p, text)| match text {
            Some(text) => checked_text(text, p),
            None => bail!("cannot read {}", p.display()),
        })
        .collect();
    if let [Ok(a), Ok(b)] = results.as_slice()
        && a.appearance == b.appearance
    {
        return Err(Failure::Profile(anyhow::anyhow!(
            "{} and {} are both {}, so there is nothing to switch between",
            variants.paths[0].display(),
            variants.paths[1].display(),
            a.appearance.map_or("unset".into(), |x| x.to_string())
        )));
    }
    let matches = |p: &Profile| appearance.is_none_or(|a| p.appearance == Some(a));
    let index = results
        .iter()
        .position(|r| r.as_ref().is_ok_and(matches))
        // with the appearance unknown or no variant matching, the first that passes is used
        .or_else(|| results.iter().position(|r| r.is_ok()));
    let broken: Vec<String> = results
        .iter()
        .filter_map(|r| r.as_ref().err().map(|e| format!("error: {e:#}")))
        .collect();
    let Some(index) = index else {
        return Err(Failure::Profile(anyhow::anyhow!(broken.join("\n"))));
    };
    // the wanted variant is broken when it is not the one chosen and the appearance asked for it
    if appearance.is_some() && results.iter().any(|r| r.is_err()) {
        let chosen = results[index].as_ref().ok().map(|p| p.appearance);
        if chosen != Some(appearance) {
            return Err(Failure::Profile(anyhow::anyhow!(broken.join("\n"))));
        }
    }
    let mut active = results[index].as_ref().ok().cloned().expect("an Ok result");
    if two {
        let first = results[0]
            .as_ref()
            .ok()
            .map_or_else(|| file_stem_id(&variants.paths[0]), Profile::slug);
        active.id = Some(shared_id(&first));
    }
    let mut written = 0;
    for t in &opts.targets {
        match write_if_changed(&active, t, &opts.out, &variants.paths) {
            Ok(true) => written += 1,
            Ok(false) => {}
            Err(err) => return Err(Failure::Write(err)),
        }
    }
    let files: Vec<String> = opts
        .targets
        .iter()
        .map(|t| {
            opts.out
                .join(render::relative_path(&active, t))
                .display()
                .to_string()
        })
        .collect();
    let mut lines = vec![format!(
        "{} {} ({}): {} of {} files changed: {}",
        if written > 0 { "rendered" } else { "unchanged" },
        variants.paths[index].display(),
        active.appearance.map_or("unset".into(), |a| a.to_string()),
        written,
        files.len(),
        files.join(", ")
    )];
    for b in broken {
        lines.push(format!("{b}\nthe other variant was rendered without it"));
    }
    Ok(lines.join("\n"))
}

/// An id from a file name, for when the first variant is broken and its own id cannot be read.
fn file_stem_id(path: &Path) -> String {
    path.file_stem()
        .map_or_else(|| "profile".into(), |s| s.to_string_lossy().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_found_by_name() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("profiles");
        let dark = root.join("zacess-deep-contrast-dark.toml");
        assert_eq!(
            sibling_variant(&dark),
            Some(root.join("zacess-deep-contrast-light.toml"))
        );
        // zacess-dark and zacess-light are the plain pair of the same family
        assert_eq!(
            sibling_variant(&root.join("zacess-dark.toml")),
            Some(root.join("zacess-light.toml"))
        );
        assert_eq!(sibling_variant(&root.join("zacess-cream.toml")), None);
    }

    #[test]
    fn the_shared_id_drops_the_appearance() {
        assert_eq!(
            shared_id("zacess-deep-contrast-dark"),
            "zacess-deep-contrast"
        );
        assert_eq!(shared_id("mine-light"), "mine");
        assert_eq!(shared_id("plain"), "plain");
        // an id that is only the ending keeps it, since an empty id cannot name a file
        assert_eq!(shared_id("dark"), "dark");
    }

    #[test]
    fn system_answers_are_read() {
        assert_eq!(parse_portal("(<<uint32 1>>,)\n"), Some(Appearance::Dark));
        assert_eq!(parse_portal("(<<uint32 2>>,)\n"), Some(Appearance::Light));
        assert_eq!(parse_portal("(<<uint32 0>>,)\n"), None);
        let reg = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\r\n    AppsUseLightTheme    REG_DWORD    0x0\r\n";
        assert_eq!(parse_reg(reg), Some(Appearance::Dark));
        assert_eq!(
            parse_reg(&reg.replace("0x0", "0x1")),
            Some(Appearance::Light)
        );
        assert_eq!(parse_appearance(" Dark\n"), Some(Appearance::Dark));
        assert_eq!(parse_appearance("dusk"), None);
    }
}
