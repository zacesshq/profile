//! Fills in every colour a profile leaves out, each one solved to meet the profile's own floors.
//!
//! A derived colour keeps a fixed hue and searches OKLCH lightness away from the background until
//! it reaches the floor plus a small margin. The margin keeps the 8 bit result above the floor once
//! it is rounded to hex. When no lightness can reach the floor the derivation fails and names the
//! role, rather than handing back a colour that would fail `check`.

use anyhow::{Result, bail};

use crate::check::Kind;
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

    // a second panel colour is never derived: left out, the panels share the surface. When it is
    // set, every colour drawn on panels is solved against it as well, as check measures it there
    let surfaces: Vec<Rgb> = [Some(background), Some(surface), p.surface_alt]
        .into_iter()
        .flatten()
        .collect();
    // interface text is also drawn over the current line, which may be a colour the person set,
    // so it is solved against that band at the overlay floor as check measures it
    let text = |role: &str, hue: f64, chroma: f64, on: &[Rgb]| {
        solve_over(
            role,
            hue,
            chroma,
            on,
            needs.text,
            None,
            &[line],
            needs.overlay,
        )
    };

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

    // syntax colours are solved against every surface they are drawn on at the text floor and over
    // both background-only highlights at the overlay floor
    let syntax_over = [
        p.line_highlight.expect("filled above"),
        p.selection_soft.expect("filled above"),
    ];
    let mut syntax = p.syntax.clone();
    for &role in Syntax::ROLES {
        if syntax.get(role).is_some() {
            continue;
        }
        let colour = match role {
            "variable" | "operator" => foreground,
            "tag" => match syntax.keyword {
                Some(k) => k,
                None => syntax_hue(role, &surfaces, &syntax_over, &needs)?,
            },
            _ => syntax_hue(role, &surfaces, &syntax_over, &needs)?,
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

    keep_pairs_apart(&mut out, &filled)?;
    refuse_failing_derivations(&out, &filled)?;
    Ok(Derived {
        profile: out,
        filled,
    })
}

/// Spreads derived colours that sit too close to a paired role. Each derived colour was solved by
/// lightness alone, so every role at the same floor shares one tone and neighbours can merge under
/// a colour vision deficiency. The derived roles are placed one at a time, in spec order, each at
/// the nearest lightness on its own hue where it stays apart from every role already placed and
/// still meets its contrast floors. Colours the person set are placed from the start and never move.
fn keep_pairs_apart(profile: &mut Profile, filled: &[String]) -> Result<()> {
    let movable = |role: &str| movable(role, filled);
    let roles: Vec<String> = Syntax::ROLES
        .iter()
        .map(|r| format!("syntax.{r}"))
        .chain(["error".to_string(), "warning".to_string()])
        .filter(|r| movable(r))
        .collect();
    // placing one role at a time can paint itself into a corner when the band of tone above the
    // floor is narrow, so a few orders are tried and the first that leaves no pair failing wins;
    // the spec order is first so ordinary profiles always get the same colours
    let mut most_constrained = roles.clone();
    most_constrained.sort_by_key(|r| {
        std::cmp::Reverse(
            crate::check::DISTINCT
                .iter()
                .filter(|(a, b, always)| {
                    (a == r || b == r) && crate::check::distinct_applies(a, b, *always)
                })
                .count(),
        )
    });
    let mut reversed = roles.clone();
    reversed.reverse();
    let start = profile.clone();
    let mut best: Option<(usize, Profile)> = None;
    let keyword_before = start.palette.syntax.keyword;
    for order in [roles.clone(), most_constrained, reversed] {
        let mut trial = start.clone();
        spread(&mut trial, &order)?;
        tag_follows_keyword(&mut trial, filled, keyword_before);
        let failing = crate::check::check(&trial)?
            .failures()
            .iter()
            .filter(|p| matches!(p.kind, Kind::Tone | Kind::Vision(_)))
            .count();
        if failing == 0 {
            *profile = trial;
            return Ok(());
        }
        if best.as_ref().is_none_or(|(n, _)| failing < *n) {
            best = Some((failing, trial));
        }
    }
    // no order clears every pair; the closest is kept and refuse_failing_derivations names the rest
    if let Some((_, trial)) = best {
        *profile = trial;
    }
    Ok(())
}

/// Whether `derive` chose this colour by hue and may move it to keep a pair apart.
fn movable(role: &str, filled: &[String]) -> bool {
    filled.iter().any(|f| f == role)
        && match role.strip_prefix("syntax.") {
            Some(token) => SYNTAX_HUES.iter().any(|(r, _)| *r == token),
            None => matches!(role, "error" | "warning"),
        }
}

/// Places each role of `order` in turn at the nearest lightness that keeps it apart from every
/// role placed before it as well as from every role outside `order`.
fn spread(profile: &mut Profile, order: &[String]) -> Result<()> {
    let mut placed: Vec<String> = Vec::new();
    for role in order {
        let partner_placed =
            |other: &str| !order.iter().any(|r| r == other) || placed.iter().any(|r| r == other);
        if let Some(colour) = nearest_passing(profile, role, &partner_placed)? {
            set_role(&mut profile.palette, role, colour);
        }
        placed.push(role.clone());
    }
    Ok(())
}

/// A derived tag is a copy of the keyword colour, so it follows the keyword wherever it moved.
fn tag_follows_keyword(profile: &mut Profile, filled: &[String], keyword_before: Option<Rgb>) {
    let p = &mut profile.palette.syntax;
    if filled.iter().any(|f| f == "syntax.tag")
        && p.tag == keyword_before
        && p.keyword != keyword_before
    {
        p.tag = p.keyword;
    }
}

/// The colour on the same OKLCH hue and chroma as `role`'s current colour, nearest in lightness
/// (the current colour first), at which every pair `role` takes part in passes, counting only the
/// paired roles `counts` accepts. `None` when no lightness does. At equal distance the step away
/// from the background wins, so a tie is settled with more contrast rather than less.
fn nearest_passing(
    profile: &Profile,
    role: &str,
    counts: &dyn Fn(&str) -> bool,
) -> Result<Option<Rgb>> {
    let current = role_colour(&profile.palette, role).expect("a complete palette");
    let bold = role
        .strip_prefix("syntax.")
        .is_some_and(|t| crate::profile::style(t).contains(&"bold"));
    let base = current.to_oklch();
    let background = profile.palette.background.expect("a complete palette");
    let away = if is_dark(background) { 1.0 } else { -1.0 };
    let mut trial = profile.clone();
    let mut passes = |candidate: Rgb| -> Result<bool> {
        set_role(&mut trial.palette, role, candidate);
        let report = crate::check::check(&trial)?;
        Ok(report.failures().iter().all(|p| {
            let other = if p.fg_role == role {
                &p.bg_role
            } else if p.bg_role == role {
                &p.fg_role
            } else {
                return true;
            };
            // a failing contrast pair always rules the colour out; a failing distinct pair only
            // once its partner is placed
            matches!(p.kind, Kind::Tone | Kind::Vision(_)) && !counts(other)
        }))
    };
    if passes(current)? {
        return Ok(Some(current));
    }
    for step in 1..=400 {
        for sign in [away, -away] {
            let l = base.l + sign * 0.0025 * f64::from(step);
            if !(0.0..=1.0).contains(&l) {
                continue;
            }
            let candidate = Oklch::new(l, base.c, base.h).to_rgb();
            // near black or white the gamut leaves too little chroma for the hue to show; a
            // grey there would read as the text colour rather than as its role
            // a bold role is already told from regular text by its weight, so it may go grey
            if base.c > 0.03 && candidate.to_oklch().c < 0.02 && !bold {
                continue;
            }
            if candidate != current && passes(candidate)? {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}

fn role_colour(p: &Palette, role: &str) -> Option<Rgb> {
    match role.strip_prefix("syntax.") {
        Some(token) => p.syntax.get(token),
        None => p.get(role),
    }
}

fn set_role(p: &mut Palette, role: &str, colour: Rgb) {
    let slot = match role.strip_prefix("syntax.") {
        Some(token) => p.syntax.slot(token),
        None => p.slot(role),
    };
    *slot.expect("role names come from the check") = Some(colour);
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
    let paired = |pair: &crate::check::Pair| matches!(pair.kind, Kind::Tone | Kind::Vision(_));
    // a paired colour the tool placed was free to move away from its partner, so a pair still
    // failing is the tool's, whoever set the other side
    let mut apart: Vec<String> = Vec::new();
    for pair in report.failures().into_iter().filter(|p| paired(p)) {
        let (fg, bg) = (pair.fg_role.as_str(), pair.bg_role.as_str());
        let name = format!("{fg} and {bg}");
        if (movable(fg, filled) || movable(bg, filled)) && !apart.contains(&name) {
            apart.push(name);
        }
    }
    let blamed: Vec<String> = report
        .failures()
        .into_iter()
        .filter(|pair| !paired(pair))
        .filter(|pair| {
            let (fg, bg) = (pair.fg_role.as_str(), pair.bg_role.as_str());
            (derived(fg) && (bg == "background" || derived(bg)))
                || (derived(bg) && (fg == "background" || derived(fg)))
        })
        .map(|pair| format!("  {pair}"))
        .collect();
    if blamed.is_empty() && apart.is_empty() {
        return Ok(());
    }
    let mut message = String::new();
    if !blamed.is_empty() {
        message.push_str(&format!(
            "no colour could be derived that meets these floors:\n{}",
            blamed.join("\n")
        ));
    }
    if !apart.is_empty() {
        // the colours tried along the way were never chosen by anyone, so the roles are named
        // against the person's own background and floors instead
        let c = &profile.needs.contrast;
        let background = profile.palette.background.expect("a complete palette");
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(&format!(
            "these paired roles cannot be kept apart on the background {background} at a text floor \
             of {}:1: {}.\nThe band of lightness above the text floor is too narrow to give each its \
             own tone {} apart and keep them {} apart under colour vision deficiencies. Set these \
             colours by hand, choose a {} background or lower needs.contrast.tone_difference or \
             needs.contrast.colour_vision_distance",
            c.text,
            apart.join(", "),
            c.tone_difference,
            c.colour_vision_distance,
            if is_dark(background) { "darker" } else { "lighter" }
        ));
    }
    // a colour the person set that already misses its floor is often why nothing could be found,
    // so it is named alongside rather than left for a later check
    let own: Vec<String> = report
        .failures()
        .into_iter()
        .filter(|pair| !paired(pair))
        .filter(|pair| !derived(&pair.fg_role) && !derived(&pair.bg_role))
        .map(|pair| format!("  {pair}"))
        .collect();
    if !own.is_empty() {
        message.push_str(&format!(
            "\nThese colours you set already miss their floors:\n{}",
            own.join("\n")
        ));
    }
    if blamed
        .iter()
        .any(|line| line.contains("highlight_distance"))
    {
        message.push_str(
            "\nA highlight can only move away from the background while the text on it keeps its \
             floor, so the foreground needs more contrast with the background than the floor alone",
        );
    }
    if !blamed.is_empty() {
        message.push_str("\nSet the colours by hand or change the floors in [needs.contrast]");
    }
    bail!("{message}")
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

fn syntax_hue(
    role: &str,
    on: &[Rgb],
    over: &[Rgb],
    needs: &crate::profile::ContrastNeeds,
) -> Result<Rgb> {
    let hue = SYNTAX_HUES
        .iter()
        .find(|(r, _)| *r == role)
        .map_or(0.0, |&(_, h)| h);
    solve_over(
        &format!("syntax.{role}"),
        hue,
        0.16,
        on,
        needs.text,
        needs.apca,
        over,
        needs.overlay,
    )
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
    solve_over(role, hue, chroma, on, floor, apca, &[], 0.0)
}

/// As `solve_text`, also meeting `over_floor` on each band in `over`, such as a current line.
#[allow(clippy::too_many_arguments)]
pub fn solve_over(
    role: &str,
    hue: f64,
    chroma: f64,
    on: &[Rgb],
    floor: f64,
    apca: Option<f64>,
    over: &[Rgb],
    over_floor: f64,
) -> Result<Rgb> {
    let anchor = on[0];
    let dark = is_dark(anchor);
    let target = floor + MARGIN;
    let apca_ok = |c: Rgb, lc: f64| apca.is_none_or(|need| apca_lc(c, anchor).abs() >= need + lc);
    let over_ok = |c: Rgb, margin: f64| {
        over.iter()
            .all(|&b| contrast_ratio(c, b) >= over_floor + margin)
    };
    let meets = |c: Rgb| {
        on.iter().all(|&s| contrast_ratio(c, s) >= target) && apca_ok(c, 1.0) && over_ok(c, MARGIN)
    };
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
    let on_ok = on.iter().all(|&s| contrast_ratio(extreme, s) >= floor);
    if on_ok && apca_ok(extreme, 0.0) && over_ok(extreme, 0.0) {
        return Ok(extreme);
    }
    if on_ok && !over_ok(extreme, 0.0) {
        let band = over
            .iter()
            .map(|b| b.hex())
            .collect::<Vec<_>>()
            .join(" and ");
        bail!(
            "cannot derive {role}: needs.contrast.overlay asks for {over_floor}:1 over {band} but \
             even {extreme} misses it there. Move the current line or the soft selection further \
             from the text or lower the overlay floor"
        );
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
    fn derived_colours_are_solved_on_surface_alt_too() {
        let plain = derive(&minimal("[palette]\nbackground = \"#ffffff\"")).unwrap();
        let d = derive(&minimal(
            "[palette]\nbackground = \"#ffffff\"\nsurface_alt = \"#d8d8d8\"",
        ))
        .unwrap();
        assert!(!d.filled.contains(&"surface_alt".to_string()));
        let report = check(&d.profile).unwrap();
        assert!(report.failures().is_empty(), "{:?}", report.failures());
        // without the second panel colour the derived muted text would sit too light for it
        let alt: Rgb = "#d8d8d8".parse().unwrap();
        let muted = |p: &Profile| p.palette.muted.unwrap();
        assert!(contrast_ratio(muted(&plain.profile), alt) < 7.0);
        assert!(contrast_ratio(muted(&d.profile), alt) >= 7.0);
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

    #[test]
    fn derived_pairs_are_spread_apart() {
        // every derived role starts at the same floor and so the same tone; the spread keeps each
        // paired role apart in tone and under every simulated deficiency
        for bg in ["#000000", "#ffffff", "#1e1e2e", "#fdf6e3"] {
            let d = derive(&minimal(&format!("[palette]\nbackground = \"{bg}\""))).unwrap();
            let report = check(&d.profile).unwrap();
            let pairs = report
                .pairs
                .iter()
                .filter(|p| matches!(p.kind, Kind::Tone | Kind::Vision(_)))
                .count();
            assert!(pairs >= 60, "{bg}: {pairs}");
            assert!(
                report.failures().is_empty(),
                "{bg}: {:?}",
                report.failures()
            );
        }
    }

    #[test]
    fn a_derived_colour_moves_but_a_set_one_does_not() {
        let p = minimal("[palette]\nbackground = \"#ffffff\"\nerror = \"#810005\"");
        let d = derive(&p).unwrap();
        assert_eq!(d.profile.palette.error, Some("#810005".parse().unwrap()));
        let warning = d.profile.palette.warning.unwrap();
        let error = d.profile.palette.error.unwrap();
        assert!(
            (warning.tone() - error.tone()).abs() >= 0.05,
            "{warning} {error}"
        );
        assert!(check(&d.profile).unwrap().failures().is_empty());
    }

    #[test]
    fn no_room_to_spread_fails_by_name() {
        // an APCA floor of Lc 90 on black leaves too narrow a band of tone for three paired roles
        let p = minimal("[needs.contrast]\napca = 90");
        let err = derive(&p).unwrap_err().to_string();
        assert!(
            err.contains("these paired roles cannot be kept apart on the background #000000"),
            "{err}"
        );
        assert!(err.contains("needs.contrast.tone_difference"), "{err}");
        // no colour tried along the way is shown as if someone had chosen it
        assert!(!err.contains(" in tone:"), "{err}");
    }

    #[test]
    fn a_derived_colour_that_cannot_move_apart_is_refused() {
        // the reviewed repro: a set error and a derived warning with no lightness far enough away
        let p = minimal(
            "[needs.contrast]\ntone_difference = 0.4\n[palette]\nbackground = \"#ffffff\"\n\
             error = \"#810005\"",
        );
        let err = derive(&p).unwrap_err().to_string();
        assert!(err.contains("error and warning"), "{err}");
        assert!(err.contains("on the background #ffffff"), "{err}");
        assert!(err.contains("own tone 0.4 apart"), "{err}");
    }

    #[test]
    fn derived_colours_meet_the_overlay_floor_on_a_set_current_line() {
        // the reviewed repro: derived colours landed at 4.3:1 over a current line set by hand
        let p = minimal("[palette]\nbackground = \"#000000\"\nline_highlight = \"#333333\"");
        let d = derive(&p).unwrap();
        let fails: Vec<String> = check(&d.profile)
            .unwrap()
            .failures()
            .iter()
            .map(|f| f.to_string())
            .collect();
        assert!(fails.is_empty(), "{fails:?}");
    }
}
