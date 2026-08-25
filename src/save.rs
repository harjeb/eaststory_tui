use std::{fs, io, path::PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

use crate::{content::world, game::Game};

pub const CURRENT_SAVE_VERSION: u32 = 4;

pub fn default_save_path() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("org", "mudchina", "dongfang-tui") {
        return dirs.data_local_dir().join("save.json");
    }
    PathBuf::from("dongfang-tui-save.json")
}

pub fn load_game(path: &std::path::Path) -> Result<Option<Game>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("无法读取存档 {}", path.display()));
        }
    };

    let mut game: Game = serde_json::from_str(&contents)
        .with_context(|| format!("存档格式损坏 {}", path.display()))?;
    match game.version {
        1 => {
            game.migrate_v1_location_ids();
            game.migrate_legacy_items();
        }
        2 => game.migrate_legacy_items(),
        3 => game.migrate_v3_statuses(),
        CURRENT_SAVE_VERSION => {}
        version => {
            bail!(
                "存档版本 {} 与程序版本 {} 不兼容",
                version,
                CURRENT_SAVE_VERSION
            );
        }
    }
    if !world().contains(&game.location) {
        bail!("存档位置 {} 不存在于当前内容中", game.location.as_str());
    }
    Ok(Some(game))
}

pub fn save_game(path: &std::path::Path, game: &Game) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建存档目录 {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(game).context("无法序列化存档")?;
    fs::write(path, contents).with_context(|| format!("无法写入存档 {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::{
        game::{
            Action, Activity, ConditionKind, ConditionState, DoorKind, EnemyKind, InteractionKind,
            LocationId,
        },
        items::{EquipmentSlot, HENGBING_SWORD_ID, ItemId, ItemInstance, WATER_MELON_ID},
    };

    #[test]
    fn save_round_trip_preserves_progress() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-{nonce}.json"));
        let mut game = Game::new();
        game.location = LocationId::from(crate::content::TEMPLE_YARD);
        game.player.reputation = 42;
        game.player.food = 73;
        game.player.conditions.push(ConditionState {
            kind: ConditionKind::Poison,
            duration: 4,
            potency: 7,
        });
        let rations = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == crate::items::DRY_RATIONS_ID)
            .unwrap()
            .instance_id;
        game.perform(Action::DropItem(rations));

        save_game(&path, &game).unwrap();
        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            restored.location,
            LocationId::from(crate::content::TEMPLE_YARD)
        );
        assert_eq!(restored.player.reputation, 42);
        assert_eq!(restored.player.food, 73);
        assert_eq!(
            restored.player.condition(ConditionKind::Poison),
            Some(&ConditionState {
                kind: ConditionKind::Poison,
                duration: 4,
                potency: 7,
            })
        );
        assert_eq!(
            restored.ground_items[&restored.location][0]
                .item_id
                .as_str(),
            crate::items::DRY_RATIONS_ID
        );
    }

    #[test]
    fn dynamic_room_state_survives_save_round_trip() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-state-{nonce}.json"));
        let mut game = Game::new();
        game.perform(Action::Interact(InteractionKind::OpenDoor(
            DoorKind::LiuGarden,
        )));
        game.location = LocationId::from(crate::content::MELON_FARM);
        game.player.perception = 30;
        game.perform(Action::Interact(InteractionKind::PickMelon));

        save_game(&path, &game).unwrap();
        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert!(matches!(
            restored.activity,
            Activity::Fighting(crate::game::CombatState {
                enemy: EnemyKind::Meloner,
                ..
            })
        ));
        restored.perform(Action::Surrender);
        assert!(
            restored
                .available_actions()
                .iter()
                .all(|action| !matches!(action, Action::Move { .. }))
        );

        restored.location = LocationId::from(crate::content::LIU_HOME);
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == crate::content::GARDEN
        )));
    }

    #[test]
    fn version_two_item_enums_are_migrated_to_instances() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v2-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(2);
        value["player"]["inventory"] = serde_json::json!(["Cloth", "DryRations", "HengbingSword"]);
        value["player"]["weapon"] = serde_json::json!("HengbingSword");
        value["player"].as_object_mut().unwrap().remove("equipment");
        let root = value.as_object_mut().unwrap();
        root.remove("ground_items");
        root.remove("next_item_instance_id");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(
            restored
                .player
                .equipped(EquipmentSlot::Weapon)
                .unwrap()
                .item_id
                .as_str(),
            HENGBING_SWORD_ID
        );
        assert!(
            restored
                .player
                .inventory
                .iter()
                .all(|item| item.instance_id > 0)
        );
    }

    #[test]
    fn version_three_items_gain_consumable_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v3-{nonce}.json"));
        let mut game = Game::new();
        game.version = 3;
        game.player
            .inventory
            .push(ItemInstance::new(99, ItemId::from(WATER_MELON_ID), 1));
        let mut value = serde_json::to_value(game).unwrap();
        let player = value["player"].as_object_mut().unwrap();
        player.remove("food");
        player.remove("max_food");
        player.remove("water");
        player.remove("max_water");
        player.remove("conditions");
        for item in player["inventory"].as_array_mut().unwrap() {
            item.as_object_mut().unwrap().remove("remaining_uses");
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored.player.food, restored.player.max_food);
        let melon = restored
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == WATER_MELON_ID)
            .unwrap();
        assert_eq!(melon.remaining_uses, Some(8));
    }

    #[test]
    fn version_one_location_ids_are_migrated() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v1-{nonce}.json"));
        let mut game = Game::new();
        game.version = 1;
        game.location = LocationId::from("TempleYard");

        save_game(&path, &game).unwrap();
        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(
            restored.location,
            LocationId::from(crate::content::TEMPLE_YARD)
        );
    }
}
