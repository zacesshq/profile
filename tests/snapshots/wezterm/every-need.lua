-- for WezTerm, from the profile Every need
-- written by zprf; change the profile and render again rather than editing this file
-- use it from wezterm.lua with: require("every-need").apply(config)

local wezterm = require("wezterm")

local M = {}

M.colors = {
  foreground = "#ffffff",
  background = "#101418",
  cursor_bg = "#dab600",
  cursor_fg = "#101418",
  cursor_border = "#dab600",
  selection_fg = "#101418",
  selection_bg = "#b68cfa",
  split = "#696969",
  scrollbar_thumb = "#696969",
  ansi = {
    "#101418",
    "#ff7867",
    "#55b64c",
    "#b6a200",
    "#6da0ff",
    "#e07ddd",
    "#00b4b4",
    "#b8b8b8",
  },
  brights = {
    "#a1a1a1",
    "#ff9e8f",
    "#6ccd63",
    "#cfb800",
    "#91b8ff",
    "#f894f5",
    "#00cccc",
    "#ffffff",
  },
  tab_bar = {
    background = "#171b1f",
    active_tab = { bg_color = "#66a9ff", fg_color = "#101418", intensity = "Bold" },
    inactive_tab = { bg_color = "#171b1f", fg_color = "#a7a7a7" },
    inactive_tab_hover = { bg_color = "#171b1f", fg_color = "#ffffff" },
    new_tab = { bg_color = "#171b1f", fg_color = "#a7a7a7" },
    new_tab_hover = { bg_color = "#171b1f", fg_color = "#ffffff" },
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
  config.cursor_thickness = "6px"
  config.font = wezterm.font_with_fallback({ "Atkinson Hyperlegible Mono" })
  config.font_size = 16.0
  config.line_height = 1.5
  -- a solid background, so text is never drawn over an image or whatever sits behind the window
  config.window_background_opacity = 1.0
  config.text_background_opacity = 1.0
  config.macos_window_background_blur = 0
  config.window_background_image = nil
  config.background = nil
  return config
end

return M
