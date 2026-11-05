//! Fills in every colour a profile leaves out, each one solved to meet the profile's own floors.
//!
//! A derived colour keeps a fixed hue and searches OKLCH lightness away from the background until
//! it reaches the floor plus a small margin. The margin keeps the 8 bit result above the floor once
//! it is rounded to hex. When no lightness can reach the floor the derivation fails and names the
//! role, rather than handing back a colour that would fail `check`.

use anyhow::{Result, bail};

use crate::colour::{Oklch, Rgb, apca_lc, contrast_ratio};
use crate::profile::{Appearance, Palette, Profile, Syntax};

/// Head room above each floor, so the rounded hex colour still clears it.
const MARGIN: f64 = 0.15;

/// The outcome of a derivation: the complete profile plus the dotted names of every colour that
/// was filled in, in the order they were derived.
#[derive(Debug, Clone)]
pub struct Derived {
    pub profile: Profile,
    pub filled: Vec<String>,
}

/// Default hues in OKLCH degrees, chosen to match the families the Zacess themes use for each role.
const SYNTAX_HUES: &[(&str, f64)] = &[
    ("keyword", 300.0),
    ("control", 345.0),
    ("function", 255.0),
    ("type", 190.0),
    ("parameter", 55.0),
    ("property", 280.0),
    ("string", 145.0),
    ("number", 40.0),
    ("comment", 125.0),
    ("attribute", 75.0),
    ("regex", 5.0),
    ("invalid", 25.0),
    ("added", 145.0),
    ("removed", 25.0),
    ("changed", 250.0),
];

/// ANSI hues for red, green, yellow, blue, magenta and cyan.
const ANSI_HUES: [f64; 6] = [29.0, 142.0, 100.0, 262.0, 328.0, 195.0];

const ACCENT_HUE: f64 = 255.0;
const SELECTION_HUE: f64 = 300.0;
const CURSOR_HUE: f64 = 95.0;

pub fn derive(profile: &Profile) -> Result<Derived> {
    let mut out = profile.clone();
    let mut filled = Vec::new();
    let needs = profile.needs.contrast.clone();
    let p = &mut out.palette;

    // the background and foreground anchor everything else, so they are settled first
    let appearance = profile.appearance;
    let background = match (p.background, p.foreground, appearance) {
        (Some(bg), _, _) => bg,
        (None, Some(fg), _) => best_of_black_and_white(fg),
        (None, None, Some(Appearance::Light)) => Rgb::WHITE,
        (None, None, _) => Rgb::BLACK,
    };
    fill(p, "background", background, &mut filled);
    let dark = is_dark(background);
    let appearance = appearance.unwrap_or(if dark {
        Appearance::Dark
    } else {
        Appearance::Light
    });
    if (appearance == Appearance::Dark) != dark {
        bail!(
            "appearance is {appearance} but the background {background} is {}: change one of them",
            if dark { "dark" } else { "light" }
        );
    }
    out.appearance = Some(appearance);
    let p = &mut out.palette;

    let foreground = p
        .foreground
        .unwrap_or_else(|| best_of_black_and_white(background));
    fill(p, "foreground", foreground, &mut filled);

    let surface = p
        .surface
        .unwrap_or_else(|| shade(background, 0.03, &[(foreground, needs.text)]));
    fill(p, "surface", surface, &mut filled);
    let line = p
        .line_highlight
        .unwrap_or_else(|| shade(background, 0.1, &[(foreground, needs.text)]));
    fill(p, "line_highlight", line, &mut filled);

    let surfaces = [background, surface];
    let text =
        |role: &str, hue: f64, chroma: f64, on: &[Rgb]| solve(role, hue, chroma, on, needs.text);

    if p.muted.is_none() {
        fill(p, "muted", text("muted", 0.0, 0.0, &surfaces)?, &mut filled);
    }
    if p.accent.is_none() {
        let accent = text("accent", ACCENT_HUE, 0.16, &surfaces)?;
        fill(p, "accent", accent, &mut filled);
    }
    let accent = p.accent.unwrap_or(foreground);
    if p.accent_foreground.is_none() {
        let on_accent = if contrast_ratio(background, accent) >= needs.text {
            background
        } else {
            best_of_black_and_white(accent)
        };
        fill(p, "accent_foreground", on_accent, &mut filled);
    }
    // the link and focus ring follow the accent only when it reads as each of them; a dim accent
    // that someone set by hand must not drag them under their floors
    let accent_meets = |floor: f64| surfaces.iter().all(|&s| contrast_ratio(accent, s) >= floor);
    if p.link.is_none() {
        let link = if accent_meets(needs.text) {
            accent
        } else {
            text("link", ACCENT_HUE, 0.16, &surfaces)?
        };
        fill(p, "link", link, &mut filled);
    }
    if p.focus_ring.is_none() {
        let ring = if accent_meets(needs.non_text) {
            accent
        } else {
            text("focus_ring", ACCENT_HUE, 0.16, &surfaces)?
        };
        fill(p, "focus_ring", ring, &mut filled);
    }
    if p.border.is_none() {
        let border = solve("border", 0.0, 0.0, &surfaces, needs.non_text)?;
        fill(p, "border", border, &mut filled);
    }
    if p.cursor.is_none() {
        let cursor = if dark {
            text("cursor", CURSOR_HUE, 0.18, &[background, line])?
        } else {
            foreground
        };
        fill(p, "cursor", cursor, &mut filled);
    }
    let cursor = p.cursor.unwrap_or(foreground);
    fill(
        p,
        "cursor_text",
        best_of(cursor, background, foreground),
        &mut filled,
    );
    if p.selection_background.is_none() {
        // measured against the background at the text floor, so the background colour can be the
        // selected text and the block clears the non-text floor at the same time
        let selection = text("selection_background", SELECTION_HUE, 0.16, &[background])?;
        fill(p, "selection_background", selection, &mut filled);
    }
    let selection = p.selection_background.unwrap_or(foreground);
    fill(
        p,
        "selection_foreground",
        best_of(selection, background, foreground),
        &mut filled,
    );
    if p.selection_soft.is_none() {
        let soft = soft_highlight(background, foreground, needs.text, needs.highlight_distance);
        fill(p, "selection_soft", soft, &mut filled);
    }
    for (role, hue) in [("error", 25.0), ("warning", 70.0), ("info", 240.0)] {
        if p.get(role).is_none() {
            fill(p, role, text(role, hue, 0.15, &surfaces)?, &mut filled);
        }
    }

    // syntax colours are solved against every surface they are drawn on at the text floor
    let syntax_on = [background, surface];
    let mut syntax = p.syntax.clone();
    for &role in Syntax::ROLES {
        if syntax.get(role).is_some() {
            continue;
        }
        let colour = match role {
            "variable" | "operator" => foreground,
            "tag" => match syntax.keyword {
                Some(k) => k,
                None => syntax_hue(role, &syntax_on, needs.text, needs.apca)?,
            },
            _ => syntax_hue(role, &syntax_on, needs.text, needs.apca)?,
        };
        *syntax.slot(role).expect("role from ROLES") = Some(colour);
        filled.push(format!("syntax.{role}"));
    }
    p.syntax = syntax;

    if p.terminal.ansi.is_none() {
        p.terminal.ansi = Some(ansi(background, foreground, needs.terminal)?);
        if dark {
            // slot 0 is black, which is the background on a dark terminal; programs use it as a fill
            p.terminal.background_slots = vec![0];
        }
        filled.push("terminal.ansi".to_string());
    }

    refuse_failing_derivations(&out, &filled)?;
    Ok(Derived {
        profile: out,
        filled,
    })
}

/// The base name a pair uses for a role, so `terminal.ansi[3]` matches the filled entry
/// `terminal.ansi`.
fn base_role(role: &str) -> &str {
    if role.starts_with("terminal.ansi[") {
        "terminal.ansi"
    } else {
        role
    }
}

/// Fails when a colour the tool chose misses a floor. Fallbacks such as a highlight that could not
/// move away from the background can miss one. Handing such a palette back would contradict
/// the spec. A failing pair is blamed on a derived colour only when its other side is the background
/// or is derived too: against a colour the person set, the failure is theirs and `check` names it.
fn refuse_failing_derivations(profile: &Profile, filled: &[String]) -> Result<()> {
    let report = crate::check::check(profile)?;
    let derived = |role: &str| filled.iter().any(|f| f == base_role(role));
    let blamed: Vec<String> = report
        .failures()
        .into_iter()
        .filter(|pair| {
            let (fg, bg) = (pair.fg_role.as_str(), pair.bg_role.as_str());
            (derived(fg) && (bg == "background" || derived(bg)))
                || (derived(bg) && (fg == "background" || derived(fg)))
        })
        .map(|pair| format!("  {pair}"))
        .collect();
    if blamed.is_empty() {
        return Ok(());
    }
    // a colour the person set that already misses its floor is often why nothing could be found,
    // so it is named alongside rather than left for a later check
    let own: Vec<String> = report
        .failures()
        .into_iter()
        .filter(|pair| !derived(&pair.fg_role) && !derived(&pair.bg_role))
        .map(|pair| format!("  {pair}"))
        .collect();
    let mut cause = if own.is_empty() {
        String::new()
    } else {
        format!(
            "\nThese colours you set already miss their floors:\n{}",
            own.join("\n")
        )
    };
    if blamed
        .iter()
        .any(|line| line.contains("highlight_distance"))
    {
        cause.push_str(
            "\nA highlight can only move away from the background while the text on it keeps its \
             floor, so the foreground needs more contrast with the background than the floor alone",
        );
    }
    bail!(
        "no colour could be derived that meets these floors:\n{}{cause}\nSet the colours by hand \
         or change the floors in [needs.contrast]",
        blamed.join("\n")
    )
}

fn fill(p: &mut Palette, role: &str, colour: Rgb, filled: &mut Vec<String>) {
    let slot = p.slot(role).expect("role names are fixed in this module");
    if slot.is_none() {
        *slot = Some(colour);
        filled.push(role.to_string());
    }
}

pub fn is_dark(colour: Rgb) -> bool {
    contrast_ratio(colour, Rgb::WHITE) > contrast_ratio(colour, Rgb::BLACK)
}

fn best_of_black_and_white(on: Rgb) -> Rgb {
    best_of(on, Rgb::BLACK, Rgb::WHITE)
}

fn best_of(on: Rgb, a: Rgb, b: Rgb) -> Rgb {
    if contrast_ratio(on, a) >= contrast_ratio(on, b) {
        a
    } else {
        b
    }
}

/// A grey step away from the background, as large as it can be while every listed colour still
/// meets its floor on it. Falls back to the background when no step keeps them all.
fn shade(background: Rgb, step: f64, keep: &[(Rgb, f64)]) -> Rgb {
    let base = background.to_oklch();
    let direction = if is_dark(background) { 1.0 } else { -1.0 };
    let mut size = step;
    while size > 0.004 {
        let candidate = Oklch::new(base.l + direction * size, base.c.min(0.02), base.h).to_rgb();
        if keep
            .iter()
            .all(|&(c, floor)| contrast_ratio(c, candidate) >= floor + MARGIN)
        {
            return candidate;
        }
        size /= 2.0;
    }
    background
}

/// A background-only highlight: the visible band that the foreground can still be read over.
fn soft_highlight(background: Rgb, foreground: Rgb, floor: f64, distance: f64) -> Rgb {
    let base = background.to_oklch();
    let direction = if is_dark(background) { 1.0 } else { -1.0 };
    let mut best = background;
    for i in 1..=60 {
        let candidate = Oklch::new(
            base.l + direction * 0.005 * f64::from(i),
            0.08,
            SELECTION_HUE,
        )
        .to_rgb();
        if contrast_ratio(foreground, candidate) < floor + MARGIN {
            break;
        }
        best = candidate;
        if candidate.to_oklab().distance(background.to_oklab()) >= distance + 0.04 {
            break;
        }
    }
    best
}

fn syntax_hue(role: &str, on: &[Rgb], floor: f64, apca: Option<f64>) -> Result<Rgb> {
    let hue = SYNTAX_HUES
        .iter()
        .find(|(r, _)| *r == role)
        .map_or(0.0, |&(_, h)| h);
    solve_text(&format!("syntax.{role}"), hue, 0.16, on, floor, apca)
}

/// The colour of the given hue and chroma nearest in lightness to the first surface that still
/// meets `floor` against every surface. Fails, naming the role, when even the lightness extreme
/// cannot.
pub fn solve(role: &str, hue: f64, chroma: f64, on: &[Rgb], floor: f64) -> Result<Rgb> {
    solve_text(role, hue, chroma, on, floor, None)
}

/// As `solve`, also meeting an APCA Lc floor against the first surface when one is given. `check`
/// measures APCA only against the background, which callers pass first.
pub fn solve_text(
    role: &str,
    hue: f64,
    chroma: f64,
    on: &[Rgb],
    floor: f64,
    apca: Option<f64>,
) -> Result<Rgb> {
    let anchor = on[0];
    let dark = is_dark(anchor);
    let target = floor + MARGIN;
    let apca_ok = |c: Rgb, lc: f64| apca.is_none_or(|need| apca_lc(c, anchor).abs() >= need + lc);
    let meets = |c: Rgb| on.iter().all(|&s| contrast_ratio(c, s) >= target) && apca_ok(c, 1.0);
    let at = |l: f64| Oklch::new(l, chroma, hue).to_rgb();
    let start = anchor.to_oklch().l;
    let end = if dark { 1.0 } else { 0.0 };
    if meets(at(end)) {
        // lightness is monotonic enough along one hue that a bisection finds the boundary; the
        // final scan below covers the small wobble that gamut mapping adds
        let (mut near, mut far) = (start, end);
        for _ in 0..48 {
            let mid = (near + far) / 2.0;
            if meets(at(mid)) {
                far = mid;
            } else {
                near = mid;
            }
        }
        let mut l = far;
        for _ in 0..200 {
            if meets(at(l)) {
                return Ok(at(l));
            }
            l += if dark { 0.002 } else { -0.002 };
        }
    }
    let extreme = if dark { Rgb::WHITE } else { Rgb::BLACK };
    if on.iter().all(|&s| contrast_ratio(extreme, s) >= floor) && apca_ok(extreme, 0.0) {
        return Ok(extreme);
    }
    if let Some(need) = apca
        && on.iter().all(|&s| contrast_ratio(extreme, s) >= floor)
    {
        bail!(
            "cannot derive {role}: needs.contrast.apca asks for Lc {need} but {anchor} allows at \
             most Lc {:.1} even for {extreme}. Lower the APCA floor or choose a {} background",
            apca_lc(extreme, anchor).abs(),
            if dark { "darker" } else { "lighter" }
        );
    }
    let best = on
        .iter()
        .map(|&s| contrast_ratio(extreme, s))
        .fold(f64::INFINITY, f64::min);
    bail!(
        "cannot derive {role}: the floor is {floor}:1 but the surfaces it sits on ({}) allow at most \
         {:.2}:1 even for {extreme}. Lower the floor or choose a {} background",
        {
            let mut seen: Vec<String> = Vec::new();
            for c in on {
                if !seen.contains(&c.hex()) {
                    seen.push(c.hex());
                }
            }
            seen.join(" and ")
        },
        crate::colour::floor2(best),
        if dark { "darker" } else { "lighter" }
    )
}

fn ansi(background: Rgb, foreground: Rgb, floor: f64) -> Result<Vec<Rgb>> {
    let dark = is_dark(background);
    let on = [background];
    let mut slots = Vec::with_capacity(16);
    let grey = |name: &str, f: f64| solve(name, 0.0, 0.0, &on, f);
    // a brighter step for the bright slots where the floor leaves room, the floor itself otherwise
    let bright = |name: &str, hue: f64, chroma: f64| {
        let stronger = (floor * 1.3).min(20.0);
        solve(name, hue, chroma, &on, stronger).or_else(|_| solve(name, hue, chroma, &on, floor))
    };
    slots.push(if dark { background } else { foreground });
    for (i, &hue) in ANSI_HUES.iter().enumerate() {
        slots.push(solve(
            &format!("terminal.ansi[{}]", i + 1),
            hue,
            0.17,
            &on,
            floor,
        )?);
    }
    // white and bright black must differ, so one of them takes the stronger step: white on a
    // dark background, bright black on a light one
    if dark {
        slots.push(bright("terminal.ansi[7]", 0.0, 0.0)?);
        slots.push(grey("terminal.ansi[8]", floor)?);
    } else {
        slots.push(grey("terminal.ansi[7]", floor)?);
        slots.push(bright("terminal.ansi[8]", 0.0, 0.0)?);
    }
    for (i, &hue) in ANSI_HUES.iter().enumerate() {
        slots.push(bright(&format!("terminal.ansi[{}]", i + 9), hue, 0.17)?);
    }
    // bright white is drawn as default text by most programs, so on a light theme it takes the
    // foreground rather than a white that would vanish
    slots.push(foreground);
    Ok(slots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::check;

    fn minimal(extra: &str) -> Profile {
        Profile::parse(&format!("spec_version = 1\nname = \"Test\"\n{extra}")).unwrap()
    }

    #[test]
    fn an_empty_palette_derives_and_passes() {
        for extra in [
            "",
            "appearance = \"light\"",
            "[palette]\nbackground = \"#1e1e2e\"",
            "[palette]\nbackground = \"#fdf6e3\"",
        ] {
            let d = derive(&minimal(extra)).unwrap();
            let report = check(&d.profile).unwrap();
            assert!(
                report.failures().is_empty(),
                "{extra}: {:?}",
                report.failures()
            );
        }
    }

    #[test]
    fn set_colours_are_never_changed() {
        let d = derive(&minimal(
            "[palette]\nbackground = \"#101010\"\naccent = \"#ffd400\"",
        ))
        .unwrap();
        assert_eq!(d.profile.palette.accent, Some("#ffd400".parse().unwrap()));
        assert!(!d.filled.contains(&"accent".to_string()));
        assert!(d.filled.contains(&"syntax.comment".to_string()));
    }

    #[test]
    fn an_impossible_floor_fails_loudly() {
        let p = minimal("[needs.contrast]\ntext = 18\n[palette]\nbackground = \"#444444\"");
        let err = derive(&p).unwrap_err().to_string();
        assert!(err.contains("cannot derive"), "{err}");
        assert!(err.contains("18:1"), "{err}");
    }

    #[test]
    fn a_fallback_that_fails_is_refused_by_name() {
        // the reviewed repro: a foreground too dim to allow any visible current line
        let p = minimal("[palette]\nbackground = \"#000000\"\nforeground = \"#959595\"");
        let err = derive(&p).unwrap_err().to_string();
        assert!(
            err.contains("line_highlight #000000 from background"),
            "{err}"
        );
        assert!(err.contains("needs.contrast.highlight_distance"), "{err}");
        assert!(err.contains("the foreground needs more contrast"), "{err}");
    }

    #[test]
    fn colours_the_person_set_are_named_as_the_cause() {
        // a dim foreground makes the derived surfaces miss too; the dim colour is named as well
        let p = minimal("[palette]\nbackground = \"#000000\"\nforeground = \"#555555\"");
        let err = derive(&p).unwrap_err().to_string();
        assert!(
            err.contains("These colours you set already miss their floors"),
            "{err}"
        );
        assert!(
            err.contains("foreground #555555 on background #000000"),
            "{err}"
        );
    }

    #[test]
    fn a_failure_against_a_set_colour_is_left_to_check() {
        let p =
            minimal("[palette]\nbackground = \"#ffffff\"\n[palette.syntax]\ncomment = \"#999999\"");
        assert!(derive(&p).is_ok());
    }

    #[test]
    fn appearance_must_match_the_background() {
        let p = minimal("appearance = \"dark\"\n[palette]\nbackground = \"#ffffff\"");
        assert!(
            derive(&p)
                .unwrap_err()
                .to_string()
                .contains("appearance is dark")
        );
    }
}
