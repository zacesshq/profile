//! Checks every foreground and background pair of a complete palette against the profile's floors.

use std::fmt;

use anyhow::{Result, bail};

use crate::colour::{Deficiency, Rgb, apca_lc, contrast_ratio, floor2, simulated_distance};
use crate::profile::{Palette, Profile, Syntax, style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A WCAG 2.2 contrast ratio.
    Ratio,
    /// An APCA lightness contrast, compared as an absolute Lc value.
    Apca,
    /// An OKLab distance, for highlights that must stay visible on the background.
    Distance,
    /// The OKLab distance between two paired colours under a simulated colour vision deficiency.
    Vision(Deficiency),
    /// The tone difference between two paired colours, for a viewer with no colour vision.
    Tone,
}

/// Pairs of roles drawn next to each other that must stay distinct to every viewer. The syntax
/// pairs are the Zacess themes' own list. A pair marked `true` is checked whatever its font styles,
/// because its colours are often red and green; the rest only when both share a style, since a
/// style difference already tells them apart.
pub const DISTINCT: &[(&str, &str, bool)] = &[
    ("syntax.keyword", "syntax.function", false),
    ("syntax.keyword", "syntax.type", false),
    ("syntax.keyword", "syntax.variable", false),
    ("syntax.control", "syntax.keyword", false),
    ("syntax.control", "syntax.variable", false),
    ("syntax.control", "syntax.function", false),
    ("syntax.function", "syntax.variable", false),
    ("syntax.function", "syntax.type", false),
    ("syntax.function", "syntax.property", false),
    ("syntax.function", "syntax.string", false),
    ("syntax.type", "syntax.variable", false),
    ("syntax.type", "syntax.property", false),
    ("syntax.variable", "syntax.property", false),
    ("syntax.variable", "syntax.parameter", false),
    ("syntax.parameter", "syntax.type", false),
    ("syntax.parameter", "syntax.property", false),
    ("syntax.property", "syntax.string", false),
    ("syntax.string", "syntax.comment", false),
    ("syntax.number", "syntax.variable", false),
    ("syntax.number", "syntax.operator", false),
    ("syntax.comment", "syntax.variable", false),
    ("syntax.tag", "syntax.attribute", false),
    ("syntax.attribute", "syntax.string", false),
    ("syntax.invalid", "syntax.variable", false),
    ("syntax.string", "syntax.number", true),
    ("syntax.string", "syntax.regex", true),
    ("syntax.string", "syntax.invalid", true),
    ("syntax.added", "syntax.removed", true),
    ("syntax.added", "syntax.changed", true),
    ("syntax.removed", "syntax.changed", true),
    ("error", "warning", true),
];

/// Whether a pair from `DISTINCT` is checked: always for the red and green pairs, otherwise only
/// when both roles share a font style.
pub fn distinct_applies(a: &str, b: &str, always: bool) -> bool {
    let style_of = |role: &str| role.strip_prefix("syntax.").map_or(&[][..], style);
    always || style_of(a) == style_of(b)
}

/// Every measurement two paired colours must pass: the tone difference plus the OKLab distance
/// under each simulated deficiency, as (kind, measured) pairs.
pub fn distinct_measures(a: Rgb, b: Rgb) -> Vec<(Kind, f64)> {
    let mut out = vec![(Kind::Tone, (a.tone() - b.tone()).abs())];
    for d in Deficiency::ALL {
        out.push((Kind::Vision(d), simulated_distance(a, b, d)));
    }
    out
}

/// Which floor in `needs.contrast` a pair is held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Floor {
    Text,
    NonText,
    Overlay,
    Terminal,
    TerminalDim,
    HighlightDistance,
    ColourVision,
    ToneDifference,
    Apca,
}

impl Floor {
    pub fn key(self) -> &'static str {
        match self {
            Floor::Text => "text",
            Floor::NonText => "non_text",
            Floor::Overlay => "overlay",
            Floor::Terminal => "terminal",
            Floor::TerminalDim => "terminal_dim",
            Floor::HighlightDistance => "highlight_distance",
            Floor::ColourVision => "colour_vision_distance",
            Floor::ToneDifference => "tone_difference",
            Floor::Apca => "apca",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pair {
    pub fg_role: String,
    pub bg_role: String,
    pub fg: Rgb,
    pub bg: Rgb,
    pub kind: Kind,
    pub floor: Floor,
    pub required: f64,
    pub measured: f64,
}

impl Pair {
    pub fn passes(&self) -> bool {
        self.measured >= self.required
    }

    /// The measured value rounded down, so a printed figure never overstates the real one.
    pub fn shown(&self) -> String {
        match self.kind {
            Kind::Ratio => format!("{:.2}:1", floor2(self.measured)),
            Kind::Apca => format!("Lc {:.1}", (self.measured * 10.0).floor() / 10.0),
            Kind::Distance | Kind::Vision(_) | Kind::Tone => {
                format!("{:.3}", (self.measured * 1000.0).floor() / 1000.0)
            }
        }
    }

    pub fn required_shown(&self) -> String {
        match self.kind {
            Kind::Ratio => format!("{}:1", self.required),
            Kind::Apca => format!("Lc {}", self.required),
            Kind::Distance | Kind::Vision(_) | Kind::Tone => format!("{}", self.required),
        }
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (fg, bg) = (&self.fg_role, &self.bg_role);
        match self.kind {
            Kind::Vision(d) => write!(
                f,
                "{fg} {} and {bg} {} under {d}: {} apart",
                self.fg,
                self.bg,
                self.shown()
            )?,
            Kind::Tone => write!(
                f,
                "{fg} {} and {bg} {} in tone: {} apart",
                self.fg,
                self.bg,
                self.shown()
            )?,
            Kind::Distance => write!(
                f,
                "{fg} {} from {bg} {}: {}",
                self.fg,
                self.bg,
                self.shown()
            )?,
            Kind::Ratio | Kind::Apca => {
                write!(f, "{fg} {} on {bg} {}: {}", self.fg, self.bg, self.shown())?
            }
        }
        if !self.passes() {
            write!(
                f,
                ", below needs.contrast.{} of {}",
                self.floor.key(),
                self.required_shown()
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub pairs: Vec<Pair>,
}

impl Report {
    pub fn failures(&self) -> Vec<&Pair> {
        self.pairs.iter().filter(|p| !p.passes()).collect()
    }

    /// The WCAG pair with the least head room over its floor.
    pub fn tightest(&self) -> Option<&Pair> {
        self.pairs
            .iter()
            .filter(|p| p.kind == Kind::Ratio)
            .min_by(|a, b| {
                (a.measured / a.required)
                    .partial_cmp(&(b.measured / b.required))
                    .expect("ratios are finite")
            })
    }
}

/// Checks a complete palette. A profile with gaps must go through `derive` first, so a missing
/// colour is reported by name instead of being silently skipped.
pub fn check(profile: &Profile) -> Result<Report> {
    let p = &profile.palette;
    let missing: Vec<String> = Palette::ROLES
        .iter()
        .filter(|r| p.get(r).is_none() && !Palette::has_fallback(r))
        .map(|r| r.to_string())
        .chain(
            Syntax::ROLES
                .iter()
                .filter(|r| p.syntax.get(r).is_none())
                .map(|r| format!("syntax.{r}")),
        )
        .chain(
            p.terminal
                .ansi
                .is_none()
                .then(|| "terminal.ansi".to_string()),
        )
        .collect();
    if !missing.is_empty() {
        bail!(
            "the palette is missing {}: run derive first",
            missing.join(", ")
        );
    }
    let ui = |role: &str| p.get(role).expect("checked above");
    let needs = &profile.needs.contrast;
    let mut pairs = Vec::new();
    let mut push = |fg_role: &str, bg_role: &str, kind: Kind, floor: Floor| {
        let colour = |role: &str| match role.strip_prefix("syntax.") {
            Some(token) => p.syntax.get(token).expect("checked above"),
            None => match role.strip_prefix("terminal.ansi[") {
                Some(rest) => {
                    let i: usize = rest.trim_end_matches(']').parse().expect("slot index");
                    p.terminal.ansi.as_ref().expect("checked above")[i]
                }
                None => ui(role),
            },
        };
        let (fg, bg) = (colour(fg_role), colour(bg_role));
        let required = match floor {
            Floor::Text => needs.text,
            Floor::NonText => needs.non_text,
            Floor::Overlay => needs.overlay,
            Floor::Terminal => needs.terminal,
            Floor::TerminalDim => needs.terminal_dim,
            Floor::HighlightDistance => needs.highlight_distance,
            Floor::ColourVision => needs.colour_vision_distance,
            Floor::ToneDifference => needs.tone_difference,
            Floor::Apca => needs.apca.unwrap_or(0.0),
        };
        let measured = match kind {
            Kind::Ratio => contrast_ratio(fg, bg),
            Kind::Apca => apca_lc(fg, bg).abs(),
            Kind::Distance => fg.to_oklab().distance(bg.to_oklab()),
            Kind::Vision(d) => simulated_distance(fg, bg, d),
            Kind::Tone => (fg.tone() - bg.tone()).abs(),
        };
        pairs.push(Pair {
            fg_role: fg_role.to_string(),
            bg_role: bg_role.to_string(),
            fg,
            bg,
            kind,
            floor,
            required,
            measured,
        });
    };

    use Floor::{HighlightDistance, NonText, Overlay, TerminalDim, Text};
    use Kind::{Distance, Ratio};
    // a second panel colour is measured only when it is set; left out, it is the surface itself and
    // measuring it again would repeat every surface pair
    let panels: &[&str] = if p.surface_alt.is_some() {
        &["surface", "surface_alt"]
    } else {
        &["surface"]
    };
    push("foreground", "background", Ratio, Text);
    for &panel in panels {
        push("foreground", panel, Ratio, Text);
    }
    for surface in ["line_highlight", "selection_soft"] {
        push("foreground", surface, Ratio, Text);
    }
    // the accent is drawn as text too (a matched bracket, the tmux clock), not only as a fill
    for role in ["muted", "link", "error", "warning", "info", "accent"] {
        push(role, "background", Ratio, Text);
        for &panel in panels {
            push(role, panel, Ratio, Text);
        }
        // diagnostics, whitespace markers and matched brackets also sit on the current line, which
        // is a background-only highlight like the soft selection, so the overlay floor applies
        push(role, "line_highlight", Ratio, Overlay);
    }
    push("cursor", "background", Ratio, Text);
    push("cursor_text", "cursor", Ratio, Text);
    push("selection_foreground", "selection_background", Ratio, Text);
    push("accent_foreground", "accent", Ratio, Text);
    for role in ["border", "focus_ring", "selection_background"] {
        push(role, "background", Ratio, NonText);
    }
    // borders divide panels and focus moves into them, so both must show on every panel colour
    for &panel in panels {
        push("border", panel, Ratio, NonText);
        push("focus_ring", panel, Ratio, NonText);
    }
    for role in ["line_highlight", "selection_soft"] {
        push(role, "background", Distance, HighlightDistance);
    }
    for token in Syntax::ROLES {
        let role = format!("syntax.{token}");
        push(&role, "background", Ratio, Text);
        for &panel in panels {
            push(&role, panel, Ratio, Text);
        }
        push(&role, "line_highlight", Ratio, Overlay);
        push(&role, "selection_soft", Ratio, Overlay);
    }
    let terminal = &p.terminal;
    for slot in 0..16u8 {
        if terminal.background_slots.contains(&slot) {
            // a slot exempt as text is still a fill that default text is drawn on, so listing
            // every slot here cannot switch the terminal check off
            push("foreground", &format!("terminal.ansi[{slot}]"), Ratio, Text);
            continue;
        }
        let floor = if terminal.dim.contains(&slot) {
            TerminalDim
        } else {
            Floor::Terminal
        };
        push(
            &format!("terminal.ansi[{slot}]"),
            "background",
            Ratio,
            floor,
        );
    }
    for &(a, b, always) in DISTINCT {
        if !distinct_applies(a, b, always) {
            continue;
        }
        if (a, b) == ("error", "warning") && !needs.check_error_warning {
            continue;
        }
        push(a, b, Kind::Tone, Floor::ToneDifference);
        for d in Deficiency::ALL {
            push(a, b, Kind::Vision(d), Floor::ColourVision);
        }
    }
    if needs.apca.is_some() {
        push("foreground", "background", Kind::Apca, Floor::Apca);
        for token in Syntax::ROLES {
            push(
                &format!("syntax.{token}"),
                "background",
                Kind::Apca,
                Floor::Apca,
            );
        }
    }
    Ok(Report { pairs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::derive;

    #[test]
    fn a_failing_pair_is_named_with_its_ratio() {
        let text = "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#000000\"\n\
                    [palette.syntax]\ncomment = \"#555555\"";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        let fails = report.failures();
        assert!(!fails.is_empty());
        let line = fails[0].to_string();
        assert!(
            line.starts_with("syntax.comment #555555 on background #000000: 2.81:1"),
            "{line}"
        );
        assert!(line.contains("below needs.contrast.text of 7:1"), "{line}");
    }

    #[test]
    fn accent_used_as_text_is_checked() {
        // the reviewed repro: a near-black accent used to pass because nothing measured it as text
        let text = "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#000000\"\n\
                    accent = \"#262626\"";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        let fails: Vec<String> = report.failures().iter().map(|p| p.to_string()).collect();
        assert!(
            fails
                .iter()
                .any(|f| f.starts_with("accent #262626 on background")),
            "{fails:?}"
        );
    }

    #[test]
    fn background_slots_cannot_exempt_the_terminal() {
        let mut ansi = String::from("[");
        for _ in 0..16 {
            ansi.push_str("\"#ffffff\", ");
        }
        ansi.push(']');
        let text = format!(
            "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#000000\"\n\
             [palette.terminal]\nansi = {ansi}\nbackground_slots = [{}]",
            (0..16)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let derived = derive(&Profile::parse(&text).unwrap()).unwrap();
        let fails = check(&derived.profile).unwrap();
        let fails: Vec<String> = fails.failures().iter().map(|p| p.to_string()).collect();
        assert_eq!(fails.len(), 16, "{fails:?}");
        assert!(fails[0].starts_with("foreground #ffffff on terminal.ansi[0] #ffffff: 1.00:1"));
    }

    #[test]
    fn a_gap_is_an_error_not_a_pass() {
        let p = Profile::parse("spec_version = 1\nname = \"t\"").unwrap();
        assert!(
            check(&p)
                .unwrap_err()
                .to_string()
                .contains("run derive first")
        );
    }

    #[test]
    fn apca_floor_adds_pairs() {
        let text = "spec_version = 1\nname = \"t\"\n[needs.contrast]\napca = 60";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        assert!(report.pairs.iter().any(|p| p.kind == Kind::Apca));
    }

    #[test]
    fn paired_roles_that_merge_are_named_like_a_contrast_failure() {
        // the Zacess Light error with the themes' own warning, which share a tone
        let text = "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#ffffff\"\n\
                    error = \"#810005\"\nwarning = \"#583600\"";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        let fails: Vec<String> = check(&derived.profile)
            .unwrap()
            .failures()
            .iter()
            .map(|p| p.to_string())
            .collect();
        assert!(
            fails.contains(
                &"error #810005 and warning #583600 in tone: 0.000 apart, below \
                  needs.contrast.tone_difference of 0.05"
                    .to_string()
            ),
            "{fails:?}"
        );
        assert!(
            fails.contains(
                &"error #810005 and warning #583600 under deuteranopia: 0.019 apart, below \
                  needs.contrast.colour_vision_distance of 0.04"
                    .to_string()
            ),
            "{fails:?}"
        );
    }

    #[test]
    fn a_style_difference_exempts_a_pair_but_not_red_and_green() {
        // keyword is bold and function is regular, so their pair is told apart by style
        assert!(!distinct_applies(
            "syntax.keyword",
            "syntax.function",
            false
        ));
        assert!(distinct_applies("syntax.function", "syntax.type", false));
        // added and removed are both regular; the pair is checked whatever their style
        assert!(distinct_applies("syntax.added", "syntax.removed", true));
        let text = "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#000000\"\n\
                    [palette.syntax]\nkeyword = \"#8ab4ff\"\nfunction = \"#8ab4ff\"";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        assert!(
            report
                .failures()
                .iter()
                .all(|p| !(p.fg_role == "syntax.keyword" && p.bg_role == "syntax.function")),
        );
    }

    #[test]
    fn the_floors_can_be_lowered_to_switch_the_pairs_off() {
        let text = "spec_version = 1\nname = \"t\"\n[needs.contrast]\ntone_difference = 0\n\
                    colour_vision_distance = 0\n[palette]\nbackground = \"#ffffff\"\n\
                    error = \"#810005\"\nwarning = \"#583600\"";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        assert!(check(&derived.profile).unwrap().failures().is_empty());
    }

    #[test]
    fn the_error_and_warning_pair_can_be_turned_off() {
        let base = "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#ffffff\"\n\
                    error = \"#810005\"\nwarning = \"#583600\"";
        let fails = |text: &str| {
            let derived = derive(&Profile::parse(text).unwrap()).unwrap();
            check(&derived.profile)
                .unwrap()
                .failures()
                .iter()
                .filter(|p| p.fg_role == "error" && p.bg_role == "warning")
                .count()
        };
        assert!(fails(base) > 0);
        let off = base.replace(
            "[palette]",
            "[needs.contrast]\ncheck_error_warning = false\n[palette]",
        );
        assert_eq!(fails(&off), 0);
    }

    #[test]
    fn surface_alt_is_measured_only_when_set() {
        let base = "spec_version = 1\nname = \"t\"\n[palette]\nbackground = \"#ffffff\"\n\
                    surface = \"#f4f4f4\"";
        let derived = derive(&Profile::parse(base).unwrap()).unwrap();
        assert_eq!(derived.profile.palette.surface_alt, None);
        assert!(!derived.filled.iter().any(|f| f == "surface_alt"));
        let without = check(&derived.profile).unwrap();
        assert!(without.failures().is_empty());
        assert!(!without.pairs.iter().any(|p| p.bg_role == "surface_alt"));

        let with = format!("{base}\nsurface_alt = \"#ededed\"");
        let derived = derive(&Profile::parse(&with).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        assert!(report.failures().is_empty(), "{:?}", report.failures());
        let on_alt: Vec<&str> = report
            .pairs
            .iter()
            .filter(|p| p.bg_role == "surface_alt")
            .map(|p| p.fg_role.as_str())
            .collect();
        // foreground, six interface text roles, the border, the focus ring and every syntax colour
        assert_eq!(on_alt.len(), 1 + 6 + 2 + Syntax::ROLES.len(), "{on_alt:?}");
        assert_eq!(report.pairs.len(), without.pairs.len() + on_alt.len());
        for role in [
            "foreground",
            "warning",
            "border",
            "focus_ring",
            "syntax.comment",
        ] {
            assert!(on_alt.contains(&role), "{role}");
        }
    }

    #[test]
    fn a_warning_that_fails_only_on_surface_alt_is_named() {
        // an earlier High Contrast Light warning candidate: it clears 7:1 on the surface
        // but not on the darker second panel colour
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/profiles/zacess-high-contrast-light.toml"
        ))
        .unwrap();
        let mut profile = Profile::parse(&text).unwrap();
        profile.palette.warning = Some("#6f4e1b".parse().unwrap());
        profile.palette.surface_alt = Some("#efefef".parse().unwrap());
        let fails: Vec<String> = check(&profile)
            .unwrap()
            .failures()
            .iter()
            .map(|p| p.to_string())
            .collect();
        assert_eq!(fails.len(), 1, "{fails:?}");
        assert!(
            fails[0].starts_with("warning #6f4e1b on surface_alt #efefef: 6.56:1"),
            "{fails:?}"
        );

        profile.palette.surface_alt = None;
        assert!(check(&profile).unwrap().failures().is_empty());
    }
}
