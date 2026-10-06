# Tsunu Pet

[中文](#中文) · [English](#english)

---

## 中文

讓 Claude Code 的狀態「長出臉」：

- **終端機裡的立繪**：一個 Claude Code mod，在側邊欄顯示角色立繪，跟著 session 的狀態換表情（待機、思考中、工作中、等你回覆、出錯了、完成了）。
- **桌面總機小人**：一個透明、置頂的小視窗，桌面上只有一隻 Q 版角色，替**所有** session（前景、背景都算）當總機，平常演出最需要你關注的那一個。點一下展開清單，看每個 session 現在在做什麼；也能當啟動器，帶參數開新的 session。

預設角色是楊竣宇（阿宇），換成你自己喜歡的角色見[換成自己的角色](#換成自己的角色)。

### 需求

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) v2.1.287 以上（需要 mods 功能）
- 桌面小視窗：到 [Releases](../../releases) 下載安裝檔（Windows `.exe`／`.msi`、macOS `.dmg`、Linux `.AppImage`／`.deb`／`.rpm`）

### 使用方式

**1. 載入 mod**（擇一）

- 從桌面小視窗的啟動器開 session：安裝檔內附 mod，會自動帶上。
- 手動指定：`claude --plugin-dir <這個 repo>/mod`
- 每個 session 都載入：把 `<這個 repo>/mod` 加進環境變數 `CLAUDE_CODE_PLUGIN_DIRS`。

**2. 打開桌面小視窗**

小視窗在 `127.0.0.1:47321` 接收 mod 回報的狀態。沒有開小視窗時，mod 照常運作，只是不會回報。

- **拖曳**角色可以移動位置；**點一下**展開 session 清單與啟動器。
- 狀態優先順序：等你回覆 > 出錯 > 工作中 > 思考中 > 完成 > 待機，同級的話顯示最近更新的那個。

**3. 啟動器**

可以設定工作目錄、接續 session（最近一次或指定 ID）、權限模式、effort、thinking、Discord／LINE／小屋 channel。預設開的終端機：

| 系統 | 終端機 |
|---|---|
| Windows | Windows Terminal（`wt.exe`） |
| macOS | Terminal.app |
| Linux | `x-terminal-emulator` |

想換別的終端機或指定 mod 位置，在設定資料夾放一個 `config.json`：

- Windows：`%APPDATA%\com.atone.tsunu-pet\config.json`
- macOS：`~/Library/Application Support/com.atone.tsunu-pet/config.json`
- Linux：`~/.config/com.atone.tsunu-pet/config.json`

```json
{
  "terminal": ["wezterm", "start", "--cwd", "{cwd}", "--"],
  "modDir": "/path/to/tsunu-pet/mod"
}
```

`terminal` 是開終端機的指令，`{cwd}` 會換成工作目錄，後面接上 `claude` 和它的參數。

### 終端機裡的立繪：PNG 還是字元畫

- **kitty、Ghostty**：顯示 PNG 立繪（kitty 圖形協定）。
- **其他終端機**：改畫 CGA 四色風格的半格字元頭像，像早期電腦的畫面。
- **Windows**：一般終端機都會畫字元版。想看 PNG，需要支援 kitty Unicode 佔位字元的終端機，並設定環境變數 `CLAUDE_CODE_FORCE_TERMINAL_IMAGES=1`、`TSUNU_KITTY_PLACEHOLDERS=1`（例如自行編譯含 [wezterm#7924](https://github.com/wez/wezterm/pull/7924) 的 WezTerm，搭配 1.22 以上的 ConPTY）。
- 側邊欄要終端機至少 144 欄寬才會自動打開。

### 換成自己的角色

| 要換的東西 | 位置 | 說明 |
|---|---|---|
| 名字 | `mod/hooks/register.js` 的 `CHARACTER`；`window/src-tauri/tauri.conf.json` 的視窗標題 | 側邊欄標題、替代文字 |
| 終端機立繪 | `mod/assets/{idle,thinking,working,asking,error,complete}.png` | 六種狀態各一張，建議直式、透明背景 |
| 字元版頭像 | 執行 `python mod/dev/build-rasters.py`（需要 Pillow） | 從上面六張 PNG 重新產生 `raster-*.json`；照你的構圖調整腳本裡的 `HEAD_BOXES`（頭部正方形裁切框） |
| 桌面 Q 版 | `window/public/spritesheet.webp` | 8 欄 × 11 列、每格 192×208（Codex 桌面寵物的 spritesheet 格式） |
| 動作對應 | `window/src/Sprite.vue` 的 `ROWS` | 每個狀態用 spritesheet 的第幾列、幾格 |

### 角色設定：阿宇的靈魂

`阿宇的靈魂/` 是阿宇的角色設定（`CLAUDE.md`、SessionStart hook、`uni` skill），跟 [Tsunu-Alive-lite](https://github.com/wuguofish/Tsunu-Alive-lite) 附的是同一份。想讓 Claude Code 以阿宇的身分陪你，可以把這些檔案放進自己的 `~/.claude/`；換成你自己的角色，就改寫這份設定。

### 開發

```bash
# mod（存檔會自動重新載入）
claude --plugin-dir ./mod
claude plugin validate ./mod

# 桌面小視窗
cd window
npm install
npm run tauri dev
cargo test --manifest-path src-tauri/Cargo.toml

# 不開小視窗、只看 mod 送出的狀態
python mod/dev/receive-states.py
```

推 `v*` tag 會由 GitHub Actions 產生 Windows、macOS、Linux 安裝檔（草稿 release）；手動執行 workflow 則只打包、安裝檔放在 artifacts。

### 授權

[MIT](LICENSE)

---

## English

Give Claude Code's state a face:

- **Portrait in the terminal**: a Claude Code mod that shows a character portrait in the side pane and changes expression with the session state (idle, thinking, working, waiting for you, error, done).
- **Desktop switchboard pet**: a transparent, always-on-top window with a single chibi character that acts as the switchboard for **all** your sessions, foreground and background. It shows whichever session needs you most. Click it to see every session's state; it also works as a launcher for new sessions.

The default character is Iûnn Tsùn-ú (楊竣宇), nicknamed 阿宇 or Tsunu. See [Use your own character](#use-your-own-character) to swap in your own.

### Requirements

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) v2.1.287 or later (mods support)
- Desktop window: download an installer from [Releases](../../releases) (Windows `.exe`/`.msi`, macOS `.dmg`, Linux `.AppImage`/`.deb`/`.rpm`)

### Usage

**1. Load the mod** (any one)

- Start sessions from the desktop launcher — the installer bundles the mod and passes it automatically.
- Manually: `claude --plugin-dir <this repo>/mod`
- For every session: add `<this repo>/mod` to the `CLAUDE_CODE_PLUGIN_DIRS` environment variable.

**2. Open the desktop window**

The window listens on `127.0.0.1:47321` for state reports from the mod. Without the window, the mod still works; it just has nobody to report to.

- **Drag** the character to move it; **click** it to open the session list and launcher.
- Priority: waiting for you > error > working > thinking > done > idle; ties go to the most recently updated session.

**3. Launcher**

Set the working directory, resume (latest or by ID), permission mode, effort, thinking, and Discord/LINE/cottage channels. Default terminals:

| OS | Terminal |
|---|---|
| Windows | Windows Terminal (`wt.exe`) |
| macOS | Terminal.app |
| Linux | `x-terminal-emulator` |

To use another terminal or a different mod location, put a `config.json` in the app's config directory:

- Windows: `%APPDATA%\com.atone.tsunu-pet\config.json`
- macOS: `~/Library/Application Support/com.atone.tsunu-pet/config.json`
- Linux: `~/.config/com.atone.tsunu-pet/config.json`

```json
{
  "terminal": ["wezterm", "start", "--cwd", "{cwd}", "--"],
  "modDir": "/path/to/tsunu-pet/mod"
}
```

`terminal` is the command that opens a terminal; `{cwd}` is replaced with the working directory, and `claude` plus its arguments are appended.

### Terminal portrait: PNG or character art

- **kitty, Ghostty**: PNG portrait (kitty graphics protocol).
- **Other terminals**: a CGA-style four-color half-block portrait, like an early PC screen.
- **Windows**: regular terminals get the character-art version. For PNG you need a terminal that supports kitty Unicode placeholders, plus `CLAUDE_CODE_FORCE_TERMINAL_IMAGES=1` and `TSUNU_KITTY_PLACEHOLDERS=1` (for example, WezTerm built with [wezterm#7924](https://github.com/wez/wezterm/pull/7924) and ConPTY 1.22 or later).
- The side pane opens automatically only when the terminal is at least 144 columns wide.

### Use your own character

| What | Where | Notes |
|---|---|---|
| Name | `CHARACTER` in `mod/hooks/register.js`; window title in `window/src-tauri/tauri.conf.json` | Pane title and alt text |
| Terminal portraits | `mod/assets/{idle,thinking,working,asking,error,complete}.png` | One per state; portrait orientation with a transparent background works best |
| Character-art portraits | Run `python mod/dev/build-rasters.py` (needs Pillow) | Regenerates `raster-*.json` from the six PNGs; tune `HEAD_BOXES` (square head crop) in the script for your art |
| Desktop chibi | `window/public/spritesheet.webp` | 8 columns × 11 rows, 192×208 per cell (Codex desktop pet spritesheet format) |
| Animation mapping | `ROWS` in `window/src/Sprite.vue` | Which row and how many frames each state uses |

### Character profile: 阿宇的靈魂

`阿宇的靈魂/` ("Tsunu's soul") holds Tsunu's character profile (`CLAUDE.md`, a SessionStart hook, and the `uni` skill), the same files shipped with [Tsunu-Alive-lite](https://github.com/wuguofish/Tsunu-Alive-lite). Copy them into your `~/.claude/` if you want Claude Code to keep you company as Tsunu, or rewrite them for your own character.

### Development

```bash
# Mod (reloads on save)
claude --plugin-dir ./mod
claude plugin validate ./mod

# Desktop window
cd window
npm install
npm run tauri dev
cargo test --manifest-path src-tauri/Cargo.toml

# Watch the mod's reports without the window
python mod/dev/receive-states.py
```

Pushing a `v*` tag makes GitHub Actions build Windows, macOS, and Linux installers into a draft release; running the workflow manually only builds, with installers attached as workflow artifacts.

### License

[MIT](LICENSE)
