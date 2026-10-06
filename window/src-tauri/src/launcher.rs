//! 啟動器：照表單組出 claude 參數，開一個新的終端機視窗執行。參數比照 tsunu-alive-lite。
//! line／cottage 的 MCP 設定檔規則也比照它：line 檔案不在就退回全域設定，cottage 檔案不在就整組跳過。
//!
//! 終端機與 mod 位置可以在 `<app config dir>/config.json` 覆寫：
//! `{ "terminal": ["wezterm", "start", "--cwd", "{cwd}", "--"], "modDir": "/path/to/mod" }`
//! `terminal` 是開終端機的指令，`{cwd}` 會換成工作目錄，後面接上 `claude` 和它的參數。
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

const BUNDLED_MOD: &str = "mod";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum Resume {
    New,
    Continue,
    Id(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchOptions {
    pub cwd: String,
    pub resume: Resume,
    pub permission_mode: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<String>,
    pub discord: bool,
    pub line: bool,
    pub cottage: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherConfig {
    terminal: Option<Vec<String>>,
    mod_dir: Option<String>,
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

fn mcp_config(file_name: &str) -> PathBuf {
    home_dir().join(".claude").join(file_name)
}

fn push(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.into());
    args.push(value.into());
}

/// `exists` 判斷 MCP 設定檔在不在，測試時可以換掉。
pub fn claude_args(o: &LaunchOptions, mod_dir: &str, exists: &dyn Fn(&Path) -> bool) -> Vec<String> {
    let mut args = Vec::new();
    push(&mut args, "--plugin-dir", mod_dir);
    if let Some(v) = &o.thinking { push(&mut args, "--thinking", v) }
    if let Some(v) = &o.permission_mode { push(&mut args, "--permission-mode", v) }
    if let Some(v) = &o.effort { push(&mut args, "--effort", v) }
    if o.discord { push(&mut args, "--channels", "plugin:discord@claude-plugins-official") }
    if o.line {
        let config = mcp_config("line-mcp.json");
        if exists(&config) { push(&mut args, "--mcp-config", &config.to_string_lossy()) }
        push(&mut args, "--dangerously-load-development-channels", "server:line");
    }
    if o.cottage {
        let config = mcp_config("cottage-mcp.json");
        if exists(&config) {
            push(&mut args, "--mcp-config", &config.to_string_lossy());
            push(&mut args, "--dangerously-load-development-channels", "server:cottage");
        }
    }
    match &o.resume {
        Resume::New => {}
        Resume::Continue => args.push("--continue".into()),
        Resume::Id(id) => push(&mut args, "--resume", id),
    }
    args
}

/// 包成單引號字串給 sh 用：內容裡的 `'` 換成 `'\''`。
fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// 開終端機的完整指令（第一個元素是執行檔）。自訂的 `terminal` 優先，否則依作業系統挑預設。
pub fn terminal_command(custom: Option<&[String]>, cwd: &str, claude: &[String]) -> Vec<String> {
    if let Some(template) = custom {
        let mut command: Vec<String> = template.iter().map(|part| part.replace("{cwd}", cwd)).collect();
        command.extend(claude.iter().cloned());
        return command;
    }
    if cfg!(target_os = "windows") {
        let mut command = vec!["wt.exe".to_string(), "-d".into(), cwd.into()];
        command.extend(claude.iter().cloned());
        command
    } else if cfg!(target_os = "macos") {
        let line = std::iter::once(format!("cd {} &&", shell_quote(cwd)))
            .chain(claude.iter().map(|part| shell_quote(part)))
            .collect::<Vec<_>>()
            .join(" ");
        let script = format!(
            "tell application \"Terminal\" to do script \"{}\"",
            line.replace('\\', "\\\\").replace('"', "\\\"")
        );
        vec!["osascript".into(), "-e".into(), script, "-e".into(), "tell application \"Terminal\" to activate".into()]
    } else {
        let mut command = vec!["x-terminal-emulator".to_string(), "-e".into()];
        command.extend(claude.iter().cloned());
        command
    }
}

fn read_config(app: &AppHandle) -> LauncherConfig {
    app.path()
        .app_config_dir()
        .ok()
        .and_then(|dir| std::fs::read_to_string(dir.join(CONFIG_FILE)).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Windows 的 resource 路徑可能帶 `\\?\` 前綴，claude 不認，拿掉。
fn plain_path(path: PathBuf) -> String {
    let text = path.to_string_lossy().into_owned();
    text.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(text)
}

fn mod_dir(app: &AppHandle, config: &LauncherConfig) -> Result<String, String> {
    if let Some(dir) = &config.mod_dir {
        return Ok(dir.clone());
    }
    app.path()
        .resolve(BUNDLED_MOD, BaseDirectory::Resource)
        .map(plain_path)
        .map_err(|e| format!("找不到內附的 mod：{e}"))
}

#[tauri::command]
pub fn launch_session(app: AppHandle, options: LaunchOptions) -> Result<(), String> {
    let config = read_config(&app);
    let mod_dir = mod_dir(&app, &config)?;
    let mut claude = vec!["claude".to_string()];
    claude.extend(claude_args(&options, &mod_dir, &|path| path.exists()));
    let command = terminal_command(config.terminal.as_deref(), &options.cwd, &claude);
    std::process::Command::new(&command[0])
        .args(&command[1..])
        .current_dir(&options.cwd)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("開不了終端機（{}）：{e}", command[0]))
}

#[tauri::command]
pub fn default_cwd() -> String {
    home_dir().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOD: &str = "/opt/tsunu-pet/mod";

    fn options() -> LaunchOptions {
        LaunchOptions {
            cwd: "/home/me/project".into(),
            resume: Resume::New,
            permission_mode: None,
            effort: None,
            thinking: None,
            discord: false,
            line: false,
            cottage: false,
        }
    }

    #[test]
    fn plain_launch_only_loads_mod() {
        assert_eq!(claude_args(&options(), MOD, &|_| true), vec!["--plugin-dir".to_string(), MOD.to_string()]);
    }

    #[test]
    fn resume_with_id_and_flags() {
        let mut o = options();
        o.resume = Resume::Id("abc".into());
        o.permission_mode = Some("acceptEdits".into());
        o.discord = true;
        let args = claude_args(&o, MOD, &|_| true);
        assert!(args.windows(2).any(|w| w == ["--resume", "abc"]));
        assert!(args.windows(2).any(|w| w == ["--permission-mode", "acceptEdits"]));
        assert!(args.windows(2).any(|w| w == ["--channels", "plugin:discord@claude-plugins-official"]));
    }

    #[test]
    fn cottage_skipped_when_config_missing() {
        let mut o = options();
        o.cottage = true;
        let args = claude_args(&o, MOD, &|_| false);
        assert!(!args.iter().any(|a| a == "server:cottage"));
    }

    #[test]
    fn line_falls_back_to_global_config_when_file_missing() {
        let mut o = options();
        o.line = true;
        let args = claude_args(&o, MOD, &|_| false);
        assert!(!args.iter().any(|a| a == "--mcp-config"));
        assert!(args.windows(2).any(|w| w == ["--dangerously-load-development-channels", "server:line"]));
    }

    #[test]
    fn custom_terminal_fills_cwd_and_appends_claude() {
        let template = vec!["wezterm".to_string(), "start".into(), "--cwd".into(), "{cwd}".into(), "--".into()];
        let claude = vec!["claude".to_string(), "--continue".into()];
        assert_eq!(
            terminal_command(Some(&template), "/work", &claude),
            vec!["wezterm", "start", "--cwd", "/work", "--", "claude", "--continue"]
        );
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }
}
