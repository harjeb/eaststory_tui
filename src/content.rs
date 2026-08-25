use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use crate::game::{EnemyKind, Exit, Location, LocationId, NpcKind, SkillKind};

pub const LIU_HOME: &str = "village.home";
pub const GARDEN: &str = "village.littlegarden";
pub const FIELD: &str = "village.field";
pub const LAKE: &str = "village.lake";
pub const LAKE_BOTTOM: &str = "village.lakebottom1";
pub const LAKESIDE: &str = "village.lakeside";
pub const LORD_HOUSE1: &str = "village.lordhouse1";
pub const LORD_HOUSE3: &str = "village.lordhouse3";
pub const MELON_FARM: &str = "village.melonfarm";
pub const ROAD3: &str = "village.road3";
pub const ROAD6: &str = "village.road6";
pub const ROAD9: &str = "village.road9";
pub const PINE_FOREST: &str = "solo.pine_forest";
pub const SNOW_TOWN: &str = "solo.snow_town";
pub const MOUNTAIN_PATH: &str = "solo.mountain_path";
pub const TEMPLE_YARD: &str = "solo.temple_yard";

static WORLD: LazyLock<World> = LazyLock::new(World::load);

pub fn world() -> &'static World {
    &WORLD
}

pub struct World {
    locations: HashMap<LocationId, Location>,
    source_room_count: usize,
}

impl World {
    fn load() -> Self {
        let catalog: AreaCatalog =
            serde_json::from_str(include_str!("../migration/catalog/village.json"))
                .expect("embedded village catalog must be valid");
        assert_eq!(catalog.schema_version, 1, "unsupported content schema");
        assert_eq!(catalog.area, "village", "unexpected embedded area");

        let source_room_count = catalog.rooms.len();
        let mut locations = HashMap::new();
        for room in catalog.rooms {
            let id = LocationId::new(room.id);
            let exits = room
                .exits
                .into_iter()
                .map(|exit| Exit {
                    direction: exit.direction,
                    target: LocationId::new(exit.target),
                    source_target: Some(exit.source_target),
                    internal: exit.internal,
                    dynamic: exit.dynamic,
                })
                .collect();
            let mut location = Location {
                id: id.clone(),
                name: room.name,
                zone: "傅家坡".into(),
                description: room.description,
                arrival: "你沿着旧路继续前行。".into(),
                exits,
                npc: None,
                training: None,
                can_rest: false,
                enemy: None,
                source_path: Some(room.source_path),
                object_sources: room.object_sources,
                behavior_flags: room.behavior_flags,
            };
            apply_village_gameplay(&mut location);
            assert!(
                locations.insert(id.clone(), location).is_none(),
                "duplicate content id {}",
                id.as_str()
            );
        }

        add_solo_adaptations(&mut locations);
        add_adapter_exits(&mut locations);

        Self {
            locations,
            source_room_count,
        }
    }

    pub fn location(&self, id: &LocationId) -> Option<&Location> {
        self.locations.get(id)
    }

    pub fn contains(&self, id: &LocationId) -> bool {
        self.locations.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.locations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn source_room_count(&self) -> usize {
        self.source_room_count
    }

    pub fn unresolved_exits(&self) -> Vec<(&Location, &Exit)> {
        self.locations
            .values()
            .flat_map(|location| {
                location
                    .exits
                    .iter()
                    .filter(|exit| !self.contains(&exit.target))
                    .map(move |exit| (location, exit))
            })
            .collect()
    }
}

fn apply_village_gameplay(location: &mut Location) {
    match location.id.as_str() {
        LIU_HOME => {
            location.arrival = "屋内仍残留着淡淡的柴火气。".into();
            location.npc = Some(NpcKind::OldLiu);
            location.can_rest = true;
        }
        GARDEN => {
            location.training = Some(SkillKind::Breathing);
            location.npc = Some(NpcKind::FlowerGirl);
            location.can_rest = true;
        }
        FIELD => location.training = Some(SkillKind::Unarmed),
        LAKESIDE => {
            location.training = Some(SkillKind::Dodge);
            location.npc = Some(NpcKind::Fisher);
            location.can_rest = true;
        }
        "village.farmhouse1" => location.can_rest = true,
        "village.fmhousback" => location.npc = Some(NpcKind::FarmWoman),
        "village.lakebottom2" => location.enemy = Some(EnemyKind::IceDragon),
        "village.melonguard" => location.npc = Some(NpcKind::Meloner),
        "village.road2" => {
            location.npc = Some(NpcKind::Trader);
            location.can_rest = true;
        }
        "village.smallstorage" => location.enemy = Some(EnemyKind::Rat),
        _ => {}
    }
}

fn add_adapter_exits(locations: &mut HashMap<LocationId, Location>) {
    locations
        .get_mut(&LocationId::from("village.road7"))
        .expect("village road7 exists")
        .exits
        .push(Exit::adapter("林间猎径", PINE_FOREST));
    locations
        .get_mut(&LocationId::from("village.road1"))
        .expect("village road1 exists")
        .exits
        .push(Exit::adapter("北行官道", SNOW_TOWN));
}

fn add_solo_adaptations(locations: &mut HashMap<LocationId, Location>) {
    for location in [
        Location::adapted(
            PINE_FOREST,
            "幽暗松林",
            "黑松山",
            "古松遮蔽天光，林间只有一条若隐若现的猎径。断枝与凌乱脚印说明这里刚有人经过。",
            "树影深处传来窸窣声。",
            vec![Exit::adapter("北", "village.road7")],
            None,
            None,
            false,
            Some(EnemyKind::Bandit),
        ),
        Location::adapted(
            SNOW_TOWN,
            "雪亭镇口",
            "雪亭镇",
            "青石官道在镇口交汇。酒旗、货担与赶路人让这里比山村热闹许多，东面的石阶通向山烟寺。",
            "城门楼上的铜铃被风吹响。",
            vec![
                Exit::adapter("南", "village.road1"),
                Exit::adapter("东", MOUNTAIN_PATH),
            ],
            None,
            Some(SkillKind::Sword),
            true,
            None,
        ),
        Location::adapted(
            MOUNTAIN_PATH,
            "山寺石阶",
            "山烟寺",
            "石阶依山盘旋，云雾时聚时散。野兽足迹留在湿泥里，赶路时不可掉以轻心。",
            "林中忽然响起一声低沉兽吼。",
            vec![
                Exit::adapter("西", SNOW_TOWN),
                Exit::adapter("上", TEMPLE_YARD),
            ],
            None,
            None,
            false,
            Some(EnemyKind::Wolf),
        ),
        Location::adapted(
            TEMPLE_YARD,
            "山烟寺前院",
            "山烟寺",
            "古柏环绕着开阔前院，年轻僧人正在木桩间练习棍阵。玄智和尚立在檐下，安静地注视众人。",
            "钟声穿过云雾，回荡在山谷中。",
            vec![Exit::adapter("下", MOUNTAIN_PATH)],
            Some(NpcKind::TempleMaster),
            Some(SkillKind::Parry),
            true,
            Some(EnemyKind::TempleDisciple),
        ),
    ] {
        let id = location.id.clone();
        assert!(locations.insert(id, location).is_none());
    }
}

#[derive(Deserialize)]
struct AreaCatalog {
    schema_version: u32,
    area: String,
    rooms: Vec<RoomRecord>,
}

#[derive(Deserialize)]
struct RoomRecord {
    id: String,
    source_path: String,
    name: String,
    description: String,
    exits: Vec<ExitRecord>,
    object_sources: Vec<String>,
    behavior_flags: Vec<String>,
}

#[derive(Deserialize)]
struct ExitRecord {
    direction: String,
    target: String,
    source_target: String,
    internal: bool,
    dynamic: bool,
}

#[cfg(test)]
mod tests {
    use std::collections::{HashSet, VecDeque};

    use super::*;

    #[test]
    fn embeds_all_village_rooms() {
        assert_eq!(world().source_room_count(), 26);
        assert!(world().contains(&LocationId::from(LIU_HOME)));
    }

    #[test]
    fn behavior_ledger_covers_every_import_warning() {
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/village.json")).unwrap();
        let entries = ledger["behaviors"].as_array().unwrap();
        let ledger_paths: HashSet<_> = entries
            .iter()
            .map(|entry| entry["source_path"].as_str().unwrap())
            .collect();
        let catalog_paths: HashSet<_> = world()
            .locations
            .values()
            .filter(|location| !location.behavior_flags.is_empty())
            .map(|location| location.source_path.as_deref().unwrap())
            .collect();
        let verified_flags: usize = entries
            .iter()
            .filter(|entry| entry["status"] == "verified")
            .map(|entry| entry["flags"].as_array().unwrap().len())
            .sum();
        let all_flags: usize = entries
            .iter()
            .map(|entry| entry["flags"].as_array().unwrap().len())
            .sum();

        assert_eq!(ledger_paths, catalog_paths);
        assert_eq!(all_flags, 19);
        assert_eq!(verified_flags, 19);
    }

    #[test]
    fn importer_marks_runtime_dynamic_exits() {
        let road6 = world().location(&LocationId::from(ROAD6)).unwrap();
        let west = road6
            .exits
            .iter()
            .find(|exit| exit.direction == "west")
            .unwrap();
        assert!(west.dynamic);
        assert_eq!(west.target.as_str(), "village.valley2");
    }

    #[test]
    fn all_internal_village_exits_resolve() {
        let unresolved_internal: Vec<_> = world()
            .unresolved_exits()
            .into_iter()
            .filter(|(_, exit)| exit.internal)
            .collect();
        assert!(unresolved_internal.is_empty(), "{unresolved_internal:?}");
    }

    #[test]
    fn unresolved_cross_area_exits_remain_tracked() {
        let targets: HashSet<_> = world()
            .unresolved_exits()
            .into_iter()
            .map(|(_, exit)| exit.target.as_str())
            .collect();
        assert_eq!(targets, HashSet::from(["canyon.canyon7", "city.nroad2"]));
    }

    #[test]
    fn every_village_room_is_reachable_from_home() {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([LocationId::from(LIU_HOME)]);
        while let Some(id) = queue.pop_front() {
            if !visited.insert(id.clone()) {
                continue;
            }
            let location = world().location(&id).unwrap();
            for exit in &location.exits {
                if exit.target.as_str().starts_with("village.") {
                    queue.push_back(exit.target.clone());
                }
            }
        }
        assert_eq!(
            visited
                .iter()
                .filter(|id| id.as_str().starts_with("village."))
                .count(),
            26
        );
    }
}
