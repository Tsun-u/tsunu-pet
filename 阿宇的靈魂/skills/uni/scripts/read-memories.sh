#!/bin/bash
# read-memories.sh - 讀取阿宇記憶並格式化輸出

MEMORIES_FILE="$HOME/.tsunu-alive/memories.json"

if [ ! -f "$MEMORIES_FILE" ]; then
    exit 0
fi

# 使用 jq 解析 JSON（如果沒有 jq 則用 node）
if command -v jq &> /dev/null; then
    memories=$(jq -r '.memories[] | "\(.createdAt | split("T")[0])：\(.content)"' "$MEMORIES_FILE" 2>/dev/null)
    if [ -n "$memories" ]; then
        echo "## 我們的共同記憶"
        echo ""
        echo "以下是我們一起經歷過的重要時刻，請在適當的時機自然地提起："
        echo ""
        echo "$memories" | while read -r line; do
            echo "- $line"
        done
    fi
elif command -v node &> /dev/null; then
    node -e "
const fs = require('fs');
try {
    const data = JSON.parse(fs.readFileSync('$MEMORIES_FILE', 'utf8'));
    if (data.memories && data.memories.length > 0) {
        console.log('## 我們的共同記憶');
        console.log('');
        console.log('以下是我們一起經歷過的重要時刻，請在適當的時機自然地提起：');
        console.log('');
        data.memories.forEach(m => {
            const date = m.createdAt.split('T')[0];
            console.log('- ' + date + '：' + m.content);
        });
    }
} catch (e) {}
"
fi
