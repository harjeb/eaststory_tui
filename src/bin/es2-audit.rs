use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result, bail};
use dongfang_tui::{
    content::{M9_HIDDEN_SOURCE_LOCATIONS, world},
    items::items,
    npcs::npcs,
    quests,
    save::CURRENT_SAVE_VERSION,
    skills::skills,
};
use serde_json::{Value, json};

const FORBIDDEN_MIGRATION_STATUSES: [&str; 4] =
    ["discovered", "structured", "implemented", "blocked"];
const EXCLUSION_STATUSES: [&str; 3] = ["excluded", "deferred", "source_noop"];

fn main() {
    if let Err(error) = run() {
        eprintln!("M9 acceptance failed: {error:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let output = output_directory()?;
    let root = workspace_root()?;
    let ledger_path = root.join("migration/overrides/m9-release.json");
    let ledger = read_json(&ledger_path)?;
    if ledger.get("milestone").and_then(Value::as_str) != Some("M9")
        || ledger.get("status").and_then(Value::as_str) != Some("complete")
    {
        bail!("migration/overrides/m9-release.json must be a complete M9 ledger");
    }

    let migration_root = root.join("migration");
    let mut migration_files = Vec::new();
    collect_json_files(&migration_root, &mut migration_files)?;
    migration_files.sort();

    let mut status_counts = BTreeMap::new();
    let mut forbidden = Vec::new();
    let mut exclusions = Vec::new();
    for file in &migration_files {
        let value = read_json(file)?;
        let relative = relative_path(&root, file);
        inspect_statuses(
            &value,
            "$",
            &relative,
            &mut status_counts,
            &mut forbidden,
            &mut exclusions,
        );
    }
    if !forbidden.is_empty() {
        bail!(
            "forbidden migration statuses remain:\n{}",
            forbidden.join("\n")
        );
    }

    let source_area_count = migration_files
        .iter()
        .filter_map(|path| read_json(path).ok())
        .filter(|value| value.get("rooms").and_then(Value::as_array).is_some())
        .count();
    let source_item_definitions = read_json(&root.join("migration/catalog/items.json"))?["items"]
        .as_array()
        .context("item catalog has no items array")?
        .len();
    let source_npc_definitions = ["npcs-m4", "npcs-m5", "npcs-m6", "npcs-m7"]
        .iter()
        .map(|name| -> Result<usize> {
            Ok(
                read_json(&root.join(format!("migration/catalog/{name}.json")))?["npcs"]
                    .as_array()
                    .with_context(|| format!("{name} catalog has no NPC array"))?
                    .len(),
            )
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<usize>();
    let hidden_source_locations = M9_HIDDEN_SOURCE_LOCATIONS
        .into_iter()
        .collect::<BTreeSet<_>>();
    let hidden_source_location_count = world()
        .locations()
        .filter(|location| hidden_source_locations.contains(location.id.as_str()))
        .count();
    if hidden_source_location_count != hidden_source_locations.len()
        || world().locations().any(|location| {
            hidden_source_locations.contains(location.id.as_str()) && location.source_path.is_none()
        })
    {
        bail!("M9 hidden source-room registry does not match the runtime world");
    }

    let mut invalid_references = Vec::new();
    let mut runtime_exit_count = 0usize;
    for location in world().locations() {
        runtime_exit_count += location.exits.len();
        for npc in &location.npcs {
            if npcs().definition(npc).is_none() {
                invalid_references.push(format!(
                    "{} references missing NPC {}",
                    location.id.as_str(),
                    npc.as_str()
                ));
            }
        }
        for item in &location.room_items {
            if items().definition(&item.item_id).is_none() {
                invalid_references.push(format!(
                    "{} references missing item {}",
                    location.id.as_str(),
                    item.item_id.as_str()
                ));
            }
        }
        if let Some(skill) = &location.training
            && skills().definition(skill).is_none()
        {
            invalid_references.push(format!(
                "{} references missing training skill {}",
                location.id.as_str(),
                skill.as_str()
            ));
        }
    }
    for (location, exit) in world().unresolved_exits() {
        invalid_references.push(format!(
            "{} has unresolved runtime exit {} -> {}",
            location.id.as_str(),
            exit.direction,
            exit.target.as_str()
        ));
    }
    if !invalid_references.is_empty() {
        bail!(
            "invalid runtime references:\n{}",
            invalid_references.join("\n")
        );
    }

    let task_definitions = quests::all_task_definitions();
    let task_unique_targets = task_definitions
        .iter()
        .map(|task| task.target.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let quest_ledger = read_json(&root.join("migration/overrides/m8-quests.json"))?;
    if quest_ledger["catalog"]["active_source_rows"].as_u64() != Some(task_definitions.len() as u64)
        || quest_ledger["catalog"]["active_unique_targets"].as_u64()
            != Some(task_unique_targets as u64)
        || quest_ledger["catalog"]["remaining"].as_u64() != Some(0)
    {
        bail!("M8 task ledger no longer matches the embedded task definitions");
    }

    let actual_contract = BTreeMap::from([
        ("source_areas", source_area_count),
        ("source_rooms", world().source_room_count()),
        (
            "non_hidden_source_rooms",
            world().source_room_count() - hidden_source_locations.len(),
        ),
        ("hidden_source_rooms", hidden_source_locations.len()),
        ("runtime_locations", world().len()),
        ("source_item_definitions", source_item_definitions),
        ("runtime_item_definitions", items().len()),
        ("source_npc_definitions", source_npc_definitions),
        ("skill_definitions", skills().len()),
        ("task_rows", task_definitions.len()),
        ("task_unique_targets", task_unique_targets),
        ("save_schema", CURRENT_SAVE_VERSION as usize),
    ]);
    verify_contract(&ledger, &actual_contract)?;

    fs::create_dir_all(&output).with_context(|| format!("create {}", output.display()))?;
    let coverage = json!({
        "schema_version": 1,
        "milestone": "M9",
        "source_commit": ledger["source_commit"],
        "generator": "es2-audit",
        "migration_json_files": migration_files.len(),
        "migration_status_counts": status_counts.clone(),
        "forbidden_statuses": FORBIDDEN_MIGRATION_STATUSES,
        "content": actual_contract,
        "runtime": {
            "source_locations": world().source_room_count(),
            "non_hidden_source_locations":
                world().source_room_count() - hidden_source_locations.len(),
            "hidden_source_locations": hidden_source_locations,
            "adapted_locations": world().len() - world().source_room_count(),
            "runtime_exits": runtime_exit_count,
            "unresolved_runtime_references": 0
        },
        "tasks": {
            "tiers": quests::TASK_TIERS.len(),
            "active_source_rows": task_definitions.len(),
            "unique_targets": task_unique_targets,
            "initially_available_rows":
                quest_ledger["catalog"]["initially_statically_available_rows"],
            "conditional_rows": quest_ledger["catalog"]["conditionally_available_rows"],
            "explicitly_filtered_rows": quest_ledger["catalog"]["excluded_rows"]
        },
        "acceptance": {
            "runtime_references": "verified",
            "task_state_machine": "verified",
            "save_upgrade_schema": CURRENT_SAVE_VERSION,
            "release_contract": "verified"
        }
    });
    let exclusion_report = json!({
        "schema_version": 1,
        "milestone": "M9",
        "source_commit": ledger["source_commit"],
        "generator": "es2-audit",
        "included_statuses": EXCLUSION_STATUSES,
        "summary": {
            "excluded": status_counts.get("excluded").copied().unwrap_or_default(),
            "deferred": status_counts.get("deferred").copied().unwrap_or_default(),
            "source_noop": status_counts.get("source_noop").copied().unwrap_or_default()
        },
        "entries": exclusions
    });
    write_json(&output.join("content-coverage.json"), &coverage)?;
    write_json(&output.join("migration-exclusions.json"), &exclusion_report)?;

    println!(
        "M9 acceptance passed: {} source rooms, {} runtime locations, {} task rows.",
        world().source_room_count(),
        world().len(),
        task_definitions.len()
    );
    println!("Wrote release metadata to {}", output.display());
    Ok(())
}

fn output_directory() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None => Ok(PathBuf::from("dist/migration-metadata")),
        Some("--output") => {
            let output = args.next().context("--output requires a directory")?;
            if args.next().is_some() {
                bail!("usage: es2-audit [--output <directory>]");
            }
            Ok(PathBuf::from(output))
        }
        Some(_) => bail!("usage: es2-audit [--output <directory>]"),
    }
}

fn workspace_root() -> Result<PathBuf> {
    let mut current = env::current_dir().context("read current directory")?;
    loop {
        if current.join("Cargo.toml").is_file() {
            return Ok(current);
        }
        if !current.pop() {
            bail!("could not find workspace Cargo.toml");
        }
    }
}

fn collect_json_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    let source = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&source).with_context(|| format!("parse {}", path.display()))
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn inspect_statuses(
    value: &Value,
    json_path: &str,
    file: &str,
    status_counts: &mut BTreeMap<String, usize>,
    forbidden: &mut Vec<String>,
    exclusions: &mut Vec<Value>,
) {
    match value {
        Value::Object(object) => {
            if let Some(status) = object.get("status").and_then(Value::as_str) {
                *status_counts.entry(status.to_string()).or_default() += 1;
                if FORBIDDEN_MIGRATION_STATUSES.contains(&status) {
                    forbidden.push(format!("{file}:{json_path} has status {status}"));
                }
                if EXCLUSION_STATUSES.contains(&status) {
                    exclusions.push(json!({
                        "file": file,
                        "json_path": json_path,
                        "status": status,
                        "entry": value
                    }));
                }
            }
            for (key, child) in object {
                inspect_statuses(
                    child,
                    &format!("{json_path}.{key}"),
                    file,
                    status_counts,
                    forbidden,
                    exclusions,
                );
            }
        }
        Value::Array(array) => {
            for (index, child) in array.iter().enumerate() {
                inspect_statuses(
                    child,
                    &format!("{json_path}[{index}]"),
                    file,
                    status_counts,
                    forbidden,
                    exclusions,
                );
            }
        }
        _ => {}
    }
}

fn verify_contract(ledger: &Value, actual: &BTreeMap<&str, usize>) -> Result<()> {
    let contract = ledger["coverage_contract"]
        .as_object()
        .context("M9 ledger has no coverage_contract object")?;
    for (key, value) in actual {
        let expected = contract
            .get(*key)
            .and_then(Value::as_u64)
            .with_context(|| format!("M9 coverage_contract has no numeric {key}"))?;
        if expected != *value as u64 {
            bail!("M9 coverage contract drift for {key}: expected {expected}, got {value}");
        }
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let serialized = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{serialized}\n")).with_context(|| format!("write {}", path.display()))
}
