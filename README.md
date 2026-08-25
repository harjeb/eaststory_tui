# 东方故事 · 独行

[![CI](https://github.com/harjeb/eaststory_tui/actions/workflows/ci.yml/badge.svg)](https://github.com/harjeb/eaststory_tui/actions/workflows/ci.yml)

这是一个从 ES2 中文多人 MUD 核心体验重构而来的单人终端游戏。它不需要 MudOS/FluffOS、Node.js、Telnet、账号或服务器，启动后直接进入本地世界。

当前版本保留了 ES2 有辨识度的玩法：

- 精、气、神三项状态与时间恢复
- 武学熟练度、实战领悟和评价，而不是传统刷怪等级
- 非致死比试、主动认输和战败回村
- 26 个从原 LPC 导入的 `village` 房间，以及 4 个单人适配地点
- 连通地点、NPC 对话、任务线、奖励和装备
- 451 个原版物品定义，以及实例堆叠、负重、装备槽、耐久和地点掉落
- 原版货币换算、商店买卖、赠予，以及食物、饮水、药物和持续状态
- 实时修炼与休息，行动期间游戏时间自动推进
- 单角色本地 JSON 存档，每 30 秒自动保存

## 运行

需要 Rust 1.85 或更高版本和支持 UTF-8 的终端。迁移工具需要原始 ES2 子模块，因此建议递归克隆：

```bash
git clone --recurse-submodules https://github.com/harjeb/eaststory_tui.git
cd eaststory_tui
cargo run --release
```

每次推送到 `main` 或提交 Pull Request 时，GitHub Actions 会自动检查格式、运行 Clippy 和测试，并构建 Linux、Windows Release 产物。产物可从对应 Actions 运行页面下载；推送 `v*` 标签会自动创建包含两个平台程序的 GitHub Release。

## 操作

| 按键 | 功能 |
| --- | --- |
| `↑` / `k` | 选择上一项行动 |
| `↓` / `j` | 选择下一项行动 |
| `Enter` / `Space` | 执行行动 |
| `Esc` | 战斗中认输 |
| `s` | 手动保存 |
| `?` | 打开或关闭帮助 |
| `q` / `Ctrl+C` | 保存并退出 |

存档写入操作系统的本地应用数据目录。Windows 通常位于 `%LOCALAPPDATA%\\mudchina\\dongfang-tui\\data\\save.json`，Linux 通常位于 `$XDG_DATA_HOME/dongfang-tui/save.json` 或 `~/.local/share/dongfang-tui/save.json`。

## 结构

- `src/game.rs`：与终端无关的角色、战斗、任务和时间状态机
- `src/content.rs`：稳定地点 ID、嵌入式区域目录和游戏规则覆盖
- `src/items.rs`：稳定物品 ID、451 个源定义和运行时物品实例
- `src/app.rs`：键盘输入和 UI 选择状态
- `src/ui.rs`：Ratatui 宽屏与紧凑布局
- `src/save.rs`：版本化本地存档
- `src/main.rs`：Crossterm 终端生命周期、事件循环和自动保存

原始 ES2 仓库保留在 `es2-utf8` 目录中，不修改其工作树。迁移工具通过 Git 对象读取固定提交并生成内容目录：

```bash
cargo run --bin es2-import -- es2-utf8 village migration/catalog/village.json
cargo run --bin es2-import -- es2-utf8 items migration/catalog/items.json
```

当前已完成 `village` 的 26 个房间和 19 项动态房间行为。M2 已完成 451 个物品候选的结构化分类，以及实例、装备、经济、消耗品和状态系统；共享动态对象的实现、延期与单人范围排除记录在 `migration/overrides/items.json`。下一阶段是 M3 完整武学与战斗。完整范围、批次和验收口径见 `docs/MIGRATION_PLAN.md`。
