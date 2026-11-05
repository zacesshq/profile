//! OSC sequences that recolour a running terminal without touching its config.
//!
//! OSC 4 sets the sixteen palette slots, OSC 10 the default text, OSC 11 the default background and
//! OSC 12 the cursor. Each ends with ST (ESC backslash), which xterm, kitty, WezTerm, Ghostty,
//! Alacritty, iTerm2 and foot all accept.

use anyhow::{Context, Result};

use crate::profile::Profile;

const ESC: &str = "\x1b";
const ST: &str = "\x1b\\";

pub fn sequences(profile: &Profile) -> Result<String> {
    let p = &profile.palette;
    let ansi = p
        .terminal
        .ansi
        .as_ref()
        .context("terminal.ansi is missing: run derive before apply")?;
    let need = |c: Option<crate::colour::Rgb>, role: &str| {
        c.with_context(|| format!("{role} is missing: run derive before apply"))
    };
    let mut out = String::new();
    for (i, colour) in ansi.iter().enumerate() {
        out.push_str(&format!("{ESC}]4;{i};{}{ST}", colour.xparse()));
    }
    out.push_str(&format!(
        "{ESC}]10;{}{ST}",
        need(p.foreground, "foreground")?.xparse()
    ));
    out.push_str(&format!(
        "{ESC}]11;{}{ST}",
        need(p.background, "background")?.xparse()
    ));
    out.push_str(&format!(
        "{ESC}]12;{}{ST}",
        need(p.cursor, "cursor")?.xparse()
    ));
    Ok(out)
}

/// Wraps sequences in the tmux passthrough form, so they reach the terminal tmux runs in.
/// tmux only forwards them when its `allow-passthrough` option is on.
pub fn tmux_passthrough(seq: &str) -> String {
    let doubled = seq.replace(ESC, "\x1b\x1b");
    format!("\x1bPtmux;{doubled}{ST}")
}

/// A printable form with ESC shown as `\e`, for reading or pasting into a script.
pub fn escaped(seq: &str) -> String {
    seq.replace(ST, "\\e\\\\\n").replace(ESC, "\\e")
}
