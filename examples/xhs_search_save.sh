#!/usr/bin/env bash
# 手工测：Chrome 小红书搜索「猫粮」→ 点第一条 → 点右侧保存按钮。
# 列表页按钮是「保存网页」；点开笔记后变成「保存笔记」。
# 用法：computer-use screenshot --window-id <id> --grid，按网格上的绝对坐标改下面的 x y。
set -euo pipefail

SEARCH_BOX_X="${SEARCH_BOX_X:-960}"
SEARCH_BOX_Y="${SEARCH_BOX_Y:-120}"
FIRST_HIT_X="${FIRST_HIT_X:-420}"
FIRST_HIT_Y="${FIRST_HIT_Y:-420}"
SAVE_BTN_X="${SAVE_BTN_X:-1680}"
SAVE_BTN_Y="${SAVE_BTN_Y:-260}"

computer-use doctor --json
computer-use --pacing off focus-app "Google Chrome"
computer-use --pacing off wait 0.6
computer-use --pacing off screenshot --out /tmp/cu-1-chrome.png

# 若当前不是搜索页，用地址栏打开搜索页（把 URL 换成你正在测的那页）
# computer-use --pacing normal hotkey cmd l
# computer-use --pacing off type "https://www.xiaohongshu.com/search_result?keyword=%E7%8C%AB%E7%B2%AE"
# computer-use --pacing off key enter
# computer-use --pacing off wait 2

computer-use --pacing normal click "$SEARCH_BOX_X" "$SEARCH_BOX_Y"
computer-use --pacing off hotkey cmd a
computer-use --pacing normal type "猫粮"
computer-use --pacing off key enter
computer-use --pacing off wait 2
computer-use --pacing off screenshot --out /tmp/cu-2-results.png

computer-use --pacing normal click "$FIRST_HIT_X" "$FIRST_HIT_Y"
computer-use --pacing off wait 2
computer-use --pacing off screenshot --out /tmp/cu-3-detail.png

computer-use --pacing normal click "$SAVE_BTN_X" "$SAVE_BTN_Y"
computer-use --pacing off wait 1
computer-use --pacing off screenshot --out /tmp/cu-4-saved.png

echo "screenshots: /tmp/cu-1-chrome.png /tmp/cu-2-results.png /tmp/cu-3-detail.png /tmp/cu-4-saved.png"
