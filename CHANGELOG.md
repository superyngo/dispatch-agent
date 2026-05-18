# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v1.1.2] - 2026-05-18

### Added
- `cli-templates.toml` resolver now searches platform-specific fallback paths after the existing chain (env var → exe dir → dev manifest). On Unix: `$HOME/.wenget/apps/dispatch-agent/config/`, `$HOME/.local/bin/config/`, `/opt/wenget/apps/dispatch-agent/config/`, `/usr/local/bin/config/`. On Windows: `%USERPROFILE%\.wenget\apps\dispatch-agent\config\`, `%LOCALAPPDATA%\Programs\dispatch-agent\config\`, `%ProgramW6432%\wenget\app\dispatch-agent\config\`, `%ProgramFiles%\gpinstall\config\`. Candidates whose required env var is unset/empty are skipped. The "not found" error now lists every path checked.
- `candidate_from_env` helper to build path candidates from environment variables.
- `find_first_existing` helper to resolve the first existing path from a candidate list.

### Fixed
- Stub platform fallback for non-Unix/non-Windows targets; assert first-hit ordering in tests.

### CI
- Release archives now bundle `config/cli-templates.toml` alongside the binary.
- Added preflight check ensuring `config/cli-templates.toml` exists before packaging.
- Unified upload-artifact steps via `matrix.archive_ext` (`.tar.gz` / `.zip`).
- Added archive content verification step to build job.
- Added installation instructions to release body template.

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
