-- for WezTerm, from the profile Zacess Deep Contrast Dark
-- written by zprf; change the profile and render again rather than editing this file
-- use it from wezterm.lua with: require("zacess-deep-contrast-dark").apply(config)

local wezterm = require("wezterm")

local M = {}

M.colors = {
  foreground = "#ffffff",
  background = "#000000",
  cursor_bg = "#ffff00",
  cursor_fg = "#000000",
  cursor_border = "#ffff00",
  selection_fg = "#000000",
  selection_bg = "#ff70ff",
  split = "#00ffff",
  scrollbar_thumb = "#00ffff",
  ansi = {
    "#000000",
    "#ff5f5f",
    "#00cd00",
    "#cdcd00",
    "#8888ff",
    "#ff2aff",
    "#00cdcd",
    "#e5e5e5",
  },
  brights = {
    "#b8b8b8",
    "#ff9e9e",
    "#00ff00",
    "#ffff00",
    "#b0b0ff",
    "#ff8fff",
    "#00ffff",
    "#ffffff",
  },
  tab_bar = {
    background = "#000000",
    active_tab = { bg_color = "#00ffff", fg_color = "#000000", intensity = "Bold" },
    inactive_tab = { bg_color = "#0a0a0a", fg_color = "#b3b3b3" },
    inactive_tab_hover = { bg_color = "#0a0a0a", fg_color = "#ffffff" },
    new_tab = { bg_color = "#000000", fg_color = "#b3b3b3" },
    new_tab_hover = { bg_color = "#000000", fg_color = "#ffffff" },
  },
}

function M.apply(config)
  config.colors = M.colors
  -- WezTerm's own cursor is a steady block, so this stops blinking without changing the shape
  config.default_cursor_style = "SteadyBlock"
  config.cursor_blink_rate = 0
  config.text_blink_rate = 0
  config.text_blink_rate_rapid = 0
  config.animation_fps = 1
  config.visual_bell = { fade_in_duration_ms = 0, fade_out_duration_ms = 0 }
  config.font = wezterm.font_with_fallback({ "Atkinson Hyperlegible Mono" })
  config.font_size = 14.0
  return config
end

return M
