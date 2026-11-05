use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use zacess_profile::check::Kind;
use zacess_profile::profile::Profile;
use zacess_profile::watch::{self, Follow};
use zacess_profile::{check, checked, derive, osc, render};

/// One accessibility profile for every app.
///
/// A profile states what you need: contrast floors, font and size, reduced motion, cursor size and
/// spacing. zprf derives any colour the profile leaves out, checks every pair against
/// your floors and writes config files for terminals and editors.
#[derive(Parser)]
#[command(name = "zprf", version, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check every colour pair of one or more profiles and exit non-zero on any failure
    Check {
        /// Profile files to check
        #[arg(required = true)]
        profiles: Vec<PathBuf>,
        /// Print every pair, not only the failures and the tightest one
        #[arg(short, long)]
        verbose: bool,
        /// Fail when the profile leaves any colour to be derived
        #[arg(long)]
        no_derive: bool,
    },
    /// Fill in every missing colour and print the complete profile
    Derive {
        profile: PathBuf,
        /// Write the complete profile to this file instead of standard output
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Write config files for a target from a profile that passes its checks
    Render {
        profile: PathBuf,
        /// A target from list-targets or "all"
        #[arg(short, long)]
        target: String,
        /// The directory to write into; each target gets its own folder inside it
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Recolour the running terminal with OSC 4, 10, 11 and 12 sequences
    Apply {
        profile: PathBuf,
        /// Send the sequences to standard output, which recolours the terminal it is shown in
        #[arg(long, required_unless_present = "escaped")]
        live: bool,
        /// Print the sequences as readable text with ESC written as \e, sending nothing; it can
        /// be given with or without --live
        #[arg(long)]
        escaped: bool,
        /// Wrap the sequences for tmux passthrough (needs `set -g allow-passthrough on`)
        #[arg(long)]
        tmux: bool,
    },
    /// Re-check and re-render a profile each time it changes, following the system's light or
    /// dark appearance when the profile has both variants
    ///
    /// A profile named name-dark.toml or name-light.toml is paired with the other one in the same
    /// folder. Name another variant with --variant. Files are written only inside --out, under the
    /// id without its -dark or -light ending, so an app can point at one file for both. It runs
    /// until Ctrl+C.
    Watch {
        profile: PathBuf,
        /// A target from list-targets or "all"
        #[arg(short, long)]
        target: String,
        /// The directory to write into; each target gets its own folder inside it
        #[arg(short, long)]
        out: PathBuf,
        /// The other variant, when it is not found by name
        #[arg(long)]
        variant: Option<PathBuf>,
        /// Which variant to use: follow the system or always use the dark or the light one
        #[arg(long, value_enum, default_value_t = AppearanceArg::System)]
        appearance: AppearanceArg,
        /// How often to look for a change, in milliseconds
        #[arg(long, default_value_t = 500, value_parser = clap::value_parser!(u64).range(50..=60_000))]
        interval_ms: u64,
        #[arg(long, hide = true)]
        appearance_from: Option<PathBuf>,
        #[arg(long, hide = true)]
        stop_after_ms: Option<u64>,
    },
    /// Print the system accessibility settings a profile maps to, without changing any
    ///
    /// The lines are `defaults write` and `defaults delete` commands on macOS, `gsettings set` on GNOME and `reg add`
    /// values with their Settings page on Windows. zprf never runs them: this is a preview, and
    /// --print is required so that is never in doubt.
    Os {
        profile: PathBuf,
        /// Print the settings; nothing is ever applied
        #[arg(long, required = true)]
        print: bool,
        /// The platform to print for, when it is not the one this runs on
        #[arg(long, value_enum)]
        platform: Option<PlatformArg>,
    },
    /// List the targets render can write
    ListTargets,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum PlatformArg {
    Macos,
    Gnome,
    Windows,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum AppearanceArg {
    System,
    Dark,
    Light,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn run(command: Command) -> Result<ExitCode> {
    match command {
        Command::Check {
            profiles,
            verbose,
            no_derive,
        } => cmd_check(&profiles, verbose, no_derive),
        Command::Derive { profile, out } => cmd_derive(&profile, out.as_deref()),
        Command::Render {
            profile,
            target,
            out,
        } => cmd_render(&profile, &target, &out),
        Command::Apply {
            profile,
            live: _,
            escaped,
            tmux,
        } => cmd_apply(&profile, escaped, tmux),
        Command::Watch {
            profile,
            target,
            out,
            variant,
            appearance,
            interval_ms,
            appearance_from,
            stop_after_ms,
        } => {
            use zacess_profile::profile::Appearance;
            watch::run(&watch::Options {
                profile,
                variant,
                targets: targets(&target)?,
                out,
                follow: match appearance {
                    AppearanceArg::System => Follow::System,
                    AppearanceArg::Dark => Follow::Fixed(Appearance::Dark),
                    AppearanceArg::Light => Follow::Fixed(Appearance::Light),
                },
                interval: std::time::Duration::from_millis(interval_ms),
                appearance_from,
                stop_after: stop_after_ms.map(std::time::Duration::from_millis),
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Os {
            profile,
            print: _,
            platform,
        } => {
            use zacess_profile::os::{Platform, preview};
            let platform = match platform {
                Some(PlatformArg::Macos) => Platform::Macos,
                Some(PlatformArg::Gnome) => Platform::Gnome,
                Some(PlatformArg::Windows) => Platform::Windows,
                None => Platform::current(),
            };
            print!("{}", preview(&checked(&profile)?, platform));
            Ok(ExitCode::SUCCESS)
        }
        Command::ListTargets => {
            let width = render::TARGETS
                .iter()
                .map(|t| t.name.len())
                .max()
                .unwrap_or(0);
            for t in render::TARGETS {
                println!(
                    "{:width$}  {:9}  {}  ({}/{})",
                    t.name, t.app, t.summary, t.name, t.file
                );
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_check(paths: &[PathBuf], verbose: bool, no_derive: bool) -> Result<ExitCode> {
    let (mut failed, mut broken) = (0usize, 0usize);
    // one bad file must not hide the results for the rest, so each is reported and the worst
    // outcome decides the exit code at the end
    for path in paths {
        match check_one(path, verbose, no_derive) {
            Ok(true) => {}
            Ok(false) => failed += 1,
            Err(err) => {
                println!("ERROR {}", path.display());
                eprintln!("error: {err:#}");
                broken += 1;
            }
        }
    }
    if paths.len() > 1 {
        println!(
            "{} of {} profiles pass",
            paths.len() - failed - broken,
            paths.len()
        );
        if broken > 0 {
            println!("{broken} could not be read or completed");
        }
    }
    Ok(if broken > 0 {
        ExitCode::from(2)
    } else if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// Checks one profile and prints its report. `Ok(false)` means a pair failed.
fn check_one(path: &Path, verbose: bool, no_derive: bool) -> Result<bool> {
    let profile = Profile::load(path)?;
    let derived = derive::derive(&profile)
        .with_context(|| format!("{} cannot be completed", path.display()))?;
    let report = check::check(&derived.profile)?;
    let fails = report.failures();
    let gaps = no_derive && !derived.filled.is_empty();
    let ok = fails.is_empty() && !gaps;
    let c = &profile.needs.contrast;
    println!(
        "{} {} ({})",
        if ok { "pass" } else { "FAIL" },
        path.display(),
        profile.name
    );
    println!(
        "     {} pairs against text {}:1, non-text {}:1, overlay {}:1, terminal {}:1 and dim terminal {}:1, \
         with paired colours {} apart in tone and {} under colour vision deficiencies{}",
        report.pairs.len(),
        c.text,
        c.non_text,
        c.overlay,
        c.terminal,
        c.terminal_dim,
        c.tone_difference,
        c.colour_vision_distance,
        c.apca
            .map(|lc| format!(" plus APCA Lc {lc}"))
            .unwrap_or_default()
    );
    if !derived.filled.is_empty() {
        println!(
            "     derived {} colours: {}",
            derived.filled.len(),
            derived.filled.join(", ")
        );
    }
    if gaps {
        println!("     FAIL --no-derive was given but the profile leaves colours to derive");
    }
    for pair in &fails {
        println!("     FAIL {pair}");
    }
    if let Some(t) = report.tightest() {
        println!("     tightest {t}");
    }
    if verbose {
        for pair in &report.pairs {
            let tag = match (pair.passes(), pair.kind) {
                (false, _) => "FAIL",
                (true, Kind::Distance) => "dist",
                (true, Kind::Vision(_)) => "cvd ",
                (true, Kind::Tone) => "tone",
                (true, Kind::Apca) => "apca",
                (true, Kind::Ratio) => "ok  ",
            };
            // a failing pair already names its floor, so only a passing one has it added
            if pair.passes() {
                println!("     {tag} {pair} (floor {})", pair.required_shown());
            } else {
                println!("     {tag} {pair}");
            }
        }
    }
    Ok(ok)
}

fn cmd_derive(path: &Path, out: Option<&Path>) -> Result<ExitCode> {
    let profile = Profile::load(path)?;
    let derived = derive::derive(&profile)
        .with_context(|| format!("{} cannot be completed", path.display()))?;
    let text = annotate(&derived.profile.to_toml()?, &derived.filled);
    let header = format!(
        "# completed by zprf from {}; lines marked derived were filled in\n",
        path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into()
        )
    );
    match out {
        Some(file) => {
            std::fs::write(file, format!("{header}{text}"))
                .with_context(|| format!("cannot write {}", file.display()))?;
            eprintln!("wrote {}", file.display());
        }
        None => print!("{header}{text}"),
    }
    eprintln!(
        "derived {} colours{}",
        derived.filled.len(),
        if derived.filled.is_empty() {
            String::new()
        } else {
            format!(": {}", derived.filled.join(", "))
        }
    );
    let report = check::check(&derived.profile)?;
    let fails = report.failures();
    for pair in &fails {
        eprintln!("FAIL {pair}");
    }
    Ok(if fails.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Marks each filled line of the TOML output with a trailing comment, so a reader can tell the
/// colours they chose from the ones the tool chose.
fn annotate(toml: &str, filled: &[String]) -> String {
    let mut table = String::new();
    let mut out = String::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            table = trimmed.trim_matches(['[', ']']).to_string();
        }
        let key = trimmed.split_once(" = ").map(|(k, _)| k.trim());
        let name = match (table.as_str(), key) {
            ("palette", Some(k)) => Some(k.to_string()),
            ("palette.syntax", Some(k)) => Some(format!("syntax.{k}")),
            ("palette.terminal", Some("ansi")) => Some("terminal.ansi".to_string()),
            _ => None,
        };
        out.push_str(line);
        if name.is_some_and(|n| filled.contains(&n)) {
            out.push_str(" # derived");
        }
        out.push('\n');
    }
    out
}

fn targets(name: &str) -> Result<Vec<&'static render::Target>> {
    Ok(if name == "all" {
        render::TARGETS.iter().collect()
    } else {
        vec![render::target(name)?]
    })
}

fn cmd_render(path: &Path, target: &str, out: &Path) -> Result<ExitCode> {
    let profile = checked(path)?;
    for t in targets(target)? {
        let written = render::write_guarded(&profile, t, out, &[path.to_path_buf()])?;
        println!("wrote {}", written.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_apply(path: &Path, escaped: bool, tmux: bool) -> Result<ExitCode> {
    let profile = checked(path)?;
    let mut seq = osc::sequences(&profile)?;
    if tmux {
        seq = osc::tmux_passthrough(&seq);
    }
    let mut stdout = std::io::stdout().lock();
    if escaped {
        stdout.write_all(osc::escaped(&seq).as_bytes())?;
    } else {
        stdout.write_all(seq.as_bytes())?;
    }
    stdout.flush()?;
    Ok(ExitCode::SUCCESS)
}
