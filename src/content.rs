use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use crate::{
    game::{EnemyKind, Exit, Location, LocationId},
    npcs::{
        FARM_WOMAN_ID, FISHER_ID, FLOWER_GIRL_ID, MELONER_ID, NpcId, OLD_LIU_ID, TEMPLE_MASTER_ID,
        TRADER_ID, npcs,
    },
    skills::{DODGE_ID, FORCE_ID, PARRY_ID, SWORD_ID, SkillId, UNARMED_ID},
};

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
pub const CANYON_FOOT: &str = "canyon.canyon1";
pub const CANYON_FOREST_ENTRANCE: &str = "canyon.stone_forest1";
pub const CANYON_ROAD: &str = "canyon.road";
pub const CANYON_SECRET_WALL: &str = "canyon.canyon4";
pub const CANYON_BLACK_MARKET: &str = "canyon.blackmarket";
pub const CANYON_CAMP2: &str = "canyon.camp2";
pub const CANYON_CAMP7: &str = "canyon.camp7";
pub const CANYON_CAMP8: &str = "canyon.camp8";
pub const CITY_INN: &str = "city.jiulou";
pub const CITY_INN_UPSTAIRS: &str = "city.jiulou_2";
pub const CITY_MANOR_GATE: &str = "city.shangshu.gate";
pub const CITY_MANOR_YARD: &str = "city.shangshu.yuan";
pub const CITY_STREET3: &str = "city.street3";
pub const CITY_WALL: &str = "city.wall";
pub const CITY_MANOR_RUIN: &str = "city.shangshu.feiwu";
pub const PINE_FOREST: &str = "solo.pine_forest";
pub const SNOW_TOWN: &str = "solo.snow_town";
pub const MOUNTAIN_PATH: &str = "solo.mountain_path";
pub const TEMPLE_YARD: &str = "solo.temple_yard";

const SOURCE_COMMIT: &str = "87bba6bd2249beec8424b0d6623486a0dd1f7b30";
const SOURCE_AREAS: [(&str, &str, &str); 5] = [
    (
        "village",
        "傅家坡",
        include_str!("../migration/catalog/village.json"),
    ),
    (
        "city",
        "京城",
        include_str!("../migration/catalog/city.json"),
    ),
    (
        "snow",
        "雪亭镇",
        include_str!("../migration/catalog/snow.json"),
    ),
    (
        "temple",
        "山烟寺",
        include_str!("../migration/catalog/temple.json"),
    ),
    (
        "canyon",
        "黄石峡",
        include_str!("../migration/catalog/canyon.json"),
    ),
];

static WORLD: LazyLock<World> = LazyLock::new(World::load);

pub fn world() -> &'static World {
    &WORLD
}

pub struct World {
    locations: HashMap<LocationId, Location>,
    source_room_counts: HashMap<String, usize>,
}

impl World {
    fn load() -> Self {
        let mut locations = HashMap::new();
        let mut source_room_counts = HashMap::new();

        for (expected_area, zone, json) in SOURCE_AREAS {
            let catalog: AreaCatalog = serde_json::from_str(json).unwrap_or_else(|error| {
                panic!("embedded {expected_area} catalog is invalid: {error}")
            });
            assert_eq!(catalog.schema_version, 1, "unsupported content schema");
            assert_eq!(
                catalog.source_commit, SOURCE_COMMIT,
                "source baseline drift"
            );
            assert_eq!(catalog.area, expected_area, "unexpected embedded area");
            source_room_counts.insert(expected_area.to_string(), catalog.rooms.len());

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
                let room_npcs = room
                    .object_sources
                    .iter()
                    .filter_map(|source_path| npcs().id_for_source(source_path).cloned())
                    .collect();
                let mut location = Location {
                    id: id.clone(),
                    name: room.name,
                    zone: zone.into(),
                    description: room.description,
                    arrival: format!("你进入{zone}地界。"),
                    exits,
                    npcs: room_npcs,
                    training: None,
                    can_rest: false,
                    enemy: None,
                    source_path: Some(room.source_path),
                    object_sources: room.object_sources,
                    behavior_flags: room.behavior_flags,
                };
                apply_location_gameplay(&mut location);
                assert!(
                    locations.insert(id.clone(), location).is_none(),
                    "duplicate content id {}",
                    id.as_str()
                );
            }
        }

        add_solo_adaptations(&mut locations);
        add_adapter_exits(&mut locations);

        Self {
            locations,
            source_room_counts,
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
        self.source_room_counts.values().sum()
    }

    pub fn area_room_count(&self, area: &str) -> usize {
        self.source_room_counts.get(area).copied().unwrap_or(0)
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

fn apply_location_gameplay(location: &mut Location) {
    match location.id.as_str() {
        LIU_HOME => {
            location.arrival = "屋内仍残留着淡淡的柴火气。".into();
            location.npcs.push(NpcId::from(OLD_LIU_ID));
            location.can_rest = true;
        }
        GARDEN => {
            location.training = Some(SkillId::from(FORCE_ID));
            location.npcs.push(NpcId::from(FLOWER_GIRL_ID));
            location.can_rest = true;
        }
        FIELD => location.training = Some(SkillId::from(UNARMED_ID)),
        LAKESIDE => {
            location.training = Some(SkillId::from(DODGE_ID));
            location.npcs.push(NpcId::from(FISHER_ID));
            location.can_rest = true;
        }
        "village.farmhouse1" => location.can_rest = true,
        "village.fmhousback" => location.npcs.push(NpcId::from(FARM_WOMAN_ID)),
        "village.lakebottom2" => location.enemy = Some(EnemyKind::IceDragon),
        "village.melonguard" => location.npcs.push(NpcId::from(MELONER_ID)),
        "village.road2" => {
            location.npcs.push(NpcId::from(TRADER_ID));
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
    locations
        .get_mut(&LocationId::from("canyon.canyon2"))
        .expect("canyon2 exists")
        .exits
        .push(Exit::adapter("west", CANYON_FOREST_ENTRANCE));
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
            Some(SkillId::from(SWORD_ID)),
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
            Some(NpcId::from(TEMPLE_MASTER_ID)),
            Some(SkillId::from(PARRY_ID)),
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
    source_commit: String,
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
    fn embeds_all_m1_and_m4_rooms() {
        assert_eq!(world().source_room_count(), 173);
        assert_eq!(world().len(), 177);
        assert_eq!(world().area_room_count("village"), 26);
        assert_eq!(world().area_room_count("city"), 55);
        assert_eq!(world().area_room_count("snow"), 38);
        assert_eq!(world().area_room_count("temple"), 27);
        assert_eq!(world().area_room_count("canyon"), 27);
        assert!(world().contains(&LocationId::from(LIU_HOME)));
    }

    #[test]
    fn behavior_ledger_covers_every_import_warning() {
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/village.json")).unwrap();
        let entries = ledger["behaviors"].as_array().unwrap();
        let ledger_flags: HashSet<_> = entries
            .iter()
            .flat_map(|entry| {
                let path = entry["source_path"].as_str().unwrap();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (path, flag.as_str().unwrap()))
            })
            .collect();
        let catalog_flags: HashSet<_> = world()
            .locations
            .values()
            .filter(|location| {
                location
                    .source_path
                    .as_deref()
                    .is_some_and(|path| path.starts_with("mudlib/d/village/"))
            })
            .flat_map(|location| {
                let path = location.source_path.as_deref().unwrap();
                location
                    .behavior_flags
                    .iter()
                    .map(move |flag| (path, flag.as_str()))
            })
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

        assert_eq!(ledger_flags, catalog_flags);
        assert_eq!(all_flags, 20);
        assert_eq!(verified_flags, 20);
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
    fn internal_source_exits_only_keep_the_registered_dangling_target() {
        let unresolved_internal: HashSet<_> = world()
            .unresolved_exits()
            .into_iter()
            .filter(|(_, exit)| exit.internal)
            .map(|(_, exit)| exit.target.as_str())
            .collect();
        assert_eq!(unresolved_internal, HashSet::from(["city.room"]));
    }

    #[test]
    fn unresolved_future_area_exits_remain_tracked() {
        let targets: HashSet<_> = world()
            .unresolved_exits()
            .into_iter()
            .map(|(_, exit)| exit.target.as_str())
            .collect();
        assert_eq!(
            targets,
            HashSet::from([
                "goathill.mroad1",
                "green.path6",
                "oldpine.npath1",
                "waterfog.sroad1",
                "wiz.entrance",
                "city.room",
            ])
        );
    }

    #[test]
    fn source_topology_has_only_registered_blockers() {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([LocationId::from(LIU_HOME)]);
        while let Some(id) = queue.pop_front() {
            if !visited.insert(id.clone()) {
                continue;
            }
            let location = world().location(&id).unwrap();
            for exit in &location.exits {
                if world().contains(&exit.target) {
                    queue.push_back(exit.target.clone());
                }
            }
            for target in match id.as_str() {
                CANYON_FOOT => &[CANYON_ROAD][..],
                CANYON_ROAD => &[CANYON_FOOT][..],
                CANYON_SECRET_WALL => &[CANYON_BLACK_MARKET][..],
                CITY_STREET3 => &[CITY_WALL][..],
                CITY_WALL => &[CITY_STREET3, CITY_MANOR_RUIN][..],
                _ => &[],
            } {
                queue.push_back(LocationId::from(*target));
            }
        }

        let unreachable: HashSet<_> = world()
            .locations
            .values()
            .filter(|location| location.source_path.is_some() && !visited.contains(&location.id))
            .map(|location| location.id.as_str())
            .collect();
        assert_eq!(
            unreachable,
            HashSet::from(["snow.herbshop1", "temple.broom1", "temple.broom2"])
        );
    }

    #[test]
    fn m4_topology_ledger_covers_adaptations_and_known_blockers() {
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m4-topology.json")).unwrap();
        assert_eq!(ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(ledger["status"], "in_progress");

        let registered_blockers: HashSet<_> = ledger["transitions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["status"] == "blocked")
            .flat_map(|entry| entry["targets"].as_array().unwrap())
            .chain(
                ledger["source_defects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|entry| entry["status"] == "blocked")
                    .map(|entry| &entry["target"]),
            )
            .map(|target| target.as_str().unwrap())
            .collect();
        assert_eq!(registered_blockers, HashSet::from(["snow.herbshop1"]));

        let forest_exit = world()
            .location(&LocationId::from("canyon.canyon2"))
            .unwrap()
            .exits
            .iter()
            .find(|exit| exit.target.as_str() == CANYON_FOREST_ENTRANCE)
            .unwrap();
        assert!(forest_exit.source_target.is_none());
    }

    #[test]
    fn m4_npc_ledger_covers_placements_behaviors_and_source_defects() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/npcs-m4.json")).unwrap();
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m4-npcs.json")).unwrap();
        assert_eq!(ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(ledger["status"], "in_progress");

        let mut catalog_flags = std::collections::BTreeMap::new();
        for npc in catalog["npcs"].as_array().unwrap() {
            for flag in npc["behavior_flags"].as_array().unwrap() {
                *catalog_flags
                    .entry(flag.as_str().unwrap())
                    .or_insert(0usize) += 1;
            }
        }
        let ledger_flags: std::collections::BTreeMap<_, _> = ledger["behavior_flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry["flag"].as_str().unwrap(),
                    entry["candidates"].as_u64().unwrap() as usize,
                )
            })
            .collect();
        assert_eq!(ledger_flags, catalog_flags);
        assert_eq!(catalog["summary"]["inquiry_npcs"], 30);
        assert_eq!(catalog["summary"]["inquiry_topics"], 75);
        assert_eq!(catalog["summary"]["static_inquiries"], 57);
        assert_eq!(catalog["summary"]["scripted_inquiries"], 18);
        assert_eq!(catalog["summary"]["runtime_inquiry_npcs"], 21);
        assert_eq!(catalog["summary"]["runtime_inquiries"], 50);
        assert_eq!(catalog["summary"]["runtime_inquiry_references"], 95);
        let runtime_features: std::collections::BTreeMap<_, _> = ledger["runtime_features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry["feature"].as_str().unwrap(),
                    entry["status"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(runtime_features["source_inquiry_catalog"], "verified");
        assert_eq!(runtime_features["placed_static_inquiries"], "verified");
        assert_eq!(runtime_features["scripted_inquiry_runtime"], "adapted");
        assert_eq!(ledger["catalog"]["runtime_scripted_inquiries"], 10);
        assert_eq!(ledger["catalog"]["total_runtime_inquiry_npcs"], 26);
        assert_eq!(ledger["catalog"]["total_runtime_inquiries"], 60);
        assert_eq!(ledger["catalog"]["total_runtime_inquiry_references"], 105);

        let catalog_scripted: HashSet<_> = catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|npc| {
                let source_path = npc["source_path"].as_str().unwrap().to_string();
                npc["inquiries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|inquiry| inquiry["scripted"] == true)
                    .map(move |inquiry| {
                        (
                            source_path.clone(),
                            inquiry["topic"].as_str().unwrap().to_string(),
                        )
                    })
            })
            .collect();
        let scripted_dispositions: HashSet<_> = ledger["scripted_inquiries"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source_path = entry["source_path"].as_str().unwrap().to_string();
                entry["topics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |topic| (source_path.clone(), topic.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(scripted_dispositions, catalog_scripted);

        let mut scripted_statuses = std::collections::BTreeMap::new();
        for entry in ledger["scripted_inquiries"].as_array().unwrap() {
            *scripted_statuses
                .entry(entry["status"].as_str().unwrap())
                .or_insert(0usize) += entry["topics"].as_array().unwrap().len();
        }
        assert_eq!(
            scripted_statuses,
            std::collections::BTreeMap::from([
                ("adapted", 6),
                ("excluded", 7),
                ("source_noop", 1),
                ("verified", 4),
            ])
        );

        let catalog_exchanges: HashSet<_> = catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|npc| {
                npc["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|flag| flag == "object_exchange")
            })
            .map(|npc| npc["source_path"].as_str().unwrap())
            .collect();
        let ledger_exchanges: HashSet<_> = ledger["object_exchanges"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["source_path"].as_str().unwrap())
            .collect();
        assert_eq!(ledger_exchanges, catalog_exchanges);
        let mut exchange_statuses = std::collections::BTreeMap::new();
        for entry in ledger["object_exchanges"].as_array().unwrap() {
            *exchange_statuses
                .entry(entry["status"].as_str().unwrap())
                .or_insert(0usize) += 1;
        }
        assert_eq!(
            exchange_statuses,
            std::collections::BTreeMap::from([
                ("adapted", 1),
                ("deferred", 6),
                ("excluded", 5),
                ("verified", 7),
            ])
        );

        let source_placements = world()
            .locations
            .values()
            .flat_map(|location| &location.npcs)
            .filter(|npc| !npc.as_str().starts_with("adapted."))
            .count();
        assert_eq!(source_placements, 78);

        let bridge_npcs: HashSet<_> = world()
            .location(&LocationId::from("city.bridge"))
            .unwrap()
            .npcs
            .iter()
            .map(NpcId::as_str)
            .collect();
        assert_eq!(
            bridge_npcs,
            HashSet::from([
                "city.npc.caker",
                "city.npc.dumpling_seller",
                "city.npc.vendor",
            ])
        );

        let npc_sources: HashSet<_> = catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|npc| npc["source_path"].as_str().unwrap())
            .collect();
        let item_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/items.json")).unwrap();
        let item_sources: HashSet<_> = item_catalog["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["source_path"].as_str().unwrap())
            .collect();
        let unresolved_objects: HashSet<_> = world()
            .locations
            .values()
            .filter(|location| {
                location.source_path.as_deref().is_some_and(|path| {
                    ["city", "snow", "temple", "canyon"]
                        .iter()
                        .any(|area| path.starts_with(&format!("mudlib/d/{area}/")))
                })
            })
            .flat_map(|location| &location.object_sources)
            .filter(|source| {
                !npc_sources.contains(source.as_str()) && !item_sources.contains(source.as_str())
            })
            .map(String::as_str)
            .collect();
        let registered_defects: HashSet<_> = ledger["source_defects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["target"].as_str().unwrap())
            .collect();
        assert_eq!(unresolved_objects, registered_defects);
    }

    #[test]
    fn imported_rooms_have_display_content_and_m4_warnings_stay_visible() {
        let source_rooms: Vec<_> = world()
            .locations
            .values()
            .filter(|location| location.source_path.is_some())
            .collect();
        assert!(source_rooms.iter().all(|room| !room.name.trim().is_empty()));
        assert!(
            source_rooms
                .iter()
                .all(|room| !room.description.trim().is_empty())
        );
        let m4_behavior_flags: usize = source_rooms
            .iter()
            .filter(|room| {
                room.source_path.as_deref().is_some_and(|path| {
                    ["city", "snow", "temple", "canyon"]
                        .iter()
                        .any(|area| path.starts_with(&format!("mudlib/d/{area}/")))
                })
            })
            .map(|room| room.behavior_flags.len())
            .sum();
        assert_eq!(m4_behavior_flags, 107);
    }
}
