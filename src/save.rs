use std::{fs, io, path::PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

use crate::{content::world, game::Game};

pub const CURRENT_SAVE_VERSION: u32 = 2;

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
        1 => game.migrate_v1_location_ids(),
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
    use crate::game::{Action, Activity, DoorKind, EnemyKind, InteractionKind, LocationId};

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

        save_game(&path, &game).unwrap();
        let restored = load_game(&path).unwrap().unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            restored.location,
            LocationId::from(crate::content::TEMPLE_YARD)
        );
        assert_eq!(restored.player.reputation, 42);
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
