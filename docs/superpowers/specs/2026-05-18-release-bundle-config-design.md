# Design: Bundle config/ into Release Archives

## Problem

The current `release.yml` workflow:
- Packages Linux/macOS binaries into `.tar.gz` containing **only** the binary
- Uploads Windows `.exe` directly with no archive at all
- Does **not** include `config/cli-templates.toml` in any release artifact

Users who download a release must manually locate and place `config/cli-templates.toml` themselves, which is error-prone and unintuitive.

## Goal

Bundle `config/cli-templates.toml` (preserving the `config/` subdirectory) together with the binary in every release archive. Windows gets `.zip`; Linux/macOS keep `.tar.gz`.

## Breaking Change

**Windows artifact format changes from `.exe` to `.zip`.** Any automation script or CI pipeline that downloads Windows releases by filename pattern (e.g., `*-windows-*.exe`) must be updated to target `.zip` instead. This should be called out explicitly in the release notes for the first release using this new format.

## Archive Structure (all platforms)

```
<archive-root>/
├── dispatch-agent        (or dispatch-agent.exe on Windows)
└── config/
    └── cli-templates.toml
```

**No top-level directory.** Files are placed directly at the archive root (not inside a `dispatch-agent-vX.Y.Z/` subdirectory). This is an intentional choice for ease of use: users can extract to any target directory without needing to strip a prefix. The trade-off is that extracting into the current directory without a target path will place files inline — users should always specify a destination.

**Bundled config scope.** Only `config/cli-templates.toml` is explicitly copied into the archive. The implementation uses `mkdir -p staging/config && cp config/cli-templates.toml staging/config/` (not `cp -r config`) to avoid accidentally including future files added to `config/` that are not intended for distribution.

## Changes Required

### 1. Matrix — Windows `asset_name` cleanup

Remove the `.exe` suffix from `asset_name` for all Windows targets, since the artifact is now a `.zip`, not a bare `.exe`:

| Before | After |
|--------|-------|
| `dispatch-agent-windows-x86_64.exe` | `dispatch-agent-windows-x86_64` |
| `dispatch-agent-windows-i686.exe` | `dispatch-agent-windows-i686` |
| `dispatch-agent-windows-aarch64.exe` | `dispatch-agent-windows-aarch64` |

`artifact_name` (the actual compiled binary filename) remains `dispatch-agent.exe`.

### 2. Build Job — Linux/macOS: Modify "Create tarball" step

Replace the current single-directory `tar` invocation with a staging-directory approach that merges the binary and the explicit config file:

```yaml
- name: Create tarball (Linux and macOS)
  if: matrix.os != 'windows-latest'
  run: |
    mkdir -p staging/config
    cp target/${{ matrix.target }}/release/${{ matrix.artifact_name }} staging/
    chmod +x staging/${{ matrix.artifact_name }}
    cp config/cli-templates.toml staging/config/
    tar czf ${{ matrix.asset_name }}.tar.gz -C staging .
```

The checkout step already provides `config/cli-templates.toml` at the repo root. `chmod +x` is included for explicitness, even though `cp` preserves the execute bit from the `target/release/` binary.

### 3. Build Job — Windows: New "Create zip" step

Add a new step after the strip steps (Windows skips stripping anyway) to produce a `.zip`, using explicit file copy:

```yaml
- name: Create zip (Windows)
  if: matrix.os == 'windows-latest'
  shell: pwsh
  run: |
    New-Item -ItemType Directory -Force -Path staging\config
    Copy-Item "target\${{ matrix.target }}\release\${{ matrix.artifact_name }}" -Destination staging\
    Copy-Item "config\cli-templates.toml" -Destination staging\config\
    Compress-Archive -Path staging\* -DestinationPath "${{ matrix.asset_name }}.zip"
```

### 4. Build Job — Merge Upload Artifacts Steps

Both platforms now upload a single archive file. The two separate upload steps can be collapsed into one:

```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: ${{ matrix.asset_name }}
    path: ${{ matrix.asset_name }}.${{ matrix.os == 'windows-latest' && 'zip' || 'tar.gz' }}
```

The `&&`/`||` ternary pattern is valid GitHub Actions expression syntax.

### 5. Release Job — Simplify "Prepare release files" step

All artifacts are now archives. Remove the entire `for dir` loop that handled the Windows `.exe` special case. Only archive files need to be collected:

```yaml
- name: Prepare release files
  run: |
    mkdir -p release_files
    find artifacts -type f \( -name "*.tar.gz" -o -name "*.zip" \) -exec cp {} release_files/ \;
    echo "Files in release_files:"
    ls -la release_files/
```

### 6. Release Job — Update "Display structure" debug step

The current step searches for `*.exe` in addition to archives. Since no bare `.exe` files will exist post-change, update to only search for archives:

```yaml
- name: Display structure
  run: |
    echo "Current directory structure:"
    ls -la
    echo "Artifacts directory:"
    ls -la artifacts/
    echo "Looking for artifacts:"
    find artifacts -type f \( -name "*.tar.gz" -o -name "*.zip" \)
```

### 7. SHA256SUMS Step — No Changes Required

The `sha256sum *` command operates on all files present in `release_files/`. Before this change, that included `.tar.gz` files and bare `.exe` files. After this change, it will include `.tar.gz` and `.zip` files. The step logic is unchanged and checksums will correctly cover all artifacts.

## Non-Changes

- Archive naming for Linux/macOS stays unchanged (e.g., `dispatch-agent-linux-x86_64.tar.gz`)
- All build, cross-compilation, and strip steps remain unchanged
- The `Generate checksums` step (`sha256sum *`) requires no modification — see Section 7

## Verification

Before tagging a production release, trigger the workflow via `workflow_dispatch` with a test version string (e.g., `v0.0.0-test`). Verify:
1. All 13 build jobs complete
2. Each Linux/macOS `.tar.gz` extracts to `./dispatch-agent` + `./config/cli-templates.toml`
3. Each Windows `.zip` extracts to `./dispatch-agent.exe` + `./config/cli-templates.toml`
4. `SHA256SUMS` lists all 13 archives

## Success Criteria

- Every release artifact (`.tar.gz` and `.zip`) extracts to a directory containing both the binary and `config/cli-templates.toml`
- SHA256SUMS covers all archives
- No bare `.exe` files appear in the release
- Release notes for the first release using this format include the breaking change notice for Windows artifact format
