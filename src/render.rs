//! Renders a checked profile into config files through the minijinja templates in `templates/`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use minijinja::syntax::SyntaxConfig;
use minijinja::{AutoEscape, Environment, UndefinedBehavior};
use serde::Serialize;

use crate::colour::Rgb;
use crate::profile::{CursorSize, Profile, Syntax};

#[derive(Debug)]
pub struct Target {
    pub name: &'static str,
    pub app: &'static str,
    pub template: &'static str,
    source: &'static str,
    /// The written path inside `<out>/<name>/`, with `{id}` replaced by the profile id.
    pub file: &'static str,
    pub summary: &'static str,
}

macro_rules! template {
    ($file:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/", $file))
    };
}

pub const TARGETS: &[Target] = &[
    Target {
        name: "ghostty",
        app: "Ghostty",
        template: "ghostty.j2",
        source: template!("ghostty.j2"),
        file: "{id}",
        summary: "config file with colours, cursor and font settings",
    },
    Target {
        name: "kitty",
        app: "kitty",
        template: "kitty.conf.j2",
        source: template!("kitty.conf.j2"),
        file: "{id}.conf",
        summary: "include file with colours, cursor and font settings",
    },
    Target {
        name: "wezterm",
        app: "WezTerm",
        template: "wezterm.lua.j2",
        source: template!("wezterm.lua.j2"),
        file: "{id}.lua",
        summary: "Lua module that applies colours and settings to a config",
    },
    Target {
        name: "alacritty",
        app: "Alacritty",
        template: "alacritty.toml.j2",
        source: template!("alacritty.toml.j2"),
        file: "{id}.toml",
        summary: "TOML import with colours, cursor and font settings",
    },
    Target {
        name: "tmux",
        app: "tmux",
        template: "tmux.conf.j2",
        source: template!("tmux.conf.j2"),
        file: "{id}.tmux.conf",
        summary: "status bar, pane border, message and copy mode colours",
    },
    Target {
        name: "neovim",
        app: "Neovim",
        template: "neovim.lua.j2",
        source: template!("neovim.lua.j2"),
        file: "colors/{id}.lua",
        summary: "Lua colour scheme with Tree-sitter and diagnostic groups",
    },
    Target {
        name: "vscode",
        app: "VS Code",
        template: "vscode.json.j2",
        source: template!("vscode.json.j2"),
        file: "{id}.settings.json",
        summary: "settings.json fragment with colour and token customisations",
    },
];

pub fn target(name: &str) -> Result<&'static Target> {
    TARGETS.iter().find(|t| t.name == name).with_context(|| {
        format!(
            "there is no target called \"{name}\". The targets are {}",
            names()
        )
    })
}

pub fn names() -> String {
    let all: Vec<&str> = TARGETS.iter().map(|t| t.name).collect();
    match all.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => all.join(""),
    }
}

/// Everything a template can read. Every field is always present, as a value or as none, so the
/// strict undefined mode catches a misspelt name in a template instead of printing nothing.
#[derive(Serialize)]
struct TemplateContext<'a> {
    profile: ProfileContext<'a>,
    needs: NeedsContext,
    c: BTreeMap<&'static str, String>,
    syntax: BTreeMap<&'static str, String>,
    ansi: Vec<String>,
    ansi_bare: Vec<String>,
    styles: BTreeMap<&'static str, Vec<&'static str>>,
}

#[derive(Serialize)]
struct ProfileContext<'a> {
    name: &'a str,
    id: String,
    appearance: String,
}

#[derive(Serialize)]
struct NeedsContext {
    contrast: f64,
    reduce_motion: bool,
    font_family: Option<String>,
    font_size_pt: Option<f64>,
    /// VS Code sizes its fonts in CSS pixels, which are three quarters of a point.
    font_size_px: Option<u32>,
    line_spacing: Option<f64>,
    /// Line spacing as a cell height percentage, for terminals that size cells rather than lines.
    cell_height_percent: Option<u32>,
    /// Extra pixels between lines, for Alacritty's font offset.
    line_gap_px: Option<i32>,
    letter_spacing_px: Option<f64>,
    column_width: Option<u16>,
    cursor_size: &'static str,
    /// Bar or line cursor width in pixels: 2, 4 or 6 for standard, large and extra large.
    cursor_width_px: u8,
    /// The same width in points, for kitty, which sizes its beam cursor in points.
    cursor_width_pt: f64,
    block_cursor: bool,
    focus_ring_width_px: Option<u8>,
}

/// Font style per syntax role, matching the Zacess themes so a token never rests on colour alone.
pub const STYLES: &[(&str, &[&str])] = &[
    ("keyword", &["bold"]),
    ("control", &["bold"]),
    ("tag", &["bold"]),
    ("parameter", &["italic"]),
    ("comment", &["italic"]),
    ("invalid", &["underline"]),
];

fn context(profile: &Profile) -> Result<TemplateContext<'_>> {
    let p = &profile.palette;
    let hex = |c: Option<Rgb>, role: &str| -> Result<String> {
        c.map(Rgb::hex)
            .with_context(|| format!("{role} is missing: run derive before render"))
    };
    let mut c = BTreeMap::new();
    for &role in crate::profile::Palette::ROLES {
        c.insert(role, hex(p.get(role), role)?);
    }
    let mut syntax = BTreeMap::new();
    for &role in Syntax::ROLES {
        syntax.insert(role, hex(p.syntax.get(role), role)?);
    }
    let ansi = p
        .terminal
        .ansi
        .clone()
        .context("terminal.ansi is missing: run derive before render")?;
    let needs = &profile.needs;
    let cursor = needs.cursor_size.unwrap_or(CursorSize::Standard);
    let size_pt = needs.font.size_pt;
    Ok(TemplateContext {
        profile: ProfileContext {
            name: &profile.name,
            id: profile.slug(),
            appearance: profile
                .appearance
                .map_or_else(|| "dark".to_string(), |a| a.to_string()),
        },
        needs: NeedsContext {
            contrast: needs.contrast.text,
            reduce_motion: needs.reduce_motion,
            font_family: needs.font.family.clone().filter(|f| !f.trim().is_empty()),
            font_size_pt: size_pt,
            font_size_px: size_pt.map(|pt| (pt * 4.0 / 3.0).round() as u32),
            line_spacing: needs.line_spacing,
            cell_height_percent: needs.line_spacing.map(|l| (l * 100.0).round() as u32),
            line_gap_px: match (needs.line_spacing, size_pt) {
                (Some(l), Some(pt)) => Some(((l - 1.0) * pt * 4.0 / 3.0).round() as i32),
                _ => None,
            },
            letter_spacing_px: match (needs.character_spacing, size_pt) {
                (Some(em), Some(pt)) => Some(((em * pt * 4.0 / 3.0) * 100.0).round() / 100.0),
                _ => None,
            },
            column_width: needs.column_width,
            cursor_size: match cursor {
                CursorSize::Standard => "standard",
                CursorSize::Large => "large",
                CursorSize::ExtraLarge => "extra_large",
            },
            cursor_width_px: match cursor {
                CursorSize::Standard => 2,
                CursorSize::Large => 4,
                CursorSize::ExtraLarge => 6,
            },
            cursor_width_pt: match cursor {
                CursorSize::Standard => 1.5,
                CursorSize::Large => 3.0,
                CursorSize::ExtraLarge => 4.5,
            },
            block_cursor: cursor != CursorSize::Standard,
            focus_ring_width_px: needs.focus_ring_width_px,
        },
        c,
        syntax,
        ansi_bare: ansi.iter().map(|c| c.hex_bare()).collect(),
        ansi: ansi.iter().map(|c| c.hex()).collect(),
        styles: Syntax::ROLES
            .iter()
            .map(|&role| {
                let style = STYLES
                    .iter()
                    .find(|(r, _)| *r == role)
                    .map_or(&[][..], |s| s.1);
                (role, style.to_vec())
            })
            .collect(),
    })
}

fn environment() -> Result<Environment<'static>> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    // auto-escaping would quote every value a second time in the JSON target; values are hex
    // colours and numbers; free text such as a font name goes through tojson in the template
    env.set_auto_escape_callback(|_| AutoEscape::None);
    env.set_syntax(
        SyntaxConfig::builder()
            .trim_blocks(true)
            .lstrip_blocks(true)
            .keep_trailing_newline(true)
            .build()?,
    );
    for t in TARGETS {
        env.add_template(t.template, t.source)
            .with_context(|| format!("template {} does not parse", t.template))?;
    }
    Ok(env)
}

/// Renders one target to a string, without touching the file system.
pub fn render(profile: &Profile, target: &Target) -> Result<String> {
    let env = environment()?;
    let ctx = context(profile)?;
    env.get_template(target.template)?
        .render(minijinja::Value::from(minijinja::value::Serde(&ctx)))
        .with_context(|| format!("rendering {} failed", target.template))
}

/// The relative path a target writes for a profile, inside `<out>/<target>/`.
pub fn relative_path(profile: &Profile, target: &Target) -> PathBuf {
    PathBuf::from(target.name).join(target.file.replace("{id}", &profile.slug()))
}

/// Writes one target below `out` and returns the path written.
pub fn write(profile: &Profile, target: &Target, out: &Path) -> Result<PathBuf> {
    if out.as_os_str().is_empty() {
        bail!("--out must name a directory");
    }
    let text = render(profile, target)?;
    let path = out.join(relative_path(profile, target));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    }
    std::fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;
    Ok(path)
}
