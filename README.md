# 东方故事 · 独行

[![CI](https://github.com/harjeb/eaststory_tui/actions/workflows/ci.yml/badge.svg)](https://github.com/harjeb/eaststory_tui/actions/workflows/ci.yml)

这是一个从 ES2 中文多人 MUD 核心体验重构而来的单人终端游戏。它不需要 MudOS/FluffOS、Node.js、Telnet、账号或服务器，启动后直接进入本地世界。

当前版本保留了 ES2 有辨识度的玩法：

- 精、气、神三项状态与时间恢复
- 70 个原版技能、114 个招式、11 位掌门，以及师承、请教、映射、练习和秘笈研读
- 20 个已验证绝招，基础内功恢复、属性成长和武学命中钩子
- 区分非致死比试与死斗，支持认输、逃跑、昏迷、杀气和通缉
- 552 个从原 LPC 导入的十七个源区域房间，以及 4 个单人适配地点；M1-M7 已完成验收
- 连通地点、NPC 对话、任务线、奖励和装备
- 73 个 M4 原版 NPC 定义、61 个运行时定义的 115 个房间实例，京城和雪亭镇 9 个商人出售 27 种原版货品
- 49 个 M5 NPC 定义展开为 128 个静态房间实例；土匪援军使来源可达上限达到 130，4 个商人的 5 项货品、31 个静态/脚本问答、战斗档案与携带物已接入
- 57 个 M6 NPC 定义展开为 92 个静态房间实例；1 个商人的 6 项货品、17 个静态/脚本问答、53 个战斗档案与 81 件携带物已结构化
- 85 个 M7 新 NPC 定义展开为 101 个静态实例，另复用 4 个既有实例；8 个商人的 29 项货品、67 个问答主题、85 个战斗档案与 122 件携带物已结构化
- 黑松土匪援军与蛇毒、牧羊山岩蛭尸体补品，以及乔阴缚仙绳/仙丹、功德箱和步玄派桃林试炼
- 绮云城堡机关与暗箭、青石八卦阵/巨石/井水/蛛网、三烟寺蒸笼，以及兼作锤兵器的烤山猪肉
- 绮云地牢小娟/刘老农营救复仇、青石老人/醉汉/沈万三玉佩与蒙汗药交换链，以及水烟阁红衣武士强化死斗
- 晚月舞步/浴池/衣物/花、幽冥五步路与投胎，以及舞谱、手镯、组合补品和青云肉类残余行为
- 晚月蓝止萍师承与线索/令牌链、振远镖局忘忧草荐书、陈天星授业、北河渡船及青云人物交换
- 30 个 M4 NPC 的 75 个原版询问主题；26 个已放置 NPC 的 50 个静态主题与 10 个已审计脚本主题可用，共形成 105 个房间主题引用
- 9 个原版 NPC 比试门槛全部处置；8 个已放置人物支持拒绝、同门比试或强制死斗
- 72 个源 NPC 战斗档案保留 316 项技能、94 项映射和 21 项修正；115 个运行时实例均可逐个比试或死斗
- 25 个 NPC 的 87 条原版战斗消息已分类；18 个运行时定义的 36 个实例可执行文本、法术、内功、反击与投降行为
- 8 个 NPC 师承标记全部审计；5 个已放置授业 NPC 按同门或学费条件提供 37 项原版课程上限
- 黄石峡黑市口令、军营许可、真假印鉴与古剑链，以及京城酒楼和尚书府的条件入口
- 祭坛与兵器库限时密道、雪亭做工、竹林巨石/书柜、茅山滑倒与藏经楼门派门槛
- 32 个 M4 房间的 42 条可查看细节，包括随开闭状态变化的 12 条门描述
- 451 个原版物品定义，以及实例堆叠、负重、装备槽、耐久和地点掉落；5 件 M4、10 件 M5、11 件 M6 与 12 件 M7 固定房间物品按源数量初始化
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
- `src/npcs.rs`：稳定 NPC ID、M4/M5/M6/M7 源定义和数据驱动商店
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
cargo run --bin es2-import -- es2-utf8 oldpine migration/catalog/oldpine.json
cargo run --bin es2-import -- es2-utf8 goathill migration/catalog/goathill.json
cargo run --bin es2-import -- es2-utf8 choyin migration/catalog/choyin.json
cargo run --bin es2-import -- es2-utf8 chuenyu migration/catalog/chuenyu.json
cargo run --bin es2-import -- es2-utf8 green migration/catalog/green.json
cargo run --bin es2-import -- es2-utf8 sanyen migration/catalog/sanyen.json
cargo run --bin es2-import -- es2-utf8 waterfog migration/catalog/waterfog.json
cargo run --bin es2-import -- es2-utf8 latemoon migration/catalog/latemoon.json
cargo run --bin es2-import -- es2-utf8 death migration/catalog/death.json
cargo run --bin es2-import -- es2-utf8 graveyard migration/catalog/graveyard.json
cargo run --bin es2-import -- es2-utf8 jail migration/catalog/jail.json
cargo run --bin es2-import -- es2-utf8 cloud migration/catalog/cloud.json
cargo run --bin es2-import -- es2-utf8 npcs-m4 migration/catalog/npcs-m4.json
cargo run --bin es2-import -- es2-utf8 npcs-m5 migration/catalog/npcs-m5.json
cargo run --bin es2-import -- es2-utf8 npcs-m6 migration/catalog/npcs-m6.json
cargo run --bin es2-import -- es2-utf8 npcs-m7 migration/catalog/npcs-m7.json
cargo run --bin es2-import -- es2-utf8 items migration/catalog/items.json
cargo run --bin es2-import -- es2-utf8 skills migration/catalog/skills.json
```

当前 M1-M7 已完成：`village` 的动态房间行为、451 个物品的实例/装备/经济/消耗状态、70 个技能的训练与战斗系统，以及北部主干区域。M4 的 147 个新房间和跨区主干已经接入；73 个 NPC 定义中，59 个静态放置定义的 78 条映射展开为 113 个实例，加上藏经楼 2 个确定化随机守卫后共形成 61 个运行时定义、115 个房间实例，京城与雪亭镇 9 个商人的 27 种货品可购买，已放置 NPC 的 50 个静态询问主题和 10 个脚本主题已接入。18 个脚本问答和 9 个 NPC 比试门槛已全部处置且无延期项；NPC 目录 schema v4 保留 72 个源战斗档案的 316 项技能、94 项映射和 21 项初始修正，115 个运行时实例均可逐个战斗；55 个定义的 93 件初始携带物已结构化，44 个已放置定义的 86 个房间实例可结算共 136 件来源一致的战利品。25 个定义的 87 条战斗消息已分类，18 个运行时定义的 36 个实例可执行当前战斗系统支持的源行为。8 个 NPC 师承标记已审计为 5 verified、1 deferred、2 excluded，5 个授业 NPC 的 37 项课程可运行。魏无极书院、刘安禄身份死斗、钱庄老板强制死斗、武馆与茅山同门比试、黄石峡黑市与真假印鉴、京城酒楼和尚书府入口可完整运行。刘老农父女剧情已补齐护送、原版双奖励与遇害复仇分支，M6 导入绮云镇地牢时会从当前黑松林单人入口迁回来源位置。19 项物品交换均有台账，其中 8 项 verified、2 项 adapted、4 项等待 M6/M8 跨区或调度状态、5 项排除；雪亭城隍庙香火捐献已按价值与灵性接入杀气削减。11 项 NPC 自定义命令已逐源处置为 9 verified、1 adapted、1 excluded；废园令牌、守城兵交验与一次性北门出城链进入存档 schema v11。四区 60 个动态房间的 107 项警告已建立精确区域/类别基线，首批 8 个房间机制覆盖 22 项警告并进入存档 schema v12；其中北驿道指向不存在 `city.room` 的命令明确排除。区域目录 schema v4 另保留 40 个固定源门定义、36 个房间的 46 条查看细节与 120 个对象实例；5 件固定房间物品按源数量初始化并进入存档 schema v14；M4 的 35 项门警告已处置为 26 verified、9 excluded，14 对有效门双向共享状态并进入存档 schema v13；32 项房间细节警告也已处置为 16 verified、16 adapted，42 条细节可执行查看；其余 40 项动态警告为 12 verified、19 adapted、1 source-noop、1 deferred、7 excluded，107 项总账全部有处置，M4 验收通过。M5 已将黑松山 41、牧羊山 16、乔阴县 62 个房间接入主世界；68 条房间对象映射保留 138 个源实例，其中 10 件固定物品按区域独立初始化并进入存档 schema v15。NPC 目录另导入 49 个定义（含全局县城官兵），46 个已放置定义展开为 128 个实例；4 个商人的 5 项货品、25 个静态问答、48 个战斗档案、29 条战斗消息和 53 件携带物均进入通用运行层，静态实例首次死斗可结算 136 件掉落，土匪动态援军使来源可达上限增至 130 个实例与 140 件掉落。59 项 M5 NPC 行为标记已全部逐源处置为 14 verified、21 adapted、23 deferred、1 excluded：酒楼守卫/县官拒绝比试、采药老者接受，守卫死斗会由同伴报官增加通缉；6 个脚本问答、官家小姐荷包与陈显祖桃木箱交换、护草神兽忘忧草与巡捕通缉死亡钩子均可运行，状态进入存档 schema v18；战斗概率回退、土匪动态援军、金银花蛇蛇毒和岩蛭尸体钩子随后进入 schema v19。23 项环境聊天、随机移动、衙门调度和尸体定时行为已准确绑定 M8/M7；县官只面向其他在线玩家的指控命令明确排除，并由单人通缉与后续司法流程替代。房间层 73/73 项警告均有处置：19 个房间的 25 条细节可查看，县衙铜门双向运行，草堂/竹篱两项源方向错误明确排除；黑松山随机洞窟/松林、攀爬藤蔓、埋骨和山寨封门可运行；乔阴绝壁、云台雷击/限时炉口、井水、石狮、草堂借书和桃林导航也已接入；骆云舟首次拜师、桃林三步试炼与第二次正式入派完成正式入口。13 项 M5 区域物品标记全部处置为 12 adapted、1 alias：稳定随机书名、功德箱、缚仙绳/仙丹、竹哨和死岩蛭补品均可运行。幽兰目标与岩蛭腐败留 M7，固定源外狮穴出口排除。三地自动端到端回归通过，M5 验收完成。醉汉的青石村线索归入 M6 并已完成；固定源复核确认陈天星荐书来自 `mudlib/u/cloud` 振远镖局，该区域现已进入 M7 静态目录，任务交换仍待 M7 行为批次；随机移动、跟随雇佣和尸体动画分别保留 M8/M7 依赖。M6 首批另接入绮云镇 37、青石村 39、三烟寺 18、水烟阁 27 个房间，以及 57 个 NPC 定义的 92 个静态实例；11 件固定房间物品进入存档 schema v20。65 项房间行为已逐源处置为 29 verified、34 adapted、2 excluded：16 个门端点、38 条查看细节、绮云城堡攀爬/石板/暗箭、青石八卦阵/巨石/井水/蛛网与三烟寺蒸笼均可运行或有明确排除，事件状态进入 schema v21。3 项区域物品标记也已处置为 2 adapted、1 excluded：两条烤山猪肉来源均保留六口食物、锤兵器及吃完变骨头语义，青石绳子的无恢复自毁命令不进入正常行动。81 项 NPC 行为已逐源处置为 17 verified、23 adapted、23 deferred、13 excluded、4 alias、1 source_noop；10 项比试门槛、7 个脚本问答、3 项交换、3 项死亡钩子、进场攻击和战斗回调均有运行时或后续绑定。小娟/刘老农及青石玉佩/蒙汗药共享剧情进入 schema v22，四区自动端到端回归通过，M6 验收完成。M7 静态批次另接入晚月庄 74、幽冥地界 12、荒冢 2、青云境 51 个房间及 204 个非房间源文件；85 个新 NPC 定义展开为 101 个实例，并复用 M4 乞丐与 M5 驻军各 2 个，世界共 105 个 M7 NPC 实例。78 条对象映射保留 117 个源实例，其中 12 件固定房间物品进入存档 schema v23；五组跨区固定源主干已接通，三条错误携带物路径与一个缺失掉落均有显式导入处置。77 项房间行为已逐源处置为 50 verified、23 adapted、1 deferred、1 excluded、2 source_noop：31 个门端点、22 条查看细节、晚月舞步/浴池/花与衣物、幽冥五步路与投胎均有运行时或准确后续绑定，事件状态进入 schema v24。22 项区域物品标记也已处置为 9 verified、13 adapted：目录驱动 no_drop、舞谱/手镯/荐书命令、五种组合补品和三种肉类残余均可运行。183 项 NPC 行为已逐源处置为 33 verified、50 adapted、51 deferred、38 excluded、10 alias、1 source_noop；晚月蓝止萍师承与竹蜻蜓/手镯/舞曲谱/令牌链、振远镖局入门/忘忧草荐书、陈天星授业、北河渡船和青云人物交换进入 schema v25。临时对象、尸体动画/腐败和环境调度明确绑定 M8 世界生命周期，五区自动回归通过，M7 验收完成。动态物品、技能脚本、M4-M7 拓扑和 NPC 行为的处置记录在 `migration/overrides/`；M7 三份台账 `m7-topology.json`、`m7-npcs.json` 与 `m7-items.json` 均为 `complete`。完整范围、批次和验收口径见 `docs/MIGRATION_PLAN.md`。
