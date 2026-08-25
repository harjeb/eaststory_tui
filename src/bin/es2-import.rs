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
    study_skill: Option<String>,
    study_exp_required: Option<i32>,
    study_spirit_cost: Option<i32>,
    study_difficulty: Option<i32>,
    study_max_level: Option<i32>,
    behavior_flags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkillCatalog {
    schema_version: u32,
    source_commit: String,
    scope: &'static str,
    status: &'static str,
    summary: SkillSummary,
    skills: Vec<SkillRecord>,
    teachers: Vec<TeacherRecord>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkillSummary {
    total: usize,
    by_category: BTreeMap<String, usize>,
    action_sets: usize,
    actions: usize,
    teachers: usize,
}

#[derive(Debug, Serialize)]
struct SkillRecord {
    id: String,
    source_path: String,
    status: &'static str,
    display_name: String,
    category: String,
    skill_type: &'static str,
    usages: Vec<String>,
    dependencies: Vec<String>,
    actions: Vec<SkillActionRecord>,
    valid_learn: Option<String>,
    practice: Option<String>,
    routers: Vec<String>,
    behavior_flags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkillActionRecord {
    name: Option<String>,
    description: Option<String>,
    weapon: Option<String>,
    damage_type: Option<String>,
    dodge: Option<i32>,
    parry: Option<i32>,
    force: Option<i32>,
    damage: Option<i32>,
}

#[derive(Debug, Serialize)]
struct TeacherRecord {
    id: String,
    source_path: String,
    status: &'static str,
    name: String,
    faction: Option<String>,
    intelligence: Option<i32>,
    skills: BTreeMap<String, i32>,
    mappings: BTreeMap<String, String>,
    apprentice_behavior: Option<String>,
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
        bail!("用法: es2-import [源仓库] [区域|items|skills] [输出文件]");
    }

    let (json, summary) = match area.as_str() {
        "items" => {
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
        }
        "skills" => {
            let catalog = import_skills(&repository)?;
            let summary = format!(
                "skills: 导入 {} 个技能、{} 个招式，{} 条告警",
                catalog.skills.len(),
                catalog.summary.actions,
                catalog.warnings.len()
            );
            (
                serde_json::to_string_pretty(&catalog).context("无法序列化技能目录")?,
                summary,
            )
        }
        _ => {
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
        }
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

    let skill_listing = git(
        repository,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            "HEAD",
            "--",
            "mudlib/daemon/skill",
        ],
    )?;
    let known_skills: BTreeSet<_> = skill_listing
        .lines()
        .filter_map(|path| {
            path.strip_prefix("mudlib/daemon/skill/")
                .and_then(|path| path.strip_suffix(".c"))
                .map(str::to_string)
        })
        .collect();

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
        let study_skill = extract_mapping_string(&source, "skill", "name");
        if let Some(skill) = &study_skill
            && !known_skills.contains(skill)
        {
            warnings.push(format!("{path}: 秘笈引用不存在的技能 [{skill}]"));
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
            study_skill,
            study_exp_required: extract_mapping_integer(&source, "skill", "exp_required"),
            study_spirit_cost: extract_mapping_integer(&source, "skill", "sen_cost"),
            study_difficulty: extract_mapping_integer(&source, "skill", "difficulty"),
            study_max_level: extract_mapping_integer(&source, "skill", "max_skill"),
            behavior_flags,
        });
    }

    items.sort_by(|left, right| left.id.cmp(&right.id));
    warnings.sort();
    Ok(ItemCatalog {
        schema_version: 3,
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

fn import_skills(repository: &Path) -> Result<SkillCatalog> {
    let source_commit = git(repository, &["rev-parse", "HEAD"])?.trim().to_string();
    let listing = git(
        repository,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            "HEAD",
            "--",
            "mudlib/daemon/skill",
        ],
    )?;

    let mut skills = Vec::new();
    let mut warnings = Vec::new();
    let mut by_category = BTreeMap::new();
    let mut action_sets = 0;
    let mut action_count = 0;

    for path in listing.lines().filter(|path| path.ends_with(".c")) {
        let source = git(repository, &["show", &format!("HEAD:{path}")])?;
        let id = path
            .strip_prefix("mudlib/daemon/skill/")
            .unwrap_or(path)
            .trim_end_matches(".c")
            .to_string();
        let valid_learn = extract_function(&source, "valid_learn(").map(normalize_fragment);
        let practice = extract_function(&source, "practice_skill(").map(normalize_fragment);
        let usages = extract_skill_usages(&source);
        let category = classify_skill(&id, &source, &usages).to_string();
        let skill_type = if source.contains("return \"knowledge\"") {
            "knowledge"
        } else {
            "martial"
        };
        let actions = extract_skill_actions(&source);
        if !actions.is_empty() {
            action_sets += 1;
            action_count += actions.len();
        }
        let routers = extract_skill_routers(&source);
        let behavior_flags = detect_skill_behaviors(&source, valid_learn.as_deref());
        for flag in &behavior_flags {
            warnings.push(format!("{path}: 技能行为待实现 [{flag}]"));
        }

        let doc_path = format!("mudlib/doc/skill/{id}");
        let display_name = git(repository, &["show", &format!("HEAD:{doc_path}")])
            .ok()
            .and_then(|document| extract_skill_doc_title(&document))
            .unwrap_or_else(|| {
                warnings.push(format!("{path}: 无法从源文档解析中文技能名"));
                id.clone()
            });
        let dependencies = extract_skill_dependencies(&source, &id);
        *by_category.entry(category.clone()).or_insert(0) += 1;

        skills.push(SkillRecord {
            id,
            source_path: path.to_string(),
            status: "structured",
            display_name,
            category,
            skill_type,
            usages,
            dependencies,
            actions,
            valid_learn,
            practice,
            routers,
            behavior_flags,
        });
    }

    let teachers = import_teachers(repository)?;
    skills.sort_by(|left, right| left.id.cmp(&right.id));
    warnings.sort();
    Ok(SkillCatalog {
        schema_version: 3,
        source_commit,
        scope: "player_skills",
        status: "structured",
        summary: SkillSummary {
            total: skills.len(),
            by_category,
            action_sets,
            actions: action_count,
            teachers: teachers.len(),
        },
        skills,
        teachers,
        warnings,
    })
}

fn import_teachers(repository: &Path) -> Result<Vec<TeacherRecord>> {
    let listing = git(
        repository,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            "HEAD",
            "--",
            "mudlib/daemon/class",
        ],
    )?;
    let mut teachers = Vec::new();
    for path in listing.lines().filter(|path| path.ends_with("/master.c")) {
        let source = git(repository, &["show", &format!("HEAD:{path}")])?;
        let id = path
            .strip_prefix("mudlib/daemon/class/")
            .and_then(|path| path.strip_suffix("/master.c"))
            .unwrap_or(path)
            .to_string();
        teachers.push(TeacherRecord {
            id,
            source_path: path.to_string(),
            status: "structured",
            name: extract_item_name(&source).unwrap_or_else(|| "未知掌门".into()),
            faction: extract_call_strings(&source, "create_family(")
                .and_then(|values| values.first().cloned()),
            intelligence: extract_set_integer(&source, "int"),
            skills: extract_named_integer_calls(&source, "set_skill("),
            mappings: extract_named_string_calls(&source, "map_skill("),
            apprentice_behavior: extract_function(&source, "attempt_apprentice(")
                .map(normalize_fragment),
        });
    }
    teachers.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(teachers)
}

fn extract_call_strings(source: &str, marker: &str) -> Option<Vec<String>> {
    let tail = &source[source.find(marker)? + marker.len()..];
    let arguments = &tail[..tail.find(");")?];
    Some(extract_quoted_strings(arguments))
}

fn extract_named_integer_calls(source: &str, marker: &str) -> BTreeMap<String, i32> {
    let mut values = BTreeMap::new();
    let mut remaining = source;
    while let Some(start) = remaining.find(marker) {
        let tail = &remaining[start + marker.len()..];
        let strings = extract_quoted_strings(
            tail.get(..tail.find(");").unwrap_or(tail.len()))
                .unwrap_or(tail),
        );
        let Some(name) = strings.first() else {
            break;
        };
        let Some(comma) = tail.find(',') else {
            break;
        };
        if let Some(value) = parse_leading_integer(tail[comma + 1..].trim_start()) {
            values.insert(name.clone(), value);
        }
        remaining = &tail[comma + 1..];
    }
    values
}

fn extract_named_string_calls(source: &str, marker: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let mut remaining = source;
    while let Some(start) = remaining.find(marker) {
        let tail = &remaining[start + marker.len()..];
        let call = tail
            .get(..tail.find(");").unwrap_or(tail.len()))
            .unwrap_or(tail);
        let strings = extract_quoted_strings(call);
        if strings.len() >= 2 {
            values.insert(strings[0].clone(), strings[1].clone());
        }
        remaining = &tail[call.len()..];
    }
    values
}

fn classify_skill(id: &str, source: &str, usages: &[String]) -> &'static str {
    const BASIC_SKILLS: &[&str] = &[
        "axe",
        "blade",
        "dagger",
        "dodge",
        "force",
        "fork",
        "hammer",
        "iron-cloth",
        "magic",
        "move",
        "parry",
        "spells",
        "staff",
        "sword",
        "throwing",
        "unarmed",
        "whip",
    ];
    let has_usage = |usage: &str| usages.iter().any(|candidate| candidate == usage);
    if source.contains("return \"knowledge\"") {
        "knowledge"
    } else if has_usage("force") {
        "internal"
    } else if has_usage("magic") || has_usage("spells") {
        "mystic"
    } else if has_usage("array") {
        "formation"
    } else if has_usage("dodge") || has_usage("move") {
        "movement"
    } else if has_usage("unarmed") {
        "unarmed"
    } else if usages.iter().any(|usage| {
        [
            "axe", "blade", "dagger", "fork", "hammer", "staff", "sword", "throwing", "whip",
        ]
        .contains(&usage.as_str())
    }) {
        "weapon"
    } else if BASIC_SKILLS.contains(&id) {
        "basic"
    } else {
        "utility"
    }
}

fn extract_skill_usages(source: &str) -> Vec<String> {
    const USAGES: &[&str] = &[
        "array",
        "axe",
        "blade",
        "dagger",
        "dodge",
        "force",
        "fork",
        "hammer",
        "iron-cloth",
        "magic",
        "move",
        "parry",
        "spells",
        "staff",
        "sword",
        "throwing",
        "unarmed",
        "whip",
    ];
    let Some(function) = extract_function(source, "valid_enable(") else {
        return Vec::new();
    };
    extract_quoted_strings(function)
        .into_iter()
        .filter(|value| USAGES.contains(&value.as_str()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn extract_skill_dependencies(source: &str, own_id: &str) -> Vec<String> {
    let mut dependencies = BTreeSet::new();
    for marker in ["query_skill(\"", "query_skill_mapped(\""] {
        let mut remaining = source;
        while let Some(start) = remaining.find(marker) {
            let tail = &remaining[start + marker.len()..];
            let Some(end) = tail.find('"') else {
                break;
            };
            let skill = &tail[..end];
            if skill != own_id {
                dependencies.insert(skill.to_string());
            }
            remaining = &tail[end + 1..];
        }
    }
    dependencies.into_iter().collect()
}

fn extract_skill_actions(source: &str) -> Vec<SkillActionRecord> {
    let Some(start) = source.find("mapping *action") else {
        return Vec::new();
    };
    let tail = &source[start..];
    let end = tail.find("});").unwrap_or(tail.len());
    let block = &tail[..end];
    let mut actions = Vec::new();
    let mut remaining = block;
    while let Some(mapping_start) = remaining.find("([") {
        let after_start = &remaining[mapping_start + 2..];
        let Some(mapping_end) = after_start.find("])") else {
            break;
        };
        let mapping = &after_start[..mapping_end];
        actions.push(SkillActionRecord {
            name: extract_action_string(mapping, "name"),
            description: extract_action_string(mapping, "action"),
            weapon: extract_action_string(mapping, "weapon"),
            damage_type: extract_action_string(mapping, "damage_type"),
            dodge: extract_action_integer(mapping, "dodge"),
            parry: extract_action_integer(mapping, "parry"),
            force: extract_action_integer(mapping, "force"),
            damage: extract_action_integer(mapping, "damage"),
        });
        remaining = &after_start[mapping_end + 2..];
    }
    actions
}

fn extract_action_value<'a>(mapping: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\"");
    let tail = &mapping[mapping.find(&marker)? + marker.len()..];
    tail.split_once(':').map(|(_, value)| value.trim_start())
}

fn extract_action_string(mapping: &str, key: &str) -> Option<String> {
    let value = extract_action_value(mapping, key)?;
    let start = value.find('"')?;
    let tail = &value[start + 1..];
    Some(tail[..tail.find('"')?].to_string())
}

fn extract_action_integer(mapping: &str, key: &str) -> Option<i32> {
    parse_leading_integer(extract_action_value(mapping, key)?)
}

fn extract_skill_routers(source: &str) -> Vec<String> {
    [
        "perform_action_file(",
        "exert_function_file(",
        "cast_spell_file(",
        "conjure_magic_file(",
        "scribe_spell_file(",
    ]
    .into_iter()
    .filter(|marker| extract_function(source, marker).is_some())
    .map(|marker| marker.trim_end_matches('(').to_string())
    .collect()
}

fn detect_skill_behaviors(source: &str, valid_learn: Option<&str>) -> Vec<String> {
    let mut flags = BTreeSet::new();
    if valid_learn.is_some_and(|body| {
        let compact: String = body
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        compact != "{return1;}"
    }) {
        flags.insert("learning_requirement");
    }
    for (marker, flag) in [
        ("practice_skill(", "practice_cost"),
        ("hit_ob(", "combat_hook"),
        ("skill_improved(", "level_hook"),
        ("perform_action_file(", "perform_router"),
        ("exert_function_file(", "exert_router"),
        ("cast_spell_file(", "cast_router"),
        ("conjure_magic_file(", "conjure_router"),
        ("scribe_spell_file(", "scribe_router"),
        ("query_dodge_msg(", "dodge_messages"),
        ("query_parry_msg(", "parry_messages"),
        ("valid_effect(", "effect_requirement"),
        ("learn_bonus(", "learning_bonus"),
        ("practice_bonus(", "practice_bonus"),
    ] {
        if extract_function(source, marker).is_some() {
            flags.insert(flag);
        }
    }
    flags.into_iter().map(str::to_string).collect()
}

fn extract_function<'a>(source: &'a str, marker: &str) -> Option<&'a str> {
    let start = source.find(marker)?;
    let tail = &source[start..];
    let brace = tail.find('{')?;
    let bytes = tail.as_bytes();
    let mut depth = 0_u32;
    for (index, byte) in bytes.iter().enumerate().skip(brace) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return tail.get(brace..=index);
                }
            }
            _ => {}
        }
    }
    None
}

fn normalize_fragment(source: &str) -> String {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_quoted_strings(source: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut remaining = source;
    while let Some(start) = remaining.find('"') {
        let tail = &remaining[start + 1..];
        let Some(end) = tail.find('"') else {
            break;
        };
        strings.push(tail[..end].to_string());
        remaining = &tail[end + 1..];
    }
    strings
}

fn extract_skill_doc_title(document: &str) -> Option<String> {
    document.lines().find_map(|line| {
        let title = line.trim().trim_start_matches('□').trim();
        (!title.is_empty()).then(|| title.to_string())
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

        assert_eq!(catalog["schema_version"], 3);
        assert_eq!(
            catalog["source_commit"],
            "87bba6bd2249beec8424b0d6623486a0dd1f7b30"
        );
        assert_eq!(items.len(), 451);
        assert_eq!(ids.len(), items.len());
        assert_eq!(category_total, items.len() as u64);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 76);
        let manual = items
            .iter()
            .find(|item| item["id"] == "village.npc.obj.parrybook")
            .unwrap();
        assert_eq!(manual["study_skill"], "parry");
        assert_eq!(manual["study_exp_required"], 2_000);
        assert_eq!(manual["study_spirit_cost"], 20);
        assert_eq!(manual["study_max_level"], 60);
    }

    #[test]
    fn parses_skill_metadata_and_actions() {
        let source = r#"
mapping *action = ({
    ([ "name": "试招", "action": "$N击向$n的$l", "dodge": -20,
       "damage": 80, "damage_type": "刺伤" ]),
});
int valid_enable(string usage) { return usage=="sword" || usage=="parry"; }
int valid_learn(object me) {
    if (me->query_skill("force", 1) < 10) return 0;
    return 1;
}
int practice_skill(object me) { return 1; }
"#;
        let usages = extract_skill_usages(source);
        let actions = extract_skill_actions(source);
        assert_eq!(usages, ["parry", "sword"]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name.as_deref(), Some("试招"));
        assert_eq!(actions[0].damage, Some(80));
        assert_eq!(actions[0].dodge, Some(-20));
        assert_eq!(extract_skill_dependencies(source, "test-sword"), ["force"]);
        assert_eq!(classify_skill("test-sword", source, &usages), "weapon");
        assert!(
            detect_skill_behaviors(
                source,
                extract_function(source, "valid_learn(")
                    .map(normalize_fragment)
                    .as_deref()
            )
            .contains(&"learning_requirement".to_string())
        );
    }

    #[test]
    fn generated_skill_catalog_matches_fixed_source_baseline() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../migration/catalog/skills.json")).unwrap();
        let skills = catalog["skills"].as_array().unwrap();
        let ids: BTreeSet<_> = skills
            .iter()
            .map(|skill| skill["id"].as_str().unwrap())
            .collect();
        let category_total: u64 = catalog["summary"]["by_category"]
            .as_object()
            .unwrap()
            .values()
            .map(|count| count.as_u64().unwrap())
            .sum();

        assert_eq!(catalog["schema_version"], 3);
        assert_eq!(
            catalog["source_commit"],
            "87bba6bd2249beec8424b0d6623486a0dd1f7b30"
        );
        assert_eq!(skills.len(), 70);
        assert_eq!(ids.len(), skills.len());
        assert_eq!(category_total, skills.len() as u64);
        assert_eq!(catalog["summary"]["action_sets"], 19);
        assert_eq!(catalog["summary"]["actions"], 114);
        assert_eq!(catalog["summary"]["teachers"], 11);
        assert_eq!(catalog["teachers"].as_array().unwrap().len(), 11);
        let fighter = catalog["teachers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|teacher| teacher["id"] == "fighter")
            .unwrap();
        assert_eq!(fighter["intelligence"], 24);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 200);
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
