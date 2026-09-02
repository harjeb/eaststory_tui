use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use crate::{
    game::{EnemyKind, Exit, Location, LocationId, RoomDetail, RoomItemPlacement, SourceDoor},
    items::items,
    npcs::{
        FARM_WOMAN_ID, FISHER_ID, FLOWER_GIRL_ID, MELONER_ID, NpcId, OLD_LIU_ID,
        TEMPLE_LIBRARY_GUARD_ID, TEMPLE_LIBRARY_GUARD_PEER_ID, TEMPLE_MASTER_ID, TRADER_ID,
        XIAO_JUAN_ID, npcs,
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
pub const CITY_MANOR_ROAD_TWO: &str = "city.shangshu.road2";
pub const CITY_STREET3: &str = "city.street3";
pub const CITY_WALL: &str = "city.wall";
pub const CITY_MANOR_RUIN: &str = "city.shangshu.feiwu";
pub const CITY_RUINED_GARDEN: &str = "city.feiyuan";
pub const CITY_NORTH_GATE: &str = "city.northdoor";
pub const CITY_NORTH_ROAD: &str = "city.nroad1";
pub const PINE_FOREST: &str = "solo.pine_forest";
pub const SNOW_TOWN: &str = "solo.snow_town";
pub const MOUNTAIN_PATH: &str = "solo.mountain_path";
pub const TEMPLE_ROAD_TWO: &str = "temple.road2";
pub const TEMPLE_YARD: &str = "solo.temple_yard";
pub const OLD_PINE_SOUTH_PATH: &str = "oldpine.spath4";
pub const CHOYIN_NORTH_GATE: &str = "choyin.n_gate";
pub const OLD_PINE_CAVE_PREFIX: &str = "oldpine.cave";
pub const OLD_PINE_FOREST_PREFIX: &str = "oldpine.pine";
pub const CHUENYU_SOUTH_ROAD: &str = "chuenyu.croad1";
pub const M9_HIDDEN_SOURCE_LOCATIONS: [&str; 3] =
    ["snow.herbshop1", "temple.broom1", "temple.broom2"];

const RUNTIME_EXCLUDED_SOURCE_EXITS: [(&str, &str); 3] = [
    ("city.nroad1", "city.room"),
    ("snow.inn", "wiz.entrance"),
    ("waterfog.entrance", "waterfog.guildhall"),
];
const OLD_PINE_CAVE_TARGETS: [&str; 4] = [
    "oldpine.cave1",
    "oldpine.cave2",
    "oldpine.cave3",
    "oldpine.cave4",
];
const OLD_PINE_FOREST_CLIFF_TARGETS: [&str; 6] = [
    "oldpine.pine1",
    "oldpine.pine2",
    "oldpine.pine3",
    "oldpine.pine4",
    "oldpine.pine5",
    "oldpine.pine6",
];
const OLD_PINE_FOREST_TARGETS: [&str; 4] = [
    "oldpine.pine2",
    "oldpine.pine3",
    "oldpine.pine4",
    "oldpine.pine5",
];

const SOURCE_COMMIT: &str = "87bba6bd2249beec8424b0d6623486a0dd1f7b30";
const SOURCE_AREAS: [(&str, &str, &str); 17] = [
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
    (
        "oldpine",
        "黑松山",
        include_str!("../migration/catalog/oldpine.json"),
    ),
    (
        "goathill",
        "牧羊山",
        include_str!("../migration/catalog/goathill.json"),
    ),
    (
        "choyin",
        "乔阴县",
        include_str!("../migration/catalog/choyin.json"),
    ),
    (
        "chuenyu",
        "绮云镇",
        include_str!("../migration/catalog/chuenyu.json"),
    ),
    (
        "green",
        "青石村",
        include_str!("../migration/catalog/green.json"),
    ),
    (
        "sanyen",
        "三烟寺",
        include_str!("../migration/catalog/sanyen.json"),
    ),
    (
        "waterfog",
        "水烟阁",
        include_str!("../migration/catalog/waterfog.json"),
    ),
    (
        "latemoon",
        "晚月庄",
        include_str!("../migration/catalog/latemoon.json"),
    ),
    (
        "death",
        "幽冥地界",
        include_str!("../migration/catalog/death.json"),
    ),
    (
        "graveyard",
        "荒冢",
        include_str!("../migration/catalog/graveyard.json"),
    ),
    (
        "jail",
        "牢狱",
        include_str!("../migration/catalog/jail.json"),
    ),
    (
        "cloud",
        "青云境",
        include_str!("../migration/catalog/cloud.json"),
    ),
];

static WORLD: LazyLock<World> = LazyLock::new(World::load);

pub fn world() -> &'static World {
    &WORLD
}

pub fn dynamic_exit_target_candidates(
    current: &LocationId,
    target: &LocationId,
) -> Option<&'static [&'static str]> {
    match (current.as_str(), target.as_str()) {
        (source, OLD_PINE_CAVE_PREFIX) if source.starts_with(OLD_PINE_CAVE_PREFIX) => {
            Some(&OLD_PINE_CAVE_TARGETS)
        }
        ("oldpine.cliffdown", OLD_PINE_FOREST_PREFIX) => Some(&OLD_PINE_FOREST_CLIFF_TARGETS),
        (source, OLD_PINE_FOREST_PREFIX) if source.starts_with(OLD_PINE_FOREST_PREFIX) => {
            Some(&OLD_PINE_FOREST_TARGETS)
        }
        _ => None,
    }
}

fn is_runtime_excluded_source_exit(location: &str, target: &str) -> bool {
    RUNTIME_EXCLUDED_SOURCE_EXITS.contains(&(location, target))
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
            assert_eq!(catalog.schema_version, 4, "unsupported content schema");
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
                    .filter(|exit| !is_runtime_excluded_source_exit(id.as_str(), &exit.target))
                    .map(|exit| Exit {
                        direction: exit.direction,
                        target: LocationId::new(exit.target),
                        source_target: Some(exit.source_target),
                        internal: exit.internal,
                        dynamic: exit.dynamic,
                    })
                    .collect();
                let doors = room
                    .doors
                    .into_iter()
                    .map(|door| SourceDoor {
                        direction: door.direction,
                        name: door.name,
                        reverse_direction: door.reverse_direction,
                        initially_closed: door.initially_closed,
                    })
                    .collect();
                let details = room
                    .details
                    .into_iter()
                    .map(|detail| RoomDetail {
                        key: detail.key,
                        description: detail.description,
                        door_direction: detail.door_direction,
                    })
                    .collect();
                let room_npcs = room
                    .object_placements
                    .iter()
                    .filter_map(|placement| {
                        npcs()
                            .id_for_source(&placement.source_path)
                            .cloned()
                            .map(|npc| (npc, placement.quantity))
                    })
                    .flat_map(|(npc, count)| std::iter::repeat_n(npc, count as usize))
                    .collect();
                let room_items = room
                    .object_placements
                    .iter()
                    .filter_map(|placement| {
                        items()
                            .id_for_source(&placement.source_path)
                            .cloned()
                            .map(|item_id| RoomItemPlacement {
                                item_id,
                                count: placement.quantity,
                            })
                    })
                    .collect();
                let mut location = Location {
                    id: id.clone(),
                    name: room.name,
                    zone: zone.into(),
                    description: room.description,
                    arrival: format!("你进入{zone}地界。"),
                    outdoors: room.outdoors,
                    exits,
                    doors,
                    details,
                    npcs: room_npcs,
                    room_items,
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

    pub fn locations(&self) -> impl Iterator<Item = &Location> {
        self.locations.values()
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
                    .filter(|exit| {
                        !self.contains(&exit.target)
                            && dynamic_exit_target_candidates(&location.id, &exit.target).is_none()
                    })
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
        TEMPLE_ROAD_TWO => {
            // The source reset chooses one NPC from each three-person guard family.
            location.npcs.push(NpcId::from(TEMPLE_LIBRARY_GUARD_ID));
            location
                .npcs
                .push(NpcId::from(TEMPLE_LIBRARY_GUARD_PEER_ID));
        }
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
    locations
        .get_mut(&LocationId::from(OLD_PINE_SOUTH_PATH))
        .expect("oldpine south path exists")
        .exits
        .push(Exit::adapter("south", CHOYIN_NORTH_GATE));
    locations
        .get_mut(&LocationId::from(CHOYIN_NORTH_GATE))
        .expect("choyin north gate exists")
        .exits
        .push(Exit::adapter("north", OLD_PINE_SOUTH_PATH));
    locations
        .get_mut(&LocationId::from("village.road1"))
        .expect("village road1 exists")
        .exits
        .push(Exit::adapter("东北岔路", CHUENYU_SOUTH_ROAD));
    locations
        .get_mut(&LocationId::from("chuenyu.east_castle"))
        .expect("chuenyu east castle exists")
        .exits
        .push(Exit {
            direction: "down".into(),
            target: LocationId::from("chuenyu.tunnel4"),
            source_target: Some("runtime reciprocal of tunnel4 slab".into()),
            internal: true,
            dynamic: true,
        });

    for (source, direction, target) in [
        ("oldpine.clearing", "climb pine", "oldpine.tree1"),
        ("oldpine.cliff1", "climb up", "oldpine.cliffside"),
        ("oldpine.cliff1", "climb down", "oldpine.riverbank1"),
        ("oldpine.cliff2", "climb up", "oldpine.cliffdown"),
        ("oldpine.cliff2", "climb down", "oldpine.epath3"),
        ("oldpine.cliffdown", "climb down", "oldpine.cliff2"),
        ("oldpine.path3", "climb up", "oldpine.stone"),
        ("oldpine.riverbank1", "climb cliff", "oldpine.cliff1"),
        ("oldpine.stone", "climb down", "oldpine.cave1"),
    ] {
        locations
            .get_mut(&LocationId::from(source))
            .unwrap_or_else(|| panic!("M5 scripted source room {source} exists"))
            .exits
            .push(Exit::adapter(direction, target));
    }
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
            Some(NpcId::from(XIAO_JUAN_ID)),
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
    #[serde(default)]
    outdoors: Option<String>,
    exits: Vec<ExitRecord>,
    doors: Vec<DoorRecord>,
    details: Vec<RoomDetailRecord>,
    object_sources: Vec<String>,
    object_placements: Vec<ObjectPlacementRecord>,
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

#[derive(Deserialize)]
struct DoorRecord {
    direction: String,
    name: String,
    reverse_direction: String,
    initially_closed: bool,
}

#[derive(Deserialize)]
struct RoomDetailRecord {
    key: String,
    description: Option<String>,
    door_direction: Option<String>,
}

#[derive(Deserialize)]
struct ObjectPlacementRecord {
    source_path: String,
    quantity: u32,
}

#[cfg(test)]
mod tests {
    use std::collections::{HashSet, VecDeque};

    use super::*;

    #[test]
    fn embeds_all_m1_through_m7_rooms() {
        assert_eq!(world().source_room_count(), 552);
        assert_eq!(world().len(), 556);
        assert_eq!(world().area_room_count("village"), 26);
        assert_eq!(world().area_room_count("city"), 55);
        assert_eq!(world().area_room_count("snow"), 38);
        assert_eq!(world().area_room_count("temple"), 27);
        assert_eq!(world().area_room_count("canyon"), 27);
        assert_eq!(world().area_room_count("oldpine"), 41);
        assert_eq!(world().area_room_count("goathill"), 16);
        assert_eq!(world().area_room_count("choyin"), 62);
        assert_eq!(world().area_room_count("chuenyu"), 37);
        assert_eq!(world().area_room_count("green"), 39);
        assert_eq!(world().area_room_count("sanyen"), 18);
        assert_eq!(world().area_room_count("waterfog"), 27);
        assert_eq!(world().area_room_count("latemoon"), 74);
        assert_eq!(world().area_room_count("death"), 12);
        assert_eq!(world().area_room_count("graveyard"), 2);
        assert_eq!(world().area_room_count("jail"), 0);
        assert_eq!(world().area_room_count("cloud"), 51);
        assert!(world().contains(&LocationId::from(LIU_HOME)));
    }

    #[test]
    fn source_outdoors_are_preserved_for_every_imported_room() {
        let source_rooms: Vec<_> = world()
            .locations()
            .filter(|location| location.source_path.is_some())
            .collect();
        assert_eq!(source_rooms.len(), 552);
        assert_eq!(
            source_rooms
                .iter()
                .filter(|location| location.outdoors.is_some())
                .count(),
            225
        );
        assert_eq!(
            world()
                .location(&LocationId::from("choyin.bridge2"))
                .unwrap()
                .outdoors
                .as_deref(),
            Some("choyin")
        );
        assert_eq!(
            world()
                .location(&LocationId::from("latemoon.entrance"))
                .unwrap()
                .outdoors
                .as_deref(),
            Some("cloud")
        );
        assert!(
            world()
                .location(&LocationId::from(LIU_HOME))
                .unwrap()
                .outdoors
                .is_none()
        );
    }

    #[test]
    fn m6_static_catalogs_and_runtime_baseline_match_the_fixed_source() {
        let catalogs: [serde_json::Value; 4] = [
            serde_json::from_str(include_str!("../migration/catalog/chuenyu.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/green.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/sanyen.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/waterfog.json")).unwrap(),
        ];
        assert_eq!(
            catalogs
                .iter()
                .map(|catalog| catalog["rooms"].as_array().unwrap().len())
                .sum::<usize>(),
            121
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["behavior_flags"].as_array().unwrap().len())
                .sum::<usize>(),
            65
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["doors"].as_array().unwrap().len())
                .sum::<usize>(),
            16
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["details"].as_array().unwrap().len())
                .sum::<usize>(),
            38
        );
        let placements = catalogs
            .iter()
            .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
            .flat_map(|room| room["object_placements"].as_array().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(placements.len(), 60);
        assert_eq!(
            placements
                .iter()
                .map(|placement| placement["quantity"].as_u64().unwrap())
                .sum::<u64>(),
            105
        );

        let topology: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m6-topology.json")).unwrap();
        assert_eq!(topology["schema_version"], 3);
        assert_eq!(topology["source_commit"], SOURCE_COMMIT);
        assert_eq!(topology["status"], "complete");
        assert_eq!(topology["catalog"]["rooms"], 121);
        assert_eq!(topology["catalog"]["warning_entries"], 65);
        assert_eq!(topology["catalog"]["npc_instances"], 92);
        assert_eq!(topology["catalog"]["fixed_room_item_instances"], 11);
        assert_eq!(topology["static_runtime"]["save_schema"], 21);
        assert_eq!(topology["behavior_flags"]["disposed"], 65);
        assert_eq!(topology["behavior_flags"]["remaining"], 0);
        assert_eq!(topology["behavior_flags"]["by_status"]["verified"], 29);
        assert_eq!(topology["behavior_flags"]["by_status"]["adapted"], 34);
        assert_eq!(topology["behavior_flags"]["by_status"]["excluded"], 2);
        assert_eq!(topology["regional_acceptance"]["status"], "complete");
        assert_eq!(topology["regional_acceptance"]["save_schema"], 22);
        let accepted_areas = topology["regional_acceptance"]["areas"].as_array().unwrap();
        assert_eq!(accepted_areas.len(), 4);
        assert!(
            accepted_areas
                .iter()
                .all(|area| area["status"] == "verified")
        );
        let room_catalog_flags: HashSet<_> = catalogs
            .iter()
            .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
            .flat_map(|room| {
                let source = room["source_path"].as_str().unwrap().to_string();
                room["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_room_flags: Vec<_> = topology["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_room_flags.len(), 65);
        assert_eq!(
            disposed_room_flags.iter().cloned().collect::<HashSet<_>>(),
            room_catalog_flags
        );

        let item_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/items.json")).unwrap();
        let item_ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m6-items.json")).unwrap();
        assert_eq!(item_ledger["schema_version"], 1);
        assert_eq!(item_ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(item_ledger["status"], "complete");
        assert_eq!(item_ledger["behavior_flags"]["total"], 3);
        assert_eq!(item_ledger["behavior_flags"]["disposed"], 3);
        assert_eq!(item_ledger["behavior_flags"]["remaining"], 0);
        assert_eq!(item_ledger["behavior_flags"]["by_status"]["adapted"], 2);
        assert_eq!(item_ledger["behavior_flags"]["by_status"]["excluded"], 1);
        let item_catalog_flags: HashSet<_> = item_catalog["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| {
                let source = item["source_path"].as_str().unwrap();
                [
                    "mudlib/d/chuenyu/",
                    "mudlib/d/green/",
                    "mudlib/d/sanyen/",
                    "mudlib/d/waterfog/",
                ]
                .iter()
                .any(|prefix| source.starts_with(prefix))
            })
            .flat_map(|item| {
                let source = item["source_path"].as_str().unwrap().to_string();
                item["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_item_flags: Vec<_> = item_ledger["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_item_flags.len(), 3);
        assert_eq!(
            disposed_item_flags.iter().cloned().collect::<HashSet<_>>(),
            item_catalog_flags
        );

        let npc_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/npcs-m6.json")).unwrap();
        let npc_ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m6-npcs.json")).unwrap();
        assert_eq!(npc_ledger["schema_version"], 2);
        assert_eq!(npc_ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(npc_ledger["status"], "complete");
        assert_eq!(npc_ledger["catalog"]["definitions"], 57);
        assert_eq!(npc_ledger["catalog"]["runtime_instances"], 92);
        assert_eq!(npc_ledger["catalog"]["save_schema"], 22);
        assert_eq!(npc_ledger["behavior_flags"]["total"], 81);
        assert_eq!(npc_ledger["behavior_flags"]["disposed"], 81);
        assert_eq!(npc_ledger["behavior_flags"]["remaining"], 0);
        for (status, expected) in [
            ("verified", 17),
            ("adapted", 23),
            ("deferred", 23),
            ("excluded", 13),
            ("alias", 4),
            ("source_noop", 1),
        ] {
            assert_eq!(npc_ledger["behavior_flags"]["by_status"][status], expected);
        }
        let npc_catalog_flags: HashSet<_> = npc_catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|npc| {
                let source = npc["source_path"].as_str().unwrap().to_string();
                npc["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_npc_flags: Vec<_> = npc_ledger["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_npc_flags.len(), 81);
        assert_eq!(
            disposed_npc_flags.iter().cloned().collect::<HashSet<_>>(),
            npc_catalog_flags
        );

        let m6_locations = world()
            .locations()
            .filter(|location| {
                ["chuenyu", "green", "sanyen", "waterfog"]
                    .iter()
                    .any(|area| location.id.as_str().starts_with(&format!("{area}.")))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            m6_locations
                .iter()
                .map(|location| location.npcs.len())
                .sum::<usize>(),
            92
        );
        assert_eq!(
            m6_locations
                .iter()
                .flat_map(|location| &location.npcs)
                .collect::<HashSet<_>>()
                .len(),
            48
        );
        assert!(
            world()
                .location(&LocationId::from("village.road1"))
                .unwrap()
                .exits
                .iter()
                .any(|exit| exit.target.as_str() == CHUENYU_SOUTH_ROAD)
        );
        assert!(
            world()
                .location(&LocationId::from("snow.crossroad"))
                .unwrap()
                .exits
                .iter()
                .any(|exit| exit.target.as_str() == "green.path6")
        );
        assert!(
            world()
                .location(&LocationId::from("snow.sroad5"))
                .unwrap()
                .exits
                .iter()
                .any(|exit| exit.target.as_str() == "waterfog.sroad1")
        );
    }

    #[test]
    fn m7_static_catalogs_and_runtime_baseline_match_the_fixed_source() {
        let catalogs: [serde_json::Value; 5] = [
            serde_json::from_str(include_str!("../migration/catalog/latemoon.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/death.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/graveyard.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/jail.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/cloud.json")).unwrap(),
        ];
        assert_eq!(
            catalogs
                .iter()
                .map(|catalog| catalog["rooms"].as_array().unwrap().len())
                .sum::<usize>(),
            139
        );
        assert_eq!(
            catalogs
                .iter()
                .map(|catalog| catalog["non_room_files"].as_array().unwrap().len())
                .sum::<usize>(),
            204
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["behavior_flags"].as_array().unwrap().len())
                .sum::<usize>(),
            77
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["doors"].as_array().unwrap().len())
                .sum::<usize>(),
            32
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["details"].as_array().unwrap().len())
                .sum::<usize>(),
            22
        );
        let placements = catalogs
            .iter()
            .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
            .flat_map(|room| room["object_placements"].as_array().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(placements.len(), 78);
        assert_eq!(
            placements
                .iter()
                .map(|placement| placement["quantity"].as_u64().unwrap())
                .sum::<u64>(),
            117
        );

        let topology: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m7-topology.json")).unwrap();
        assert_eq!(topology["schema_version"], 3);
        assert_eq!(topology["source_commit"], SOURCE_COMMIT);
        assert_eq!(topology["status"], "complete");
        assert_eq!(topology["catalog"]["rooms"], 139);
        assert_eq!(topology["catalog"]["non_room_files"], 204);
        assert_eq!(topology["catalog"]["warning_entries"], 77);
        assert_eq!(topology["catalog"]["object_instances"], 117);
        assert_eq!(topology["catalog"]["npc_instances"], 105);
        assert_eq!(topology["catalog"]["m7_npc_instances"], 101);
        assert_eq!(topology["catalog"]["fixed_room_item_instances"], 12);
        assert_eq!(topology["static_runtime"]["save_schema"], 25);
        assert_eq!(topology["behavior_flags"]["total"], 77);
        assert_eq!(topology["behavior_flags"]["disposed"], 77);
        assert_eq!(topology["behavior_flags"]["remaining"], 0);
        assert_eq!(topology["behavior_flags"]["by_status"]["verified"], 50);
        assert_eq!(topology["behavior_flags"]["by_status"]["adapted"], 23);
        assert_eq!(topology["behavior_flags"]["by_status"]["deferred"], 1);
        assert_eq!(topology["behavior_flags"]["by_status"]["excluded"], 1);
        assert_eq!(topology["behavior_flags"]["by_status"]["source_noop"], 2);
        let room_catalog_flags: HashSet<_> = catalogs
            .iter()
            .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
            .flat_map(|room| {
                let source = room["source_path"].as_str().unwrap().to_string();
                room["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_room_flags: Vec<_> = topology["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_room_flags.len(), 77);
        assert_eq!(
            disposed_room_flags.iter().cloned().collect::<HashSet<_>>(),
            room_catalog_flags
        );
        assert_eq!(topology["regional_acceptance"]["status"], "complete");
        assert_eq!(topology["regional_acceptance"]["save_schema"], 25);
        assert_eq!(
            topology["regional_acceptance"]["areas"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|area| area["status"] == "verified")
                .count(),
            5
        );

        let item_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/items.json")).unwrap();
        let item_ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m7-items.json")).unwrap();
        assert_eq!(item_ledger["schema_version"], 1);
        assert_eq!(item_ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(item_ledger["status"], "complete");
        assert_eq!(item_ledger["behavior_flags"]["total"], 22);
        assert_eq!(item_ledger["behavior_flags"]["disposed"], 22);
        assert_eq!(item_ledger["behavior_flags"]["remaining"], 0);
        assert_eq!(item_ledger["behavior_flags"]["by_status"]["verified"], 9);
        assert_eq!(item_ledger["behavior_flags"]["by_status"]["adapted"], 13);
        let item_catalog_flags: HashSet<_> = item_catalog["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| {
                let source = item["source_path"].as_str().unwrap();
                [
                    "mudlib/d/latemoon/",
                    "mudlib/d/death/",
                    "mudlib/d/graveyard/",
                    "mudlib/d/jail/",
                    "mudlib/u/cloud/",
                ]
                .iter()
                .any(|prefix| source.starts_with(prefix))
            })
            .flat_map(|item| {
                let source = item["source_path"].as_str().unwrap().to_string();
                item["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_item_flags: Vec<_> = item_ledger["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_item_flags.len(), 22);
        assert_eq!(
            disposed_item_flags.iter().cloned().collect::<HashSet<_>>(),
            item_catalog_flags
        );

        let npc_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/npcs-m7.json")).unwrap();
        let npc_ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m7-npcs.json")).unwrap();
        assert_eq!(npc_ledger["schema_version"], 2);
        assert_eq!(npc_ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(npc_ledger["status"], "complete");
        assert_eq!(npc_ledger["catalog"]["definitions"], 85);
        assert_eq!(npc_ledger["catalog"]["placed_definitions"], 65);
        assert_eq!(npc_ledger["catalog"]["runtime_instances"], 101);
        assert_eq!(npc_ledger["catalog"]["world_m7_npc_instances"], 105);
        assert_eq!(npc_ledger["catalog"]["vendors"], 8);
        assert_eq!(npc_ledger["catalog"]["vendor_goods"], 29);
        assert_eq!(npc_ledger["catalog"]["inquiry_topics"], 67);
        assert_eq!(npc_ledger["catalog"]["combat_profiles"], 85);
        assert_eq!(npc_ledger["catalog"]["carried_item_entries"], 122);
        assert_eq!(npc_ledger["catalog"]["save_schema"], 25);
        assert_eq!(npc_ledger["behavior_flags"]["total"], 183);
        assert_eq!(npc_ledger["behavior_flags"]["disposed"], 183);
        assert_eq!(npc_ledger["behavior_flags"]["remaining"], 0);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["verified"], 33);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["adapted"], 50);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["deferred"], 51);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["excluded"], 38);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["alias"], 10);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["source_noop"], 1);
        assert_eq!(
            npc_catalog["npcs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|npc| npc["placement_count"].as_u64().unwrap())
                .sum::<u64>(),
            101
        );
        assert_eq!(
            npc_catalog["npcs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|npc| npc["behavior_flags"].as_array().unwrap().len())
                .sum::<usize>(),
            183
        );
        let npc_catalog_flags: HashSet<_> = npc_catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|npc| {
                let source = npc["source_path"].as_str().unwrap().to_string();
                npc["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_npc_flags: Vec<_> = npc_ledger["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let flags = entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|flag| flag.as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                entry["source_paths"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(move |source| {
                        let source = source.as_str().unwrap().to_string();
                        flags
                            .clone()
                            .into_iter()
                            .map(move |flag| (source.clone(), flag))
                    })
            })
            .collect();
        assert_eq!(disposed_npc_flags.len(), 183);
        assert_eq!(
            disposed_npc_flags.iter().cloned().collect::<HashSet<_>>(),
            npc_catalog_flags
        );
        assert_eq!(
            crate::npcs::npcs()
                .id_for_source("mudlib/d/snow/npc/beggar.c")
                .unwrap()
                .as_str(),
            "snow.npc.beggar"
        );
        assert_eq!(
            crate::npcs::npcs()
                .id_for_source("mudlib/obj/npc/garrison.c")
                .unwrap()
                .as_str(),
            "obj.npc.garrison"
        );

        let m7_locations = world()
            .locations()
            .filter(|location| {
                ["latemoon", "death", "graveyard", "jail", "u.cloud"]
                    .iter()
                    .any(|prefix| location.id.as_str().starts_with(&format!("{prefix}.")))
            })
            .collect::<Vec<_>>();
        assert_eq!(m7_locations.len(), 139);
        assert_eq!(
            m7_locations
                .iter()
                .map(|location| location.npcs.len())
                .sum::<usize>(),
            105
        );
        for (from, target) in [
            ("latemoon.entrance", "u.cloud.wroad0"),
            ("u.cloud.wroad0", "latemoon.entrance"),
            ("latemoon.sroad5", "sanyen.tunnel"),
            ("u.cloud.dragonhill.nroad", "snow.sroad1"),
            ("u.cloud.sunhill.road1", "choyin.n_gate"),
            ("u.cloud.sunhill.road4", "sanyen.sroad1"),
        ] {
            assert!(
                world()
                    .location(&LocationId::from(from))
                    .unwrap()
                    .exits
                    .iter()
                    .any(|exit| exit.target.as_str() == target),
                "{from} lacks exit to {target}"
            );
        }
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
    fn m9_runtime_exits_and_dynamic_targets_are_resolved() {
        assert!(world().unresolved_exits().is_empty());

        let dynamic_targets: HashSet<_> = world()
            .locations()
            .flat_map(|location| {
                location.exits.iter().filter_map(move |exit| {
                    dynamic_exit_target_candidates(&location.id, &exit.target)
                        .map(|targets| (location.id.as_str(), exit.target.as_str(), targets))
                })
            })
            .collect();
        assert_eq!(
            dynamic_targets
                .iter()
                .map(|(_, target, _)| *target)
                .collect::<HashSet<_>>(),
            HashSet::from([OLD_PINE_CAVE_PREFIX, OLD_PINE_FOREST_PREFIX])
        );
        for (_, _, targets) in dynamic_targets {
            assert!(
                targets
                    .iter()
                    .all(|target| world().contains(&LocationId::from(*target)))
            );
        }
    }

    #[test]
    fn m9_every_runtime_reference_has_a_registered_definition() {
        for location in world().locations() {
            assert!(
                location
                    .npcs
                    .iter()
                    .all(|npc| crate::npcs::npcs().definition(npc).is_some()),
                "{} contains an unknown NPC",
                location.id.as_str()
            );
            assert!(
                location
                    .room_items
                    .iter()
                    .all(|item| crate::items::items().definition(&item.item_id).is_some()),
                "{} contains an unknown item",
                location.id.as_str()
            );
            assert!(
                location
                    .training
                    .as_ref()
                    .is_none_or(|skill| crate::skills::skills().definition(skill).is_some()),
                "{} has an unknown training skill",
                location.id.as_str()
            );
        }
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
            .filter(|location| {
                location.source_path.as_deref().is_some_and(|path| {
                    ["village", "city", "snow", "temple", "canyon"]
                        .iter()
                        .any(|area| path.starts_with(&format!("mudlib/d/{area}/")))
                }) && !visited.contains(&location.id)
            })
            .map(|location| location.id.as_str())
            .collect();
        assert_eq!(unreachable, HashSet::from(M9_HIDDEN_SOURCE_LOCATIONS));
    }

    #[test]
    fn m5_static_catalogs_and_mainline_adapter_match_the_fixed_source() {
        let catalogs: [serde_json::Value; 3] = [
            serde_json::from_str(include_str!("../migration/catalog/oldpine.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/goathill.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/choyin.json")).unwrap(),
        ];
        assert_eq!(
            catalogs
                .iter()
                .map(|catalog| catalog["rooms"].as_array().unwrap().len())
                .sum::<usize>(),
            119
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["behavior_flags"].as_array().unwrap().len())
                .sum::<usize>(),
            73
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["details"].as_array().unwrap().len())
                .sum::<usize>(),
            25
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .map(|room| room["object_placements"].as_array().unwrap().len())
                .sum::<usize>(),
            68
        );
        assert_eq!(
            catalogs
                .iter()
                .flat_map(|catalog| catalog["rooms"].as_array().unwrap())
                .flat_map(|room| room["object_placements"].as_array().unwrap())
                .map(|placement| placement["quantity"].as_u64().unwrap())
                .sum::<u64>(),
            138
        );

        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m5-topology.json")).unwrap();
        assert_eq!(ledger["schema_version"], 6);
        assert_eq!(ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(ledger["status"], "complete");
        assert_eq!(ledger["catalog"]["fixed_room_item_instances"], 10);
        assert_eq!(ledger["static_runtime"]["save_schema"], 15);
        let generic_features = &ledger["generic_room_features"];
        assert_eq!(generic_features["warning_entries_disposed"], 23);
        assert_eq!(generic_features["door_audit"]["warning_entries"], 4);
        assert_eq!(generic_features["door_audit"]["source_definitions"], 4);
        assert_eq!(generic_features["door_audit"]["runtime_pairs"], 1);
        assert_eq!(generic_features["door_audit"]["runtime_endpoints"], 2);
        assert_eq!(
            generic_features["door_audit"]["status_counts"]["verified"],
            2
        );
        assert_eq!(
            generic_features["door_audit"]["status_counts"]["excluded"],
            2
        );
        assert_eq!(generic_features["room_detail_audit"]["warning_entries"], 19);
        assert_eq!(generic_features["room_detail_audit"]["rooms"], 19);
        assert_eq!(generic_features["room_detail_audit"]["details"], 25);
        assert_eq!(generic_features["room_detail_audit"]["text_details"], 25);
        assert_eq!(
            generic_features["room_detail_audit"]["status_counts"]["verified"],
            16
        );
        assert_eq!(
            generic_features["room_detail_audit"]["status_counts"]["adapted"],
            3
        );
        let scripted_batch = &ledger["scripted_room_batch_1"];
        assert_eq!(scripted_batch["warning_entries_disposed"], 29);
        assert_eq!(scripted_batch["rooms"], 19);
        assert_eq!(scripted_batch["status_counts"]["adapted"], 28);
        assert_eq!(scripted_batch["status_counts"]["source_noop"], 1);
        assert_eq!(
            scripted_batch["dispositions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["warning_entries"].as_u64().unwrap())
                .sum::<u64>(),
            29
        );
        assert!(
            ledger["random_topology"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entry| entry["status"] == "adapted")
        );
        let choyin_batch = &ledger["scripted_room_batch_2"];
        assert_eq!(choyin_batch["warning_entries_disposed"], 9);
        assert_eq!(choyin_batch["rooms"], 4);
        assert_eq!(choyin_batch["save_schema"], 16);
        assert_eq!(choyin_batch["status_counts"]["verified"], 1);
        assert_eq!(choyin_batch["status_counts"]["adapted"], 8);
        assert_eq!(
            choyin_batch["dispositions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["warning_entries"].as_u64().unwrap())
                .sum::<u64>(),
            9
        );
        let final_room_batch = &ledger["scripted_room_batch_3"];
        assert_eq!(final_room_batch["warning_entries_disposed"], 12);
        assert_eq!(final_room_batch["rooms"], 7);
        assert_eq!(final_room_batch["save_schema"], 19);
        assert_eq!(final_room_batch["status_counts"]["adapted"], 10);
        assert_eq!(final_room_batch["status_counts"]["deferred"], 1);
        assert_eq!(final_room_batch["status_counts"]["excluded"], 1);
        assert_eq!(
            final_room_batch["dispositions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["warning_entries"].as_u64().unwrap())
                .sum::<u64>(),
            12
        );
        assert_eq!(
            generic_features["warning_entries_disposed"]
                .as_u64()
                .unwrap()
                + scripted_batch["warning_entries_disposed"].as_u64().unwrap()
                + choyin_batch["warning_entries_disposed"].as_u64().unwrap()
                + final_room_batch["warning_entries_disposed"]
                    .as_u64()
                    .unwrap(),
            ledger["catalog"]["warning_entries"].as_u64().unwrap()
        );
        assert_eq!(ledger["regional_gameplay"]["status"], "complete");
        assert_eq!(ledger["regional_gameplay"]["save_schema"], 19);
        assert!(
            ledger["regional_gameplay"]["acceptance"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entry| entry["status"] == "verified")
        );
        let regional_features: std::collections::HashSet<_> =
            ledger["regional_gameplay"]["runtime_features"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["feature"].as_str().unwrap())
                .collect();
        assert_eq!(
            regional_features,
            std::collections::HashSet::from([
                "oldpine_reinforcement_and_poison",
                "goathill_leech_corpse_tonic",
                "choyin_rope_tablet_and_scholar_trial",
                "source_unplaced_definitions",
            ])
        );
        assert_eq!(
            ledger["importer_resolutions"][0]["source_path"],
            "mudlib/d/choyin/stove.c"
        );
        assert!(
            world()
                .location(&LocationId::from("choyin.stove"))
                .unwrap()
                .exits
                .iter()
                .any(|exit| exit.target.as_str() == "choyin.tongbhill")
        );

        let npc_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/npcs-m5.json")).unwrap();
        let npc_ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m5-npcs.json")).unwrap();
        assert_eq!(npc_ledger["schema_version"], 3);
        assert_eq!(npc_ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(npc_ledger["status"], "complete");
        assert_eq!(npc_ledger["catalog"]["definitions"], 49);
        assert_eq!(npc_ledger["catalog"]["placed_definitions"], 46);
        assert_eq!(npc_ledger["catalog"]["runtime_instances"], 128);
        assert_eq!(npc_ledger["catalog"]["dynamic_spawn_definitions"], 1);
        assert_eq!(npc_ledger["catalog"]["dynamic_spawn_sites"], 2);
        assert_eq!(
            npc_ledger["catalog"]["runtime_instances_with_dynamic_max"],
            130
        );
        assert_eq!(npc_ledger["catalog"]["runtime_drop_entries"], 136);
        assert_eq!(
            npc_ledger["catalog"]["runtime_drop_entries_with_dynamic_max"],
            140
        );
        assert_eq!(npc_ledger["behavior_flags"]["total"], 59);
        assert_eq!(
            npc_catalog["npcs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|npc| npc["behavior_flags"].as_array().unwrap().len())
                .sum::<usize>(),
            59
        );
        let npc_features: std::collections::BTreeMap<_, _> = npc_ledger["runtime_features"]
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
        assert_eq!(npc_features["source_npc_placements"], "adapted");
        assert_eq!(npc_features["source_static_vendors"], "verified");
        assert_eq!(npc_features["source_static_inquiries"], "verified");
        assert_eq!(npc_features["source_combat_profiles"], "adapted");
        assert_eq!(npc_features["source_combat_chat"], "adapted");
        assert_eq!(npc_features["source_carried_items"], "adapted");
        assert_eq!(npc_features["source_fight_gates"], "verified");
        assert_eq!(npc_features["hotel_guard_kill_gate"], "adapted");
        assert_eq!(npc_features["source_scripted_inquiries"], "adapted");
        assert_eq!(npc_features["source_object_exchanges"], "adapted");
        assert_eq!(npc_features["source_death_hooks"], "adapted");
        assert_eq!(npc_features["regional_npc_combat_hooks"], "adapted");
        assert_eq!(npc_features["behavior_flag_audit"], "audited");
        assert_eq!(npc_ledger["behavior_flags"]["disposed"], 59);
        assert_eq!(npc_ledger["behavior_flags"]["remaining"], 0);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["verified"], 14);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["adapted"], 21);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["deferred"], 23);
        assert_eq!(npc_ledger["behavior_flags"]["by_status"]["excluded"], 1);

        let catalog_flags: std::collections::HashSet<_> = npc_catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|npc| {
                let source_path = npc["source_path"].as_str().unwrap().to_string();
                npc["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source_path.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_flags: Vec<_> = npc_ledger["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source_path = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source_path.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_flags.len(), 59);
        assert_eq!(
            disposed_flags
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>(),
            catalog_flags
        );

        let item_catalog: serde_json::Value =
            serde_json::from_str(include_str!("../migration/catalog/items.json")).unwrap();
        let item_ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m5-items.json")).unwrap();
        assert_eq!(item_ledger["schema_version"], 1);
        assert_eq!(item_ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(item_ledger["status"], "complete");
        assert_eq!(item_ledger["behavior_flags"]["total"], 13);
        assert_eq!(item_ledger["behavior_flags"]["disposed"], 13);
        assert_eq!(item_ledger["behavior_flags"]["remaining"], 0);
        assert_eq!(item_ledger["behavior_flags"]["by_status"]["adapted"], 12);
        assert_eq!(item_ledger["behavior_flags"]["by_status"]["alias"], 1);
        let m5_item_flags: std::collections::HashSet<_> = item_catalog["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| {
                let source = item["source_path"].as_str().unwrap();
                [
                    "mudlib/d/oldpine/",
                    "mudlib/d/goathill/",
                    "mudlib/d/choyin/",
                ]
                .iter()
                .any(|prefix| source.starts_with(prefix))
            })
            .flat_map(|item| {
                let source = item["source_path"].as_str().unwrap().to_string();
                item["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        let disposed_item_flags: Vec<_> = item_ledger["behavior_flags"]["dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| {
                let source = entry["source_path"].as_str().unwrap().to_string();
                entry["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |flag| (source.clone(), flag.as_str().unwrap().to_string()))
            })
            .collect();
        assert_eq!(disposed_item_flags.len(), 13);
        assert_eq!(
            disposed_item_flags
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>(),
            m5_item_flags
        );
        let item_features: std::collections::HashSet<_> = item_ledger["runtime_features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["feature"].as_str().unwrap())
            .collect();
        assert!(item_features.contains("choyin_donation_box"));
        assert!(item_features.contains("choyin_golden_rope_and_tablet"));
        assert!(item_features.contains("oldpine_bamboo_pipe"));
        assert!(item_features.contains("goathill_dead_leech_tonic"));

        assert_eq!(
            world()
                .locations()
                .filter(|location| {
                    ["oldpine", "goathill", "choyin"]
                        .iter()
                        .any(|area| location.id.as_str().starts_with(&format!("{area}.")))
                })
                .map(|location| location.npcs.len())
                .sum::<usize>(),
            128
        );
        assert_eq!(
            world()
                .locations()
                .filter(|location| {
                    ["oldpine", "goathill", "choyin"]
                        .iter()
                        .any(|area| location.id.as_str().starts_with(&format!("{area}.")))
                })
                .flat_map(|location| &location.npcs)
                .collect::<HashSet<_>>()
                .len(),
            46
        );

        for (source, target) in [
            (OLD_PINE_SOUTH_PATH, CHOYIN_NORTH_GATE),
            (CHOYIN_NORTH_GATE, OLD_PINE_SOUTH_PATH),
        ] {
            let exit = world()
                .location(&LocationId::from(source))
                .unwrap()
                .exits
                .iter()
                .find(|exit| exit.target.as_str() == target)
                .unwrap();
            assert!(exit.source_target.is_none());
        }
        assert_eq!(
            world()
                .locations()
                .filter(|location| {
                    ["oldpine", "goathill", "choyin"]
                        .iter()
                        .any(|area| location.id.as_str().starts_with(&format!("{area}.")))
                })
                .flat_map(|location| &location.room_items)
                .map(|placement| placement.count as usize)
                .sum::<usize>(),
            10
        );
    }

    #[test]
    fn m4_topology_ledger_covers_adaptations_and_known_blockers() {
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/m4-topology.json")).unwrap();
        assert_eq!(ledger["schema_version"], 6);
        assert_eq!(ledger["source_commit"], SOURCE_COMMIT);
        assert_eq!(ledger["status"], "complete");
        let behavior_audit = &ledger["room_behavior_audit"];
        assert_eq!(behavior_audit["warning_entries"], 107);
        assert_eq!(behavior_audit["rooms_with_warnings"], 60);
        let event_batch = &behavior_audit["event_batch_1"];
        assert_eq!(event_batch["warning_entries_disposed"], 22);
        let dispositions = event_batch["dispositions"].as_array().unwrap();
        assert_eq!(dispositions.len(), 8);
        assert_eq!(
            dispositions
                .iter()
                .map(|entry| entry["flags"].as_array().unwrap().len())
                .sum::<usize>(),
            22
        );
        let disposition_statuses =
            dispositions
                .iter()
                .fold(std::collections::BTreeMap::new(), |mut counts, entry| {
                    *counts
                        .entry(entry["status"].as_str().unwrap())
                        .or_insert(0usize) += 1;
                    counts
                });
        assert_eq!(
            disposition_statuses,
            std::collections::BTreeMap::from([("adapted", 5), ("excluded", 1), ("verified", 2),])
        );

        let door_audit = &behavior_audit["door_audit"];
        assert_eq!(door_audit["warning_entries_disposed"], 35);
        assert_eq!(door_audit["source_definitions"], 36);
        assert_eq!(door_audit["runtime_pairs"], 14);
        assert_eq!(door_audit["runtime_endpoints"], 28);
        assert_eq!(door_audit["status_counts"]["verified"], 26);
        assert_eq!(door_audit["status_counts"]["excluded"], 9);
        let door_dispositions = door_audit["dispositions"].as_array().unwrap();
        assert_eq!(door_dispositions.len(), 3);
        assert_eq!(
            door_dispositions
                .iter()
                .map(|entry| entry["warning_entries"].as_u64().unwrap())
                .sum::<u64>(),
            35
        );
        assert_eq!(
            door_dispositions
                .iter()
                .map(|entry| entry["source_definitions"].as_u64().unwrap())
                .sum::<u64>(),
            36
        );

        let detail_audit = &behavior_audit["room_detail_audit"];
        assert_eq!(detail_audit["warning_entries_disposed"], 32);
        assert_eq!(detail_audit["rooms"], 32);
        assert_eq!(detail_audit["details"], 42);
        assert_eq!(detail_audit["text_details"], 30);
        assert_eq!(detail_audit["door_details"], 12);
        assert_eq!(detail_audit["status_counts"]["verified"], 16);
        assert_eq!(detail_audit["status_counts"]["adapted"], 16);
        let detail_dispositions = detail_audit["dispositions"].as_array().unwrap();
        assert_eq!(detail_dispositions.len(), 2);
        assert_eq!(
            detail_dispositions
                .iter()
                .map(|entry| entry["warning_entries"].as_u64().unwrap())
                .sum::<u64>(),
            32
        );
        assert_eq!(
            detail_dispositions
                .iter()
                .map(|entry| entry["details"].as_u64().unwrap())
                .sum::<u64>(),
            42
        );

        let dynamic_audit = &behavior_audit["dynamic_behavior_audit"];
        assert_eq!(dynamic_audit["warning_entries_disposed"], 40);
        let dynamic_dispositions = dynamic_audit["dispositions"].as_array().unwrap();
        assert_eq!(dynamic_dispositions.len(), 23);
        assert_eq!(
            dynamic_dispositions
                .iter()
                .map(|entry| entry["warning_entries"].as_u64().unwrap())
                .sum::<u64>(),
            40
        );
        assert_eq!(dynamic_audit["status_counts"]["verified"], 12);
        assert_eq!(dynamic_audit["status_counts"]["adapted"], 19);
        assert_eq!(dynamic_audit["status_counts"]["source_noop"], 1);
        assert_eq!(dynamic_audit["status_counts"]["deferred"], 1);
        assert_eq!(dynamic_audit["status_counts"]["excluded"], 7);
        assert_eq!(
            door_audit["warning_entries_disposed"].as_u64().unwrap()
                + detail_audit["warning_entries_disposed"].as_u64().unwrap()
                + dynamic_audit["warning_entries_disposed"].as_u64().unwrap(),
            behavior_audit["warning_entries"].as_u64().unwrap()
        );

        let object_audit = &ledger["room_object_audit"];
        assert_eq!(object_audit["source_mappings"], 84);
        assert_eq!(object_audit["source_instances"], 120);
        assert_eq!(object_audit["runtime_npc_definitions"], 61);
        assert_eq!(object_audit["runtime_npc_instances"], 115);
        assert_eq!(object_audit["runtime_item_placements"], 4);
        assert_eq!(object_audit["runtime_item_instances"], 5);
        let object_dispositions = object_audit["dispositions"].as_array().unwrap();
        assert_eq!(object_dispositions.len(), 3);
        assert_eq!(
            object_dispositions
                .iter()
                .map(|entry| entry["source_mappings"].as_u64().unwrap())
                .sum::<u64>(),
            84
        );
        assert_eq!(
            object_dispositions
                .iter()
                .map(|entry| entry["source_instances"].as_u64().unwrap())
                .sum::<u64>(),
            120
        );

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
        assert_eq!(registered_blockers, HashSet::<&str>::new());

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
        assert_eq!(ledger["status"], "complete");

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
        assert_eq!(runtime_features["source_placements"], "adapted");
        assert_eq!(runtime_features["source_inquiry_catalog"], "verified");
        assert_eq!(runtime_features["placed_static_inquiries"], "verified");
        assert_eq!(runtime_features["scripted_inquiry_runtime"], "adapted");
        assert_eq!(runtime_features["object_exchange_audit"], "verified");
        assert_eq!(runtime_features["snow_temple_donation"], "verified");
        assert_eq!(runtime_features["source_fight_gates"], "verified");
        assert_eq!(runtime_features["source_npc_lessons"], "verified");
        assert_eq!(runtime_features["source_npc_combat_profiles"], "adapted");
        assert_eq!(runtime_features["source_npc_carried_items"], "adapted");
        assert_eq!(runtime_features["source_npc_combat_chat"], "adapted");
        assert_eq!(runtime_features["source_custom_commands"], "adapted");
        assert_eq!(runtime_features["city_exit_token"], "adapted");
        assert_eq!(ledger["catalog"]["runtime_scripted_inquiries"], 10);
        assert_eq!(ledger["catalog"]["total_runtime_inquiry_npcs"], 26);
        assert_eq!(ledger["catalog"]["total_runtime_inquiries"], 60);
        assert_eq!(ledger["catalog"]["total_runtime_inquiry_references"], 105);
        assert_eq!(ledger["catalog"]["fight_gate_npcs"], 9);
        assert_eq!(ledger["catalog"]["runtime_fight_gate_npcs"], 8);
        assert_eq!(ledger["catalog"]["apprenticeship_npcs"], 8);
        assert_eq!(ledger["catalog"]["placed_apprenticeship_npcs"], 6);
        assert_eq!(ledger["catalog"]["runtime_lesson_npcs"], 5);
        assert_eq!(ledger["catalog"]["runtime_lessons"], 37);
        assert_eq!(ledger["catalog"]["combat_profiles"], 72);
        assert_eq!(ledger["catalog"]["combat_skill_entries"], 316);
        assert_eq!(ledger["catalog"]["combat_mapping_entries"], 94);
        assert_eq!(ledger["catalog"]["combat_apply_entries"], 21);
        assert_eq!(ledger["catalog"]["combat_chat_npcs"], 25);
        assert_eq!(ledger["catalog"]["combat_chat_entries"], 87);
        assert_eq!(ledger["catalog"]["carried_item_npcs"], 55);
        assert_eq!(ledger["catalog"]["carried_item_entries"], 93);
        assert_eq!(ledger["catalog"]["carried_items"], 5);
        assert_eq!(ledger["catalog"]["worn_items"], 56);
        assert_eq!(ledger["catalog"]["wielded_items"], 32);
        assert_eq!(ledger["catalog"]["static_placed_definitions"], 59);
        assert_eq!(ledger["catalog"]["placed_definitions"], 61);
        assert_eq!(ledger["catalog"]["static_room_mappings"], 78);
        assert_eq!(ledger["catalog"]["static_room_instances"], 113);
        assert_eq!(ledger["catalog"]["runtime_room_references"], 115);
        assert_eq!(ledger["catalog"]["runtime_carried_item_npcs"], 44);
        assert_eq!(ledger["catalog"]["runtime_carried_item_references"], 86);
        assert_eq!(ledger["catalog"]["runtime_drop_entries"], 136);
        assert_eq!(ledger["catalog"]["runtime_combat_npcs"], 61);
        assert_eq!(ledger["catalog"]["runtime_combat_npc_instances"], 115);
        assert_eq!(ledger["catalog"]["runtime_combat_chat_npcs"], 18);
        assert_eq!(ledger["catalog"]["runtime_combat_chat_instances"], 36);
        assert_eq!(ledger["catalog"]["custom_command_npcs"], 11);
        assert_eq!(ledger["catalog"]["placed_custom_command_npcs"], 10);
        assert_eq!(ledger["catalog"]["runtime_custom_command_npcs"], 10);
        assert_eq!(ledger["catalog"]["runtime_object_exchange_npcs"], 10);

        let catalog_apprenticeship: HashSet<_> = catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|npc| {
                npc["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|flag| flag == "apprenticeship")
            })
            .map(|npc| npc["source_path"].as_str().unwrap())
            .collect();
        let apprenticeship_dispositions: HashSet<_> = ledger["apprenticeship_dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["source_path"].as_str().unwrap())
            .collect();
        assert_eq!(apprenticeship_dispositions, catalog_apprenticeship);
        let mut apprenticeship_statuses = std::collections::BTreeMap::new();
        for entry in ledger["apprenticeship_dispositions"].as_array().unwrap() {
            *apprenticeship_statuses
                .entry(entry["status"].as_str().unwrap())
                .or_insert(0usize) += 1;
        }
        assert_eq!(
            apprenticeship_statuses,
            std::collections::BTreeMap::from([("deferred", 1), ("excluded", 2), ("verified", 5),])
        );

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

        let catalog_custom_commands: HashSet<_> = catalog["npcs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|npc| {
                npc["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|flag| flag == "custom_command")
            })
            .map(|npc| npc["source_path"].as_str().unwrap())
            .collect();
        let custom_command_dispositions: HashSet<_> = ledger["custom_command_dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["source_path"].as_str().unwrap())
            .collect();
        assert_eq!(custom_command_dispositions, catalog_custom_commands);
        let mut custom_command_statuses = std::collections::BTreeMap::new();
        for entry in ledger["custom_command_dispositions"].as_array().unwrap() {
            *custom_command_statuses
                .entry(entry["status"].as_str().unwrap())
                .or_insert(0usize) += 1;
        }
        assert_eq!(
            custom_command_statuses,
            std::collections::BTreeMap::from([("adapted", 1), ("excluded", 1), ("verified", 9),])
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
                ("adapted", 2),
                ("deferred", 4),
                ("excluded", 5),
                ("verified", 8),
            ])
        );

        let source_placements = world()
            .locations
            .values()
            .filter(|location| {
                location.source_path.as_deref().is_some_and(|path| {
                    ["city", "snow", "temple", "canyon"]
                        .iter()
                        .any(|area| path.starts_with(&format!("mudlib/d/{area}/")))
                })
            })
            .flat_map(|location| &location.npcs)
            .filter(|npc| !npc.as_str().starts_with("adapted."))
            .count();
        assert_eq!(source_placements, 115);

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
    fn fixed_source_room_items_are_structured_with_exact_counts() {
        let placements: std::collections::BTreeMap<_, _> = world()
            .locations()
            .filter(|location| {
                location.source_path.as_deref().is_some_and(|path| {
                    ["city", "snow", "temple", "canyon"]
                        .iter()
                        .any(|area| path.starts_with(&format!("mudlib/d/{area}/")))
                })
            })
            .flat_map(|location| {
                location.room_items.iter().map(|placement| {
                    (
                        (
                            location.id.as_str().to_string(),
                            placement.item_id.as_str().to_string(),
                        ),
                        placement.count,
                    )
                })
            })
            .collect();
        assert_eq!(placements.len(), 4);
        assert_eq!(placements.values().sum::<u32>(), 5);
        assert_eq!(
            placements,
            std::collections::BTreeMap::from([
                (("snow.secret_storage".into(), "snow.obj.shield".into()), 1,),
                (("snow.temple".into(), "obj.paper_seal".into()), 2,),
                (("snow.temple".into(), "snow.obj.denotation".into()), 1,),
                (
                    (
                        "snow.weapon_storage".into(),
                        "snow.npc.obj.bamboo_sword".into(),
                    ),
                    1,
                ),
            ])
        );
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
        let m4_rooms: Vec<_> = source_rooms
            .iter()
            .filter(|room| {
                room.source_path.as_deref().is_some_and(|path| {
                    ["city", "snow", "temple", "canyon"]
                        .iter()
                        .any(|area| path.starts_with(&format!("mudlib/d/{area}/")))
                })
            })
            .collect();
        let m4_behavior_flags: usize = m4_rooms.iter().map(|room| room.behavior_flags.len()).sum();
        assert_eq!(m4_behavior_flags, 107);
        assert_eq!(
            m4_rooms.iter().map(|room| room.doors.len()).sum::<usize>(),
            36
        );
        let rooms_with_details: Vec<_> = m4_rooms
            .iter()
            .filter(|room| !room.details.is_empty())
            .collect();
        assert_eq!(rooms_with_details.len(), 32);
        assert_eq!(
            rooms_with_details
                .iter()
                .map(|room| room.details.len())
                .sum::<usize>(),
            42
        );
        assert_eq!(
            rooms_with_details
                .iter()
                .flat_map(|room| &room.details)
                .filter(|detail| detail.description.is_some())
                .count(),
            30
        );
        assert_eq!(
            rooms_with_details
                .iter()
                .flat_map(|room| &room.details)
                .filter(|detail| detail.door_direction.is_some())
                .count(),
            12
        );
        assert_eq!(
            m4_rooms
                .iter()
                .filter(|room| !room.behavior_flags.is_empty())
                .count(),
            60
        );
        let mut flags_by_region = std::collections::BTreeMap::new();
        let mut flags_by_kind = std::collections::BTreeMap::new();
        for room in m4_rooms {
            let path = room.source_path.as_deref().unwrap();
            let region = ["city", "snow", "temple", "canyon"]
                .into_iter()
                .find(|region| path.starts_with(&format!("mudlib/d/{region}/")))
                .unwrap();
            *flags_by_region.entry(region).or_insert(0usize) += room.behavior_flags.len();
            for flag in &room.behavior_flags {
                *flags_by_kind.entry(flag.as_str()).or_insert(0usize) += 1;
            }
        }
        assert_eq!(
            flags_by_region,
            std::collections::BTreeMap::from([
                ("canyon", 15),
                ("city", 48),
                ("snow", 24),
                ("temple", 20),
            ])
        );
        assert_eq!(
            flags_by_kind,
            std::collections::BTreeMap::from([
                ("conditional_exit", 15),
                ("custom_command", 12),
                ("door", 35),
                ("dynamic_exit", 4),
                ("environment_damage", 4),
                ("item_interaction", 32),
                ("random_behavior", 3),
                ("timed_behavior", 2),
            ])
        );
    }
}
