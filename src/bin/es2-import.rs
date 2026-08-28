use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde::Serialize;

const SOURCE_REV: &str = "87bba6b";

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
    doors: Vec<DoorRecord>,
    details: Vec<RoomDetailRecord>,
    object_sources: Vec<String>,
    object_placements: Vec<ObjectReference>,
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
struct DoorRecord {
    direction: String,
    name: String,
    reverse_direction: String,
    initially_closed: bool,
}

#[derive(Debug, Serialize)]
struct RoomDetailRecord {
    key: String,
    description: Option<String>,
    door_direction: Option<String>,
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

#[derive(Debug, Serialize)]
struct NpcCatalog {
    schema_version: u32,
    source_commit: String,
    scope: &'static str,
    status: &'static str,
    summary: NpcSummary,
    npcs: Vec<NpcRecord>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NpcSummary {
    total: usize,
    placed: usize,
    vendors: usize,
    vendor_goods: usize,
    inquiry_npcs: usize,
    inquiry_topics: usize,
    static_inquiries: usize,
    scripted_inquiries: usize,
    runtime_inquiry_npcs: usize,
    runtime_inquiries: usize,
    runtime_inquiry_references: usize,
    combat_profiles: usize,
    combat_skill_entries: usize,
    combat_mapping_entries: usize,
    combat_apply_entries: usize,
    combat_chat_npcs: usize,
    combat_chat_entries: usize,
    carried_item_npcs: usize,
    carried_item_entries: usize,
    worn_item_entries: usize,
    wielded_item_entries: usize,
    placed_combat_npcs: usize,
    placed_combat_chat_npcs: usize,
    placed_carried_item_npcs: usize,
    by_area: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct NpcRecord {
    id: String,
    source_path: String,
    status: &'static str,
    area: String,
    name: String,
    description: Option<String>,
    combat_exp: Option<i32>,
    attributes: BTreeMap<String, i32>,
    resources: BTreeMap<String, i32>,
    skills: BTreeMap<String, i32>,
    mappings: BTreeMap<String, String>,
    combat_apply: BTreeMap<String, i32>,
    combat_chat: Option<NpcCombatChatRecord>,
    carried_items: Vec<NpcCarriedItemRecord>,
    placement_count: usize,
    vendor_goods: Vec<VendorGoodRecord>,
    inquiries: Vec<InquiryRecord>,
    behavior_flags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NpcCombatChatRecord {
    chance: Option<i32>,
    chance_expression: Option<String>,
    entries: Vec<NpcCombatChatEntryRecord>,
}

#[derive(Debug, Serialize)]
struct NpcCombatChatEntryRecord {
    kind: &'static str,
    value: String,
}

#[derive(Debug, Serialize)]
struct NpcCarriedItemRecord {
    item_id: String,
    source_path: String,
    state: &'static str,
}

#[derive(Debug, Serialize)]
struct VendorGoodRecord {
    item_id: String,
    source_path: String,
}

#[derive(Debug, Serialize)]
struct InquiryRecord {
    topic: String,
    response: Option<String>,
    scripted: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectReference {
    source_path: String,
    quantity: usize,
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
        bail!(
            "用法: es2-import [源仓库] [区域|items|skills|npcs-m4|npcs-m5|npcs-m6|npcs-m7] [输出文件]"
        );
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
        "npcs-m4" | "npcs-m5" | "npcs-m6" | "npcs-m7" => {
            let catalog = match area.as_str() {
                "npcs-m4" => import_npcs(
                    &repository,
                    &["city", "snow", "temple", "canyon"],
                    "m4-npcs",
                    &[],
                    &[],
                )?,
                "npcs-m5" => import_npcs(
                    &repository,
                    &["oldpine", "goathill", "choyin"],
                    "m5-npcs",
                    &[],
                    &[],
                )?,
                "npcs-m6" => import_npcs(
                    &repository,
                    &["chuenyu", "green", "sanyen", "waterfog"],
                    "m6-npcs",
                    &[],
                    &[],
                )?,
                "npcs-m7" => import_npcs(
                    &repository,
                    &["latemoon", "death", "graveyard", "jail", "cloud"],
                    "m7-npcs",
                    &["mudlib/d/snow/npc/beggar.c", "mudlib/obj/npc/garrison.c"],
                    &[
                        ("mudlib/d/latemoon/room/npc/houndbane.c", None),
                        (
                            "mudlib/d/latemoon/room/npc/obj/deer_boot.c",
                            Some("mudlib/d/latemoon/npc/obj/deer_boot.c"),
                        ),
                        (
                            "mudlib/d/latemoon/room/npc/obj/blue_dress.c",
                            Some("mudlib/d/latemoon/npc/obj/blue_dress.c"),
                        ),
                        (
                            "mudlib/d/latemoon/room/npc/obj/redbelt.c",
                            Some("mudlib/d/latemoon/npc/obj/redbelt.c"),
                        ),
                    ],
                )?,
                _ => unreachable!(),
            };
            let summary = format!(
                "{area}: 导入 {} 个 NPC、{} 个商人、{} 条告警",
                catalog.npcs.len(),
                catalog.summary.vendors,
                catalog.warnings.len()
            );
            (
                serde_json::to_string_pretty(&catalog).context("无法序列化 NPC 目录")?,
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

fn area_source_root(area: &str) -> String {
    match area {
        "cloud" => "mudlib/u/cloud".into(),
        _ => format!("mudlib/d/{area}"),
    }
}

fn area_id_prefix(area: &str) -> &str {
    match area {
        "cloud" => "u.cloud",
        _ => area,
    }
}

fn import_area(repository: &Path, area: &str) -> Result<AreaCatalog> {
    let source_commit = git(repository, &["rev-parse", SOURCE_REV])?
        .trim()
        .to_string();
    let root = area_source_root(area);
    let listing = git(
        repository,
        &["ls-tree", "-r", "--name-only", SOURCE_REV, "--", &root],
    )?;

    let mut rooms = Vec::new();
    let mut non_room_files = Vec::new();
    let mut warnings = Vec::new();

    for path in listing.lines().filter(|path| path.ends_with(".c")) {
        let source = git(repository, &["show", &format!("{SOURCE_REV}:{path}")])?;
        if !is_room(&source) {
            non_room_files.push(path.to_string());
            continue;
        }

        let id = source_path_to_id(path);
        let name = extract_set_string(&source, "short").unwrap_or_else(|| {
            warnings.push(format!("{path}: 无法解析 short"));
            id.clone()
        });
        let description = extract_long(&source).unwrap_or_else(|| {
            warnings.push(format!("{path}: 无法解析 long"));
            String::new()
        });
        let exits = extract_exits(&source, area_id_prefix(area), path);
        let doors = extract_doors(&source);
        let details = extract_room_details(&source);
        let object_placements = extract_object_references(&source, path);
        let object_sources = object_placements
            .iter()
            .map(|object| object.source_path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
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
            doors,
            details,
            object_sources,
            object_placements,
            behavior_flags,
        });
    }

    rooms.sort_by(|left, right| left.id.cmp(&right.id));
    non_room_files.sort();
    warnings.sort();

    Ok(AreaCatalog {
        schema_version: 4,
        source_commit,
        area: area.to_string(),
        status: "structured",
        rooms,
        non_room_files,
        warnings,
    })
}

fn import_npcs(
    repository: &Path,
    areas: &[&str],
    scope: &'static str,
    excluded_sources: &[&str],
    carried_item_resolutions: &[(&str, Option<&str>)],
) -> Result<NpcCatalog> {
    let source_commit = git(repository, &["rev-parse", SOURCE_REV])?
        .trim()
        .to_string();
    let mut sources = BTreeMap::new();
    let mut placements = BTreeMap::<String, usize>::new();

    for area in areas {
        let root = area_source_root(area);
        let listing = git(
            repository,
            &["ls-tree", "-r", "--name-only", SOURCE_REV, "--", &root],
        )?;
        for path in listing.lines().filter(|path| path.ends_with(".c")) {
            let source = git(repository, &["show", &format!("{SOURCE_REV}:{path}")])?;
            if is_room(&source) {
                for object in extract_object_references(&source, path) {
                    *placements.entry(object.source_path).or_insert(0) += object.quantity;
                }
            }
            if is_npc(&source) && !excluded_sources.contains(&path) {
                sources.insert(path.to_string(), source);
            }
        }
    }

    for path in placements.keys() {
        if sources.contains_key(path) || excluded_sources.contains(&path.as_str()) {
            continue;
        }
        if let Ok(source) = git(repository, &["show", &format!("{SOURCE_REV}:{path}")])
            && is_npc(&source)
        {
            sources.insert(path.clone(), source);
        }
    }

    let mut npcs = Vec::new();
    let mut warnings = Vec::new();
    let mut by_area = BTreeMap::new();
    for (path, source) in sources {
        let area = if path.starts_with("mudlib/u/cloud/") {
            "cloud".to_string()
        } else {
            path.strip_prefix("mudlib/d/")
                .or_else(|| path.strip_prefix("mudlib/"))
                .and_then(|tail| tail.split('/').next())
                .expect("NPC path has a scope")
                .to_string()
        };
        let id = source_path_to_id(&path);
        let name = extract_item_name(&source)
            .or_else(|| extract_set_string(&source, "title"))
            .unwrap_or_else(|| {
                warnings.push(format!("{path}: 无法解析 NPC 名称"));
                id.clone()
            });
        let vendor_goods = extract_vendor_goods(&source, &path);
        let inquiries = extract_inquiries(&source);
        let behavior_flags = detect_npc_behaviors(&source);
        let create = extract_function(&source, "create(").unwrap_or(&source);
        let attributes = extract_static_set_integers(
            create,
            &["str", "int", "con", "spi", "kar", "per", "cor", "cps"],
        );
        let resources = extract_static_set_integers(
            create,
            &[
                "gin",
                "eff_gin",
                "max_gin",
                "sen",
                "eff_sen",
                "max_sen",
                "force",
                "max_force",
                "force_factor",
                "atman",
                "max_atman",
                "atman_factor",
                "mana",
                "max_mana",
                "mana_factor",
            ],
        );
        let skills = extract_named_integer_calls(create, "set_skill(");
        let mappings = extract_named_string_calls(create, "map_skill(");
        let combat_apply = extract_prefixed_named_integer_calls(create, "set_temp(", "apply/");
        let combat_chat = extract_npc_combat_chat(create);
        let carried_items = extract_npc_carried_items(create, &path)
            .into_iter()
            .filter_map(|mut item| {
                match carried_item_resolutions
                    .iter()
                    .find(|(source, _)| *source == item.source_path)
                    .map(|(_, replacement)| *replacement)
                {
                    Some(Some(replacement)) => {
                        item.source_path = replacement.to_string();
                        item.item_id = source_path_to_id(replacement);
                        Some(item)
                    }
                    Some(None) => None,
                    None => Some(item),
                }
            })
            .collect();
        for flag in &behavior_flags {
            warnings.push(format!("{path}: NPC 行为待实现 [{flag}]"));
        }
        *by_area.entry(area.clone()).or_insert(0) += 1;
        npcs.push(NpcRecord {
            id,
            source_path: path.clone(),
            status: "structured",
            area,
            name,
            description: extract_set_text(&source, "long"),
            combat_exp: extract_set_integer(&source, "combat_exp"),
            attributes,
            resources,
            skills,
            mappings,
            combat_apply,
            combat_chat,
            carried_items,
            placement_count: placements.get(&path).copied().unwrap_or(0),
            vendor_goods,
            inquiries,
            behavior_flags,
        });
    }
    npcs.sort_by(|left, right| left.id.cmp(&right.id));
    warnings.sort();

    let placed = npcs.iter().filter(|npc| npc.placement_count > 0).count();
    let vendors = npcs
        .iter()
        .filter(|npc| !npc.vendor_goods.is_empty())
        .count();
    let vendor_goods = npcs.iter().map(|npc| npc.vendor_goods.len()).sum();
    let inquiry_npcs = npcs.iter().filter(|npc| !npc.inquiries.is_empty()).count();
    let inquiry_topics = npcs.iter().map(|npc| npc.inquiries.len()).sum();
    let static_inquiries = npcs
        .iter()
        .flat_map(|npc| &npc.inquiries)
        .filter(|inquiry| !inquiry.scripted && inquiry.response.is_some())
        .count();
    let scripted_inquiries = inquiry_topics - static_inquiries;
    let runtime_inquiry_npcs = npcs
        .iter()
        .filter(|npc| {
            npc.placement_count > 0
                && npc
                    .inquiries
                    .iter()
                    .any(|inquiry| !inquiry.scripted && inquiry.response.is_some())
        })
        .count();
    let runtime_inquiries = npcs
        .iter()
        .filter(|npc| npc.placement_count > 0)
        .flat_map(|npc| &npc.inquiries)
        .filter(|inquiry| !inquiry.scripted && inquiry.response.is_some())
        .count();
    let runtime_inquiry_references = npcs
        .iter()
        .map(|npc| {
            npc.placement_count
                * npc
                    .inquiries
                    .iter()
                    .filter(|inquiry| !inquiry.scripted && inquiry.response.is_some())
                    .count()
        })
        .sum();
    let combat_profiles = npcs
        .iter()
        .filter(|npc| {
            npc.combat_exp.is_some()
                || !npc.attributes.is_empty()
                || !npc.skills.is_empty()
                || !npc.combat_apply.is_empty()
        })
        .count();
    let combat_skill_entries = npcs.iter().map(|npc| npc.skills.len()).sum();
    let combat_mapping_entries = npcs.iter().map(|npc| npc.mappings.len()).sum();
    let combat_apply_entries = npcs.iter().map(|npc| npc.combat_apply.len()).sum();
    let combat_chat_npcs = npcs.iter().filter(|npc| npc.combat_chat.is_some()).count();
    let combat_chat_entries = npcs
        .iter()
        .filter_map(|npc| npc.combat_chat.as_ref())
        .map(|chat| chat.entries.len())
        .sum();
    let carried_item_npcs = npcs
        .iter()
        .filter(|npc| !npc.carried_items.is_empty())
        .count();
    let carried_item_entries = npcs.iter().map(|npc| npc.carried_items.len()).sum();
    let worn_item_entries = npcs
        .iter()
        .flat_map(|npc| &npc.carried_items)
        .filter(|item| item.state == "worn")
        .count();
    let wielded_item_entries = npcs
        .iter()
        .flat_map(|npc| &npc.carried_items)
        .filter(|item| item.state == "wielded")
        .count();
    let placed_combat_npcs = npcs.iter().filter(|npc| npc.placement_count > 0).count();
    let placed_combat_chat_npcs = npcs
        .iter()
        .filter(|npc| npc.placement_count > 0 && npc.combat_chat.is_some())
        .count();
    let placed_carried_item_npcs = npcs
        .iter()
        .filter(|npc| npc.placement_count > 0 && !npc.carried_items.is_empty())
        .count();

    Ok(NpcCatalog {
        schema_version: 4,
        source_commit,
        scope,
        status: "structured",
        summary: NpcSummary {
            total: npcs.len(),
            placed,
            vendors,
            vendor_goods,
            inquiry_npcs,
            inquiry_topics,
            static_inquiries,
            scripted_inquiries,
            runtime_inquiry_npcs,
            runtime_inquiries,
            runtime_inquiry_references,
            combat_profiles,
            combat_skill_entries,
            combat_mapping_entries,
            combat_apply_entries,
            combat_chat_npcs,
            combat_chat_entries,
            carried_item_npcs,
            carried_item_entries,
            worn_item_entries,
            wielded_item_entries,
            placed_combat_npcs,
            placed_combat_chat_npcs,
            placed_carried_item_npcs,
            by_area,
        },
        npcs,
        warnings,
    })
}

fn import_items(repository: &Path) -> Result<ItemCatalog> {
    const ITEM_INHERITS: &str = "inherit (ITEM|COMBINED_ITEM|MONEY|EQUIP|CLOTH|HEAD|BOOTS|WAIST|NECK|SURCOAT|SHIELD|WRISTS|FINGER|HANDS|ARMOR|SWORD|BLADE|HAMMER|THROWING|STAFF|WHIP|DAGGER|AXE|FORK|POWDER|PILL|F_FOOD|F_LIQUID);|inherit \"/std/(item|weapon|armor)";
    let source_commit = git(repository, &["rev-parse", SOURCE_REV])?
        .trim()
        .to_string();
    let listing = git(
        repository,
        &[
            "grep",
            "-I",
            "-l",
            "-E",
            ITEM_INHERITS,
            SOURCE_REV,
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
            SOURCE_REV,
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
    let revision_prefix = format!("{SOURCE_REV}:");
    for entry in listing.lines() {
        let path = entry.strip_prefix(&revision_prefix).unwrap_or(entry);
        if !path.ends_with(".c") || !is_player_item_path(path) {
            continue;
        }
        let source = git(repository, &["show", &format!("{SOURCE_REV}:{path}")])?;
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
    let source_commit = git(repository, &["rev-parse", SOURCE_REV])?
        .trim()
        .to_string();
    let listing = git(
        repository,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            SOURCE_REV,
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
        let source = git(repository, &["show", &format!("{SOURCE_REV}:{path}")])?;
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
        let display_name = git(repository, &["show", &format!("{SOURCE_REV}:{doc_path}")])
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
            SOURCE_REV,
            "--",
            "mudlib/daemon/class",
        ],
    )?;
    let mut teachers = Vec::new();
    for path in listing.lines().filter(|path| path.ends_with("/master.c")) {
        let source = git(repository, &["show", &format!("{SOURCE_REV}:{path}")])?;
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

fn extract_static_set_integers(source: &str, keys: &[&str]) -> BTreeMap<String, i32> {
    keys.iter()
        .filter_map(|key| extract_set_integer(source, key).map(|value| ((*key).into(), value)))
        .collect()
}

fn extract_prefixed_named_integer_calls(
    source: &str,
    marker: &str,
    prefix: &str,
) -> BTreeMap<String, i32> {
    extract_named_integer_calls(source, marker)
        .into_iter()
        .filter_map(|(key, value)| key.strip_prefix(prefix).map(|key| (key.to_string(), value)))
        .collect()
}

fn extract_npc_carried_items(source: &str, owner_path: &str) -> Vec<NpcCarriedItemRecord> {
    let code = strip_lpc_comments(source);
    let mut items = Vec::new();
    let mut remaining = code.as_str();
    while let Some(start) = remaining.find("carry_object(") {
        let tail = &remaining[start + "carry_object(".len()..];
        let Some(end) = tail.find(')') else {
            break;
        };
        let call = &tail[..end];
        let suffix = tail[end + 1..].trim_start();
        let state = if suffix.starts_with("->wear()") {
            "worn"
        } else if suffix.starts_with("->wield()") {
            "wielded"
        } else {
            "carried"
        };
        if let Some(reference) = extract_path_reference(call)
            && let Some(source_path) = resolve_source_path(&reference, owner_path)
        {
            items.push(NpcCarriedItemRecord {
                item_id: source_path_to_id(&source_path),
                source_path,
                state,
            });
        }
        remaining = &tail[end + 1..];
    }
    items
}

fn strip_lpc_comments(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        String,
        LineComment,
        BlockComment,
    }

    let mut output = String::with_capacity(source.len());
    let mut state = State::Code;
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        match state {
            State::Code if character == '"' => {
                output.push(character);
                state = State::String;
            }
            State::Code if character == '/' && characters.peek() == Some(&'/') => {
                characters.next();
                state = State::LineComment;
            }
            State::Code if character == '/' && characters.peek() == Some(&'*') => {
                characters.next();
                state = State::BlockComment;
            }
            State::Code => output.push(character),
            State::String if character == '\\' => {
                output.push(character);
                if let Some(escaped) = characters.next() {
                    output.push(escaped);
                }
            }
            State::String => {
                output.push(character);
                if character == '"' {
                    state = State::Code;
                }
            }
            State::LineComment if character == '\n' => {
                output.push(character);
                state = State::Code;
            }
            State::LineComment => {}
            State::BlockComment if character == '*' && characters.peek() == Some(&'/') => {
                characters.next();
                state = State::Code;
            }
            State::BlockComment if character == '\n' => output.push(character),
            State::BlockComment => {}
        }
    }
    output
}

fn extract_npc_combat_chat(source: &str) -> Option<NpcCombatChatRecord> {
    let value = find_set_value(source, "chat_msg_combat")?;
    let start = value.find("({")? + 2;
    let end = value.get(start..)?.find("})")? + start;
    let entries = split_lpc_top_level(value.get(start..end)?, b',')
        .into_iter()
        .filter_map(parse_npc_combat_chat_entry)
        .collect();
    let chance_value = find_set_value(source, "chat_chance_combat")
        .or_else(|| find_set_value(source, "chat_chance"));
    let chance = chance_value.and_then(parse_leading_integer);
    let chance_expression = chance_value.and_then(|value| {
        chance.is_none().then(|| {
            normalize_fragment(
                value
                    .get(..value.find(");").unwrap_or(value.len()))
                    .unwrap_or(value),
            )
        })
    });
    Some(NpcCombatChatRecord {
        chance,
        chance_expression,
        entries,
    })
}

fn parse_npc_combat_chat_entry(source: &str) -> Option<NpcCombatChatEntryRecord> {
    let source = source.trim();
    if source.is_empty() {
        return None;
    }
    if source.contains("(:") {
        let kind = if source.contains("cast_spell") {
            "spell"
        } else if source.contains("exert_function") {
            "force"
        } else if source.contains("perform_action") {
            "perform"
        } else if source.contains("random_move") {
            "movement"
        } else if source.contains("destruct") {
            "destruct"
        } else if source.contains("command") {
            "command"
        } else {
            "callback"
        };
        return Some(NpcCombatChatEntryRecord {
            kind,
            value: normalize_fragment(source),
        });
    }

    let value = extract_quoted_strings(source)
        .concat()
        .replace("\\n", "\n")
        .trim()
        .to_string();
    (!value.is_empty()).then_some(NpcCombatChatEntryRecord {
        kind: "text",
        value,
    })
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
    let tail = find_call_arguments(source, "set_name")?;
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
    let value = find_set_value(source, key)?;
    let value = value.strip_prefix('"')?;
    Some(value.get(..value.find('"')?)?.trim().to_string())
}

fn extract_set_integer(source: &str, key: &str) -> Option<i32> {
    parse_leading_integer(find_set_value(source, key)?)
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
    let block = mapping_block(source, mapping)?;
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
        matches!(
            line,
            "inherit ROOM;" | "inherit BANK;" | "inherit HOCKSHOP;"
        ) || line.starts_with("inherit ROOM ")
            || line.contains("inherit \"/std/room\"")
    })
}

fn is_npc(source: &str) -> bool {
    source.lines().any(|line| {
        let line = line.trim();
        line == "inherit NPC;"
            || line.starts_with("inherit NPC ")
            || line.contains("inherit \"/std/char/npc\"")
    })
}

fn find_set_call<F>(source: &str, key_matches: F) -> Option<(&str, &str)>
where
    F: Fn(&str) -> bool,
{
    for (start, _) in source.match_indices("set") {
        let tail = source.get(start + "set".len()..)?.trim_start();
        let Some(tail) = tail.strip_prefix('(') else {
            continue;
        };
        let tail = tail.trim_start();
        let Some(tail) = tail.strip_prefix('"') else {
            continue;
        };
        let Some(key_end) = tail.find('"') else {
            continue;
        };
        let key = tail.get(..key_end)?;
        if !key_matches(key) {
            continue;
        }
        let tail = tail.get(key_end + 1..)?.trim_start();
        let Some(value) = tail.strip_prefix(',') else {
            continue;
        };
        return Some((key, value.trim_start()));
    }
    None
}

fn find_set_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    find_set_call(source, |candidate| candidate == key).map(|(_, value)| value)
}

fn extract_long(source: &str) -> Option<String> {
    let value = find_set_value(source, "long")?;
    if let Some(tail) = value.strip_prefix("@LONG") {
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
    let value = value.strip_prefix('"')?;
    Some(value.get(..value.find('"')?)?.trim().to_string())
}

fn extract_set_text(source: &str, key: &str) -> Option<String> {
    let value = find_set_value(source, key)?;
    let expression = value.get(..value.find(");").unwrap_or(value.len()))?;
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
        Some(fragments.concat().replace("\\n", "\n").trim().to_string())
    }
}

fn extract_exits(source: &str, area: &str, source_path: &str) -> Vec<ExitRecord> {
    let mut exits = Vec::new();
    if let Some(block) = mapping_block(source, "exits") {
        for line in block.lines() {
            if let Some((direction, source_target)) = parse_mapping_entry(line)
                && let Some(target) = normalize_target(&source_target, source_path)
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

    for line in source.lines() {
        let Some((key, value)) = find_set_call(line, |key| key.starts_with("exits/")) else {
            continue;
        };
        let Some(direction) = key.strip_prefix("exits/") else {
            continue;
        };
        let Some(source_target) = extract_path_reference(value) else {
            continue;
        };
        let Some(target) = normalize_target(&source_target, source_path) else {
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

fn extract_doors(source: &str) -> Vec<DoorRecord> {
    let code = strip_lpc_comments(source);
    let mut doors = Vec::new();
    let mut remaining = code.as_str();
    while let Some(start) = remaining.find("create_door") {
        let tail = &remaining[start + "create_door".len()..];
        let Some(open) = tail.find('(') else {
            break;
        };
        let arguments = &tail[open + 1..];
        let Some(end) = arguments.find(");") else {
            break;
        };
        let call = &arguments[..end];
        let strings = extract_quoted_strings(call);
        if strings.len() >= 3 {
            doors.push(DoorRecord {
                direction: strings[0].clone(),
                name: strings[1].clone(),
                reverse_direction: strings[2].clone(),
                initially_closed: call.contains("DOOR_CLOSED"),
            });
        }
        remaining = &arguments[end + 2..];
    }
    doors
}

fn extract_room_details(source: &str) -> Vec<RoomDetailRecord> {
    let code = strip_lpc_comments(source);
    let Some(block) = mapping_block(&code, "item_desc") else {
        return Vec::new();
    };
    let block = block.trim();
    let Some(inner) = block
        .strip_prefix("([")
        .and_then(|value| value.strip_suffix("])"))
    else {
        return Vec::new();
    };

    let mut details = split_lpc_top_level(inner, b',')
        .into_iter()
        .filter_map(|entry| {
            let separator = find_lpc_top_level(entry, b':')?;
            let key = parse_lpc_string_expression(entry.get(..separator)?.trim())?;
            let value = entry.get(separator + 1..)?.trim();
            let door_direction = value
                .contains("look_door")
                .then(|| extract_quoted_strings(value).last().cloned())
                .flatten();
            let description = if door_direction.is_some() {
                None
            } else {
                parse_room_detail_description(&code, value)
            };
            (description.is_some() || door_direction.is_some()).then_some(RoomDetailRecord {
                key,
                description,
                door_direction,
            })
        })
        .collect::<Vec<_>>();
    details.sort_by(|left, right| left.key.cmp(&right.key));
    details
}

fn parse_room_detail_description(source: &str, value: &str) -> Option<String> {
    if let Some(heredoc) = parse_lpc_heredoc(value) {
        return Some(heredoc);
    }
    if let Some(description) = parse_lpc_string_expression(value) {
        return Some(description);
    }

    let closure = value.strip_prefix("(:")?.strip_suffix(":)")?.trim();
    if let Some(description) = parse_lpc_string_expression(closure) {
        return Some(description);
    }
    let function_name = closure
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .find(|fragment| !fragment.is_empty() && *fragment != "this_object")?;
    let function = extract_function_definition(source, function_name)?;
    extract_quoted_strings(function)
        .last()
        .map(|description| description.replace("\\n", "\n").trim().to_string())
        .filter(|description| !description.is_empty())
}

fn parse_lpc_heredoc(value: &str) -> Option<String> {
    let value = value.strip_prefix('@')?;
    let marker_end = value.find(['\r', '\n'])?;
    let marker = value.get(..marker_end)?.trim();
    let body = value.get(marker_end..)?.trim_start_matches(['\r', '\n']);
    let end_marker = format!("\n{marker}");
    let end = body.find(&end_marker)?;
    Some(
        body.get(..end)?
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string(),
    )
}

fn extract_function_definition<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    for (start, _) in source
        .match_indices(name)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let tail = source.get(start + name.len()..)?.trim_start();
        if !tail.starts_with('(') {
            continue;
        }
        let close = find_matching_delimiter(tail, b'(', b')')?;
        let body = tail.get(close + 1..)?.trim_start();
        if !body.starts_with('{') {
            continue;
        }
        let end = find_matching_delimiter(body, b'{', b'}')?;
        return body.get(..=end);
    }
    None
}

fn find_matching_delimiter(source: &str, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0_u32;
    for (index, byte) in source.bytes().enumerate() {
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

#[cfg(test)]
fn extract_objects(source: &str, source_path: &str) -> Vec<String> {
    extract_object_references(source, source_path)
        .into_iter()
        .map(|object| object.source_path)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn extract_object_references(source: &str, source_path: &str) -> Vec<ObjectReference> {
    let Some(block) = mapping_block(source, "objects") else {
        return Vec::new();
    };
    block
        .lines()
        .filter_map(|line| {
            let reference = extract_path_reference(line)?;
            let source_path = resolve_source_path(&reference, source_path)?;
            let quantity = line
                .rsplit_once(':')
                .and_then(|(_, value)| parse_leading_integer(value))
                .unwrap_or(1)
                .max(1) as usize;
            Some(ObjectReference {
                source_path,
                quantity,
            })
        })
        .collect()
}

fn extract_inquiries(source: &str) -> Vec<InquiryRecord> {
    let Some(block) = mapping_block(source, "inquiry") else {
        return Vec::new();
    };
    let Some(start) = block.find("([") else {
        return Vec::new();
    };
    let Some(end) = block.rfind("])") else {
        return Vec::new();
    };

    split_lpc_top_level(&block[start + 2..end], b',')
        .into_iter()
        .filter_map(|entry| {
            let separator = find_lpc_top_level(entry, b':')?;
            let topic = parse_lpc_string_expression(entry.get(..separator)?.trim())?;
            let value = entry.get(separator + 1..)?.trim();
            let response = parse_lpc_string_expression(value).or_else(|| {
                value
                    .strip_prefix("(:")?
                    .strip_suffix(":)")
                    .and_then(|inner| parse_lpc_string_expression(inner.trim()))
            });
            Some(InquiryRecord {
                topic,
                scripted: response.is_none(),
                response,
            })
        })
        .collect()
}

fn split_lpc_top_level(source: &str, delimiter: u8) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut cursor = 0;
    while let Some(index) = find_lpc_top_level(source.get(cursor..).unwrap_or_default(), delimiter)
    {
        let absolute = cursor + index;
        parts.push(&source[start..absolute]);
        start = absolute + 1;
        cursor = start;
    }
    parts.push(&source[start..]);
    parts
}

fn find_lpc_top_level(source: &str, delimiter: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut parens = 0_u32;
    let mut brackets = 0_u32;
    let mut braces = 0_u32;
    let mut quoted = false;
    let mut escaped = false;

    for (index, byte) in bytes.iter().copied().enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'{' => braces += 1,
            b'}' => braces = braces.saturating_sub(1),
            _ if byte == delimiter && parens == 0 && brackets == 0 && braces == 0 => {
                return Some(index);
            }
            _ => {}
        }
    }
    None
}

fn parse_lpc_string_expression(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut fragments = Vec::new();

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        if bytes[cursor] != b'"' {
            return None;
        }
        cursor += 1;
        let mut fragment = String::new();
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'"' => {
                    cursor += 1;
                    break;
                }
                b'\\' if cursor + 1 < bytes.len() => {
                    cursor += 1;
                    match bytes[cursor] {
                        b'n' => {
                            fragment.push('\n');
                            cursor += 1;
                        }
                        b'r' => {
                            fragment.push('\r');
                            cursor += 1;
                        }
                        b't' => {
                            fragment.push('\t');
                            cursor += 1;
                        }
                        b'"' => {
                            fragment.push('"');
                            cursor += 1;
                        }
                        b'\\' => {
                            fragment.push('\\');
                            cursor += 1;
                        }
                        _ => {
                            let character = source.get(cursor..)?.chars().next()?;
                            fragment.push(character);
                            cursor += character.len_utf8();
                        }
                    }
                }
                _ => {
                    let tail = source.get(cursor..)?;
                    let character = tail.chars().next()?;
                    fragment.push(character);
                    cursor += character.len_utf8();
                }
            }
        }
        fragments.push(fragment);
    }

    (!fragments.is_empty()).then(|| fragments.concat().trim().to_string())
}

fn extract_vendor_goods(source: &str, source_path: &str) -> Vec<VendorGoodRecord> {
    let Some(block) = mapping_block(source, "vendor_goods") else {
        return Vec::new();
    };
    let mut goods = BTreeMap::new();
    for line in block.lines() {
        let Some(reference) = extract_path_reference(line) else {
            continue;
        };
        let Some(item_path) = resolve_source_path(&reference, source_path) else {
            continue;
        };
        goods.entry(item_path.clone()).or_insert(VendorGoodRecord {
            item_id: source_path_to_id(&item_path),
            source_path: item_path,
        });
    }
    goods.into_values().collect()
}

fn mapping_block<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let tail = find_set_value(source, key)?;
    let end = tail.find("])")? + 2;
    tail.get(..end)
}

fn parse_mapping_entry(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let line = line.strip_prefix("([").unwrap_or(line).trim_start();
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
    let mut remaining = line;
    while let Some(start) = remaining.find('"') {
        let quoted = remaining.get(start + 1..)?;
        let end = quoted.find('"')?;
        let value = quoted.get(..end)?;
        if [
            "/d/", "/u/", "/obj/", "/daemon/", "d/", "u/", "obj/", "daemon/",
        ]
        .iter()
        .any(|prefix| value.starts_with(prefix))
        {
            return Some(value.to_string());
        }
        remaining = quoted.get(end + 1..)?;
    }
    None
}

fn resolve_source_path(reference: &str, owner_path: &str) -> Option<String> {
    let path = if let Some(local) = reference.strip_prefix("__DIR__") {
        let source_dir = owner_path.rsplit_once('/')?.0;
        format!("{source_dir}/{local}")
    } else if reference.starts_with("mudlib/") {
        reference.to_string()
    } else if reference.starts_with('/') {
        format!("mudlib{reference}")
    } else {
        format!("mudlib/{reference}")
    };
    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            value => components.push(value),
        }
    }
    let mut normalized = components.join("/");
    if !normalized.ends_with(".c") {
        normalized.push_str(".c");
    }
    Some(normalized)
}

fn normalize_target(source_target: &str, source_path: &str) -> Option<String> {
    let path = resolve_source_path(source_target, source_path)?;
    path.starts_with("mudlib/")
        .then(|| source_path_to_id(&path))
}

fn source_path_to_id(path: &str) -> String {
    let path = path
        .strip_prefix("mudlib/d/")
        .or_else(|| path.strip_prefix("mudlib/"))
        .unwrap_or(path);
    path.strip_suffix(".c").unwrap_or(path).replace('/', ".")
}

fn find_call_arguments<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    source.match_indices(name).find_map(|(start, _)| {
        source
            .get(start + name.len()..)?
            .trim_start()
            .strip_prefix('(')
    })
}

fn contains_call(source: &str, name: &str) -> bool {
    find_call_arguments(source, name).is_some()
}

fn detect_behaviors(source: &str) -> Vec<String> {
    let mut behaviors = Vec::new();
    for (name, flag) in [
        ("add_action", "custom_command"),
        ("valid_leave", "conditional_exit"),
        ("create_door", "door"),
        ("call_out", "timed_behavior"),
        ("receive_damage", "environment_damage"),
        ("random", "random_behavior"),
    ] {
        if contains_call(source, name) {
            behaviors.push(flag.to_string());
        }
    }
    if find_set_call(source, |key| key.starts_with("exits/")).is_some() {
        behaviors.push("dynamic_exit".into());
    }
    if find_set_value(source, "item_desc").is_some() {
        behaviors.push("item_interaction".into());
    }
    behaviors
}

fn detect_npc_behaviors(source: &str) -> Vec<String> {
    let mut behaviors = BTreeSet::new();
    for (key, flag) in [
        ("inquiry", "inquiry"),
        ("chat_msg", "ambient_chat"),
        ("chat_msg_combat", "combat_chat"),
    ] {
        if find_set_value(source, key).is_some() {
            behaviors.insert(flag);
        }
    }
    for (function, flag) in [
        ("accept_object", "object_exchange"),
        ("receive_object", "object_exchange"),
        ("accept_fight", "fight_gate"),
        ("accept_kill", "kill_gate"),
        ("add_action", "custom_command"),
        ("call_out", "timed_behavior"),
        ("die", "death_hook"),
    ] {
        if contains_call(source, function) {
            behaviors.insert(flag);
        }
    }
    if source.contains("random_move") {
        behaviors.insert("random_movement");
    }
    if ["recognize_apprentice", "accept_apprentice", "create_family"]
        .iter()
        .any(|function| contains_call(source, function))
    {
        behaviors.insert("apprenticeship");
    }
    if find_call_arguments(source, "set_name").is_some_and(|arguments| {
        arguments
            .get(..arguments.find(',').unwrap_or(arguments.len()))
            .is_some_and(|name| name.contains("random("))
    }) {
        behaviors.insert("random_identity");
    }
    behaviors.into_iter().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_single_line_exit_mapping() {
        let source = r#"void create() { set("exits",(["out":__DIR__"tongbhill",])); }"#;
        let exits = extract_exits(source, "choyin", "mudlib/d/choyin/stove.c");
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].direction, "out");
        assert_eq!(exits[0].target, "choyin.tongbhill");
        assert_eq!(exits[0].source_target, "__DIR__tongbhill");
        assert!(exits[0].internal);
        assert!(!exits[0].dynamic);
    }

    #[test]
    fn parses_static_and_dynamic_exits() {
        let source = r#"
set("exits", ([
  "north" : __DIR__"road2",
  "south" : "/d/city/nroad2",
]));
set("exits/west",__DIR__"valley2");
"#;
        let exits = extract_exits(source, "village", "mudlib/d/village/road1.c");
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
        assert_eq!(
            normalize_target("__DIR__xiaowu", "mudlib/d/city/shangshu/road1.c").as_deref(),
            Some("city.shangshu.xiaowu")
        );
        assert_eq!(
            normalize_target("__DIR__../road", "mudlib/d/canyon/bamboo/train.c").as_deref(),
            Some("canyon.road")
        );
    }

    #[test]
    fn parses_source_doors_and_ignores_commented_calls() {
        let source = r#"
create_door ("east", "大铁门", "west", DOOR_CLOSED);
create_door("north", "帘子", "south");
// create_door("out", "假门", "enter");
/* create_door("down", "旧门", "up", DOOR_CLOSED); */
"#;
        let doors = extract_doors(source);
        assert_eq!(doors.len(), 2);
        assert_eq!(doors[0].direction, "east");
        assert_eq!(doors[0].name, "大铁门");
        assert_eq!(doors[0].reverse_direction, "west");
        assert!(doors[0].initially_closed);
        assert_eq!(doors[1].direction, "north");
        assert!(!doors[1].initially_closed);
    }

    #[test]
    fn parses_static_callback_and_door_room_details() {
        let source = r#"
set("item_desc", ([
    "sign": @TEXT
告示第一行
告示第二行
TEXT
    ,
    "book": (: "一本旧书。\n" :),
    "notice": (: look_notice :),
    "door": (: look_door, "east" :),
]));
string look_notice(object me)
{
    if (wizardp(me)) return "管理员告示。\n";
    return "行人告示。\n";
}
"#;
        let details = extract_room_details(source);
        assert_eq!(details.len(), 4);
        assert_eq!(details[0].key, "book");
        assert_eq!(details[0].description.as_deref(), Some("一本旧书。"));
        assert_eq!(details[1].key, "door");
        assert_eq!(details[1].door_direction.as_deref(), Some("east"));
        assert_eq!(details[2].key, "notice");
        assert_eq!(details[2].description.as_deref(), Some("行人告示。"));
        assert_eq!(details[3].key, "sign");
        assert_eq!(
            details[3].description.as_deref(),
            Some("告示第一行\n告示第二行")
        );
    }

    #[test]
    fn generated_area_catalogs_retain_fixed_source_doors_and_details() {
        let catalogs = [
            (
                "village",
                include_str!("../../migration/catalog/village.json"),
                4,
                4,
                4,
                8,
                27,
            ),
            (
                "city",
                include_str!("../../migration/catalog/city.json"),
                15,
                13,
                21,
                38,
                51,
            ),
            (
                "snow",
                include_str!("../../migration/catalog/snow.json"),
                11,
                9,
                11,
                25,
                44,
            ),
            (
                "temple",
                include_str!("../../migration/catalog/temple.json"),
                10,
                5,
                5,
                10,
                10,
            ),
            (
                "canyon",
                include_str!("../../migration/catalog/canyon.json"),
                0,
                5,
                5,
                11,
                15,
            ),
            (
                "oldpine",
                include_str!("../../migration/catalog/oldpine.json"),
                0,
                11,
                14,
                19,
                37,
            ),
            (
                "goathill",
                include_str!("../../migration/catalog/goathill.json"),
                0,
                0,
                0,
                10,
                18,
            ),
            (
                "choyin",
                include_str!("../../migration/catalog/choyin.json"),
                4,
                8,
                11,
                39,
                83,
            ),
            (
                "chuenyu",
                include_str!("../../migration/catalog/chuenyu.json"),
                8,
                8,
                23,
                27,
                59,
            ),
            (
                "green",
                include_str!("../../migration/catalog/green.json"),
                6,
                10,
                10,
                14,
                20,
            ),
            (
                "sanyen",
                include_str!("../../migration/catalog/sanyen.json"),
                2,
                2,
                3,
                9,
                11,
            ),
            (
                "waterfog",
                include_str!("../../migration/catalog/waterfog.json"),
                0,
                2,
                2,
                10,
                15,
            ),
            (
                "latemoon",
                include_str!("../../migration/catalog/latemoon.json"),
                28,
                6,
                6,
                46,
                55,
            ),
            (
                "death",
                include_str!("../../migration/catalog/death.json"),
                0,
                1,
                1,
                3,
                6,
            ),
            (
                "graveyard",
                include_str!("../../migration/catalog/graveyard.json"),
                0,
                0,
                0,
                0,
                0,
            ),
            (
                "jail",
                include_str!("../../migration/catalog/jail.json"),
                0,
                0,
                0,
                0,
                0,
            ),
            (
                "cloud",
                include_str!("../../migration/catalog/cloud.json"),
                4,
                14,
                15,
                29,
                56,
            ),
        ];

        for (
            area,
            json,
            expected_doors,
            expected_detail_rooms,
            expected_details,
            expected_object_placements,
            expected_object_instances,
        ) in catalogs
        {
            let catalog: serde_json::Value = serde_json::from_str(json).unwrap();
            assert_eq!(catalog["schema_version"], 4, "{area}");
            assert_eq!(
                catalog["rooms"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|room| room["doors"].as_array().unwrap().len())
                    .sum::<usize>(),
                expected_doors,
                "{area}"
            );
            let detail_rooms = catalog["rooms"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|room| !room["details"].as_array().unwrap().is_empty())
                .collect::<Vec<_>>();
            assert_eq!(detail_rooms.len(), expected_detail_rooms, "{area}");
            assert_eq!(
                detail_rooms
                    .iter()
                    .map(|room| room["details"].as_array().unwrap().len())
                    .sum::<usize>(),
                expected_details,
                "{area}"
            );
            let object_placements = catalog["rooms"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|room| room["object_placements"].as_array().unwrap());
            let (placements, instances) =
                object_placements.fold((0usize, 0u64), |(placements, instances), object| {
                    (
                        placements + 1,
                        instances + object["quantity"].as_u64().unwrap(),
                    )
                });
            assert_eq!(placements, expected_object_placements, "{area}");
            assert_eq!(instances, expected_object_instances, "{area}");
        }
    }

    #[test]
    fn parses_heredoc_description() {
        let source = r#"
set ("short", "谷物加工厂");
set ("long", @LONG
第一行
第二行
LONG
);
set ("exits", ([
    "west": "/d/snow/mstreet2",
]));
set ("item_desc", ([ "sign": "告示" ]));
add_action ("do_work", "work");
"#;
        assert_eq!(
            extract_set_string(source, "short").as_deref(),
            Some("谷物加工厂")
        );
        assert_eq!(extract_long(source).as_deref(), Some("第一行\n第二行"));
        assert_eq!(
            extract_exits(source, "snow", "mudlib/d/snow/workplace.c").len(),
            1
        );
        assert_eq!(
            detect_behaviors(source),
            ["custom_command", "item_interaction"]
        );
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
    fn parses_static_and_scripted_npc_inquiries() {
        let source = r#"
set("inquiry", ([
    "静态" : "第一句" "第二句\n",
    "静态回调" : (: "直接回复。\n" :),
    "函数" : (: ask_me :),
    "序列" : ({ "开场", (: command, "nod" :), "收尾" }),
]));
"#;
        let inquiries = extract_inquiries(source);
        assert_eq!(inquiries.len(), 4);
        assert_eq!(inquiries[0].topic, "静态");
        assert_eq!(inquiries[0].response.as_deref(), Some("第一句第二句"));
        assert!(!inquiries[0].scripted);
        assert_eq!(inquiries[1].response.as_deref(), Some("直接回复。"));
        assert!(!inquiries[1].scripted);
        assert!(inquiries[2].scripted);
        assert!(inquiries[2].response.is_none());
        assert!(inquiries[3].scripted);
        assert!(inquiries[3].response.is_none());
    }

    #[test]
    fn parses_npc_combat_profile_and_typed_chat() {
        let source = r#"
set("str", 29);
set("max_force", 1800);
set_skill("sword", 80);
map_skill("sword", "six-chaos-sword");
set_temp("apply/attack", 90);
set("chat_chance_combat", random(40));
set("chat_msg_combat", ({
    CYN "第一句。\n" NOR,
    (: cast_spell, "drainerbolt" :),
    (: command, "surrender" :),
}) );
"#;
        assert_eq!(extract_static_set_integers(source, &["str"])["str"], 29);
        assert_eq!(
            extract_static_set_integers(source, &["max_force"])["max_force"],
            1800
        );
        assert_eq!(
            extract_named_integer_calls(source, "set_skill(")["sword"],
            80
        );
        assert_eq!(
            extract_named_string_calls(source, "map_skill(")["sword"],
            "six-chaos-sword"
        );
        assert_eq!(
            extract_prefixed_named_integer_calls(source, "set_temp(", "apply/")["attack"],
            90
        );

        let chat = extract_npc_combat_chat(source).unwrap();
        assert_eq!(chat.chance, None);
        assert_eq!(chat.chance_expression.as_deref(), Some("random(40)"));
        assert_eq!(chat.entries.len(), 3);
        assert_eq!(chat.entries[0].kind, "text");
        assert_eq!(chat.entries[0].value, "第一句。");
        assert_eq!(chat.entries[1].kind, "spell");
        assert_eq!(chat.entries[2].kind, "command");

        let ambient_chance_chat = extract_npc_combat_chat(
            r#"
set("chat_chance", 15);
set("chat_msg_combat", ({ "战斗消息。" }));
"#,
        )
        .unwrap();
        assert_eq!(ambient_chance_chat.chance, Some(15));
        assert_eq!(ambient_chance_chat.chance_expression, None);

        let carried = extract_npc_carried_items(
            r#"
carry_object("/obj/cloth")->wear();
carry_object(__DIR__"obj/blade")->wield();
carry_object("/obj/old_book");
// carry_object("/obj/commented");
/* carry_object("/obj/commented_too"); */
"#,
            "mudlib/d/snow/npc/example.c",
        );
        assert_eq!(carried.len(), 3);
        assert_eq!(carried[0].item_id, "obj.cloth");
        assert_eq!(carried[0].state, "worn");
        assert_eq!(carried[1].item_id, "snow.npc.obj.blade");
        assert_eq!(carried[1].state, "wielded");
        assert_eq!(carried[2].state, "carried");
    }

    #[test]
    fn generated_m4_npc_catalog_matches_fixed_source_baseline() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../migration/catalog/npcs-m4.json")).unwrap();
        let npcs = catalog["npcs"].as_array().unwrap();
        let behavior_total: u64 = npcs
            .iter()
            .flat_map(|npc| npc["behavior_flags"].as_array().unwrap())
            .count() as u64;

        assert_eq!(catalog["schema_version"], 4);
        assert_eq!(
            catalog["source_commit"],
            "87bba6bd2249beec8424b0d6623486a0dd1f7b30"
        );
        assert_eq!(npcs.len(), 73);
        assert_eq!(catalog["summary"]["placed"], 59);
        assert_eq!(catalog["summary"]["vendors"], 9);
        assert_eq!(catalog["summary"]["vendor_goods"], 27);
        assert_eq!(catalog["summary"]["inquiry_npcs"], 30);
        assert_eq!(catalog["summary"]["inquiry_topics"], 75);
        assert_eq!(catalog["summary"]["static_inquiries"], 57);
        assert_eq!(catalog["summary"]["scripted_inquiries"], 18);
        assert_eq!(catalog["summary"]["runtime_inquiry_npcs"], 21);
        assert_eq!(catalog["summary"]["runtime_inquiries"], 50);
        assert_eq!(catalog["summary"]["runtime_inquiry_references"], 95);
        assert_eq!(catalog["summary"]["combat_profiles"], 72);
        assert_eq!(catalog["summary"]["combat_skill_entries"], 316);
        assert_eq!(catalog["summary"]["combat_mapping_entries"], 94);
        assert_eq!(catalog["summary"]["combat_apply_entries"], 21);
        assert_eq!(catalog["summary"]["combat_chat_npcs"], 25);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 87);
        assert_eq!(catalog["summary"]["carried_item_npcs"], 55);
        assert_eq!(catalog["summary"]["carried_item_entries"], 93);
        assert_eq!(catalog["summary"]["worn_item_entries"], 56);
        assert_eq!(catalog["summary"]["wielded_item_entries"], 32);
        assert_eq!(catalog["summary"]["placed_combat_npcs"], 59);
        assert_eq!(catalog["summary"]["placed_combat_chat_npcs"], 16);
        assert_eq!(catalog["summary"]["placed_carried_item_npcs"], 42);
        assert_eq!(behavior_total, 156);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 156);
    }

    #[test]
    fn generated_m5_npc_catalog_matches_fixed_source_baseline() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../migration/catalog/npcs-m5.json")).unwrap();
        let npcs = catalog["npcs"].as_array().unwrap();
        let behavior_total = npcs
            .iter()
            .map(|npc| npc["behavior_flags"].as_array().unwrap().len())
            .sum::<usize>();

        assert_eq!(catalog["schema_version"], 4);
        assert_eq!(catalog["scope"], "m5-npcs");
        assert_eq!(npcs.len(), 49);
        assert_eq!(catalog["summary"]["placed"], 46);
        assert_eq!(catalog["summary"]["vendors"], 4);
        assert_eq!(catalog["summary"]["vendor_goods"], 5);
        assert_eq!(catalog["summary"]["inquiry_npcs"], 12);
        assert_eq!(catalog["summary"]["inquiry_topics"], 31);
        assert_eq!(catalog["summary"]["combat_profiles"], 48);
        assert_eq!(catalog["summary"]["combat_skill_entries"], 105);
        assert_eq!(catalog["summary"]["combat_mapping_entries"], 10);
        assert_eq!(catalog["summary"]["combat_apply_entries"], 85);
        assert_eq!(catalog["summary"]["combat_chat_npcs"], 10);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 29);
        assert_eq!(catalog["summary"]["carried_item_npcs"], 29);
        assert_eq!(catalog["summary"]["carried_item_entries"], 53);
        assert_eq!(catalog["summary"]["placed_combat_chat_npcs"], 9);
        assert_eq!(catalog["summary"]["placed_carried_item_npcs"], 28);
        assert_eq!(behavior_total, 59);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 59);
        assert!(npcs.iter().any(|npc| {
            npc["source_path"] == "mudlib/obj/npc/garrison.c" && npc["placement_count"] == 2
        }));
    }

    #[test]
    fn generated_m6_npc_catalog_matches_fixed_source_baseline() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../migration/catalog/npcs-m6.json")).unwrap();
        let npcs = catalog["npcs"].as_array().unwrap();
        let behavior_total = npcs
            .iter()
            .map(|npc| npc["behavior_flags"].as_array().unwrap().len())
            .sum::<usize>();

        assert_eq!(catalog["schema_version"], 4);
        assert_eq!(catalog["scope"], "m6-npcs");
        assert_eq!(npcs.len(), 57);
        assert_eq!(catalog["summary"]["placed"], 48);
        assert_eq!(catalog["summary"]["vendors"], 1);
        assert_eq!(catalog["summary"]["vendor_goods"], 6);
        assert_eq!(catalog["summary"]["inquiry_topics"], 17);
        assert_eq!(catalog["summary"]["static_inquiries"], 10);
        assert_eq!(catalog["summary"]["scripted_inquiries"], 7);
        assert_eq!(catalog["summary"]["combat_profiles"], 53);
        assert_eq!(catalog["summary"]["combat_skill_entries"], 162);
        assert_eq!(catalog["summary"]["combat_mapping_entries"], 31);
        assert_eq!(catalog["summary"]["combat_apply_entries"], 28);
        assert_eq!(catalog["summary"]["combat_chat_npcs"], 18);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 33);
        assert_eq!(catalog["summary"]["carried_item_npcs"], 42);
        assert_eq!(catalog["summary"]["carried_item_entries"], 81);
        assert_eq!(behavior_total, 81);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 81);
    }

    #[test]
    fn generated_m7_npc_catalog_matches_fixed_source_baseline() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../migration/catalog/npcs-m7.json")).unwrap();
        let npcs = catalog["npcs"].as_array().unwrap();
        let behavior_total = npcs
            .iter()
            .map(|npc| npc["behavior_flags"].as_array().unwrap().len())
            .sum::<usize>();

        assert_eq!(catalog["schema_version"], 4);
        assert_eq!(catalog["scope"], "m7-npcs");
        assert_eq!(npcs.len(), 85);
        assert_eq!(catalog["summary"]["placed"], 65);
        assert_eq!(catalog["summary"]["vendors"], 8);
        assert_eq!(catalog["summary"]["vendor_goods"], 29);
        assert_eq!(catalog["summary"]["inquiry_topics"], 67);
        assert_eq!(catalog["summary"]["static_inquiries"], 56);
        assert_eq!(catalog["summary"]["scripted_inquiries"], 11);
        assert_eq!(catalog["summary"]["combat_profiles"], 85);
        assert_eq!(catalog["summary"]["combat_skill_entries"], 343);
        assert_eq!(catalog["summary"]["combat_mapping_entries"], 75);
        assert_eq!(catalog["summary"]["combat_apply_entries"], 36);
        assert_eq!(catalog["summary"]["combat_chat_npcs"], 13);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 27);
        assert_eq!(catalog["summary"]["carried_item_npcs"], 60);
        assert_eq!(catalog["summary"]["carried_item_entries"], 122);
        assert_eq!(behavior_total, 183);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 183);
        assert!(
            npcs.iter()
                .find(|npc| npc["id"] == "latemoon.room.npc.aaa")
                .unwrap()["carried_items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(npcs.iter().any(|npc| {
            npc["id"] == "latemoon.room.npc.fong"
                && npc["carried_items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| item["source_path"] == "mudlib/d/latemoon/npc/obj/deer_boot.c")
        }));
        assert!(npcs.iter().all(|npc| {
            !matches!(
                npc["source_path"].as_str().unwrap(),
                "mudlib/d/snow/npc/beggar.c" | "mudlib/obj/npc/garrison.c"
            )
        }));
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
    fn source_ids_remove_only_one_c_extension() {
        assert_eq!(
            source_path_to_id("mudlib/u/cloud/biaoju.c"),
            "u.cloud.biaoju"
        );
        assert_eq!(
            source_path_to_id("mudlib/d/goathill/cavern1.c.c"),
            "goathill.cavern1.c"
        );
        assert_eq!(
            normalize_target("__DIR__cavern1.c", "mudlib/d/goathill/cavern2.c"),
            Some("goathill.cavern1".into())
        );
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
        let exits = extract_exits(source, "village", "mudlib/d/village/farmhouse1.c");
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].target, "village.farmhouse1");
        assert_eq!(
            extract_objects(source, "mudlib/d/village/farmhouse1.c"),
            ["mudlib/d/village/npc/woman1.c"]
        );
    }

    #[test]
    fn parses_npc_vendor_goods_and_room_quantities() {
        let room = r#"
set("objects", ([
  __DIR__"npc/trainee": 6,
  "/obj/weapon/shield": 1,
]));
"#;
        let objects = extract_object_references(room, "mudlib/d/snow/school2.c");
        assert_eq!(objects.len(), 2);
        assert!(objects.iter().any(|object| {
            object.source_path == "mudlib/d/snow/npc/trainee.c" && object.quantity == 6
        }));
        let cloud_objects = extract_object_references(
            "set(\"objects\", ([ \"/u/cloud/npc/b_header\": 1, ]));",
            "mudlib/u/cloud/biaoju.c",
        );
        assert_eq!(cloud_objects.len(), 1);
        assert_eq!(
            cloud_objects[0].source_path,
            "mudlib/u/cloud/npc/b_header.c"
        );

        let vendor = r#"
set_name("店小二", ({ "waiter" }));
set("long",
    "第一行。\n"
    "第二行。\n");
set("vendor_goods", ([
  "dumpling": "obj/example/dumpling",
  "cake": __DIR__"obj/cake",
]));
"#;
        assert_eq!(extract_item_name(vendor).as_deref(), Some("店小二"));
        assert_eq!(
            extract_set_text(vendor, "long").as_deref(),
            Some("第一行。\n第二行。")
        );
        let goods = extract_vendor_goods(vendor, "mudlib/d/city/npc/waiter.c");
        assert_eq!(goods.len(), 2);
        assert!(
            goods
                .iter()
                .any(|good| good.item_id == "obj.example.dumpling")
        );
        assert!(goods.iter().any(|good| good.item_id == "city.npc.obj.cake"));
    }
}
