# cli-templates.toml 搜尋路徑擴充

Date: 2026-05-18
Status: Approved (design)
Scope: `src/templates.rs::resolve_templates_path()`

## 目標

在 `cli-templates.toml` 的解析鏈尾端追加平台特定 fallback 路徑，使從常見安裝位置（`wenget`、`/opt`、`/usr/local/bin/config`、Windows Program Files 等）執行的 `dispatch-agent` 能自動找到模板檔。

## 現況

`resolve_templates_path()` 目前依序搜尋：

1. `DISPATCH_AGENT_TEMPLATES` 環境變數
2. `<exe_dir>/config/cli-templates.toml`
3. `<CARGO_MANIFEST_DIR>/config/cli-templates.toml`（dev fallback）

找不到時回傳錯誤。

## 變更後搜尋順序

既有 1–3 保持不變；於其後依平台追加：

### Unix（含 linux 與 macOS）

4. `$HOME/.wenget/apps/dispatch-agent/config/cli-templates.toml`
5. `$HOME/.local/bin/config/cli-templates.toml`
6. `/opt/wenget/apps/dispatch-agent/config/cli-templates.toml`
7. `/usr/local/bin/config/cli-templates.toml`

### Windows

4. `%USERPROFILE%\.wenget\apps\dispatch-agent\config\cli-templates.toml`
5. `%LOCALAPPDATA%\Programs\dispatch-agent\config\cli-templates.toml`
6. `%ProgramW6432%\wenget\app\dispatch-agent\config\cli-templates.toml`
7. `%ProgramFiles%\gpinstall\config\cli-templates.toml`

## 行為規則

- 每條候選路徑：若依賴的環境變數未定義或為空字串，**跳過該條**，繼續下一條，不視為錯誤。
- 候選路徑存在時取第一個命中者；不存在則往下一條走。
- 全部候選都沒命中才回傳錯誤；錯誤訊息列出所有實際被檢查過的路徑（環境變數已被解析後的最終形式），環境變數未定義者標註 `<unset>` 或省略。

## 實作

### 修改

- `src/templates.rs`
  - 為 `resolve_templates_path()` 新增 helper：
    - `fn candidate_from_env(var: &str, suffix: &[&str]) -> Option<PathBuf>`：讀取環境變數並 join `suffix`；變數未定義/為空回 `None`。
    - `fn candidate_absolute(parts: &[&str]) -> PathBuf`：純絕對路徑。
  - 用 `#[cfg(unix)]` / `#[cfg(windows)]` 把平台特定候選清單放進私有函式 `platform_fallback_candidates() -> Vec<PathBuf>`，回傳實際可檢查的候選路徑（已解析環境變數）。
  - 在現有 dev fallback 失敗後，遍歷 `platform_fallback_candidates()`，取首個 `exists()`。
  - 更新最終錯誤訊息，包含所有實際檢查過的路徑列表。

### 不變動

- `DISPATCH_AGENT_TEMPLATES` 行為。
- `exe_dir` 與 `CARGO_MANIFEST_DIR` 路徑與順序。
- `src/detect.rs` 與 `src/config_cmd.rs`（後者解析的是 `dispatch-agent.toml`，與本變更無關）。

## 測試

於 `src/templates.rs` `#[cfg(test)]` 區塊新增：

1. **Unix fallback 命中**（`#[cfg(unix)]`）：
   - 在 tempdir 建構 `$HOME/.wenget/apps/dispatch-agent/config/cli-templates.toml`。
   - 用 `EnvGuard` 設 `HOME` 指向 tempdir、`unset` `DISPATCH_AGENT_TEMPLATES`，並讓 `exe_dir/config/...` 與 `CARGO_MANIFEST_DIR/config/...` 均不命中（後者已存在於專案中，需特別處理 — 可改為驗證解析函式直接回傳的路徑，而非透過 `load_templates()` 觸發實際讀檔）。
   - 驗證 `resolve_templates_path()` 回傳該 fallback 路徑。

2. **未定義環境變數跳過**：
   - 模擬 `HOME`（或 Windows 上 `LOCALAPPDATA`）unset，驗證 `platform_fallback_candidates()` 不包含該條，且不 panic。

3. **完全沒命中時錯誤訊息**：
   - 所有候選都不存在；斷言錯誤訊息包含至少一條平台特定路徑字串。

> 註：因 `CARGO_MANIFEST_DIR/config/cli-templates.toml` 在開發環境中實際存在，會干擾 fallback 測試。實作時將「逐一檢查候選」抽成可測試函式 `find_first_existing(candidates: &[PathBuf]) -> Option<PathBuf>`，平台 fallback 測試直接驗證該函式與 `platform_fallback_candidates()`，繞過 dev fallback 干擾。

## 文件

- `CHANGELOG.md` 追加 Unreleased 條目：說明新增的平台 fallback 搜尋路徑。
- README.md 目前未列舉搜尋路徑，本次不新增段落（避免擴張範疇）；若後續需要，另開 issue。

## 風險與權衡

- 新增多條路徑增加 stat 次數，但僅在前 3 條都未命中時觸發，影響可忽略。
- `%ProgramFiles%` 在 32-bit 行程中會解析為 `Program Files (x86)`，與 `%ProgramW6432%` 不同；兩者都列出符合使用者要求。
- `/usr/local/bin/config/` 並非標準 FHS 配置位置，但依使用者既有部署慣例保留。
