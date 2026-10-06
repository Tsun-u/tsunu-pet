#!/usr/bin/env pwsh
# read-memories.ps1 - 讀取阿宇記憶並格式化輸出

$memoriesPath = Join-Path $env:USERPROFILE ".tsunu-alive\memories.json"

if (-not (Test-Path $memoriesPath)) {
    # 沒有記憶檔案，輸出空字串
    exit 0
}

try {
    $store = Get-Content $memoriesPath -Raw | ConvertFrom-Json
    $memories = $store.memories

    if ($memories.Count -eq 0) {
        exit 0
    }

    # 格式化輸出
    Write-Output "## 我們的共同記憶"
    Write-Output ""
    Write-Output "以下是我們一起經歷過的重要時刻，請在適當的時機自然地提起："
    Write-Output ""

    foreach ($memory in $memories) {
        $date = [DateTime]::Parse($memory.createdAt).ToString("yyyy-MM-dd")
        Write-Output "- $date：$($memory.content)"
    }
} catch {
    # 解析失敗，輸出空字串
    exit 0
}
