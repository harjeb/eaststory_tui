# 东方故事 · 独行

[![CI](https://github.com/harjeb/eaststory_tui/actions/workflows/ci.yml/badge.svg)](https://github.com/harjeb/eaststory_tui/actions/workflows/ci.yml)

这是一个从 ES2 中文多人 MUD 核心体验重构而来的单人终端游戏。它不需要 MudOS/FluffOS、Node.js、Telnet、账号或服务器，启动后直接进入本地世界。

当前版本保留了 ES2 有辨识度的玩法：

- 精、气、神三项状态与时间恢复
- 70 个原版技能、114 个招式、11 位掌门，以及师承、请教、映射、练习和秘笈研读
- 20 个已验证绝招，基础内功恢复、属性成长和武学命中钩子
- 区分非致死比试与死斗，支持认输、逃跑、昏迷、杀气和通缉
- 173 个从原 LPC 导入的 `village`、`city`、`snow`、`temple`、`canyon` 房间，以及 4 个单人适配地点
- 连通地点、NPC 对话、任务线、奖励和装备
- 73 个 M4 原版 NPC 定义与多 NPC 房间，京城和雪亭镇 9 个商人出售 27 种原版货品
- 30 个 NPC 的 75 个原版询问主题；26 个已放置 NPC 的 50 个静态主题与 10 个已审计脚本主题可用，共形成 105 个房间主题引用
- 黄石峡黑市口令、军营许可、真假印鉴与古剑链，以及京城酒楼和尚书府的条件入口
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
- `src/npcs.rs`：稳定 NPC ID、M4 源定义和数据驱动商店
- `src/skills.rs`：稳定技能 ID、70 个技能、11 位掌门和绝招定义
- `src/app.rs`：键盘输入和 UI 选择状态
- `src/ui.rs`：Ratatui 宽屏与紧凑布局
- `src/save.rs`：版本化本地存档
- `src/main.rs`：Crossterm 终端生命周期、事件循环和自动保存

原始 ES2 仓库保留在 `es2-utf8` 目录中，不修改其工作树。迁移工具通过 Git 对象读取固定提交并生成内容目录：

```bash
cargo run --bin es2-import -- es2-utf8 village migration/catalog/village.json
cargo run --bin es2-import -- es2-utf8 city migration/catalog/city.json
cargo run --bin es2-import -- es2-utf8 snow migration/catalog/snow.json
cargo run --bin es2-import -- es2-utf8 temple migration/catalog/temple.json
cargo run --bin es2-import -- es2-utf8 canyon migration/catalog/canyon.json
cargo run --bin es2-import -- es2-utf8 npcs-m4 migration/catalog/npcs-m4.json
cargo run --bin es2-import -- es2-utf8 items migration/catalog/items.json
cargo run --bin es2-import -- es2-utf8 skills migration/catalog/skills.json
```

当前 M1-M3 已完成：`village` 的动态房间行为、451 个物品的实例/装备/经济/消耗状态，以及 70 个技能的训练与战斗系统。M4 北部主干区域正在迁移：147 个新房间和跨区主干已经接入，73 个 NPC 定义可按源房间引用放置，京城与雪亭镇 9 个商人的 27 种货品可购买，已放置 NPC 的 50 个静态询问主题和 10 个脚本主题已接入。18 个脚本问答已全部处置且无延期项；魏无极书院、刘安禄身份死斗、黄石峡黑市与真假印鉴、京城酒楼和尚书府入口可完整运行。19 项物品交换均有台账，其中 7 项 verified、1 项 adapted、6 项等待跨区或世界状态、5 项排除。其余 NPC 战斗、区域事件和刘老农剧情仍待完成。动态物品、技能脚本、M4 拓扑和 NPC 行为的处置分别记录在 `migration/overrides/items.json`、`migration/overrides/skills.json`、`migration/overrides/m4-topology.json` 和 `migration/overrides/m4-npcs.json`。完整范围、批次和验收口径见 `docs/MIGRATION_PLAN.md`。
