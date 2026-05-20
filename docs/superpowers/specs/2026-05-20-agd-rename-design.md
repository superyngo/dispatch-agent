# agd — v2.0.0 Rename & Companion Improvements

**Status:** Draft
**Date:** 2026-05-20
**Target release:** v2.0.0 (breaking)

## 1. Goals

Ship a single coordinated v2.0.0 release that:

1. Renames the project from `dispatch-agent` to `agd` (agent dispatcher) — binary, crate, GitHub repo, environment variables, on-disk paths, config file names, and user-visible strings. No backwards-compatible aliases. No migration shims.
2. Adds a working `agd --version` flag.
3. Enriches template-resolution warnings/errors with the actual `cli-templates.toml` path that was loaded, so operators can debug a misconfigured template entry without having to grep nine candidate paths.
4. Lands the currently-uncommitted `config/cli-templates.toml` edits (the new `[agy]` template, five explicit `version_flag` declarations, the npx-gemini behavior changes) and fixes their accompanying misleading comments.
5. Replaces the README `## Usage` placeholder with a complete operator-facing reference: configuration example, subcommand summaries, dispatch examples, environment variables.

## 2. Non-goals

- No backwards compatibility for the old `dispatch-agent` binary, env vars, paths, or config file names. The project is early-stage; existing local installs will be replaced rather than migrated.
- No introduction of an `APP_NAME` constant or other rename-friendly abstraction (YAGNI — the project is not expected to rename again).
- No rewriting of historical artifacts. `CHANGELOG.md` entries for v1.0.0 – v1.1.2, and the spec/plan documents in `docs/superpowers/{specs,plans}/2026-05-18-*.md`, retain their original `dispatch-agent` wording as period-accurate history.
- No changes to the `fake_agent` test binary name.
- No edit to the installer Gist (`a6b786af38b8b4c2ce15a70ae5387bd7`). That script is parameterized by `APP_NAME` / `REPO` env vars passed from the install command; only the values in `README.md` change.
- The GitHub repository rename itself (`gh repo rename agd`) is an operator action, not a codebase action. This spec does not execute it.

## 3. Rename mapping (authoritative)

The following table is the complete identifier replacement set. Anything not listed is unchanged.

| Category | Old | New |
|---|---|---|
| Cargo `package.name` | `dispatch-agent` | `agd` |
| Cargo `[[bin]].name` (main binary) | `dispatch-agent` | `agd` |
| Cargo `[[bin]].name` (test helper) | `fake_agent` | `fake_agent` *(unchanged)* |
| Clap `#[command(name = ...)]` | `"dispatch-agent"` | `"agd"` |
| Env var (template override) — appears in `src/templates.rs:123` (read), `src/templates.rs:156` (`format_not_found_error` user message), `src/templates.rs:211,222,235,246,258,277,290,299,478` (tests), `src/detect.rs:107` (test), `tests/detect.rs:15`, `tests/dispatch.rs:51,79` | `DISPATCH_AGENT_TEMPLATES` | `AGD_TEMPLATES` |
| Env var (re-entry depth guard) — appears in `src/dispatch/mod.rs:32` (`env::var` read), `src/dispatch/mod.rs:37` (error message string `"invalid DISPATCH_AGENT_DEPTH value"`), `src/env.rs:55` (doc comment), `src/env.rs:66,154` (env-map literal and test read) | `DISPATCH_AGENT_DEPTH` | `AGD_DEPTH` |
| Cache directory | `<cache_dir>/dispatch-agent/rr-state.json` | `<cache_dir>/agd/rr-state.json` |
| User config file | `~/.config/dispatch-agent.toml` | `~/.config/agd.toml` |
| Project config file | `<git-root>/.config/dispatch-agent.toml` | `<git-root>/.config/agd.toml` |
| Fallback install paths in `src/templates.rs::platform_fallback_candidates` (both `unix` and `windows` `cfg` branches: path-segment array literals plus `/opt/wenget/apps/dispatch-agent/...` and `/usr/local/bin/...` string literals) | every `dispatch-agent` path segment | `agd` |
| Test fixture strings in `src/detect.rs:123,166` (`"dispatch-agent-fake-nonexistent-xyz"` synthetic binary name used to assert detect-miss behavior) | `dispatch-agent-fake-nonexistent-xyz` | `agd-fake-nonexistent-xyz` |
| User-facing strings in `src/config_cmd.rs:58,61,64,99,143` (config-path hints in `cmd_config_path` / `cmd_config_show`; error message `"Run 'dispatch-agent init' to create one."` in two sites) | `dispatch-agent` | `agd` |
| User-facing error prefixes in `src/env.rs:15,22,40` (`"dispatch-agent: ..."` prefix on three eprintln sites) and the env-var literal at `src/env.rs:66,154` plus the doc-comment at `src/env.rs:55` | `dispatch-agent` / `DISPATCH_AGENT_DEPTH` | `agd` / `AGD_DEPTH` |
| Path-label match and test fixtures in `src/dispatch/display.rs:119,155,168` (display logic checks `.contains(".config/dispatch-agent.toml")`; tests assert against fake CLI name and `[✗] missing (...)` line) | `dispatch-agent` / `dispatch-agent-fake-nonexistent-cli-xyz` | `agd` / `agd-fake-nonexistent-cli-xyz` |
| Test code in `src/config.rs:95,100` (test joins `dispatch-agent.toml` to fixture dirs) | `dispatch-agent.toml` | `agd.toml` |
| Test fixtures in `tests/snapshots_test.rs:25,48,95` (`"dispatch-agent-nonexistent-xyz"` and `/tmp/dispatch-agent-test.toml`) | `dispatch-agent*` | `agd*` |
| `env!("CARGO_BIN_EXE_dispatch-agent")` references across `tests/init.rs:14,68,99`, `tests/detect.rs:13`, `tests/config_cmd.rs:7,35`, `tests/dispatch.rs:44,74,96,116,144,169,197,222,243,265` (16 occurrences). The macro key derives from the Cargo `[[bin]].name`, so after that change every such literal must be rewritten. | `CARGO_BIN_EXE_dispatch-agent` | `CARGO_BIN_EXE_agd` |
| Top-of-file documentation comment block in `config/cli-templates.toml:2-10` (mentions of `dispatch-agent`, `~/.config/dispatch-agent.toml`, `<git-root>/.config/dispatch-agent.toml`, `dispatch-agent init`, `dispatch-agent config edit`) | `dispatch-agent` / `dispatch-agent.toml` | `agd` / `agd.toml` |
| Crate-name imports in tests (`use dispatch_agent::...`) | follow automatically from the Cargo `package.name` change — Rust crate name derives from package name with `-` → `_` | `use agd::...` |
| GitHub repository | `superyngo/dispatch-agent` | `superyngo/agd` |
| `README.md` `APP_NAME` and `REPO` install-script values | `dispatch-agent` / `superyngo/dispatch-agent` | `agd` / `superyngo/agd` |
| User-facing strings in `src/init.rs` (`INIT_USAGE`, hint messages, examples) | `dispatch-agent` | `agd` |
| User-facing strings in `src/cli.rs` (help text, command name) | `dispatch-agent` | `agd` |
| `.github/workflows/release.yml` asset names, paths, install hints | `dispatch-agent` | `agd` |
| `scripts/parity_check.sh`, `scripts/regen_golden.sh` binary references | `dispatch-agent` | `agd` |
| Tests under `tests/` referencing the binary name, env vars, or config-file names | `dispatch-agent*` / `DISPATCH_AGENT_*` | `agd*` / `AGD_*` |
| Snapshot files under `tests/snapshots/` | regenerate with `cargo insta review` after code rename | new strings |

### Files explicitly NOT modified

- `CHANGELOG.md` — historical entries (v1.0.0 – v1.1.2) are left untouched. A new top-of-file entry for v2.0.0 announces the rename.
- `docs/superpowers/specs/2026-05-18-cli-templates-search-paths-design.md`
- `docs/superpowers/specs/2026-05-18-release-bundle-config-design.md`
- `docs/superpowers/plans/2026-05-18-cli-templates-search-paths.md`
- `docs/superpowers/plans/2026-05-18-release-bundle-config.md`

## 4. Feature: `agd --version`

`src/cli.rs:5` (the `#[command(name = "dispatch-agent")]` attribute on the `Cli` struct) is modified to also carry `version`. Clap will read `CARGO_PKG_VERSION` at compile time and emit `agd 2.0.0` when the user runs `agd --version`.

```rust
#[derive(Parser)]
#[command(name = "agd", version)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,
    #[command(subcommand)]
    pub command: Commands,
}
```

A new integration test (location: `tests/version.rs`) spawns the built binary with `--version` and asserts the stdout contains `env!("CARGO_PKG_VERSION")`.

## 5. Feature: template path in warnings / errors

### 5.1 Current state

`src/templates.rs::load_templates()` returns `IndexMap<String, Template>` and does not expose the resolved `PathBuf` to its callers. The six warning/error sites in `src/dispatch/mod.rs` (lines 137, 145, 188, 197, 280, 289) print only the literal string `"cli-templates.toml"`. When a user sees:

```
warning: template 'agy' for agent 'antigravity-cli' not found in cli-templates.toml
warning: agent 'antigravity-cli' uses unverified template 'agy', skipping
```

they cannot tell *which* `cli-templates.toml` was loaded — the resolver checks the `AGD_TEMPLATES` env var, the executable directory, `CARGO_MANIFEST_DIR`, and up to four platform-specific fallback paths.

### 5.2 Change

1. `load_templates()` signature changes from `-> anyhow::Result<IndexMap<String, Template>>` to `-> anyhow::Result<(IndexMap<String, Template>, PathBuf)>`. The `PathBuf` is the absolute path returned by `resolve_templates_path()`.
2. `src/dispatch/mod.rs::cmd_dispatch()` is updated to destructure the tuple and thread `templates_path: &Path` through to `dispatch_single`, `dispatch_tiers`, and `dry_run`. Each of those three functions grows one parameter.

   **Caller fan-out:** every other caller of `load_templates()` must also destructure the new tuple. Confirmed sites include `cmd_detect` in `src/detect.rs`, `cmd_config_list` in `src/config_cmd.rs`, the `detect.rs` test helper, and unit tests inside `src/templates.rs`. Sites that do not need the path discard it with `let (templates, _) = load_templates()?;`. These are purely mechanical destructures — no behavioral change.
3. The six warning/error strings are rewritten as follows (using `templates_path.display()`):

   | Site | New format |
   |---|---|
   | `mod.rs:137` (dry_run, template not found) | `warning: template '{tmpl}' for agent '{agent}' not found in {path}` |
   | `mod.rs:145` (dry_run, unverified template) | `warning: agent '{agent}' uses unverified template '{tmpl}', skipping ({path})` |
   | `mod.rs:188` (dispatch_single, template not found) | `error: template '{tmpl}' for agent '{agent}' not found in {path}` |
   | `mod.rs:197` (dispatch_single, unverified template) | `error: agent '{agent}' uses unverified template '{tmpl}'; cannot dispatch ({path})` |
   | `mod.rs:280` (dispatch_tiers, template not found) | `warning: template '{tmpl}' for agent '{agent}' not found in {path}` |
   | `mod.rs:289` (dispatch_tiers, unverified template) | `warning: agent '{agent}' uses unverified template '{tmpl}', skipping ({path})` |

4. `src/detect.rs` does not emit template-resolution warnings, so §5's warning-message changes do not touch it. It does, however, fall under the mechanical destructure described in step 2 (its `cmd_detect` calls `load_templates()`), and its test fixture strings are still renamed per §3.

### 5.3 Tests

- One `tests/dispatch.rs` case per warning kind: build a fixture pointing `AGD_TEMPLATES` at a temp file with a missing template name, run `agd dispatch --dry-run`, assert stderr contains both the missing-template phrase and the absolute fixture path.
- Existing insta snapshots that capture these warnings will be regenerated.

## 6. Feature: `config/cli-templates.toml` cleanup

Land the uncommitted working-tree changes with two corrections:

| Change | Action |
|---|---|
| New `[agy]` template (prompt_flag `-p`, default version probe) | Keep |
| Seven explicit `version_flag = "--version"` rows on `[claude]`, `[gemini]`, `[codex]`, `[copilot]`, `[opencode]`, `[gemini-npx]`, `[agy]` | Keep the explicit declarations — documents intent even though it matches the default applied at load time |
| `[gemini-npx]` switched from `version_flag = ""` to `version_flag = "--version"` | Keep — enables version probing for the npx-wrapped binary |
| `[gemini-npx]` removed `--skip-trust` from `extra_args` | Keep |
| Misleading inline comment `# detect reports version as null` next to **all seven** `version_flag = "--version"` rows | **Fix** — replace each with `# explicit (matches default)` (or equivalent phrasing). All seven rows must be updated. |
| Comment block immediately below `[gemini-npx]` (`config/cli-templates.toml:119-120`) showing `--skip-trust` in an example invocation | **Fix** — drop the `--skip-trust` token from the example invocation to match the new `extra_args` |

## 7. Feature: README `## Usage`

The current `## Usage` placeholder (`...`) is replaced with the following sections, inserted between `## Installation` and `## License`. Final wording may be tightened during the writing-plans phase, but the section list and the configuration example are normative.

### 7.1 Section list

1. **Subcommands** — one-line description for each of `agd detect`, `agd init`, `agd config`, `agd dispatch`, plus `agd --version`.
2. **Configuration** — default file locations (`~/.config/agd.toml` and `<git-root>/.config/agd.toml`), and a complete worked TOML example (see §7.2).
3. **Common commands** — runnable examples for `agd config path`, `agd config edit`, `agd config show`, `agd dispatch -p "..." --tier primary`, `agd dispatch -p "..." --agent <id> --dry-run`.
4. **Environment variables** — `AGD_TEMPLATES` (operator-overridable) and `AGD_DEPTH` (internal — should not be set manually).

### 7.2 Configuration example (normative)

```toml
version = 1

[[tiers]]
id = "primary"

  [[tiers.agents]]
  id = "claude-claude"
  cli = "claude"
  model = "default"
  args = ["--dangerously-skip-permissions"]
  enabled = true
    [[tiers.agents.env]]
    type = "source"
    path = "~/.zshrc.d/cclaude.env"

  [[tiers.agents]]
  id = "antigravity-cli"
  cli = "agy"
  model = "default"
  args = ["--dangerously-skip-permissions"]
  enabled = true

  [[tiers.agents]]
  id = "claude-glm"
  cli = "claude"
  model = "default"
  args = ["--dangerously-skip-permissions"]
  enabled = true
    [[tiers.agents.env]]
    type = "source"
    path = "~/.zshrc.d/zclaude.env"
```

### 7.3 Install commands

`README.md` install commands are updated to set `APP_NAME=agd` and `REPO=superyngo/agd`. The Gist URL is unchanged.

## 8. Implementation strategy

Mechanical search-and-replace driven by the compiler:

1. Apply the §3 mapping in passes (Rust source → tests → config-resolved paths → workflows / scripts → README / installer values → snapshot regeneration).
2. After each pass: `cargo build` to surface symbol-level misses, then `cargo test` to surface string-level misses (env vars, paths, error messages).
3. Snapshot updates via `cargo insta review` are accepted in a single commit at the end.
4. The `config/cli-templates.toml` cleanup (§6) lands as its own commit; the README rewrite (§7) lands as its own commit; the rename (§3) lands in 2–3 commits split by surface (code, workflows/scripts, tests/snapshots).

No new modules, no abstractions, no refactors beyond what the rename and the `load_templates` signature change require.

## 9. Acceptance criteria

The release is shippable when:

1. `cargo build --release` succeeds.
2. `cargo test` passes; no test references `dispatch-agent`, `DISPATCH_AGENT_*`, or the old config-file names.
3. `rg -i 'dispatch[-_ ]?agent' -g '!CHANGELOG.md' -g '!docs/superpowers/specs/2026-05-18-*' -g '!docs/superpowers/plans/2026-05-18-*' -g '!docs/tmp' -g '!target'` returns zero matches.
4. The built binary responds to `agd --version` with the package version.
5. Manual smoke test passes: `agd detect`, `agd config path`, `agd dispatch --dry-run --tier primary -p "test"` — all output references `agd` (never `dispatch-agent`) and any template warning includes the absolute path to the loaded `cli-templates.toml`.
6. `README.md` renders with all four §7.1 sections present, install commands using `agd` / `superyngo/agd`, and the §7.2 TOML example verbatim.

## 10. Risks and accepted tradeoffs

| Risk | Severity | Mitigation / Acceptance |
|---|---|---|
| Existing local installs with `~/.config/dispatch-agent.toml` and `<cache>/dispatch-agent/` break | Low | Accepted. Project is early-stage; user explicitly declined migration. New install creates fresh `~/.config/agd.toml`. |
| Old release URLs (`releases/download/v1.1.2/dispatch-agent-*`) still exist post-repo-rename | Negligible | GitHub auto-creates permanent redirects on repo rename. Old release assets remain accessible. |
| Snapshot test churn obscures intentional changes | Low | Regenerated snapshots are diff-reviewed; insta presents each side-by-side before accepting. |
| Template-path threading (§5.2) ripples through three function signatures | Low | Compile errors surface every call site; no dynamic behavior to break. |
| Operator forgets `gh repo rename` step | Low | Listed explicitly as an operator action in this spec; will appear in the implementation plan checklist. |
