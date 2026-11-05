# Security Policy

Zacess Profile reads files people share with each other and writes config files that terminals and
editors load. Some of those files are code: the WezTerm and Neovim targets are Lua. A flaw that lets
a profile reach further than its colours and settings is taken seriously.

## Scope

In scope:

- A profile that makes `render` write outside the folder given to `--out`, for example through a
  crafted `id` or name
- A profile whose text ends up as code in a rendered file, such as a font name that breaks out of a
  Lua or JSON string
- A profile that makes `apply --live` send anything other than OSC 4, 10, 11 and 12 colour
  sequences to the terminal
- A crash or unbounded resource use from a malformed profile
- A template that makes an app load an outside resource or run a command

## Out of scope

- How a terminal or editor itself handles its config or escape sequences (report those to their
  makers)
- Vulnerabilities in Rust crates with no demonstrated impact on this tool (report those upstream)
- Running a rendered file you have edited by hand

## Supported versions

Only the latest release and the latest commit on `main` receive security fixes.

## Reporting a vulnerability

> [!IMPORTANT]
> Report privately, never in a public issue or discussion. Use
> [GitHub private vulnerability reporting](https://github.com/zacesshq/profile/security/advisories/new)
> or email [security@zacess.com](mailto:security@zacess.com) with details and reproduction steps.
> Expect an acknowledgement within a few days.

The organisation's [security policy](https://github.com/zacesshq/.github/blob/main/SECURITY.md)
covers every Zacess repository. This file takes precedence where the two differ.
