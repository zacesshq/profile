-- for Neovim, from the profile Zacess Deep Contrast Dark
-- written by zprf; change the profile and render again rather than editing this file
-- put this file in a colors folder on the runtimepath, then: colorscheme zacess-deep-contrast-dark

vim.cmd("highlight clear")
if vim.fn.exists("syntax_on") == 1 then
  vim.cmd("syntax reset")
end
vim.o.background = "dark"
vim.o.termguicolors = true
vim.g.colors_name = "zacess-deep-contrast-dark"

local c = {
  accent = "#00ffff",
  accent_foreground = "#000000",
  background = "#000000",
  border = "#00ffff",
  cursor = "#ffff00",
  cursor_text = "#000000",
  error = "#fe8167",
  focus_ring = "#ffff00",
  foreground = "#ffffff",
  info = "#59a4f9",
  line_highlight = "#262626",
  link = "#1db4ff",
  muted = "#b3b3b3",
  selection_background = "#ff70ff",
  selection_foreground = "#000000",
  selection_soft = "#720072",
  surface = "#000000",
  warning = "#ffd370",
}

local s = {
  added = "#00f000",
  attribute = "#c195ff",
  changed = "#67aeff",
  comment = "#00c600",
  control = "#ffabfd",
  ["function"] = "#ffdf5a",
  invalid = "#fd8b78",
  keyword = "#67aeff",
  number = "#e19e00",
  operator = "#ffffff",
  parameter = "#ffffff",
  property = "#c195ff",
  regex = "#ff80c5",
  removed = "#ffaa7c",
  string = "#ffaa7c",
  tag = "#67aeff",
  type = "#00ebb1",
  variable = "#ffffff",
}

local function hi(group, spec)
  vim.api.nvim_set_hl(0, group, spec)
end

-- interface
hi("Normal", { fg = c.foreground, bg = c.background })
hi("NormalNC", { fg = c.foreground, bg = c.background })
hi("NormalFloat", { fg = c.foreground, bg = c.surface })
hi("FloatBorder", { fg = c.border, bg = c.surface })
hi("Cursor", { fg = c.cursor_text, bg = c.cursor })
hi("TermCursor", { fg = c.cursor_text, bg = c.cursor })
hi("CursorLine", { bg = c.line_highlight })
hi("CursorColumn", { bg = c.line_highlight })
hi("ColorColumn", { bg = c.line_highlight })
hi("LineNr", { fg = c.muted })
hi("CursorLineNr", { fg = c.foreground, bold = true })
hi("SignColumn", { fg = c.muted, bg = c.background })
hi("Visual", { fg = c.selection_foreground, bg = c.selection_background })
hi("Search", { fg = c.foreground, bg = c.selection_soft, underline = true })
hi("CurSearch", { fg = c.accent_foreground, bg = c.accent, bold = true })
hi("IncSearch", { fg = c.accent_foreground, bg = c.accent, bold = true })
hi("MatchParen", { fg = c.accent, bold = true, underline = true })
hi("StatusLine", { fg = c.foreground, bg = c.surface, bold = true })
hi("StatusLineNC", { fg = c.muted, bg = c.surface })
hi("TabLine", { fg = c.muted, bg = c.surface })
hi("TabLineSel", { fg = c.accent_foreground, bg = c.accent, bold = true })
hi("TabLineFill", { bg = c.surface })
hi("WinSeparator", { fg = c.border })
hi("Pmenu", { fg = c.foreground, bg = c.surface })
hi("PmenuSel", { fg = c.selection_foreground, bg = c.selection_background, bold = true })
hi("PmenuThumb", { bg = c.border })
hi("NonText", { fg = c.muted })
hi("Whitespace", { fg = c.muted })
hi("Folded", { fg = c.muted, bg = c.surface, italic = true })
hi("Directory", { fg = s["function"] })
hi("Title", { fg = s.keyword, bold = true })
hi("Underlined", { fg = c.link, underline = true })
hi("ErrorMsg", { fg = c.error, bold = true })
hi("WarningMsg", { fg = c.warning, bold = true })
hi("MoreMsg", { fg = c.info })
hi("Question", { fg = c.info })

-- syntax, with the font styles that keep each role readable without colour
hi("Comment", { fg = s.comment, italic = true })
hi("Keyword", { fg = s.keyword, bold = true })
hi("Statement", { fg = s.keyword, bold = true })
hi("StorageClass", { fg = s.keyword, bold = true })
hi("Conditional", { fg = s.control, bold = true })
hi("Repeat", { fg = s.control, bold = true })
hi("Exception", { fg = s.control, bold = true })
hi("Include", { fg = s.control, bold = true })
hi("Function", { fg = s["function"] })
hi("Type", { fg = s["type"] })
hi("Identifier", { fg = s.variable })
hi("String", { fg = s.string })
hi("Character", { fg = s.string })
hi("Number", { fg = s.number })
hi("Float", { fg = s.number })
hi("Boolean", { fg = s.number })
hi("Constant", { fg = s.number })
hi("Operator", { fg = s.operator })
hi("Delimiter", { fg = s.operator })
hi("Special", { fg = s.regex })
hi("Error", { fg = s.invalid, underline = true })
hi("Todo", { fg = s.comment, bold = true, italic = true })
hi("DiffAdd", { fg = s.added })
hi("DiffDelete", { fg = s.removed })
hi("DiffChange", { fg = s.changed })
hi("DiffText", { fg = s.changed, bold = true, underline = true })
hi("Added", { fg = s.added })
hi("Removed", { fg = s.removed })
hi("Changed", { fg = s.changed })

-- tree-sitter captures
hi("@keyword", { link = "Keyword" })
hi("@keyword.function", { link = "Keyword" })
hi("@keyword.conditional", { link = "Conditional" })
hi("@keyword.repeat", { link = "Repeat" })
hi("@keyword.return", { link = "Conditional" })
hi("@keyword.import", { link = "Include" })
hi("@function", { link = "Function" })
hi("@function.call", { link = "Function" })
hi("@function.method", { link = "Function" })
hi("@type", { link = "Type" })
hi("@type.builtin", { link = "Type" })
hi("@variable", { link = "Identifier" })
hi("@variable.parameter", { fg = s.parameter, italic = true })
hi("@property", { fg = s.property })
hi("@variable.member", { fg = s.property })
hi("@string", { link = "String" })
hi("@string.regexp", { fg = s.regex })
hi("@string.escape", { fg = s.regex })
hi("@number", { link = "Number" })
hi("@boolean", { link = "Boolean" })
hi("@constant", { link = "Constant" })
hi("@comment", { link = "Comment" })
hi("@operator", { link = "Operator" })
hi("@punctuation", { link = "Delimiter" })
hi("@tag", { fg = s.tag, bold = true })
hi("@tag.attribute", { fg = s.attribute })
hi("@markup.link", { fg = c.link, underline = true })
hi("@diff.plus", { link = "Added" })
hi("@diff.minus", { link = "Removed" })
hi("@diff.delta", { link = "Changed" })

-- diagnostics are underlined as well as coloured, so they never rest on colour alone
hi("DiagnosticError", { fg = c.error })
hi("DiagnosticWarn", { fg = c.warning })
hi("DiagnosticInfo", { fg = c.info })
hi("DiagnosticHint", { fg = c.info })
hi("DiagnosticUnderlineError", { sp = c.error, undercurl = true })
hi("DiagnosticUnderlineWarn", { sp = c.warning, undercurl = true })
hi("DiagnosticUnderlineInfo", { sp = c.info, underline = true })
hi("DiagnosticUnderlineHint", { sp = c.info, underline = true })

vim.g.terminal_color_0 = "#000000"
vim.g.terminal_color_1 = "#ff5f5f"
vim.g.terminal_color_2 = "#00cd00"
vim.g.terminal_color_3 = "#cdcd00"
vim.g.terminal_color_4 = "#8888ff"
vim.g.terminal_color_5 = "#ff2aff"
vim.g.terminal_color_6 = "#00cdcd"
vim.g.terminal_color_7 = "#e5e5e5"
vim.g.terminal_color_8 = "#b8b8b8"
vim.g.terminal_color_9 = "#ff9e9e"
vim.g.terminal_color_10 = "#00ff00"
vim.g.terminal_color_11 = "#ffff00"
vim.g.terminal_color_12 = "#b0b0ff"
vim.g.terminal_color_13 = "#ff8fff"
vim.g.terminal_color_14 = "#00ffff"
vim.g.terminal_color_15 = "#ffffff"

-- the profile's other needs; set vim.g.zacess_profile_options = false before loading to skip them
if vim.g.zacess_profile_options ~= false then
  -- removed first so loading the scheme again does not add the entry a second time
  vim.opt.guicursor:remove("a:blinkon0")
  vim.opt.guicursor:append("a:blinkon0")
end
