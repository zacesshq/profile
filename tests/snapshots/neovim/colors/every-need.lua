-- for Neovim, from the profile Every need
-- written by zprf; change the profile and render again rather than editing this file
-- put this file in a colors folder on the runtimepath, then: colorscheme every-need

vim.cmd("highlight clear")
if vim.fn.exists("syntax_on") == 1 then
  vim.cmd("syntax reset")
end
vim.o.background = "dark"
vim.o.termguicolors = true
vim.g.colors_name = "every-need"

local c = {
  accent = "#66a9ff",
  accent_foreground = "#101418",
  background = "#101418",
  border = "#696969",
  cursor = "#dab600",
  cursor_text = "#101418",
  error = "#fd837b",
  focus_ring = "#66a9ff",
  foreground = "#ffffff",
  info = "#31b0f9",
  line_highlight = "#272c30",
  link = "#66a9ff",
  muted = "#a7a7a7",
  selection_background = "#b68cfa",
  selection_foreground = "#101418",
  selection_soft = "#301e4b",
  surface = "#171b1f",
  surface_alt = "#171b1f",
  warning = "#f6a73a",
}

local s = {
  added = "#57bb5e",
  attribute = "#e19800",
  changed = "#9accff",
  comment = "#8cb431",
  control = "#ff95d2",
  ["function"] = "#66a9ff",
  invalid = "#ffb4ae",
  keyword = "#bb93ff",
  number = "#ffb69d",
  operator = "#ffffff",
  parameter = "#f68c36",
  property = "#bdc3ff",
  regex = "#ff7da0",
  removed = "#ff9c94",
  string = "#68cc6e",
  tag = "#bb93ff",
  type = "#2fcbc4",
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
hi("StatusLine", { fg = c.foreground, bg = c.surface_alt, bold = true })
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

vim.g.terminal_color_0 = "#101418"
vim.g.terminal_color_1 = "#ff7867"
vim.g.terminal_color_2 = "#55b64c"
vim.g.terminal_color_3 = "#b6a200"
vim.g.terminal_color_4 = "#6da0ff"
vim.g.terminal_color_5 = "#e07ddd"
vim.g.terminal_color_6 = "#00b4b4"
vim.g.terminal_color_7 = "#b8b8b8"
vim.g.terminal_color_8 = "#a1a1a1"
vim.g.terminal_color_9 = "#ff9e8f"
vim.g.terminal_color_10 = "#6ccd63"
vim.g.terminal_color_11 = "#cfb800"
vim.g.terminal_color_12 = "#91b8ff"
vim.g.terminal_color_13 = "#f894f5"
vim.g.terminal_color_14 = "#00cccc"
vim.g.terminal_color_15 = "#ffffff"

-- the profile's other needs; set vim.g.zacess_profile_options = false before loading to skip them
if vim.g.zacess_profile_options ~= false then
  -- removed first so loading the scheme again does not add the entry a second time
  vim.opt.guicursor:remove("a:blinkon0")
  vim.opt.guicursor:append("a:blinkon0")
  vim.opt.colorcolumn = "73"
  -- a diff opens one buffer above the other instead of side by side
  vim.opt.diffopt:remove("vertical")
  vim.opt.diffopt:remove("horizontal")
  vim.opt.diffopt:append("horizontal")
  -- no tildes past the end of a buffer and no see-through floating windows or menus
  vim.opt.fillchars:append({ eob = " " })
  vim.o.winblend = 0
  vim.o.pumblend = 0
end
