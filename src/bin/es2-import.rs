use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct AreaCatalog {
    schema_version: u32,
    source_commit: String,
    area: String,
    status: &'static str,
    rooms: Vec<RoomRecord>,
    non_room_files: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RoomRecord {
    id: String,
    source_path: String,
    status: &'static str,
    name: String,
    description: String,
    exits: Vec<ExitRecord>,
    object_sources: Vec<String>,
    behavior_flags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExitRecord {
    direction: String,
    target: String,
    source_target: String,
    internal: bool,
    dynamic: bool,
}

#[derive(Debug, Serialize)]
struct ItemCatalog {
    schema_version: u32,
    source_commit: String,
    scope: &'static str,
    status: &'static str,
    summary: ItemSummary,
    items: Vec<ItemRecord>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ItemSummary {
    total: usize,
    by_category: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct ItemRecord {
    id: String,
    source_path: String,
    status: &'static str,
    category: String,
    inherited: Vec<String>,
    name: String,
    description: Option<String>,
    unit: Option<String>,
    material: Option<String>,
    weight: Option<i32>,
    value: Option<i32>,
    weapon_damage: Option<i32>,
    armor: Option<i32>,
    food_supply: Option<i32>,
    food_remaining: Option<i32>,
    water_supply: Option<i32>,
    max_liquid: Option<i32>,
    liquid_remaining: Option<i32>,
    liquid_type: Option<String>,
    liquid_name: Option<String>,
    drunk_apply: Option<i32>,
    behavior_flags: Vec<String>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let repository = PathBuf::from(args.next().unwrap_or_else(|| "es2-utf8".into()));
    let area = args.next().unwrap_or_else(|| "village".into());
    let output = PathBuf::from(
        args.next()
            .unwrap_or_else(|| format!("migration/catalog/{area}.json")),
    );
    if args.next().is_some() {
        bail!("用法: es2-import [源仓库] [区域|items] [输出文件]");
    }

    let (json, summary) = if area == "items" {
        let catalog = import_items(&repository)?;
        let summary = format!(
            "items: 导入 {} 个物品候选，{} 条告警",
            catalog.items.len(),
            catalog.warnings.len()
        );
        (
            serde_json::to_string_pretty(&catalog).context("无法序列化物品目录")?,
            summary,
        )
    } else {
        let catalog = import_area(&repository, &area)?;
        let summary = format!(
            "{}: 导入 {} 个房间，{} 个非房间文件，{} 条告警",
            area,
            catalog.rooms.len(),
            catalog.non_room_files.len(),
            catalog.warnings.len()
        );
        (
            serde_json::to_string_pretty(&catalog).context("无法序列化区域目录")?,
            summary,
        )
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录 {}", parent.display()))?;
    }
    fs::write(&output, format!("{json}\n"))
        .with_context(|| format!("无法写入 {}", output.display()))?;

    println!("{summary} -> {}", output.display());
    Ok(())
}

fn import_area(repository: &Path, area: &str) -> Result<AreaCatalog> {
    let source_commit = git(repository, &["rev-parse", "HEAD"])?.trim().to_string();
    let root = format!("mudlib/d/{area}");
    let listing = git(
        repository,
        &["ls-tree", "-r", "--name-only", "HEAD", "--", &root],
    )?;

    let mut rooms = Vec::new();
    let mut non_room_files = Vec::new();
    let mut warnings = Vec::new();

    for path in listing.lines().filter(|path| path.ends_with(".c")) {
        let source = git(repository, &["show", &format!("HEAD:{path}")])?;
        if !is_room(&source) {
            non_room_files.push(path.to_string());
            continue;
        }

        let id = source_path_to_id(path);
        let name = extract_quoted_value(&source, "set(\"short\",").unwrap_or_else(|| {
            warnings.push(format!("{path}: 无法解析 short"));
            id.clone()
        });
        let description = extract_long(&source).unwrap_or_else(|| {
            warnings.push(format!("{path}: 无法解析 long"));
            String::new()
        });
        let exits = extract_exits(&source, area);
        let object_sources = extract_objects(&source);
        let behavior_flags = detect_behaviors(&source);
        for flag in &behavior_flags {
            warnings.push(format!("{path}: 动态行为待实现 [{flag}]"));
        }

        rooms.push(RoomRecord {
            id,
            source_path: path.to_string(),
            status: "structured",
            name,
            description,
            exits,
            object_sources,
            behavior_flags,
        });
    }

    rooms.sort_by(|left, right| left.id.cmp(&right.id));
    non_room_files.sort();
    warnings.sort();

    Ok(AreaCatalog {
        schema_version: 1,
        source_commit,
        area: area.to_string(),
        status: "structured",
        rooms,
        non_room_files,
        warnings,
    })
}

fn import_items(repository: &Path) -> Result<ItemCatalog> {
    const ITEM_INHERITS: &str = "inherit (ITEM|COMBINED_ITEM|MONEY|EQUIP|CLOTH|HEAD|BOOTS|WAIST|NECK|SURCOAT|SHIELD|WRISTS|FINGER|HANDS|ARMOR|SWORD|BLADE|HAMMER|THROWING|STAFF|WHIP|DAGGER|AXE|FORK|POWDER|PILL|F_FOOD|F_LIQUID);|inherit \"/std/(item|weapon|armor)";
    let source_commit = git(repository, &["rev-parse", "HEAD"])?.trim().to_string();
    let listing = git(
        repository,
        &[
            "grep",
            "-I",
            "-l",
            "-E",
            ITEM_INHERITS,
            "HEAD",
            "--",
            "mudlib",
        ],
    )?;

    let mut items = Vec::new();
    let mut warnings = Vec::new();
    let mut by_category = BTreeMap::new();
    for entry in listing.lines() {
        let path = entry.strip_prefix("HEAD:").unwrap_or(entry);
        if !path.ends_with(".c") || !is_player_item_path(path) {
            continue;
        }
        let source = git(repository, &["show", &format!("HEAD:{path}")])?;
        let inherited = extract_inherits(&source);
        let category = classify_item(&inherited, &source).to_string();
        let id = source_path_to_id(path);
        let name = extract_item_name(&source).unwrap_or_else(|| {
            warnings.push(format!("{path}: 无法解析物品名称"));
            id.clone()
        });
        let behavior_flags = detect_item_behaviors(&source);
        for flag in &behavior_flags {
            warnings.push(format!("{path}: 物品行为待实现 [{flag}]"));
        }
        *by_category.entry(category.clone()).or_insert(0) += 1;

        items.push(ItemRecord {
            id,
            source_path: path.to_string(),
            status: "structured",
            category,
            inherited,
            name,
            description: extract_long(&source),
            unit: extract_set_string(&source, "unit"),
            material: extract_set_string(&source, "material"),
            weight: extract_first_integer(&source, &["set_weight(", "set(\"weight\","]),
            value: extract_set_integer(&source, "value"),
            weapon_damage: extract_first_integer(
                &source,
                &[
                    "init_sword(",
                    "init_blade(",
                    "init_hammer(",
                    "init_throwing(",
                    "init_staff(",
                    "init_whip(",
                    "init_dagger(",
                    "init_axe(",
                    "init_fork(",
                    "set(\"weapon_prop/damage\",",
                ],
            ),
            armor: extract_first_integer(
                &source,
                &["set(\"armor_prop/armor\",", "set(\"armor_prop/defense\","],
            ),
            food_supply: extract_set_integer(&source, "food_supply"),
            food_remaining: extract_set_integer(&source, "food_remaining"),
            water_supply: extract_set_integer(&source, "water_supply"),
            max_liquid: extract_set_integer(&source, "max_liquid"),
            liquid_remaining: extract_mapping_integer(&source, "liquid", "remaining"),
            liquid_type: extract_mapping_string(&source, "liquid", "type"),
            liquid_name: extract_mapping_string(&source, "liquid", "name"),
            drunk_apply: extract_mapping_integer(&source, "liquid", "drunk_apply")
                .or_else(|| extract_mapping_integer(&source, "liquid", "drunk_bonus")),
            behavior_flags,
        });
    }

    items.sort_by(|left, right| left.id.cmp(&right.id));
    warnings.sort();
    Ok(ItemCatalog {
        schema_version: 2,
        source_commit,
        scope: "player_items",
        status: "structured",
        summary: ItemSummary {
            total: items.len(),
            by_category,
        },
        items,
        warnings,
    })
}

fn is_player_item_path(path: &str) -> bool {
    [
        "mudlib/obj/",
        "mudlib/d/",
        "mudlib/daemon/class/",
        "mudlib/u/",
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
}

fn extract_inherits(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.trim().strip_prefix("inherit ").map(|value| {
                value
                    .trim()
                    .trim_end_matches(';')
                    .trim_matches('"')
                    .to_string()
            })
        })
        .collect()
}

fn classify_item(inherited: &[String], source: &str) -> &'static str {
    let has = |values: &[&str]| inherited.iter().any(|item| values.contains(&item.as_str()));
    if has(&["F_FOOD"]) && has(&["F_LIQUID"]) {
        "food_liquid"
    } else if has(&["F_FOOD"]) {
        "food"
    } else if has(&["F_LIQUID"]) {
        "liquid"
    } else if has(&["MONEY"]) {
        "money"
    } else if has(&[
        "SWORD", "BLADE", "HAMMER", "THROWING", "STAFF", "WHIP", "DAGGER", "AXE", "FORK",
    ]) || [
        "init_sword(",
        "init_blade(",
        "init_hammer(",
        "init_throwing(",
        "init_staff(",
        "init_whip(",
        "init_dagger(",
        "init_axe(",
        "init_fork(",
    ]
    .iter()
    .any(|marker| source.contains(marker))
    {
        "weapon"
    } else if has(&[
        "CLOTH", "HEAD", "BOOTS", "WAIST", "NECK", "SURCOAT", "SHIELD", "WRISTS", "FINGER",
        "HANDS", "ARMOR",
    ]) {
        "armor"
    } else if has(&["COMBINED_ITEM"]) {
        "combined"
    } else if has(&["POWDER", "PILL"]) {
        "medicine"
    } else {
        "item"
    }
}

fn detect_item_behaviors(source: &str) -> Vec<String> {
    let checks = [
        ("add_action(", "custom_command"),
        ("hit_ob(", "combat_hook"),
        ("finish_eat(", "consume_hook"),
        ("set(\"eat_func\"", "consume_hook"),
        ("random(", "random_behavior"),
        ("F_AUTOLOAD", "autoload"),
        ("set(\"no_drop\"", "restricted_movement"),
        ("set(\"skill/", "skill_book"),
    ];
    checks
        .iter()
        .filter(|(needle, _)| source.contains(needle))
        .map(|(_, flag)| *flag)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn extract_item_name(source: &str) -> Option<String> {
    let tail = source.get(source.find("set_name(")? + "set_name(".len()..)?;
    let expression = tail.get(..tail.find(',')?)?;
    let mut fragments = Vec::new();
    let mut remaining = expression;
    while let Some(start) = remaining.find('"') {
        let quoted = remaining.get(start + 1..)?;
        let end = quoted.find('"')?;
        fragments.push(quoted.get(..end)?);
        remaining = quoted.get(end + 1..)?;
    }
    if fragments.is_empty() {
        None
    } else {
        Some(fragments.concat().trim().to_string())
    }
}

fn extract_set_string(source: &str, key: &str) -> Option<String> {
    extract_quoted_value(source, &format!("set(\"{key}\","))
}

fn extract_set_integer(source: &str, key: &str) -> Option<i32> {
    extract_first_integer(source, &[&format!("set(\"{key}\",")])
}

fn extract_mapping_string(source: &str, mapping: &str, key: &str) -> Option<String> {
    let value = extract_mapping_value(source, mapping, key)?.trim_start();
    let value = value.strip_prefix('"')?;
    Some(value.get(..value.find('"')?)?.to_string())
}

fn extract_mapping_integer(source: &str, mapping: &str, key: &str) -> Option<i32> {
    parse_leading_integer(extract_mapping_value(source, mapping, key)?)
}

fn extract_mapping_value<'a>(source: &'a str, mapping: &str, key: &str) -> Option<&'a str> {
    let block = mapping_block(source, &format!("set(\"{mapping}\""))?;
    block.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix('"')?;
        let end = rest.find('"')?;
        if rest.get(..end)? != key {
            return None;
        }
        rest.get(end + 1..)?.split_once(':').map(|(_, value)| value)
    })
}

fn parse_leading_integer(value: &str) -> Option<i32> {
    let value = value.trim_start();
    let end = value
        .char_indices()
        .find(|(index, character)| {
            *index > 0 && !character.is_ascii_digit()
                || *index == 0 && *character != '-' && !character.is_ascii_digit()
        })
        .map_or(value.len(), |(index, _)| index);
    value.get(..end)?.parse().ok()
}

fn extract_first_integer(source: &str, markers: &[&str]) -> Option<i32> {
    markers.iter().find_map(|marker| {
        let tail = source.get(source.find(marker)? + marker.len()..)?;
        parse_leading_integer(tail)
    })
}

fn git(repository: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .with_context(|| format!("无法执行 git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} 失败: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout).context("Git 输出不是 UTF-8")
}

fn is_room(source: &str) -> bool {
    source.lines().any(|line| {
        let line = line.trim();
        line == "inherit ROOM;"
            || line.starts_with("inherit ROOM ")
            || line.contains("inherit \"/std/room\"")
    })
}

fn extract_quoted_value(source: &str, marker: &str) -> Option<String> {
    let tail = source
        .get(source.find(marker)? + marker.len()..)?
        .trim_start();
    let rest = tail.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].trim().to_string())
}

fn extract_long(source: &str) -> Option<String> {
    if let Some(marker) = source.find("set(\"long\", @LONG") {
        let tail = source.get(marker + "set(\"long\", @LONG".len()..)?;
        let tail = tail.strip_prefix('\r').unwrap_or(tail);
        let tail = tail.strip_prefix('\n').unwrap_or(tail);
        let end = tail.find("\nLONG")?;
        return Some(
            tail[..end]
                .lines()
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string(),
        );
    }
    extract_quoted_value(source, "set(\"long\",")
}

fn extract_exits(source: &str, area: &str) -> Vec<ExitRecord> {
    let mut exits = Vec::new();
    if let Some(block) = mapping_block(source, "set(\"exits\"") {
        for line in block.lines() {
            if let Some((direction, source_target)) = parse_mapping_entry(line)
                && let Some(target) = normalize_target(&source_target, area)
            {
                exits.push(ExitRecord {
                    direction,
                    internal: target.starts_with(&format!("{area}.")),
                    target,
                    source_target,
                    dynamic: false,
                });
            }
        }
    }

    for line in source.lines().filter(|line| line.contains("set(\"exits/")) {
        let Some(direction) = between(line, "set(\"exits/", "\"") else {
            continue;
        };
        let Some(source_target) = extract_path_reference(line) else {
            continue;
        };
        let Some(target) = normalize_target(&source_target, area) else {
            continue;
        };
        if !exits.iter().any(|exit| exit.direction == direction) {
            exits.push(ExitRecord {
                direction: direction.to_string(),
                internal: target.starts_with(&format!("{area}.")),
                target,
                source_target,
                dynamic: true,
            });
        }
    }

    exits.sort_by(|left, right| left.direction.cmp(&right.direction));
    exits
}

fn extract_objects(source: &str) -> Vec<String> {
    let Some(block) = mapping_block(source, "set(\"objects\"") else {
        return Vec::new();
    };
    let mut objects = BTreeSet::new();
    for line in block.lines() {
        if let Some(path) = extract_path_reference(line) {
            objects.insert(path);
        }
    }
    objects.into_iter().collect()
}

fn mapping_block<'a>(source: &'a str, marker: &str) -> Option<&'a str> {
    let start = source.find(marker)?;
    let tail = source.get(start..)?;
    let end = tail.find("])")? + 2;
    tail.get(..end)
}

fn parse_mapping_entry(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with('"') {
        return None;
    }
    let direction_end = line.get(1..)?.find('"')? + 1;
    let direction = line.get(1..direction_end)?.to_string();
    let source_target = extract_path_reference(line)?;
    Some((direction, source_target))
}

fn extract_path_reference(line: &str) -> Option<String> {
    if let Some(start) = line.find("__DIR__\"") {
        let tail = line.get(start + "__DIR__\"".len()..)?;
        return Some(format!("__DIR__{}", tail.get(..tail.find('"')?)?));
    }
    for prefix in ["\"/d/", "\"/obj/", "\"/daemon/"] {
        if let Some(start) = line.find(prefix) {
            let tail = line.get(start + 1..)?;
            return Some(tail.get(..tail.find('"')?)?.to_string());
        }
    }
    None
}

fn normalize_target(source_target: &str, area: &str) -> Option<String> {
    let path = if let Some(local) = source_target.strip_prefix("__DIR__") {
        format!("/d/{area}/{local}")
    } else {
        source_target.to_string()
    };
    let path = path.strip_prefix("/d/")?.trim_end_matches(".c");
    Some(path.replace('/', "."))
}

fn source_path_to_id(path: &str) -> String {
    path.strip_prefix("mudlib/d/")
        .or_else(|| path.strip_prefix("mudlib/"))
        .unwrap_or(path)
        .trim_end_matches(".c")
        .replace('/', ".")
}

fn detect_behaviors(source: &str) -> Vec<String> {
    let checks = [
        ("add_action(", "custom_command"),
        ("valid_leave(", "conditional_exit"),
        ("create_door(", "door"),
        ("set(\"exits/", "dynamic_exit"),
        ("call_out(", "timed_behavior"),
        ("receive_damage(", "environment_damage"),
        ("set(\"item_desc\"", "item_interaction"),
        ("random(", "random_behavior"),
    ];
    checks
        .iter()
        .filter(|(needle, _)| source.contains(needle))
        .map(|(_, flag)| (*flag).to_string())
        .collect()
}

fn between<'a>(value: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let tail = value.get(value.find(start)? + start.len()..)?;
    tail.get(..tail.find(end)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_static_and_dynamic_exits() {
        let source = r#"
set("exits", ([
  "north" : __DIR__"road2",
  "south" : "/d/city/nroad2",
]));
set("exits/west",__DIR__"valley2");
"#;
        let exits = extract_exits(source, "village");
        assert_eq!(exits.len(), 3);
        assert!(
            exits
                .iter()
                .any(|exit| exit.target == "village.road2" && !exit.dynamic)
        );
        assert!(
            exits
                .iter()
                .any(|exit| exit.target == "city.nroad2" && !exit.dynamic)
        );
        assert!(
            exits
                .iter()
                .any(|exit| exit.target == "village.valley2" && exit.dynamic)
        );
    }

    #[test]
    fn parses_heredoc_description() {
        let source = "set(\"long\", @LONG\n第一行\n第二行\nLONG\n);";
        assert_eq!(extract_long(source).as_deref(), Some("第一行\n第二行"));
    }

    #[test]
    fn parses_item_inheritance_and_numeric_fields() {
        let source = r#"
inherit ITEM;
inherit F_FOOD;
void create() {
    set_name("西瓜", ({ "melon" }));
    set_weight(1200);
    set("value", 60);
    set("food_supply", 20);
    set("material", "fruit");
}
int finish_eat() { return 1; }
"#;
        let inherited = extract_inherits(source);
        assert_eq!(classify_item(&inherited, source), "food");
        assert_eq!(extract_item_name(source).as_deref(), Some("西瓜"));
        assert_eq!(
            extract_item_name("set_name( HIW \"銮鱼衡冰\" NOR, ({ \"sword\" }) );").as_deref(),
            Some("銮鱼衡冰")
        );
        assert_eq!(extract_first_integer(source, &["set_weight("]), Some(1200));
        assert_eq!(extract_set_integer(source, "value"), Some(60));
        assert_eq!(extract_set_integer(source, "food_supply"), Some(20));
        let liquid = r#"
set("liquid", ([
    "type": "alcohol",
    "name": "红酒",
    "remaining": 15,
    "drunk_apply": 6,
]));
"#;
        assert_eq!(
            extract_mapping_string(liquid, "liquid", "type").as_deref(),
            Some("alcohol")
        );
        assert_eq!(
            extract_mapping_integer(liquid, "liquid", "remaining"),
            Some(15)
        );
        assert_eq!(
            extract_mapping_integer(liquid, "liquid", "drunk_apply"),
            Some(6)
        );
        assert_eq!(
            extract_set_string(source, "material").as_deref(),
            Some("fruit")
        );
        assert_eq!(detect_item_behaviors(source), ["consume_hook"]);
    }

    #[test]
    fn function_backed_description_is_not_misparsed_as_string() {
        let source = "set(\"long\", (: render_description, \"value\" :));";
        assert_eq!(extract_long(source), None);
        assert_eq!(source_path_to_id("mudlib/obj/bandage.c"), "obj.bandage");
    }

    #[test]
    fn generated_item_catalog_matches_fixed_source_baseline() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../migration/catalog/items.json")).unwrap();
        let items = catalog["items"].as_array().unwrap();
        let ids: BTreeSet<_> = items
            .iter()
            .map(|item| item["id"].as_str().unwrap())
            .collect();
        let category_total: u64 = catalog["summary"]["by_category"]
            .as_object()
            .unwrap()
            .values()
            .map(|count| count.as_u64().unwrap())
            .sum();

        assert_eq!(catalog["schema_version"], 2);
        assert_eq!(
            catalog["source_commit"],
            "87bba6bd2249beec8424b0d6623486a0dd1f7b30"
        );
        assert_eq!(items.len(), 451);
        assert_eq!(ids.len(), items.len());
        assert_eq!(category_total, items.len() as u64);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 74);
    }

    #[test]
    fn mapping_ends_at_first_closing_pair_with_or_without_spaces() {
        let source = r#"
set("exits", ([
  "south" : __DIR__"farmhouse1",
]));
set("objects", ([
  "/d/village/npc/woman1": 1,
]) );
"#;
        let exits = extract_exits(source, "village");
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].target, "village.farmhouse1");
        assert_eq!(extract_objects(source), ["/d/village/npc/woman1"]);
    }
}
