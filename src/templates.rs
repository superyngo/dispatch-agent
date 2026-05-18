use anyhow::{anyhow, Context};
use indexmap::IndexMap;
use std::{env, fs};

use crate::types::Template;

#[allow(dead_code)]
pub fn load_templates() -> anyhow::Result<IndexMap<String, Template>> {
    let path = resolve_templates_path()?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("reading templates file {}", path.display()))?;
    let content = raw.strip_prefix('\u{FEFF}').unwrap_or(&raw);
    let mut map: IndexMap<String, Template> = toml::from_str(content)
        .with_context(|| format!("parsing templates file {}", path.display()))?;
    for tmpl in map.values_mut() {
        if tmpl.version_flag.is_none() {
            tmpl.version_flag = Some("--version".to_string());
        }
    }
    Ok(map)
}

#[allow(dead_code)]
#[cfg(windows)]
fn platform_fallback_candidates() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = candidate_from_env(
        "USERPROFILE",
        &[
            ".wenget",
            "apps",
            "dispatch-agent",
            "config",
            "cli-templates.toml",
        ],
    ) {
        out.push(p);
    }
    if let Some(p) = candidate_from_env(
        "LOCALAPPDATA",
        &["Programs", "dispatch-agent", "config", "cli-templates.toml"],
    ) {
        out.push(p);
    }
    if let Some(p) = candidate_from_env(
        "ProgramW6432",
        &[
            "wenget",
            "app",
            "dispatch-agent",
            "config",
            "cli-templates.toml",
        ],
    ) {
        out.push(p);
    }
    if let Some(p) = candidate_from_env(
        "ProgramFiles",
        &["gpinstall", "config", "cli-templates.toml"],
    ) {
        out.push(p);
    }
    out
}

#[allow(dead_code)]
#[cfg(unix)]
fn platform_fallback_candidates() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = candidate_from_env(
        "HOME",
        &[
            ".wenget",
            "apps",
            "dispatch-agent",
            "config",
            "cli-templates.toml",
        ],
    ) {
        out.push(p);
    }
    if let Some(p) = candidate_from_env("HOME", &[".local", "bin", "config", "cli-templates.toml"])
    {
        out.push(p);
    }
    out.push(std::path::PathBuf::from(
        "/opt/wenget/apps/dispatch-agent/config/cli-templates.toml",
    ));
    out.push(std::path::PathBuf::from(
        "/usr/local/bin/config/cli-templates.toml",
    ));
    out
}

#[allow(dead_code)]
#[cfg(not(any(unix, windows)))]
fn platform_fallback_candidates() -> Vec<std::path::PathBuf> {
    Vec::new()
}

#[allow(dead_code)]
fn candidate_from_env(var: &str, suffix: &[&str]) -> Option<std::path::PathBuf> {
    let base = env::var(var).ok()?;
    if base.is_empty() {
        return None;
    }
    let mut p = std::path::PathBuf::from(base);
    for s in suffix {
        p.push(s);
    }
    Some(p)
}

#[allow(dead_code)]
fn find_first_existing(candidates: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
    candidates.iter().find(|p| p.exists()).cloned()
}

#[allow(dead_code)]
fn resolve_templates_path() -> anyhow::Result<std::path::PathBuf> {
    let mut checked: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(p) = env::var("DISPATCH_AGENT_TEMPLATES") {
        return Ok(std::path::PathBuf::from(p));
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let p = exe_dir.join("config/cli-templates.toml");
            if p.exists() {
                return Ok(p);
            }
            checked.push(p);
        }
    }

    let dev_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/cli-templates.toml");
    if dev_path.exists() {
        return Ok(dev_path);
    }
    checked.push(dev_path);

    let platform = platform_fallback_candidates();
    if let Some(hit) = find_first_existing(&platform) {
        return Ok(hit);
    }
    checked.extend(platform);

    Err(anyhow!("{}", format_not_found_error(&checked)))
}

#[allow(dead_code)]
fn format_not_found_error(checked: &[std::path::PathBuf]) -> String {
    let mut s = String::from(
        "cli-templates.toml not found. Set DISPATCH_AGENT_TEMPLATES to override. Searched:\n",
    );
    for p in checked {
        s.push_str("  ");
        s.push_str(&p.display().to_string());
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: String,
        old: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &str, value: &str) -> Self {
            let old = env::var(key).ok();
            env::set_var(key, value);
            Self {
                key: key.to_string(),
                old,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(v) => env::set_var(&self.key, v),
                None => env::remove_var(&self.key),
            }
        }
    }

    fn write_toml(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
        let path = dir.join("cli-templates.toml");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn load_templates_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = write_toml(dir.path(), "[test-cli]\nprompt_flag = \"-p\"\n");
        let _guard = EnvGuard::set("DISPATCH_AGENT_TEMPLATES", path.to_str().unwrap());
        let map = load_templates().unwrap();
        assert!(map.contains_key("test-cli"));
        assert_eq!(map["test-cli"].prompt_flag.as_deref(), Some("-p"));
    }

    #[test]
    fn version_flag_default_applied() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = write_toml(dir.path(), "[cli]\nprompt_flag = \"-p\"\n");
        let _guard = EnvGuard::set("DISPATCH_AGENT_TEMPLATES", path.to_str().unwrap());
        let map = load_templates().unwrap();
        assert_eq!(map["cli"].version_flag.as_deref(), Some("--version"));
    }

    #[test]
    fn version_flag_explicit_not_overwritten() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = write_toml(
            dir.path(),
            "[cli]\nprompt_flag = \"-p\"\nversion_flag = \"\"\n",
        );
        let _guard = EnvGuard::set("DISPATCH_AGENT_TEMPLATES", path.to_str().unwrap());
        let map = load_templates().unwrap();
        assert_eq!(map["cli"].version_flag.as_deref(), Some(""));
    }

    #[test]
    fn bom_stripped() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let content = "\u{FEFF}[cli]\nprompt_flag = \"-p\"\n";
        let path = write_toml(dir.path(), content);
        let _guard = EnvGuard::set("DISPATCH_AGENT_TEMPLATES", path.to_str().unwrap());
        let map = load_templates().unwrap();
        assert!(map.contains_key("cli"));
    }

    #[test]
    fn insertion_order_preserved() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let toml =
            "[b]\nprompt_flag = \"-p\"\n[a]\nprompt_flag = \"-p\"\n[c]\nprompt_flag = \"-p\"\n";
        let path = write_toml(dir.path(), toml);
        let _guard = EnvGuard::set("DISPATCH_AGENT_TEMPLATES", path.to_str().unwrap());
        let map = load_templates().unwrap();
        let keys: Vec<&String> = map.keys().collect();
        assert_eq!(keys, vec!["b", "a", "c"]);
    }

    #[cfg(unix)]
    #[test]
    fn resolve_templates_path_uses_unix_fallback() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        // Build $HOME/.wenget/apps/dispatch-agent/config/cli-templates.toml
        let nested = dir.path().join(".wenget/apps/dispatch-agent/config");
        std::fs::create_dir_all(&nested).unwrap();
        let target = nested.join("cli-templates.toml");
        std::fs::write(&target, "[cli]\nprompt_flag = \"-p\"\n").unwrap();

        let _h = EnvGuard::set("HOME", dir.path().to_str().unwrap());
        // Ensure earlier links in the chain miss:
        env::remove_var("DISPATCH_AGENT_TEMPLATES");
        // exe_dir/config/cli-templates.toml normally won't exist for cargo-test binaries;
        // CARGO_MANIFEST_DIR/config/cli-templates.toml DOES exist in this repo, so
        // resolve_templates_path() will short-circuit there. We therefore validate the
        // fallback by checking find_first_existing(platform_fallback_candidates()) directly.
        let got = super::find_first_existing(&super::platform_fallback_candidates())
            .expect("expected fallback hit");
        assert_eq!(got, target);
    }

    #[test]
    fn resolve_templates_path_error_lists_paths() {
        let _lock = ENV_MUTEX.lock().unwrap();
        env::remove_var("DISPATCH_AGENT_TEMPLATES");
        // We cannot force exe_dir and CARGO_MANIFEST_DIR misses without filesystem
        // gymnastics; instead, exercise the error formatter directly.
        let msg = super::format_not_found_error(&[
            std::path::PathBuf::from("/tmp/a/cli-templates.toml"),
            std::path::PathBuf::from("/tmp/b/cli-templates.toml"),
        ]);
        assert!(msg.contains("/tmp/a/cli-templates.toml"));
        assert!(msg.contains("/tmp/b/cli-templates.toml"));
        assert!(msg.contains("DISPATCH_AGENT_TEMPLATES"));
    }

    #[cfg(windows)]
    #[test]
    fn platform_fallback_candidates_windows_uses_env_vars() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let _u = EnvGuard::set("USERPROFILE", dir.path().to_str().unwrap());
        let _l = EnvGuard::set("LOCALAPPDATA", dir.path().to_str().unwrap());
        let _w = EnvGuard::set("ProgramW6432", dir.path().to_str().unwrap());
        let _p = EnvGuard::set("ProgramFiles", dir.path().to_str().unwrap());

        let candidates = super::platform_fallback_candidates();
        let base = dir.path();

        let exp_userprofile = base
            .join(".wenget")
            .join("apps")
            .join("dispatch-agent")
            .join("config")
            .join("cli-templates.toml");
        let exp_localappdata = base
            .join("Programs")
            .join("dispatch-agent")
            .join("config")
            .join("cli-templates.toml");
        let exp_progw6432 = base
            .join("wenget")
            .join("app")
            .join("dispatch-agent")
            .join("config")
            .join("cli-templates.toml");
        let exp_progfiles = base
            .join("gpinstall")
            .join("config")
            .join("cli-templates.toml");

        assert!(candidates.contains(&exp_userprofile));
        assert!(candidates.contains(&exp_localappdata));
        assert!(candidates.contains(&exp_progw6432));
        assert!(candidates.contains(&exp_progfiles));

        let pos =
            |needle: &std::path::PathBuf| candidates.iter().position(|c| c == needle).unwrap();
        assert!(pos(&exp_userprofile) < pos(&exp_localappdata));
        assert!(pos(&exp_localappdata) < pos(&exp_progw6432));
        assert!(pos(&exp_progw6432) < pos(&exp_progfiles));
    }

    #[cfg(windows)]
    #[test]
    fn platform_fallback_candidates_windows_skips_unset_vars() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _u = EnvGuard::set("USERPROFILE", "");
        let _l = EnvGuard::set("LOCALAPPDATA", "");
        let _w = EnvGuard::set("ProgramW6432", "");
        let _p = EnvGuard::set("ProgramFiles", "");
        let candidates = super::platform_fallback_candidates();
        assert!(
            candidates.is_empty(),
            "expected empty, got {:?}",
            candidates
        );
    }

    #[cfg(unix)]
    #[test]
    fn platform_fallback_candidates_unix_uses_home_and_absolutes() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let _g = EnvGuard::set("HOME", dir.path().to_str().unwrap());
        let candidates = super::platform_fallback_candidates();

        let home = dir.path();
        let expected_home_wenget =
            home.join(".wenget/apps/dispatch-agent/config/cli-templates.toml");
        let expected_home_local = home.join(".local/bin/config/cli-templates.toml");
        let expected_opt =
            std::path::PathBuf::from("/opt/wenget/apps/dispatch-agent/config/cli-templates.toml");
        let expected_usr = std::path::PathBuf::from("/usr/local/bin/config/cli-templates.toml");

        assert!(
            candidates.contains(&expected_home_wenget),
            "missing {} in {:?}",
            expected_home_wenget.display(),
            candidates
        );
        assert!(candidates.contains(&expected_home_local));
        assert!(candidates.contains(&expected_opt));
        assert!(candidates.contains(&expected_usr));

        // Order: HOME entries come before absolute /opt and /usr/local entries
        let pos =
            |needle: &std::path::PathBuf| candidates.iter().position(|c| c == needle).unwrap();
        assert!(pos(&expected_home_wenget) < pos(&expected_opt));
        assert!(pos(&expected_home_local) < pos(&expected_opt));
        assert!(pos(&expected_opt) < pos(&expected_usr));
    }

    #[cfg(unix)]
    #[test]
    fn platform_fallback_candidates_unix_skips_when_home_unset() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _g = EnvGuard::set("HOME", "");
        let candidates = super::platform_fallback_candidates();
        // Absolute paths still present
        assert!(candidates.iter().any(|c| c.starts_with("/opt/wenget")));
        assert!(candidates.iter().any(|c| c.starts_with("/usr/local/bin")));
        // No path should contain ".wenget/apps/dispatch-agent" rooted in empty/HOME
        assert!(
            !candidates
                .iter()
                .any(|c| c.to_string_lossy().starts_with(".wenget")),
            "candidates leaked relative HOME path: {:?}",
            candidates
        );
    }

    #[test]
    fn candidate_from_env_returns_none_when_unset() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // Use a name that is extremely unlikely to be set
        env::remove_var("DA_TEST_UNSET_VAR_XYZ");
        let result = super::candidate_from_env("DA_TEST_UNSET_VAR_XYZ", &["sub", "file.toml"]);
        assert!(result.is_none());
    }

    #[test]
    fn candidate_from_env_returns_none_when_empty() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _g = EnvGuard::set("DA_TEST_EMPTY_VAR_XYZ", "");
        let result = super::candidate_from_env("DA_TEST_EMPTY_VAR_XYZ", &["x"]);
        assert!(result.is_none());
    }

    #[test]
    fn candidate_from_env_joins_suffix() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let _g = EnvGuard::set("DA_TEST_BASE_XYZ", dir.path().to_str().unwrap());
        let result = super::candidate_from_env("DA_TEST_BASE_XYZ", &["a", "b.toml"]).unwrap();
        assert_eq!(result, dir.path().join("a").join("b.toml"));
    }

    #[test]
    fn find_first_existing_returns_first_hit() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.toml");
        let b = dir.path().join("b.toml");
        std::fs::File::create(&a).unwrap();
        std::fs::File::create(&b).unwrap();
        let result = super::find_first_existing(&[a.clone(), b.clone()]);
        assert_eq!(result, Some(a), "should return first existing in order");
    }

    #[test]
    fn find_first_existing_skips_missing_then_returns_existing() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.toml");
        let b = dir.path().join("b.toml");
        std::fs::File::create(&b).unwrap();
        let result = super::find_first_existing(&[a, b.clone()]);
        assert_eq!(result, Some(b));
    }

    #[test]
    fn find_first_existing_returns_none_when_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.toml");
        let b = dir.path().join("b.toml");
        let result = super::find_first_existing(&[a, b]);
        assert_eq!(result, None);
    }

    #[test]
    fn missing_file_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _guard = EnvGuard::set(
            "DISPATCH_AGENT_TEMPLATES",
            "/nonexistent/path/cli-templates.toml",
        );
        let result = load_templates();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("reading templates file"),
            "expected 'reading templates file' in error, got: {msg}"
        );
    }
}
