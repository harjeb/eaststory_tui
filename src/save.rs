use std::{fs, io, path::PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

use crate::{content::world, game::Game};

pub const CURRENT_SAVE_VERSION: u32 = 7;

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
            game.migrate_v3_statuses();
            game.migrate_v4_skills();
        }
        2 => {
            game.migrate_legacy_items();
            game.migrate_v3_statuses();
            game.migrate_v4_skills();
        }
        3 => {
            game.migrate_v3_statuses();
            game.migrate_v4_skills();
        }
        4 => game.migrate_v4_skills(),
        5 => game.migrate_v5_npc_events(),
        6 => game.migrate_v6_m4_access(),
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
            Action, Activity, CombatMode, ConditionKind, ConditionState, DoorKind, EnemyKind,
            InteractionKind, LocationId,
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
        game.player.faction = Some("天邪派".into());
        game.player.teacher = Some("fighter".into());
        game.player.combat_experience = 88_000;
        game.player.potential = 321;
        game.player.learned_points = 77;
        game.player.bellicosity = 9;
        game.player.wanted = 2;
        game.player.constitution = 17;
        game.player.force = 140;
        game.player.max_force = 180;
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
        game.activity = Activity::Fighting(crate::game::CombatState {
            enemy: EnemyKind::TempleDisciple,
            health: 51,
            max_health: 95,
            rounds: 3,
            mode: CombatMode::Lethal,
            attack_bonus: 12,
            dodge_bonus: 8,
            enemy_busy_rounds: 2,
            technique_cooldown: 1,
            power_up_active: true,
            fake_fault_active: true,
        });

        save_game(&path, &game).unwrap();
        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            restored.location,
            LocationId::from(crate::content::TEMPLE_YARD)
        );
        assert_eq!(restored.player.reputation, 42);
        assert_eq!(restored.player.food, 73);
        assert_eq!(restored.player.faction.as_deref(), Some("天邪派"));
        assert_eq!(restored.player.teacher.as_deref(), Some("fighter"));
        assert_eq!(restored.player.combat_experience, 88_000);
        assert_eq!(restored.player.potential, 321);
        assert_eq!(restored.player.learned_points, 77);
        assert_eq!(restored.player.bellicosity, 9);
        assert_eq!(restored.player.wanted, 2);
        assert_eq!(restored.player.constitution, 17);
        assert_eq!(restored.player.force, 140);
        assert_eq!(restored.player.max_force, 180);
        assert!(matches!(
            restored.activity,
            Activity::Fighting(crate::game::CombatState {
                mode: CombatMode::Lethal,
                attack_bonus: 12,
                dodge_bonus: 8,
                enemy_busy_rounds: 2,
                technique_cooldown: 1,
                power_up_active: true,
                fake_fault_active: true,
                ..
            })
        ));
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
    fn m4_npc_event_state_survives_save_round_trip() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-m4-npc-{nonce}.json"));
        let mut game = Game::new();
        game.location = LocationId::from("snow.herbshop");
        game.perform(Action::BuyItem {
            item_id: ItemId::from(crate::items::WOUND_MEDICINE_ID),
            npc: crate::npcs::NpcId::from("snow.npc.herbalist"),
        });
        let medicine = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == crate::items::WOUND_MEDICINE_ID)
            .unwrap()
            .instance_id;
        game.location = LocationId::from("snow.school");
        game.perform(Action::GiveItem {
            instance_id: medicine,
            npc: crate::npcs::NpcId::from(crate::npcs::SNOW_TEACHER_ID),
        });
        game.player.combat_experience = 20_000;
        game.location = LocationId::from("snow.school1");
        game.perform(Action::AskNpc {
            npc: crate::npcs::NpcId::from(crate::npcs::SNOW_GUARD_ID),
            topic: "血手刘三".into(),
        });
        assert!(matches!(
            game.activity,
            Activity::Fighting(crate::game::CombatState {
                enemy: EnemyKind::BloodHandLiuSan,
                ..
            })
        ));

        save_game(&path, &game).unwrap();
        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert!(matches!(
            restored.activity,
            Activity::Fighting(crate::game::CombatState {
                enemy: EnemyKind::BloodHandLiuSan,
                ..
            })
        ));
        restored.activity = Activity::Idle;
        restored.location = LocationId::from("snow.school");
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::LearnFromNpc { skill, npc }
                if skill.as_str() == "literate" && npc.as_str() == crate::npcs::SNOW_TEACHER_ID
        )));
    }

    #[test]
    fn m4_access_state_survives_save_round_trip() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-m4-access-{nonce}.json"));
        let mut game = Game::new();
        game.player.banknotes = 1;
        game.location = LocationId::from(crate::content::CANYON_CAMP8);
        game.perform(Action::OfferMoney {
            amount: 800,
            npc: crate::npcs::NpcId::from(crate::npcs::CANYON_ADVISER_ID),
        });
        game.location = LocationId::from("canyon.camp6");
        game.perform(Action::OfferMoney {
            amount: 3_000,
            npc: crate::npcs::NpcId::from(crate::npcs::CANYON_CAPTAIN_ID),
        });
        game.location = LocationId::from(crate::content::CITY_INN);
        game.perform(Action::OfferMoney {
            amount: 1_000,
            npc: crate::npcs::NpcId::from(crate::npcs::CITY_WAITER_ID),
        });

        save_game(&path, &game).unwrap();
        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);

        restored.location = LocationId::from(crate::content::CANYON_SECRET_WALL);
        assert!(restored.available_actions().contains(&Action::Interact(
            crate::game::InteractionKind::SwearCanyonSecret
        )));
        restored.location = LocationId::from(crate::content::CANYON_CAMP7);
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == crate::content::CANYON_CAMP8
        )));
        restored.location = LocationId::from(crate::content::CITY_INN);
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == crate::content::CITY_INN_UPSTAIRS
        )));
    }

    #[test]
    fn version_five_saves_gain_default_m4_npc_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v5-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(5);
        let root = value.as_object_mut().unwrap();
        root.remove("snow_teacher_paid");
        root.remove("snow_guard_revealed");
        root.remove("snow_guard_defeated");
        root.remove("canyon_secret_clue");
        root.remove("canyon_camp_access");
        root.remove("canyon_fake_seal_bought");
        root.remove("canyon_general_rejected_fake");
        root.remove("canyon_general_rewarded");
        root.remove("city_inn_access");
        root.remove("city_manor_pass");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        restored.location = LocationId::from("snow.school");
        assert!(
            restored
                .available_actions()
                .iter()
                .all(|action| { !matches!(action, Action::LearnFromNpc { .. }) })
        );
    }

    #[test]
    fn version_six_saves_gain_default_m4_access_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v6-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(6);
        let root = value.as_object_mut().unwrap();
        for field in [
            "canyon_secret_clue",
            "canyon_camp_access",
            "canyon_fake_seal_bought",
            "canyon_general_rejected_fake",
            "canyon_general_rewarded",
            "city_inn_access",
            "city_manor_pass",
        ] {
            root.remove(field);
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        restored.location = LocationId::from(crate::content::CANYON_CAMP7);
        assert!(restored.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == crate::content::CANYON_CAMP8
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
    fn version_four_skills_gain_stable_basics_and_mappings() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v4-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(4);
        value["player"]["skills"] = serde_json::json!([
            { "kind": "Unarmed", "level": 18, "progress": 3 },
            { "kind": "Sword", "level": 12, "progress": 2 },
            { "kind": "Dodge", "level": 15, "progress": 1 },
            { "kind": "Breathing", "level": 9, "progress": 0 },
            { "kind": "Parry", "level": 7, "progress": 0 }
        ]);
        let player = value["player"].as_object_mut().unwrap();
        player.remove("skill_mappings");
        player.remove("force");
        player.remove("max_force");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored.player.skill_level(crate::skills::UNARMED_ID), 18);
        assert_eq!(restored.player.skill_level(crate::skills::SWORD_ID), 12);
        assert_eq!(restored.player.skill_level(crate::skills::MOVE_ID), 15);
        assert_eq!(restored.player.skill_mappings.len(), 5);
        assert_eq!(restored.player.force, 50);
        assert_eq!(restored.player.max_force, 100);
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
