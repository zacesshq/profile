//! The profile format: what a person needs, plus an optional palette.
//!
//! The field names and their meaning are specified in `spec/SPEC.md` and `spec/profile.schema.json`.
//! A change here is a change to the spec, so the two are kept in step by `tests/schema.rs`.

use std::fmt;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::colour::Rgb;

/// The newest spec version this build reads and writes.
pub const SPEC_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub spec_version: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<Appearance>,
    #[serde(default)]
    pub needs: Needs,
    #[serde(default)]
    pub palette: Palette,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    Dark,
    Light,
}

impl fmt::Display for Appearance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Appearance::Dark => "dark",
            Appearance::Light => "light",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Needs {
    #[serde(default)]
    pub contrast: ContrastNeeds,
    #[serde(default)]
    pub font: FontNeeds,
    /// No animation, blinking or smooth scrolling. On when missing, as in the Zacess reader.
    #[serde(default = "yes")]
    pub reduce_motion: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_size: Option<CursorSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_ring_width_px: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_spacing: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_spacing: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_width: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_column: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_decoration: Option<bool>,
}

fn yes() -> bool {
    true
}

impl Default for Needs {
    fn default() -> Self {
        Needs {
            contrast: ContrastNeeds::default(),
            font: FontNeeds::default(),
            reduce_motion: true,
            cursor_size: None,
            focus_ring_width_px: None,
            line_spacing: None,
            character_spacing: None,
            column_width: None,
            single_column: None,
            hide_decoration: None,
        }
    }
}

/// The contrast floors every pair is checked against.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContrastNeeds {
    /// Text and syntax colours on every surface they are drawn on. WCAG AAA is 7.
    #[serde(default = "seven")]
    pub text: f64,
    /// Borders, the focus ring and the selection block against the background. WCAG asks for 3.
    #[serde(default = "three")]
    pub non_text: f64,
    /// Syntax colours drawn over a background-only highlight: the current line and a soft selection.
    #[serde(default = "four_and_a_half")]
    pub overlay: f64,
    /// The sixteen terminal colours on the terminal background.
    #[serde(default = "seven")]
    pub terminal: f64,
    /// Terminal slots listed in `palette.terminal.dim`, which may sit lower to stay vivid.
    #[serde(default = "four_and_a_half")]
    pub terminal_dim: f64,
    /// The smallest OKLab distance between a highlight and the background, so it stays visible.
    #[serde(default = "highlight_distance")]
    pub highlight_distance: f64,
    /// The smallest OKLab distance between two paired colours as a person with protanopia,
    /// deuteranopia or tritanopia sees them, so neighbouring roles never merge.
    #[serde(default = "colour_vision_distance")]
    pub colour_vision_distance: f64,
    /// The smallest tone difference (the cube root of relative luminance) between two paired
    /// colours, so they still differ with no colour vision at all.
    #[serde(default = "tone_difference")]
    pub tone_difference: f64,
    /// Whether errors and warnings are held to the paired colour floors. The Zacess themes do not
    /// pair them, so turning this off keeps a themes palette as it is.
    #[serde(default = "yes")]
    pub check_error_warning: bool,
    /// An optional APCA floor (absolute Lc) for text, checked on top of the WCAG ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apca: Option<f64>,
}

fn seven() -> f64 {
    7.0
}
fn three() -> f64 {
    3.0
}
fn four_and_a_half() -> f64 {
    4.5
}
fn highlight_distance() -> f64 {
    0.08
}
fn colour_vision_distance() -> f64 {
    0.04
}
fn tone_difference() -> f64 {
    0.05
}

impl Default for ContrastNeeds {
    fn default() -> Self {
        ContrastNeeds {
            text: 7.0,
            non_text: 3.0,
            overlay: 4.5,
            terminal: 7.0,
            terminal_dim: 4.5,
            highlight_distance: 0.08,
            colour_vision_distance: 0.04,
            tone_difference: 0.05,
            check_error_warning: true,
            apca: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FontNeeds {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<FontFallback>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_pt: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FontFallback {
    SansSerif,
    Serif,
    Monospace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CursorSize {
    Standard,
    Large,
    ExtraLarge,
}

/// Generates a struct of optional colours plus name based access, so derivation, checking and the
/// schema test can walk every role without a hand-kept list drifting out of step.
macro_rules! colour_roles {
    (
        $(#[$meta:meta])* $name:ident { $($field:ident => $key:literal),* $(,)? }
        $(extra { $($extra:tt)* })?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(
                #[serde(rename = $key, default, skip_serializing_if = "Option::is_none")]
                pub $field: Option<Rgb>,
            )*
            $($($extra)*)?
        }

        impl $name {
            pub const ROLES: &'static [&'static str] = &[$($key),*];

            pub fn get(&self, role: &str) -> Option<Rgb> {
                match role {
                    $($key => self.$field,)*
                    _ => None,
                }
            }

            pub fn slot(&mut self, role: &str) -> Option<&mut Option<Rgb>> {
                match role {
                    $($key => Some(&mut self.$field),)*
                    _ => None,
                }
            }
        }
    };
}

colour_roles!(
    /// The palette: interface colours at the top level, then syntax and terminal tables.
    Palette {
        background => "background",
        foreground => "foreground",
        surface => "surface",
        surface_alt => "surface_alt",
        muted => "muted",
        border => "border",
        accent => "accent",
        accent_foreground => "accent_foreground",
        link => "link",
        focus_ring => "focus_ring",
        cursor => "cursor",
        cursor_text => "cursor_text",
        selection_background => "selection_background",
        selection_foreground => "selection_foreground",
        selection_soft => "selection_soft",
        line_highlight => "line_highlight",
        error => "error",
        warning => "warning",
        info => "info",
    }
    extra {
        #[serde(default)]
        pub syntax: Syntax,
        #[serde(default)]
        pub terminal: Terminal,
    }
);

colour_roles!(
    /// Syntax colours, one per token role. The roles match the Zacess themes.
    Syntax {
        keyword => "keyword",
        control => "control",
        function => "function",
        kind => "type",
        variable => "variable",
        parameter => "parameter",
        property => "property",
        string => "string",
        number => "number",
        comment => "comment",
        operator => "operator",
        tag => "tag",
        attribute => "attribute",
        regex => "regex",
        invalid => "invalid",
        added => "added",
        removed => "removed",
        changed => "changed",
    }
);

/// Font style per syntax role, matching the Zacess themes so a token never rests on colour alone.
/// Roles missing here are drawn in the regular style.
pub const STYLES: &[(&str, &[&str])] = &[
    ("keyword", &["bold"]),
    ("control", &["bold"]),
    ("tag", &["bold"]),
    ("parameter", &["italic"]),
    ("comment", &["italic"]),
    ("invalid", &["underline"]),
];

/// The font style of a syntax role, empty for the regular style.
pub fn style(role: &str) -> &'static [&'static str] {
    STYLES
        .iter()
        .find(|(r, _)| *r == role)
        .map_or(&[][..], |s| s.1)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Terminal {
    /// The sixteen ANSI colours: 0 to 7 normal, 8 to 15 bright.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ansi: Option<Vec<Rgb>>,
    /// Slots checked at `needs.contrast.terminal_dim` rather than `needs.contrast.terminal`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dim: Vec<u8>,
    /// Slots that programs use as a background colour, so they are not checked as text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub background_slots: Vec<u8>,
}

impl Palette {
    /// Roles that may stay missing after derivation, each with the role it falls back to. A second
    /// panel colour is only worth naming when an app draws one, so leaving it out means the panels
    /// share `surface` and nothing new is measured.
    pub const FALLBACKS: &'static [(&'static str, &'static str)] = &[("surface_alt", "surface")];

    /// The colour a role resolves to, following its fallback when it is not set.
    pub fn resolved(&self, role: &str) -> Option<Rgb> {
        self.get(role).or_else(|| {
            Self::FALLBACKS
                .iter()
                .find(|(r, _)| *r == role)
                .and_then(|(_, fallback)| self.get(fallback))
        })
    }

    /// Whether a role may be left out of a complete palette.
    pub fn has_fallback(role: &str) -> bool {
        Self::FALLBACKS.iter().any(|(r, _)| *r == role)
    }
}

impl Profile {
    pub fn load(path: &Path) -> Result<Profile> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        Profile::parse(&text).with_context(|| format!("{} is not a valid profile", path.display()))
    }

    pub fn parse(text: &str) -> Result<Profile> {
        let profile: Profile = toml::from_str(text)?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn to_toml(&self) -> Result<String> {
        Ok(toml::to_string(self)?)
    }

    /// The id used in file names: the `id` field when set, otherwise the name in lowercase with dashes.
    pub fn slug(&self) -> String {
        if let Some(id) = &self.id {
            return id.clone();
        }
        let mut slug = String::new();
        for ch in self.name.chars() {
            if ch.is_ascii_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
            } else if !slug.is_empty() && !slug.ends_with('-') {
                slug.push('-');
            }
        }
        slug.trim_end_matches('-').to_string()
    }

    /// Range checks the schema also states, so a profile read without the schema fails the same way.
    pub fn validate(&self) -> Result<()> {
        if self.spec_version == 0 || self.spec_version > SPEC_VERSION {
            bail!(
                "spec_version {} is not supported: this build reads spec version {}",
                self.spec_version,
                SPEC_VERSION
            );
        }
        if self.name.trim().is_empty() {
            bail!("name must not be empty");
        }
        // the name lands in comment lines of every rendered file, so a line break would let a
        // shared profile write its own lines into a Lua or shell config
        if self.name.chars().any(char::is_control) {
            bail!("name must be one line with no control characters");
        }
        // the font name lands inside quoted strings in Lua, TOML and JSON, so a quote or a
        // backslash could end the string early
        if let Some(family) = &self.needs.font.family
            && family
                .chars()
                .any(|c| c.is_control() || c == '"' || c == '\\')
        {
            bail!("needs.font.family must not contain quotes, backslashes or control characters");
        }
        if let Some(id) = &self.id {
            let ok = !id.is_empty()
                && id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !id.starts_with('-')
                && !id.ends_with('-')
                && !id.contains("--");
            if !ok {
                bail!(
                    "id \"{id}\" must be lowercase letters and digits joined by single dashes, such as large-text-dark"
                );
            }
        }
        // file names come from the slug, so a name with no ASCII letter or digit would write files
        // with an empty name
        if self.id.is_none() && self.slug().is_empty() {
            bail!(
                "name \"{}\" has no ASCII letters or digits to make file names from: add an id, \
                 such as id = \"my-profile\"",
                self.name
            );
        }
        let c = &self.needs.contrast;
        for (key, value) in [
            ("text", c.text),
            ("non_text", c.non_text),
            ("overlay", c.overlay),
            ("terminal", c.terminal),
            ("terminal_dim", c.terminal_dim),
        ] {
            if !(1.0..=21.0).contains(&value) {
                bail!("needs.contrast.{key} is {value}: a WCAG ratio runs from 1 to 21");
            }
        }
        for (key, value) in [
            ("highlight_distance", c.highlight_distance),
            ("colour_vision_distance", c.colour_vision_distance),
        ] {
            if !(0.0..=1.0).contains(&value) {
                bail!("needs.contrast.{key} is {value}: an OKLab distance runs from 0 to 1");
            }
        }
        if !(0.0..=1.0).contains(&c.tone_difference) {
            bail!(
                "needs.contrast.tone_difference is {}: a tone difference runs from 0 to 1",
                c.tone_difference
            );
        }
        if let Some(lc) = c.apca
            && !(0.0..=108.0).contains(&lc)
        {
            bail!("needs.contrast.apca is {lc}: an APCA Lc floor runs from 0 to 108");
        }
        range("needs.font.size_pt", self.needs.font.size_pt, 6.0, 96.0)?;
        range("needs.line_spacing", self.needs.line_spacing, 1.0, 3.0)?;
        range(
            "needs.character_spacing",
            self.needs.character_spacing,
            0.0,
            0.5,
        )?;
        range(
            "needs.focus_ring_width_px",
            self.needs.focus_ring_width_px.map(f64::from),
            1.0,
            10.0,
        )?;
        range(
            "needs.column_width",
            self.needs.column_width.map(f64::from),
            20.0,
            150.0,
        )?;
        if let Some(ansi) = &self.palette.terminal.ansi
            && ansi.len() != 16
        {
            bail!(
                "palette.terminal.ansi has {} colours: it needs all sixteen, 0 to 15",
                ansi.len()
            );
        }
        for (key, slots) in [
            ("dim", &self.palette.terminal.dim),
            ("background_slots", &self.palette.terminal.background_slots),
        ] {
            if let Some(bad) = slots.iter().find(|&&s| s > 15) {
                bail!("palette.terminal.{key} lists slot {bad}: slots run from 0 to 15");
            }
            if let Some((_, dup)) = slots
                .iter()
                .enumerate()
                .find(|(i, s)| slots[..*i].contains(s))
            {
                bail!("palette.terminal.{key} lists slot {dup} twice");
            }
        }
        Ok(())
    }
}

fn range(key: &str, value: Option<f64>, lo: f64, hi: f64) -> Result<()> {
    match value {
        Some(v) if !(lo..=hi).contains(&v) => bail!("{key} is {v}: it must be from {lo} to {hi}"),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_alone_is_a_profile() {
        let p = Profile::parse("spec_version = 1\nname = \"Plain\"").unwrap();
        assert_eq!(p.needs.contrast.text, 7.0);
        assert!(p.needs.reduce_motion);
        assert_eq!(p.slug(), "plain");
    }

    #[test]
    fn slug_from_name() {
        let p =
            Profile::parse("spec_version = 1\nname = \"Zacess Deep  Contrast: Dark!\"").unwrap();
        assert_eq!(p.slug(), "zacess-deep-contrast-dark");
    }

    #[test]
    fn unknown_fields_are_refused() {
        let err =
            Profile::parse("spec_version = 1\nname = \"x\"\n[needs]\ncontrst = 7").unwrap_err();
        assert!(format!("{err:#}").contains("contrst"), "{err:#}");
    }

    #[test]
    fn bad_ranges_are_refused() {
        for body in [
            "[needs.contrast]\ntext = 30",
            "[needs.font]\nsize_pt = 2",
            "[palette]\nbackground = \"black\"",
            "[palette.terminal]\nansi = [\"#000000\"]",
        ] {
            let text = format!("spec_version = 1\nname = \"x\"\n{body}");
            assert!(Profile::parse(&text).is_err(), "{body}");
        }
        assert!(Profile::parse("spec_version = 9\nname = \"x\"").is_err());
    }

    #[test]
    fn a_name_with_no_file_name_needs_an_id() {
        let err = Profile::parse("spec_version = 1\nname = \"日本語\"").unwrap_err();
        assert!(err.to_string().contains("add an id"), "{err}");
        let p = Profile::parse("spec_version = 1\nname = \"日本語\"\nid = \"nihongo\"").unwrap();
        assert_eq!(p.slug(), "nihongo");
    }

    #[test]
    fn ids_and_slot_lists_match_the_schema() {
        for body in [
            "id = \"a--b\"",
            "id = \"-a\"",
            "id = \"A\"",
            "[palette.terminal]\ndim = [1, 1]",
            "[palette.terminal]\nbackground_slots = [0, 3, 0]",
        ] {
            let text = format!("spec_version = 1\nname = \"x\"\n{body}");
            assert!(Profile::parse(&text).is_err(), "{body}");
        }
        assert!(Profile::parse("spec_version = 1\nname = \"x\"\nid = \"a-b-2\"").is_ok());
    }

    #[test]
    fn text_that_could_escape_a_rendered_file_is_refused() {
        for body in [
            "name = \"x\\nos.execute('rm')\"",
            "name = \"x\"\n[needs.font]\nfamily = 'Mono\" }) os.execute(\"id\") --'",
            "name = \"x\"\n[needs.font]\nfamily = 'Mono\\'",
        ] {
            let text = format!("spec_version = 1\n{body}");
            assert!(Profile::parse(&text).is_err(), "{body}");
        }
    }
}
