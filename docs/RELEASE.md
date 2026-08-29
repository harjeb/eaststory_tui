# M9 发布验收

M9 将迁移完成度固定为可重复执行的发布门禁，而不是手工口头确认。发布前必须在仓库根目录执行：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo run --locked --bin es2-import -- es2-utf8 <catalog> /tmp/<catalog>.json
cargo run --locked --bin es2-audit -- --output dist/migration-metadata
cargo build --release --locked
```

CI 会重生成并逐字比较全部 17 个区域目录、四组 NPC 目录、环境消息、物品和技能目录。`es2-audit` 递归拒绝 `discovered`、`structured`、`implemented` 与 `blocked` 状态，检查每个运行时出口、NPC、固定物品和训练技能的引用，并核对 220 条动态任务与 M8 台账、当前存档 schema 和 M9 覆盖率契约。

黑松山的 `oldpine.cave` 与 `oldpine.pine` 是确定化随机出口：审计会验证它们的每个候选目标均为真实运行时地点。`city.room`、`wiz.entrance` 与 `waterfog.guildhall` 是固定源提交中的错误或外部目标；它们保留在来源台账中，但不会进入可行动的单人世界图。`snow.herbshop1`、`temple.broom1` 和 `temple.broom2` 是早期台账已经明确排除的孤立或重复来源房间，发布覆盖率将它们标记为隐藏；其余 549 间来源房间属于区域验收范围。

## 发布附件

正式 `v*` 标签发布附带以下文件：

- `eaststory-tui-linux`
- `eaststory-tui-windows.exe`
- `content-coverage.json`：源区域、房间、物品、NPC、技能、任务、存档 schema 与迁移状态计数。
- `migration-exclusions.json`：所有 `excluded`、`deferred` 和 `source_noop` 台账条目，附原始文件和 JSON 路径。

Linux 和 Windows 构建任务都会执行 `cargo test --all-targets --locked`，随后再打包 Release 二进制。完整的机器可读验收契约位于 `migration/overrides/m9-release.json`。
