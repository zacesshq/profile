-- for Neovim, from the profile Warm dark, large text
-- written by zprf; change the profile and render again rather than editing this file
-- put this file in a colors folder on the runtimepath, then: colorscheme warm-dark-large-text

vim.cmd("highlight clear")
if vim.fn.exists("syntax_on") == 1 then
  vim.cmd("syntax reset")
end
vim.o.background = "dark"
vim.o.termguicolors = true
vim.g.colors_name = "warm-dark-large-text"

local c = {
  accent = "#6badff",
  accent_foreground = "#1b1612",
  background = "#1b1612",
  border = "#6c6c6c",
  cursor = "#debb00",
  cursor_text = "#1b1612",
  error = "#ff877f",
  focus_ring = "#6badff",
  foreground = "#ffffff",
  info = "#35b3fc",
  line_highlight = "#332e29",
  link = "#6badff",
  muted = "#aaaaaa",
  selection_background = "#b98ffd",
  selection_foreground = "#1b1612",
  selection_soft = "#32204d",
  surface = "#221d19",
  warning = "#e89927",
}

local s = {
  added = "#5bbf62",
  attribute = "#e59b00",
  changed = "#5dafff",
  comment = "#90b735",
  control = "#f683c8",
  ["function"] = "#6badff",
  invalid = "#ff877f",
  keyword = "#be96ff",
  number = "#ff895f",
  operator = "#ffffff",
  parameter = "#f98f3a",
  property = "#9ca2ff",
  regex = "#ff83a3",
  removed = "#ff877f",
  string = "#5bbf62",
  tag = "#be96ff",
  type = "#00beb6",
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

vim.g.terminal_color_0 = "#1b1612"
vim.g.terminal_color_1 = "#ff7d6c"
vim.g.terminal_color_2 = "#57b84e"
vim.g.terminal_color_3 = "#b9a400"
vim.g.terminal_color_4 = "#71a3ff"
vim.g.terminal_color_5 = "#e280e0"
vim.g.terminal_color_6 = "#00b6b6"
vim.g.terminal_color_7 = "#bbbbbb"
vim.g.terminal_color_8 = "#a4a4a4"
vim.g.terminal_color_9 = "#ffa294"
vim.g.terminal_color_10 = "#6fd066"
vim.g.terminal_color_11 = "#d2bb00"
vim.g.terminal_color_12 = "#95bbff"
vim.g.terminal_color_13 = "#fb97f8"
vim.g.terminal_color_14 = "#00cfcf"
vim.g.terminal_color_15 = "#ffffff"

-- the profile's other needs; set vim.g.zacess_profile_options = false before loading to skip them
if vim.g.zacess_profile_options ~= false then
  -- removed first so loading the scheme again does not add the entry a second time
  vim.opt.guicursor:remove("a:blinkon0")
  vim.opt.guicursor:append("a:blinkon0")
  vim.opt.colorcolumn = "81"
end
