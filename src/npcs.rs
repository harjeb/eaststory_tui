use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::items::{self, ItemId, items};

const CATALOG_JSON: &str = include_str!("../migration/catalog/npcs-m4.json");
const SOURCE_COMMIT: &str = "87bba6bd2249beec8424b0d6623486a0dd1f7b30";

pub const OLD_LIU_ID: &str = "adapted.old_liu";
pub const TEA_SELLER_ID: &str = "adapted.tea_seller";
pub const TEMPLE_MASTER_ID: &str = "adapted.temple_master";
pub const FISHER_ID: &str = "adapted.fisher";
pub const FLOWER_GIRL_ID: &str = "adapted.flower_girl";
pub const FARM_WOMAN_ID: &str = "adapted.farm_woman";
pub const MELONER_ID: &str = "adapted.meloner";
pub const TRADER_ID: &str = "adapted.trader";
pub const CANYON_ADVISER_ID: &str = "canyon.npc.adviser";
pub const CANYON_CAPTAIN_ID: &str = "canyon.npc.captain";
pub const CANYON_GENERAL_ID: &str = "canyon.npc.general";
pub const CANYON_SELLER_ID: &str = "canyon.npc.seller";
pub const CITY_SHANGSHU_GUARD_ID: &str = "city.shangshu.npc.guard";
pub const CITY_WAITER_ID: &str = "city.npc.waiter";
pub const SNOW_GUARD_ID: &str = "snow.npc.guard";
pub const SNOW_TEACHER_ID: &str = "snow.npc.teacher";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NpcId(String);

impl NpcId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn name(&self) -> &'static str {
        npcs()
            .definition(self)
            .map_or("未知人物", |npc| npc.name.as_str())
    }
}

impl From<&str> for NpcId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VendorGood {
    pub item_id: ItemId,
    pub source_path: String,
    #[serde(default)]
    pub price: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptedInquiryKind {
    VendorList,
    CanyonHistory,
    HerbalistAdvice,
    SnowGuardReveal,
    TeacherTuition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectExchangeKind {
    CanyonAdviser,
    CanyonCaptain,
    CanyonGeneral,
    CanyonSeller,
    CityWaiter,
    CityShangshuGuard,
    ScavengerDonation,
    TeacherTuition,
}

impl ObjectExchangeKind {
    pub fn money_offer(self) -> Option<u64> {
        match self {
            Self::CanyonAdviser => Some(800),
            Self::CanyonCaptain => Some(3_000),
            Self::CanyonSeller => Some(30_000),
            Self::CityWaiter => Some(1_000),
            Self::CityShangshuGuard => Some(30_000),
            Self::CanyonGeneral | Self::ScavengerDonation | Self::TeacherTuition => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InquiryDefinition {
    pub topic: String,
    pub response: Option<String>,
    pub scripted: bool,
}

impl InquiryDefinition {
    pub fn is_runtime_available(&self) -> bool {
        !self.scripted && self.response.is_some()
    }

    pub fn scripted_runtime_kind(&self, npc: &NpcId) -> Option<ScriptedInquiryKind> {
        match (npc.as_str(), self.topic.as_str()) {
            ("city.npc.caker", "大饼")
            | ("city.npc.dumpling_seller", "包子")
            | ("city.npc.vendor", "金疮药") => Some(ScriptedInquiryKind::VendorList),
            ("canyon.npc.captain", "黄石隘口") => Some(ScriptedInquiryKind::CanyonHistory),
            ("snow.npc.herbalist", "治伤" | "疗伤" | "开药") => {
                Some(ScriptedInquiryKind::HerbalistAdvice)
            }
            (SNOW_GUARD_ID, "刘老三" | "血手刘三") => {
                Some(ScriptedInquiryKind::SnowGuardReveal)
            }
            (SNOW_TEACHER_ID, "学费") => Some(ScriptedInquiryKind::TeacherTuition),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcDefinition {
    pub id: NpcId,
    pub source_path: String,
    pub status: String,
    pub area: String,
    pub name: String,
    pub description: Option<String>,
    pub combat_exp: Option<i32>,
    pub placement_count: usize,
    pub vendor_goods: Vec<VendorGood>,
    pub inquiries: Vec<InquiryDefinition>,
    pub behavior_flags: Vec<String>,
}

impl NpcDefinition {
    pub fn price_for(&self, item_id: &ItemId) -> Option<u64> {
        let good = self
            .vendor_goods
            .iter()
            .find(|good| &good.item_id == item_id)?;
        good.price.or_else(|| {
            items()
                .definition(item_id)
                .and_then(|item| item.value)
                .map(|value| value.max(0) as u64)
        })
    }

    pub fn object_exchange_kind(&self) -> Option<ObjectExchangeKind> {
        match self.id.as_str() {
            CANYON_ADVISER_ID => Some(ObjectExchangeKind::CanyonAdviser),
            CANYON_CAPTAIN_ID => Some(ObjectExchangeKind::CanyonCaptain),
            CANYON_GENERAL_ID => Some(ObjectExchangeKind::CanyonGeneral),
            CANYON_SELLER_ID => Some(ObjectExchangeKind::CanyonSeller),
            CITY_WAITER_ID => Some(ObjectExchangeKind::CityWaiter),
            CITY_SHANGSHU_GUARD_ID => Some(ObjectExchangeKind::CityShangshuGuard),
            "snow.npc.scavenger" => Some(ObjectExchangeKind::ScavengerDonation),
            SNOW_TEACHER_ID => Some(ObjectExchangeKind::TeacherTuition),
            _ => None,
        }
    }

    pub fn accepts_runtime_gifts(&self) -> bool {
        self.source_path == "adapted" || self.object_exchange_kind().is_some()
    }
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    source_commit: String,
    npcs: Vec<NpcDefinition>,
}

pub struct NpcRepository {
    definitions: HashMap<NpcId, NpcDefinition>,
    source_ids: HashMap<String, NpcId>,
    source_count: usize,
    source_commit: String,
}

impl NpcRepository {
    fn load() -> Self {
        let catalog: Catalog = serde_json::from_str(CATALOG_JSON).expect("invalid M4 NPC catalog");
        assert_eq!(catalog.schema_version, 2, "unsupported NPC catalog schema");
        assert_eq!(catalog.source_commit, SOURCE_COMMIT, "NPC source drift");
        assert_eq!(catalog.npcs.len(), 73, "unexpected M4 NPC count");

        let source_count = catalog.npcs.len();
        let mut definitions = HashMap::new();
        let mut source_ids = HashMap::new();
        for npc in catalog.npcs {
            for good in &npc.vendor_goods {
                assert!(
                    items().contains(&good.item_id),
                    "NPC {} references missing item {}",
                    npc.id.as_str(),
                    good.item_id.as_str()
                );
            }
            assert!(
                source_ids
                    .insert(npc.source_path.clone(), npc.id.clone())
                    .is_none(),
                "duplicate NPC source path {}",
                npc.source_path
            );
            assert!(
                definitions.insert(npc.id.clone(), npc).is_none(),
                "duplicate NPC ID"
            );
        }

        for npc in adapted_npcs() {
            assert!(definitions.insert(npc.id.clone(), npc).is_none());
        }

        Self {
            definitions,
            source_ids,
            source_count,
            source_commit: catalog.source_commit,
        }
    }

    pub fn definition(&self, id: &NpcId) -> Option<&NpcDefinition> {
        self.definitions.get(id)
    }

    pub fn id_for_source(&self, source_path: &str) -> Option<&NpcId> {
        self.source_ids.get(source_path)
    }

    pub fn source_npc_count(&self) -> usize {
        self.source_count
    }

    pub fn source_commit(&self) -> &str {
        &self.source_commit
    }
}

fn adapted_npcs() -> Vec<NpcDefinition> {
    vec![
        adapted(OLD_LIU_ID, "刘老农", "刘家小房的主人。", vec![]),
        adapted(TEA_SELLER_ID, "茶摊老板", "在路边摆摊的茶商。", vec![]),
        adapted(
            TEMPLE_MASTER_ID,
            "玄智和尚",
            "山烟寺中指点后辈的和尚。",
            vec![],
        ),
        adapted(FISHER_ID, "渔夫", "熟悉湖面水情的渔夫。", vec![]),
        adapted(FLOWER_GIRL_ID, "采花妞", "在花园里忙碌的姑娘。", vec![]),
        adapted(FARM_WOMAN_ID, "农妇", "正在为失踪孩子担忧的农妇。", vec![]),
        adapted(
            MELONER_ID,
            "瓜农",
            "看守瓜田的农夫。",
            vec![(items::WATER_MELON_ID, 60)],
        ),
        adapted(
            TRADER_ID,
            "关外商人",
            "从关外来往于官道的商人。",
            vec![
                (items::DRY_RATIONS_ID, 15),
                ("obj.weapon.dagger", 50),
                ("obj.weapon.longsword", 400),
            ],
        ),
    ]
}

fn adapted(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    goods: Vec<(&'static str, u64)>,
) -> NpcDefinition {
    NpcDefinition {
        id: NpcId::from(id),
        source_path: "adapted".into(),
        status: "adapted".into(),
        area: "adapted".into(),
        name: name.into(),
        description: Some(description.into()),
        combat_exp: None,
        placement_count: 0,
        vendor_goods: goods
            .into_iter()
            .map(|(item_id, price)| VendorGood {
                item_id: ItemId::from(item_id),
                source_path: "adapted".into(),
                price: Some(price),
            })
            .collect(),
        inquiries: Vec::new(),
        behavior_flags: Vec::new(),
    }
}

static NPCS: LazyLock<NpcRepository> = LazyLock::new(NpcRepository::load);

pub fn npcs() -> &'static NpcRepository {
    &NPCS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_m4_catalog_and_vendor_references_are_complete() {
        assert_eq!(npcs().source_npc_count(), 73);
        assert_eq!(npcs().source_commit(), SOURCE_COMMIT);
        let catalog: serde_json::Value = serde_json::from_str(CATALOG_JSON).unwrap();
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
    }

    #[test]
    fn source_paths_and_adapted_npcs_have_stable_ids() {
        assert_eq!(
            npcs()
                .id_for_source("mudlib/d/snow/npc/herbalist.c")
                .unwrap()
                .as_str(),
            "snow.npc.herbalist"
        );
        assert_eq!(NpcId::from(OLD_LIU_ID).name(), "刘老农");
    }

    #[test]
    fn static_inquiries_are_distinguished_from_scripted_callbacks() {
        let boss = npcs().definition(&NpcId::from("city.npc.boss")).unwrap();
        let emperor = boss
            .inquiries
            .iter()
            .find(|inquiry| inquiry.topic == "皇上")
            .unwrap();
        assert!(emperor.is_runtime_available());
        assert_eq!(
            emperor.response.as_deref(),
            Some("小声点。偷偷地告述你，我真见过皇上的。")
        );

        let guard = npcs().definition(&NpcId::from("snow.npc.guard")).unwrap();
        assert!(
            guard
                .inquiries
                .iter()
                .all(|inquiry| !inquiry.is_runtime_available())
        );
    }

    #[test]
    fn only_audited_scripted_inquiries_have_runtime_handlers() {
        let ids = [
            "canyon.npc.captain",
            "city.npc.caker",
            "city.npc.dumpling_seller",
            "city.npc.vendor",
            "snow.npc.guard",
            "snow.npc.herbalist",
            "snow.npc.mercenary",
            "snow.npc.post_officer",
            "snow.npc.proposer",
            "snow.npc.teacher",
            "snow.npc.teacher1",
        ];
        let handled: Vec<_> = ids
            .into_iter()
            .flat_map(|id| {
                let npc_id = NpcId::from(id);
                npcs()
                    .definition(&npc_id)
                    .unwrap()
                    .inquiries
                    .iter()
                    .filter_map(move |inquiry| {
                        inquiry
                            .scripted_runtime_kind(&npc_id)
                            .map(|kind| (id, inquiry.topic.as_str(), kind))
                    })
            })
            .collect();

        assert_eq!(handled.len(), 10);
        assert_eq!(
            handled
                .iter()
                .filter(|(_, _, kind)| *kind == ScriptedInquiryKind::VendorList)
                .count(),
            3
        );
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == "canyon.npc.captain"
                && *topic == "黄石隘口"
                && *kind == ScriptedInquiryKind::CanyonHistory
        }));
        assert_eq!(
            handled
                .iter()
                .filter(|(_, _, kind)| *kind == ScriptedInquiryKind::HerbalistAdvice)
                .count(),
            3
        );
        assert_eq!(
            handled
                .iter()
                .filter(|(_, _, kind)| *kind == ScriptedInquiryKind::SnowGuardReveal)
                .count(),
            2
        );
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == SNOW_TEACHER_ID
                && *topic == "学费"
                && *kind == ScriptedInquiryKind::TeacherTuition
        }));
    }

    #[test]
    fn only_audited_source_npcs_accept_runtime_gifts() {
        let accepting: Vec<_> = npcs()
            .definitions
            .values()
            .filter(|npc| npc.source_path != "adapted" && npc.accepts_runtime_gifts())
            .map(|npc| npc.id.as_str())
            .collect();
        assert_eq!(
            accepting
                .into_iter()
                .collect::<std::collections::HashSet<_>>(),
            std::collections::HashSet::from([
                CANYON_ADVISER_ID,
                CANYON_CAPTAIN_ID,
                CANYON_GENERAL_ID,
                CANYON_SELLER_ID,
                CITY_SHANGSHU_GUARD_ID,
                CITY_WAITER_ID,
                "snow.npc.scavenger",
                SNOW_TEACHER_ID,
            ])
        );
    }
}
