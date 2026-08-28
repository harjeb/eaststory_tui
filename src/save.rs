use std::{fs, io, path::PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

use crate::{content::world, game::Game};

pub const CURRENT_SAVE_VERSION: u32 = 25;

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
        7 => game.migrate_v7_m4_combat(),
        8 => game.migrate_v8_old_liu_plot(),
        9 => game.migrate_v9_m4_npc_combat(),
        10 => game.migrate_v10_city_exit_permit(),
        11 => game.migrate_v11_m4_room_events(),
        12 => game.migrate_v12_source_doors(),
        13 | 14 => {}
        15 => game.migrate_v15_m5_choyin_events(),
        16 => game.migrate_v16_m5_room_events(),
        17 => game.migrate_v17_m5_npc_events(),
        18 => game.migrate_v18_m5_regional_gameplay(),
        19 => {}
        20 => game.migrate_v20_m6_room_events(),
        21 => game.migrate_v21_m6_npc_events(),
        22 => game.migrate_v22_m7_source_room_items(),
        23 => game.migrate_v23_m7_room_and_item_events(),
        24 => game.migrate_v24_m7_npc_events(),
        CURRENT_SAVE_VERSION => {}
        version => {
            bail!(
                "存档版本 {} 与程序版本 {} 不兼容",
                version,
                CURRENT_SAVE_VERSION
            );
        }
    }
    game.migrate_v13_source_room_items();
    game.migrate_v14_m5_source_room_items();
    game.migrate_v19_m6_source_room_items();
    game.migrate_v22_m7_source_room_items();
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
            enemy_attack_bonus: 7,
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
                enemy_attack_bonus: 7,
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
    fn version_seven_saves_gain_default_m4_combat_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v7-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(7);
        value.as_object_mut().unwrap().remove("defeated_npcs");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        restored.location = LocationId::from("city.bank");
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::Fight(EnemyKind::Npc(npc)) if npc.as_str() == "city.npc.microsof"
        )));
    }

    #[test]
    fn version_eight_saves_upgrade_to_old_liu_plot_schema() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v8-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(8);
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored.quest, crate::game::QuestStage::Unasked);
    }

    #[test]
    fn version_nine_saves_gain_per_location_npc_combat_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v9-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(9);
        let root = value.as_object_mut().unwrap();
        root.remove("defeated_npc_instances");
        if let Some(activity) = root
            .get_mut("activity")
            .and_then(|value| value.as_object_mut())
            && let Some(combat) = activity
                .get_mut("Fighting")
                .and_then(|value| value.as_object_mut())
        {
            combat.remove("enemy_attack_bonus");
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::Talk(npc) if npc.as_str() == crate::npcs::OLD_LIU_ID
        )));
    }

    #[test]
    fn version_ten_saves_gain_default_city_exit_permit() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v10-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(10);
        value.as_object_mut().unwrap().remove("city_exit_permit");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let mut restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        restored.location = LocationId::from(crate::content::CITY_NORTH_GATE);
        assert!(restored.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == crate::content::CITY_NORTH_ROAD
        )));
    }

    #[test]
    fn version_eleven_saves_gain_default_m4_room_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v11-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(11);
        value["location"] = serde_json::json!("city.jitan");
        let root = value.as_object_mut().unwrap();
        for field in [
            "city_altar_forward_turns",
            "city_altar_backward_turns",
            "city_altar_passage_ticks",
            "snow_shelf_pushes",
            "snow_storage_passage_ticks",
            "canyon_boulder_open",
            "canyon_bookcase_searched",
        ] {
            root.remove(field);
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert!(
            restored
                .available_actions()
                .contains(&Action::Interact(InteractionKind::PressAltarButton))
        );
        assert!(restored.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "city.midao1"
        )));
    }

    #[test]
    fn version_twelve_saves_gain_default_source_door_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v12-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(12);
        value["location"] = serde_json::json!("city.shangshu.road1");
        value.as_object_mut().unwrap().remove("source_door_states");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert!(restored.available_actions().iter().any(|action| matches!(
            action,
            Action::OpenSourceDoor { target } if target.as_str() == "city.shangshu.xiaowu"
        )));
        assert!(restored.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "city.shangshu.xiaowu"
        )));
    }

    #[test]
    fn version_thirteen_saves_gain_source_room_items_once() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v13-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(13);
        let root = value.as_object_mut().unwrap();
        root.remove("source_room_items_initialized");
        root.remove("m5_source_room_items_initialized");
        root.remove("m6_source_room_items_initialized");
        root.remove("m7_source_room_items_initialized");
        root.insert("ground_items".into(), serde_json::json!({}));
        root.insert("next_item_instance_id".into(), serde_json::json!(3));
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        save_game(&path, &restored).unwrap();
        let roundtrip = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(roundtrip.version, CURRENT_SAVE_VERSION);
        assert_eq!(roundtrip.ground_items.values().flatten().count(), 38);
        assert_eq!(
            roundtrip.ground_items[&LocationId::from("snow.temple")]
                .iter()
                .filter(|item| item.item_id.as_str() == "obj.paper_seal")
                .count(),
            2
        );
    }

    #[test]
    fn version_fourteen_saves_gain_m5_m6_and_m7_room_items_once() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v14-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(14);
        let root = value.as_object_mut().unwrap();
        root.remove("m5_source_room_items_initialized");
        root.remove("m6_source_room_items_initialized");
        root.remove("m7_source_room_items_initialized");
        root["ground_items"]
            .as_object_mut()
            .unwrap()
            .retain(|location, _| {
                ![
                    "oldpine",
                    "goathill",
                    "choyin",
                    "chuenyu",
                    "green",
                    "sanyen",
                    "waterfog",
                    "latemoon",
                    "death",
                    "graveyard",
                    "jail",
                    "u.cloud",
                ]
                .iter()
                .any(|area| location.starts_with(&format!("{area}.")))
            });
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        save_game(&path, &restored).unwrap();
        let roundtrip = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(roundtrip.version, CURRENT_SAVE_VERSION);
        assert_eq!(roundtrip.ground_items.values().flatten().count(), 38);
        assert_eq!(
            roundtrip.ground_items[&LocationId::from("choyin.stove")]
                .iter()
                .filter(|item| item.item_id.as_str() == "choyin.obj.tablet")
                .count(),
            5
        );
    }

    #[test]
    fn version_fifteen_defaults_m5_choyin_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v15-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(15);
        let root = value.as_object_mut().unwrap();
        root.remove("choyin_platform_passage_ticks");
        root.remove("choyin_thunder_ticks");
        root.remove("choyin_lion_lift_count");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored_value["choyin_platform_passage_ticks"], 0);
        assert_eq!(restored_value["choyin_thunder_ticks"], 0);
        assert_eq!(restored_value["choyin_lion_lift_count"], 0);
    }

    #[test]
    fn version_sixteen_defaults_remaining_m5_room_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v16-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(16);
        let root = value.as_object_mut().unwrap();
        root.remove("oldpine_keep_sealed");
        root.remove("choyin_taolin_steps");
        root.remove("choyin_taolin_clue");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored_value["oldpine_keep_sealed"], false);
        assert_eq!(restored_value["choyin_taolin_steps"], 0);
        assert_eq!(restored_value["choyin_taolin_clue"], 0);
    }

    #[test]
    fn version_seventeen_defaults_m5_npc_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v17-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(17);
        let root = value.as_object_mut().unwrap();
        root.remove("choyin_silk_bag_received");
        root.remove("choyin_silk_bag_delivered");
        root.remove("choyin_chest_rewarded");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored_value["choyin_silk_bag_received"], false);
        assert_eq!(restored_value["choyin_silk_bag_delivered"], false);
        assert_eq!(restored_value["choyin_chest_rewarded"], false);
    }

    #[test]
    fn version_eighteen_defaults_m5_regional_gameplay_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v18-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(18);
        let altar_ground = value["ground_items"]["choyin.altar"]
            .as_array_mut()
            .unwrap();
        let box_index = altar_ground
            .iter()
            .position(|item| item["item_id"] == "choyin.obj.denotation")
            .unwrap();
        let moved_box = altar_ground.remove(box_index);
        value["player"]["inventory"]
            .as_array_mut()
            .unwrap()
            .push(moved_box.clone());
        let mut duplicate_box = moved_box;
        duplicate_box["instance_id"] = serde_json::json!(999_999);
        duplicate_box["item_id"] = serde_json::json!("choyin.npc.obj.denotation");
        value["ground_items"].as_object_mut().unwrap().insert(
            "legacy.moved_box".into(),
            serde_json::json!([duplicate_box]),
        );
        let root = value.as_object_mut().unwrap();
        root.remove("choyin_scholar_trial_started");
        root.remove("choyin_scholar_trial_completed");
        root.remove("spawned_npc_instances");
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored_value["choyin_scholar_trial_started"], false);
        assert_eq!(restored_value["choyin_scholar_trial_completed"], false);
        assert_eq!(
            restored_value["spawned_npc_instances"],
            serde_json::json!([])
        );
        assert!(restored.player.inventory.iter().all(|item| {
            !matches!(
                item.item_id.as_str(),
                "choyin.obj.denotation" | "choyin.npc.obj.denotation"
            )
        }));
        let donation_boxes = restored
            .ground_items
            .iter()
            .flat_map(|(location, ground)| {
                ground.iter().filter_map(move |item| {
                    matches!(
                        item.item_id.as_str(),
                        "choyin.obj.denotation" | "choyin.npc.obj.denotation"
                    )
                    .then_some((location.as_str(), item.item_id.as_str()))
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            donation_boxes,
            vec![("choyin.altar", "choyin.obj.denotation")]
        );
    }

    #[test]
    fn version_nineteen_adds_m6_room_items_once() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v19-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(19);
        let root = value.as_object_mut().unwrap();
        root.remove("m6_source_room_items_initialized");
        root["ground_items"]
            .as_object_mut()
            .unwrap()
            .retain(|location, _| {
                !["chuenyu", "green", "sanyen", "waterfog"]
                    .iter()
                    .any(|area| location.starts_with(&format!("{area}.")))
            });
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        save_game(&path, &restored).unwrap();
        let roundtrip = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(roundtrip.version, CURRENT_SAVE_VERSION);
        let m6_items = roundtrip
            .ground_items
            .iter()
            .filter(|(location, _)| {
                ["chuenyu", "green", "sanyen", "waterfog"]
                    .iter()
                    .any(|area| location.as_str().starts_with(&format!("{area}.")))
            })
            .flat_map(|(_, items)| items)
            .collect::<Vec<_>>();
        assert_eq!(m6_items.len(), 11);
        for (item_id, expected) in [
            ("chuenyu.obj.pigmeat", 3),
            ("chuenyu.obj.qiwine", 3),
            ("obj.longsword", 5),
        ] {
            assert_eq!(
                m6_items
                    .iter()
                    .filter(|item| item.item_id.as_str() == item_id)
                    .count(),
                expected
            );
        }
    }

    #[test]
    fn version_twenty_defaults_m6_room_events_and_liquid_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v20-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(20);
        value["location"] = serde_json::json!("chuenyu.trap_castle");
        let root = value.as_object_mut().unwrap();
        for field in [
            "chuenyu_slab_pushes",
            "chuenyu_slab_passage_ticks",
            "chuenyu_trap_arrow_ticks",
            "green_bagua_completed",
            "green_windsword_rewarded",
            "sanyen_buns_taken",
            "green_elder_jade_clue",
            "green_drunk_jade_clue",
            "green_drunk_drug_clue",
            "green_jade_received",
            "green_drug_offer_unlocked",
        ] {
            root.remove(field);
        }
        for item in root["player"]["inventory"].as_array_mut().unwrap() {
            item.as_object_mut().unwrap().remove("filled_with_water");
        }
        for ground in root["ground_items"].as_object_mut().unwrap().values_mut() {
            for item in ground.as_array_mut().unwrap() {
                item.as_object_mut().unwrap().remove("filled_with_water");
            }
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored_value["chuenyu_slab_pushes"], 0);
        assert_eq!(restored_value["chuenyu_slab_passage_ticks"], 0);
        assert_eq!(restored_value["chuenyu_trap_arrow_ticks"], 2);
        assert_eq!(restored_value["green_bagua_completed"], false);
        assert_eq!(restored_value["green_windsword_rewarded"], false);
        assert_eq!(restored_value["sanyen_buns_taken"], 0);
        assert_eq!(restored_value["green_elder_jade_clue"], false);
        assert_eq!(restored_value["green_drunk_jade_clue"], false);
        assert_eq!(restored_value["green_drunk_drug_clue"], false);
        assert_eq!(restored_value["green_jade_received"], false);
        assert_eq!(restored_value["green_drug_offer_unlocked"], false);
        assert!(
            restored
                .player
                .inventory
                .iter()
                .chain(restored.ground_items.values().flatten())
                .all(|item| !item.filled_with_water)
        );
    }

    #[test]
    fn version_twenty_one_defaults_m6_npc_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v21-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(21);
        let root = value.as_object_mut().unwrap();
        for field in [
            "green_elder_jade_clue",
            "green_drunk_jade_clue",
            "green_drunk_drug_clue",
            "green_jade_received",
            "green_drug_offer_unlocked",
        ] {
            root.remove(field);
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        for field in [
            "green_elder_jade_clue",
            "green_drunk_jade_clue",
            "green_drunk_drug_clue",
            "green_jade_received",
            "green_drug_offer_unlocked",
        ] {
            assert_eq!(restored_value[field], false, "{field}");
        }
    }

    #[test]
    fn version_twenty_two_adds_m7_room_items_once() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v22-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(22);
        let root = value.as_object_mut().unwrap();
        root.remove("m7_source_room_items_initialized");
        root["ground_items"]
            .as_object_mut()
            .unwrap()
            .retain(|location, _| {
                !["latemoon", "death", "graveyard", "jail", "u.cloud"]
                    .iter()
                    .any(|prefix| location.starts_with(&format!("{prefix}.")))
            });
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        save_game(&path, &restored).unwrap();
        let roundtrip = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(roundtrip.version, CURRENT_SAVE_VERSION);
        let m7_items = roundtrip
            .ground_items
            .iter()
            .filter(|(location, _)| {
                ["latemoon", "death", "graveyard", "jail", "u.cloud"]
                    .iter()
                    .any(|prefix| location.as_str().starts_with(&format!("{prefix}.")))
            })
            .flat_map(|(_, items)| items)
            .collect::<Vec<_>>();
        assert_eq!(m7_items.len(), 12);
        for (item_id, expected) in [
            ("latemoon.obj.bamboo", 1),
            ("latemoon.obj.hankie", 1),
            ("latemoon.obj.food2", 1),
            ("latemoon.obj.food3", 2),
            ("latemoon.obj.wine", 1),
            ("latemoon.obj.food", 1),
            ("latemoon.room.npc.obj.fire", 1),
            ("latemoon.obj.cake", 1),
            ("obj.example.dumpling", 3),
        ] {
            assert_eq!(
                m7_items
                    .iter()
                    .filter(|item| item.item_id.as_str() == item_id)
                    .count(),
                expected,
                "{item_id}"
            );
        }
    }

    #[test]
    fn version_twenty_three_defaults_m7_room_and_item_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v23-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(23);
        let root = value.as_object_mut().unwrap();
        for field in [
            "latemoon_clothes_taken",
            "latemoon_flowers_picked",
            "death_road_steps",
        ] {
            root.remove(field);
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        assert_eq!(restored_value["latemoon_clothes_taken"], 0);
        assert_eq!(restored_value["latemoon_flowers_picked"], 0);
        assert_eq!(restored_value["death_road_steps"], 0);
    }

    #[test]
    fn version_twenty_four_defaults_m7_npc_event_state() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dongfang-tui-v24-{nonce}.json"));
        let mut value = serde_json::to_value(Game::new()).unwrap();
        value["version"] = serde_json::json!(24);
        let root = value.as_object_mut().unwrap();
        for field in [
            "latemoon_dragonfly_received",
            "latemoon_bracelet_clue",
            "latemoon_bracelet_received",
            "latemoon_dance_book_clue",
            "latemoon_dance_book_received",
            "latemoon_token_rewarded",
            "cloud_escort_member",
            "cloud_escort_letter_received",
            "city_chen_letter_delivered",
            "cloud_boater_paid",
            "cloud_gangster_pass",
            "cloud_girl_recognized",
        ] {
            root.remove(field);
        }
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let restored = load_game(&path).unwrap().unwrap();
        let restored_value = serde_json::to_value(&restored).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
        for field in [
            "latemoon_dragonfly_received",
            "latemoon_bracelet_clue",
            "latemoon_bracelet_received",
            "latemoon_dance_book_clue",
            "latemoon_dance_book_received",
            "latemoon_token_rewarded",
            "cloud_escort_member",
            "cloud_escort_letter_received",
            "city_chen_letter_delivered",
            "cloud_boater_paid",
            "cloud_gangster_pass",
            "cloud_girl_recognized",
        ] {
            assert_eq!(restored_value[field], false, "{field}");
        }
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
