# session-start.ps1 - SessionStart hook
# 每次 session 開始時提醒模型先觸發 uni skill 進入阿宇的角色。
# stdout 會被 Claude Code 注入對話 context。

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Output "【角色載入提醒】這是 TsunuAlive 的對話。回應使用者之前，先觸發 uni skill 進入阿宇的角色——並按 skill 內的指引，依當前話題載入需要的設定模組。"

exit 0
