-- for WezTerm, from the profile Warm dark, large text
-- written by zprf; change the profile and render again rather than editing this file
-- use it from wezterm.lua with: require("warm-dark-large-text").apply(config)

local wezterm = require("wezterm")

local M = {}

M.colors = {
  foreground = "#ffffff",
  background = "#1b1612",
  cursor_bg = "#debb00",
  cursor_fg = "#1b1612",
  cursor_border = "#debb00",
  selection_fg = "#1b1612",
  selection_bg = "#b98ffd",
  split = "#6c6c6c",
  scrollbar_thumb = "#6c6c6c",
  ansi = {
    "#1b1612",
    "#ff7d6c",
    "#57b84e",
    "#b9a400",
    "#71a3ff",
    "#e280e0",
    "#00b6b6",
    "#bbbbbb",
  },
  brights = {
    "#a4a4a4",
    "#ffa294",
    "#6fd066",
    "#d2bb00",
    "#95bbff",
    "#fb97f8",
    "#00cfcf",
    "#ffffff",
  },
  tab_bar = {
    background = "#221d19",
    active_tab = { bg_color = "#6badff", fg_color = "#1b1612", intensity = "Bold" },
    inactive_tab = { bg_color = "#221d19", fg_color = "#aaaaaa" },
    inactive_tab_hover = { bg_color = "#221d19", fg_color = "#ffffff" },
    new_tab = { bg_color = "#221d19", fg_color = "#aaaaaa" },
    new_tab_hover = { bg_color = "#221d19", fg_color = "#ffffff" },
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
  config.cursor_thickness = "4px"
  config.font = wezterm.font_with_fallback({ "Atkinson Hyperlegible Mono" })
  config.font_size = 18.0
  config.line_height = 1.3
  return config
end

return M
