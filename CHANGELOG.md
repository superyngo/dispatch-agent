# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `cli-templates.toml` resolver now searches platform-specific fallback paths after the existing chain (env var → exe dir → dev manifest). On Unix: `$HOME/.wenget/apps/dispatch-agent/config/`, `$HOME/.local/bin/config/`, `/opt/wenget/apps/dispatch-agent/config/`, `/usr/local/bin/config/`. On Windows: `%USERPROFILE%\.wenget\apps\dispatch-agent\config\`, `%LOCALAPPDATA%\Programs\dispatch-agent\config\`, `%ProgramW6432%\wenget\app\dispatch-agent\config\`, `%ProgramFiles%\gpinstall\config\`. Candidates whose required env var is unset/empty are skipped. The "not found" error now lists every path checked.

## [v0.1.1] - 2026-05-15

### Fixed
- Gate `dispatch::process::unix` / `windows` submodules by target OS so Windows builds no longer fail trying to compile unix-only code (libc `setsid`/`killpg`, `pre_exec`, `signal_hook::iterator`).
- Add Windows stub for `start_signal_watcher`.

## [v0.1.0] - 2026-05-15

### Added
- Initial release of `dispatch-agent`: dispatch tasks to other agent CLIs with tier-based fallback.
- `init`, `detect`, `config`, and `dispatch` subcommands.
- CLI templates and round-robin tier state.
- GitHub Actions release workflow for multi-platform binary builds (Linux/Windows/macOS).
