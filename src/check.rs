//! Checks every foreground and background pair of a complete palette against the profile's floors.

use std::fmt;

use anyhow::{Result, bail};

use crate::colour::{Rgb, apca_lc, contrast_ratio, floor2};
use crate::profile::{Palette, Profile, Syntax};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A WCAG 2.2 contrast ratio.
    Ratio,
    /// An APCA lightness contrast, compared as an absolute Lc value.
    Apca,
    /// An OKLab distance, for highlights that must stay visible on the background.
    Distance,
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
            Kind::Distance => format!("{:.3}", (self.measured * 1000.0).floor() / 1000.0),
        }
    }

    pub fn required_shown(&self) -> String {
        match self.kind {
            Kind::Ratio => format!("{}:1", self.required),
            Kind::Apca => format!("Lc {}", self.required),
            Kind::Distance => format!("{}", self.required),
        }
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let relation = match self.kind {
            Kind::Distance => "from",
            _ => "on",
        };
        write!(
            f,
            "{} {} {relation} {} {}: {}",
            self.fg_role,
            self.fg,
            self.bg_role,
            self.bg,
            self.shown()
        )?;
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
        .filter(|r| p.get(r).is_none())
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
            Floor::Apca => needs.apca.unwrap_or(0.0),
        };
        let measured = match kind {
            Kind::Ratio => contrast_ratio(fg, bg),
            Kind::Apca => apca_lc(fg, bg).abs(),
            Kind::Distance => fg.to_oklab().distance(bg.to_oklab()),
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
    for surface in ["background", "surface", "line_highlight", "selection_soft"] {
        push("foreground", surface, Ratio, Text);
    }
    // the accent is drawn as text too (a matched bracket, the tmux clock), not only as a fill
    for role in ["muted", "link", "error", "warning", "info", "accent"] {
        push(role, "background", Ratio, Text);
        push(role, "surface", Ratio, Text);
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
    push("border", "surface", Ratio, NonText);
    // focus moves into panels and sidebars, so the ring must show on them as well
    push("focus_ring", "surface", Ratio, NonText);
    for role in ["line_highlight", "selection_soft"] {
        push(role, "background", Distance, HighlightDistance);
    }
    for token in Syntax::ROLES {
        let role = format!("syntax.{token}");
        push(&role, "background", Ratio, Text);
        push(&role, "surface", Ratio, Text);
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
        let text = "spec_version = 1\nname = \"t\"\n[needs.contrast]\napca = 90";
        let derived = derive(&Profile::parse(text).unwrap()).unwrap();
        let report = check(&derived.profile).unwrap();
        assert!(report.pairs.iter().any(|p| p.kind == Kind::Apca));
    }
}
