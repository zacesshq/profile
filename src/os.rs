//! `zprf os --print`: the system accessibility settings a profile maps to, printed as the commands
//! that would set them.
//!
//! This module only builds text. It never runs a command, writes a setting or reads the system.
//! `tests/os.rs` checks that it cannot: there is no way to apply what it prints from `zprf`.
//! Reading the lines and running any of them is left to the person.

use std::fmt::Write as _;

use crate::colour::Rgb;
use crate::profile::{Appearance, CursorSize, FontFallback, Profile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Macos,
    Gnome,
    Windows,
}

impl Platform {
    /// The platform this build runs on, with every Unix other than macOS taken as GNOME.
    pub fn current() -> Platform {
        if cfg!(target_os = "macos") {
            Platform::Macos
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Gnome
        }
    }

    fn name(self) -> &'static str {
        match self {
            Platform::Macos => "macOS",
            Platform::Gnome => "GNOME",
            Platform::Windows => "Windows",
        }
    }

    /// The header names the shell too, since the Windows lines are written for PowerShell.
    fn heading(self) -> &'static str {
        match self {
            Platform::Windows => "Windows, as PowerShell lines",
            other => other.name(),
        }
    }
}

/// GNOME's interface font is 11 points and Windows sizes text from 9 point Segoe UI, so a profile's
/// size becomes a scale factor against each.
const GNOME_BASE_PT: f64 = 11.0;
const WINDOWS_BASE_PT: f64 = 9.0;

/// The preview for one platform. Every line is either a comment starting with `#` or a command
/// for that platform's own settings tool.
pub fn preview(profile: &Profile, platform: Platform) -> String {
    let mut out = String::new();
    let needs = &profile.needs;
    let line = |out: &mut String, text: &str| {
        out.push_str(text);
        out.push('\n');
    };
    let _ = writeln!(
        out,
        "# system settings for {} from the profile {}",
        platform.heading(),
        profile.name
    );
    line(
        &mut out,
        "# preview only: zprf has not run any of these and has no option to",
    );
    line(
        &mut out,
        "# colours and the appearance the profile leaves out are the ones zprf derive chooses",
    );
    if platform == Platform::Macos {
        line(
            &mut out,
            "# writing com.apple.universalaccess needs a terminal with Full Disk Access on recent macOS",
        );
    }
    let mut unmapped: Vec<&str> = Vec::new();
    let aaa = needs.contrast.text >= 7.0;
    let caret_px = match needs.cursor_size {
        Some(CursorSize::Large) => Some(4),
        Some(CursorSize::ExtraLarge) => Some(6),
        _ => None,
    };
    let font = needs
        .font
        .family
        .as_deref()
        .filter(|f| !f.trim().is_empty());

    match platform {
        Platform::Macos => {
            if needs.reduce_motion {
                line(
                    &mut out,
                    "\n# reduce_motion: Accessibility > Display > Reduce motion",
                );
                line(
                    &mut out,
                    "defaults write com.apple.universalaccess reduceMotion -bool true",
                );
            }
            if aaa {
                line(
                    &mut out,
                    "\n# a text floor of 7:1 or more: Accessibility > Display > Increase contrast",
                );
                line(
                    &mut out,
                    "defaults write com.apple.universalaccess increaseContrast -bool true",
                );
            }
            if needs.hide_decoration == Some(true) {
                line(
                    &mut out,
                    "\n# hide_decoration: Accessibility > Display > Reduce transparency",
                );
                line(
                    &mut out,
                    "defaults write com.apple.universalaccess reduceTransparency -bool true",
                );
            }
            match profile.appearance {
                Some(Appearance::Dark) => {
                    line(&mut out, "\n# appearance = dark: Appearance > Dark");
                    line(
                        &mut out,
                        "defaults write -g AppleInterfaceStyle -string Dark",
                    );
                }
                Some(Appearance::Light) => {
                    line(
                        &mut out,
                        "\n# appearance = light: Appearance > Light, which macOS stores by removing the key",
                    );
                    line(&mut out, "defaults delete -g AppleInterfaceStyle");
                }
                None => {}
            }
            if let Some(soft) = profile.palette.selection_soft {
                let _ = writeln!(
                    out,
                    "\n# selection_soft {soft}: Appearance > Highlight colour, drawn behind selected text"
                );
                let _ = writeln!(
                    out,
                    "defaults write -g AppleHighlightColor -string \"{} Other\"",
                    unit_rgb(soft)
                );
            }
            for (key, set) in [
                ("font.family", font.is_some()),
                ("font.size_pt", needs.font.size_pt.is_some()),
                ("cursor_size", caret_px.is_some()),
                ("focus_ring_width_px", needs.focus_ring_width_px.is_some()),
            ] {
                if set {
                    unmapped.push(key);
                }
            }
        }
        Platform::Gnome => {
            let set = |out: &mut String, schema: &str, key: &str, value: &str| {
                let _ = writeln!(out, "gsettings set {schema} {key} {value}");
            };
            let iface = "org.gnome.desktop.interface";
            if needs.reduce_motion {
                line(
                    &mut out,
                    "\n# reduce_motion: Accessibility > Seeing > Reduce animation plus a steady text cursor",
                );
                set(&mut out, iface, "enable-animations", "false");
                set(&mut out, iface, "cursor-blink", "false");
            }
            if aaa {
                line(
                    &mut out,
                    "\n# a text floor of 7:1 or more: Accessibility > Seeing > High contrast",
                );
                set(
                    &mut out,
                    "org.gnome.desktop.a11y.interface",
                    "high-contrast",
                    "true",
                );
            }
            if let Some(a) = profile.appearance {
                let value = match a {
                    Appearance::Dark => "'prefer-dark'",
                    Appearance::Light => "'prefer-light'",
                };
                let _ = writeln!(out, "\n# appearance = {a}: Appearance > Style");
                set(&mut out, iface, "color-scheme", value);
            }
            if let Some(pt) = needs.font.size_pt {
                let factor = (pt / GNOME_BASE_PT).clamp(0.5, 3.0);
                let _ = writeln!(
                    out,
                    "\n# size_pt = {pt}: Accessibility > Seeing > Large text, as a scale of the {GNOME_BASE_PT} point interface font"
                );
                set(
                    &mut out,
                    iface,
                    "text-scaling-factor",
                    &format!("{:.2}", (factor * 100.0).round() / 100.0),
                );
            }
            if let Some(family) = font {
                let key = match needs.font.fallback {
                    Some(FontFallback::Serif | FontFallback::SansSerif) => "document-font-name",
                    Some(FontFallback::Monospace) | None => "monospace-font-name",
                };
                let size = needs
                    .font
                    .size_pt
                    .map_or(String::new(), |pt| format!(" {pt}"));
                let _ = writeln!(out, "\n# font.family: the {key} setting");
                // the value is a GVariant string in double quotes, which the spec already keeps
                // free of quotes and backslashes; a single quote in the name is closed and
                // escaped for the shell so a shared profile cannot end the argument and add a command
                let value = format!("\"{family}{size}\"").replace('\'', "'\\''");
                set(&mut out, iface, key, &format!("'{value}'"));
            }
            for (key, set) in [
                ("cursor_size", caret_px.is_some()),
                ("focus_ring_width_px", needs.focus_ring_width_px.is_some()),
                ("hide_decoration", needs.hide_decoration == Some(true)),
            ] {
                if set {
                    unmapped.push(key);
                }
            }
        }
        Platform::Windows => {
            let reg = |out: &mut String, key: &str, name: &str, kind: &str, data: &str| {
                let _ = writeln!(out, "reg add \"{key}\" /v {name} /t {kind} /d {data} /f");
            };
            let desktop = r"HKCU\Control Panel\Desktop";
            let personalise = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
            if needs.reduce_motion {
                line(
                    &mut out,
                    "\n# reduce_motion: Settings > Accessibility > Visual effects > Animation effects off.\n\
                     # That switch also stops animation inside apps, which has no single registry value;\n\
                     # the values below stop window animation and the blinking text cursor",
                );
                reg(
                    &mut out,
                    r"HKCU\Control Panel\Desktop\WindowMetrics",
                    "MinAnimate",
                    "REG_SZ",
                    "0",
                );
                reg(&mut out, desktop, "CursorBlinkRate", "REG_SZ", "-1");
            }
            if aaa {
                line(
                    &mut out,
                    "\n# a text floor of 7:1 or more: Settings > Accessibility > Contrast themes.\n\
                     # A contrast theme replaces every colour, so it is chosen there rather than set here",
                );
            }
            if let Some(a) = profile.appearance {
                let light = if a == Appearance::Light { "1" } else { "0" };
                let _ = writeln!(
                    out,
                    "\n# appearance = {a}: Settings > Personalisation > Colours > Choose your mode"
                );
                reg(
                    &mut out,
                    personalise,
                    "AppsUseLightTheme",
                    "REG_DWORD",
                    light,
                );
                reg(
                    &mut out,
                    personalise,
                    "SystemUsesLightTheme",
                    "REG_DWORD",
                    light,
                );
            }
            if needs.hide_decoration == Some(true) {
                line(
                    &mut out,
                    "\n# hide_decoration: Settings > Accessibility > Visual effects > Transparency effects off",
                );
                reg(
                    &mut out,
                    personalise,
                    "EnableTransparency",
                    "REG_DWORD",
                    "0",
                );
            }
            if let Some(px) = caret_px {
                let _ = writeln!(
                    out,
                    "\n# cursor_size: Settings > Accessibility > Text cursor > Text cursor thickness, {px} pixels"
                );
                reg(
                    &mut out,
                    desktop,
                    "CaretWidth",
                    "REG_DWORD",
                    &px.to_string(),
                );
            }
            if let Some(w) = needs.focus_ring_width_px {
                let _ = writeln!(
                    out,
                    "\n# focus_ring_width_px = {w}: the width Windows draws its own focus rectangle at"
                );
                reg(
                    &mut out,
                    desktop,
                    "FocusBorderWidth",
                    "REG_DWORD",
                    &w.to_string(),
                );
                reg(
                    &mut out,
                    desktop,
                    "FocusBorderHeight",
                    "REG_DWORD",
                    &w.to_string(),
                );
            }
            if let Some(pt) = needs.font.size_pt {
                let percent = ((pt / WINDOWS_BASE_PT) * 100.0).round().clamp(100.0, 225.0);
                let _ = writeln!(
                    out,
                    "\n# size_pt = {pt}: Settings > Accessibility > Text size, as a scale of 9 point Segoe UI from 100 to 225 per cent"
                );
                reg(
                    &mut out,
                    r"HKCU\Software\Microsoft\Accessibility",
                    "TextScaleFactor",
                    "REG_DWORD",
                    &format!("{percent}"),
                );
            }
            if font.is_some() {
                unmapped.push("font.family");
            }
        }
    }
    for (key, set) in [
        ("line_spacing", needs.line_spacing.is_some()),
        ("character_spacing", needs.character_spacing.is_some()),
        ("column_width", needs.column_width.is_some()),
        ("single_column", needs.single_column == Some(true)),
    ] {
        if set {
            unmapped.push(key);
        }
    }
    if !unmapped.is_empty() {
        let _ = writeln!(
            out,
            "\n# {} has no system setting for: {}",
            platform.name(),
            unmapped.join(", ")
        );
    }
    out
}

/// `AppleHighlightColor` stores each channel as a fraction of one.
fn unit_rgb(c: Rgb) -> String {
    let f = |v: u8| format!("{:.6}", f64::from(v) / 255.0);
    format!("{} {} {}", f(c.r), f(c.g), f(c.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_need() -> Profile {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/examples/every-need.toml");
        crate::checked(&path).unwrap()
    }

    #[test]
    fn macos_lines_are_defaults_writes() {
        let text = preview(&every_need(), Platform::Macos);
        assert!(text.contains("defaults write com.apple.universalaccess reduceMotion -bool true"));
        assert!(
            text.contains("defaults write com.apple.universalaccess reduceTransparency -bool true")
        );
        assert!(text.contains("defaults write -g AppleInterfaceStyle -string Dark"));
        assert!(text.contains("AppleHighlightColor"));
        assert!(text.contains("has no system setting for: font.family, font.size_pt, cursor_size, focus_ring_width_px, line_spacing, character_spacing, column_width, single_column"));
    }

    #[test]
    fn gnome_lines_are_gsettings() {
        let text = preview(&every_need(), Platform::Gnome);
        assert!(text.contains("gsettings set org.gnome.desktop.interface enable-animations false"));
        assert!(
            text.contains("gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'")
        );
        assert!(
            text.contains("gsettings set org.gnome.desktop.interface text-scaling-factor 1.45")
        );
        assert!(text.contains(
            "gsettings set org.gnome.desktop.interface monospace-font-name '\"Atkinson Hyperlegible Mono 16\"'"
        ));
    }

    #[test]
    fn windows_lines_are_registry_values() {
        let text = preview(&every_need(), Platform::Windows);
        assert!(text.contains(
            r#"reg add "HKCU\Control Panel\Desktop" /v CaretWidth /t REG_DWORD /d 6 /f"#
        ));
        assert!(text.contains(
            r#"reg add "HKCU\Control Panel\Desktop" /v FocusBorderWidth /t REG_DWORD /d 4 /f"#
        ));
        assert!(text.contains("/v TextScaleFactor /t REG_DWORD /d 178 /f"));
        assert!(text.contains("/v AppsUseLightTheme /t REG_DWORD /d 0 /f"));
    }

    #[test]
    fn every_line_is_a_comment_or_a_settings_command() {
        for platform in [Platform::Macos, Platform::Gnome, Platform::Windows] {
            let tool = match platform {
                Platform::Macos => "defaults ",
                Platform::Gnome => "gsettings set ",
                Platform::Windows => "reg add ",
            };
            for line in preview(&every_need(), platform).lines() {
                assert!(
                    line.is_empty() || line.starts_with('#') || line.starts_with(tool),
                    "{line}"
                );
            }
        }
    }

    #[test]
    fn a_need_that_is_not_set_prints_nothing() {
        let p = crate::derive(
            &Profile::parse("spec_version = 1\nname = \"Plain\"\n[needs]\nreduce_motion = false")
                .unwrap(),
        )
        .unwrap()
        .profile;
        let text = preview(&p, Platform::Gnome);
        assert!(!text.contains("enable-animations"), "{text}");
        assert!(!text.contains("text-scaling-factor"), "{text}");
    }

    #[test]
    fn a_font_name_cannot_add_a_shell_command() {
        let mut p = every_need();
        p.needs.font.family = Some("Mono'; touch pwned; echo '".into());
        let text = preview(&p, Platform::Gnome);
        let line = text
            .lines()
            .find(|l| l.starts_with("gsettings") && l.contains("monospace-font-name"))
            .unwrap();
        assert_eq!(
            line,
            r#"gsettings set org.gnome.desktop.interface monospace-font-name '"Mono'\''; touch pwned; echo '\'' 16"'"#
        );
    }
}
