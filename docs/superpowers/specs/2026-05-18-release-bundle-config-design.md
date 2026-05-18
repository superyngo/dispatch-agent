# Design: Bundle config/ into Release Archives

## Problem

The current `release.yml` workflow:
- Packages Linux/macOS binaries into `.tar.gz` containing **only** the binary
- Uploads Windows `.exe` directly with no archive at all
- Does **not** include `config/cli-templates.toml` in any release artifact

Users who download a release must manually locate and place `config/cli-templates.toml` themselves, which is error-prone and unintuitive.

## Goal

Bundle `config/cli-templates.toml` (preserving the `config/` subdirectory) together with the binary in every release archive. Windows gets `.zip`; Linux/macOS keep `.tar.gz`.

## Archive Structure (all platforms)

```
<archive-root>/
├── dispatch-agent        (or dispatch-agent.exe on Windows)
└── config/
    └── cli-templates.toml
```

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

Replace the current single-directory `tar` invocation with a staging-directory approach that merges the binary and the `config/` folder:

```yaml
- name: Create tarball (Linux and macOS)
  if: matrix.os != 'windows-latest'
  run: |
    mkdir -p staging
    cp target/${{ matrix.target }}/release/${{ matrix.artifact_name }} staging/
    cp -r config staging/
    tar czf ${{ matrix.asset_name }}.tar.gz -C staging .
```

The checkout step already provides the `config/` folder at the repo root.

### 3. Build Job — Windows: New "Create zip" step

Add a new step after the strip steps (Windows skips stripping anyway) to produce a `.zip`:

```yaml
- name: Create zip (Windows)
  if: matrix.os == 'windows-latest'
  shell: pwsh
  run: |
    New-Item -ItemType Directory -Force -Path staging
    Copy-Item "target\${{ matrix.target }}\release\${{ matrix.artifact_name }}" -Destination staging\
    Copy-Item -Recurse config -Destination staging\
    Compress-Archive -Path staging\* -DestinationPath "${{ matrix.asset_name }}.zip"
```

### 4. Build Job — Windows: Update "Upload artifacts" step

Change the upload path from the raw `.exe` to the `.zip`:

```yaml
- name: Upload artifacts (Windows)
  if: matrix.os == 'windows-latest'
  uses: actions/upload-artifact@v4
  with:
    name: ${{ matrix.asset_name }}
    path: ${{ matrix.asset_name }}.zip
```

### 5. Release Job — Simplify "Prepare release files" step

All artifacts are now archives. Remove the Windows `.exe` special-case logic:

```yaml
- name: Prepare release files
  run: |
    mkdir -p release_files
    find artifacts -type f \( -name "*.tar.gz" -o -name "*.zip" \) -exec cp {} release_files/ \;
    echo "Files in release_files:"
    ls -la release_files/
```

## Non-Changes

- Archive naming for Linux/macOS stays unchanged (e.g., `dispatch-agent-linux-x86_64.tar.gz`)
- All build, cross-compilation, strip, and checksum steps remain unchanged
- The `Display structure` debug step in the release job remains unchanged

## Success Criteria

- Every release artifact (`.tar.gz` and `.zip`) extracts to a directory containing both the binary and `config/cli-templates.toml`
- SHA256SUMS covers all archives
- No bare `.exe` files appear in the release
