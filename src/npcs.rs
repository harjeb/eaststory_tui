use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::items::{self, ItemId, items};

const M4_CATALOG_JSON: &str = include_str!("../migration/catalog/npcs-m4.json");
const M5_CATALOG_JSON: &str = include_str!("../migration/catalog/npcs-m5.json");
const M6_CATALOG_JSON: &str = include_str!("../migration/catalog/npcs-m6.json");
const M7_CATALOG_JSON: &str = include_str!("../migration/catalog/npcs-m7.json");
const NPC_AMBIENT_CATALOG_JSON: &str = include_str!("../migration/catalog/npc-ambient.json");
const SOURCE_COMMIT: &str = "87bba6bd2249beec8424b0d6623486a0dd1f7b30";

pub const OLD_LIU_ID: &str = "adapted.old_liu";
pub const XIAO_JUAN_ID: &str = "adapted.xiao_juan";
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
pub const CHOYIN_CAKE_VENDOR_ID: &str = "choyin.npc.cake_vendor";
pub const CHOYIN_CUCURBIT_SELLER_ID: &str = "choyin.npc.cucurbit_seller";
pub const CHOYIN_DUMPLING_SELLER_ID: &str = "choyin.npc.dumpling_seller";
pub const CHOYIN_GIRL_ID: &str = "choyin.npc.girl";
pub const CHOYIN_HOTEL_GUARD_ID: &str = "choyin.npc.guard";
pub const CHOYIN_LION_ID: &str = "choyin.npc.lion";
pub const CHOYIN_MAGISTRATE_ID: &str = "choyin.npc.judgeman";
pub const CHOYIN_OLD_MAN_ID: &str = "choyin.npc.oldman";
pub const CHOYIN_POLICE_ID: &str = "choyin.npc.yamen_po";
pub const CHOYIN_SERGEANT_ID: &str = "choyin.npc.sergeant";
pub const CHOYIN_YOUNG_MAN_ID: &str = "choyin.npc.youngman";
pub const CHUENYU_OLD_LIU_ID: &str = "chuenyu.npc.oldliu";
pub const CHUENYU_XIAO_JUAN_ID: &str = "chuenyu.npc.xiaojuan";
pub const CHUENYU_XIAO_JUAN_PLACED_ID: &str = "chuenyu.xiaojuan2";
pub const GREEN_MASTER_ID: &str = "green.npc.master";
pub const GREEN_OLD_MAN_ID: &str = "green.npc.oldman2";
pub const GREEN_SHEN_ID: &str = "green.npc.shen";
pub const CITY_BANKER_ID: &str = "city.npc.microsof";
pub const CITY_GUARD_ID: &str = "city.npc.guard";
pub const CITY_MASTER_CHEN_ID: &str = "city.npc.chen";
pub const CITY_SHANGSHU_GUARD_ID: &str = "city.shangshu.npc.guard";
pub const CITY_SHANGSHU_PATROL_ID: &str = "city.shangshu.npc.huyuan";
pub const CITY_SHANGSHU_PATROL_ELITE_ID: &str = "city.shangshu.npc.huyuan1";
pub const CITY_WAITER_ID: &str = "city.npc.waiter";
pub const SNOW_BANKER_ID: &str = "snow.npc.annihir";
pub const SNOW_FIST_TRAINER_ID: &str = "snow.npc.fist_trainer";
pub const SNOW_GIRL_ID: &str = "snow.npc.girl";
pub const SNOW_GUARD_ID: &str = "snow.npc.guard";
pub const SNOW_DRUNK_ID: &str = "snow.npc.drunk";
pub const SNOW_KEEPER_ID: &str = "snow.npc.keeper";
pub const SNOW_MERCENARY_ID: &str = "snow.npc.mercenary";
pub const SNOW_SCAVENGER_ID: &str = "snow.npc.scavenger";
pub const SNOW_TEACHER_ID: &str = "snow.npc.teacher";
pub const SNOW_TEACHER_DUPLICATE_ID: &str = "snow.npc.teacher1";
pub const TEMPLE_LIBRARY_GUARD_ID: &str = "temple.npc.guard_taoist1";
pub const TEMPLE_LIBRARY_GUARD_PEER_ID: &str = "temple.npc.taoist_guard1";
pub const TEMPLE_OLD_TAOIST_ID: &str = "temple.npc.old_taoist";
pub const TEMPLE_PROTECTOR_ID: &str = "temple.npc.tfighter";
pub const TEMPLE_TRAINER_ID: &str = "temple.npc.trainer";
pub const WATERFOG_ELITE_GUARD_ID: &str = "waterfog.npc.elite_guard";
pub const LATEMOON_FUNLIN_ID: &str = "latemoon.npc.funlin";
pub const LATEMOON_GIRL_ID: &str = "latemoon.npc.girl";
pub const LATEMOON_SHAOWEI_ID: &str = "latemoon.npc.shaowei";
pub const LATEMOON_YUMAY_ID: &str = "latemoon.npc.yumay";
pub const LATEMOON_OLD_ID: &str = "latemoon.room.npc.old";
pub const LATEMOON_SHINFUN_ID: &str = "latemoon.upstar.npc.shinfun";
pub const CLOUD_B_HEADER_ID: &str = "u.cloud.npc.b_header";
pub const CLOUD_BOATER_ID: &str = "u.cloud.npc.boater";
pub const CLOUD_GANGSTER_ID: &str = "u.cloud.npc.gangster";
pub const CLOUD_GIRL_ID: &str = "u.cloud.npc.girl";
pub const CLOUD_GOD_ID: &str = "u.cloud.npc.god";
pub const CLOUD_JUDGE_ID: &str = "u.cloud.npc.judge";
pub const CLOUD_MONK_ID: &str = "u.cloud.npc.monk";
pub const CLOUD_THIEF_ID: &str = "u.cloud.npc.thief";

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

#[derive(Debug, Clone, Deserialize)]
pub struct NpcCarriedItem {
    pub item_id: ItemId,
    pub source_path: String,
    pub state: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptedInquiryKind {
    VendorList,
    CanyonHistory,
    ChoyinPoliceBribery,
    ChoyinSilkBag,
    ChoyinYoungManTrouble,
    GreenOldManJade,
    GreenShenJade,
    GreenShenSlumberDrug,
    LatemoonGirlDance,
    LatemoonGirlDragonDance,
    LatemoonShaoweiDragonfly,
    LatemoonShinfunDanceBook,
    LatemoonYumayFunlin,
    LatemoonYumayLearnDance,
    CloudBoaterCross,
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
    ChoyinSergeant,
    ChoyinYoungMan,
    CityGuardToken,
    CityWaiter,
    CityShangshuGuard,
    GreenShen,
    LatemoonFunlin,
    LatemoonOld,
    LatemoonShaowei,
    CloudBHeader,
    CloudBoater,
    CloudGangster,
    CloudGirl,
    CloudJudge,
    CloudMonk,
    CityChenLetter,
    ScavengerDonation,
    SnowDrunk,
    SnowTempleDonation,
    TeacherTuition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcFightPolicy {
    Allow,
    ForceLethal,
    RequireFaction(&'static str),
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcApprenticeshipPolicy {
    DeferredLetter,
    RecognizeFaction(&'static str),
    SameFaction(&'static str),
    PaidStudent,
    PlotGated,
    ExcludedUnplaced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcLesson {
    pub skill: &'static str,
    pub max_level: u32,
}

const fn lesson(skill: &'static str, max_level: u32) -> NpcLesson {
    NpcLesson { skill, max_level }
}

const CITY_CHEN_LESSONS: &[NpcLesson] = &[
    lesson("blade", 150),
    lesson("celestial", 80),
    lesson("celestrike", 80),
    lesson("dodge", 80),
    lesson("force", 100),
    lesson("parry", 100),
    lesson("pyrobat-steps", 70),
    lesson("spring-blade", 150),
    lesson("unarmed", 100),
];
const FIST_TRAINER_LESSONS: &[NpcLesson] = &[
    lesson("unarmed", 30),
    lesson("liuh-ken", 20),
    lesson("dodge", 30),
];
const SNOW_GIRL_LESSONS: &[NpcLesson] = &[
    lesson("unarmed", 20),
    lesson("parry", 40),
    lesson("dodge", 50),
    lesson("sword", 30),
    lesson("force", 30),
    lesson("literate", 70),
    lesson("fonxanforce", 40),
    lesson("fonxansword", 40),
    lesson("liuh-ken", 40),
    lesson("chaos-steps", 70),
];
const SNOW_TEACHER_LESSONS: &[NpcLesson] = &[lesson("literate", 60)];
const TEMPLE_PROTECTOR_LESSONS: &[NpcLesson] = &[
    lesson("literate", 50),
    lesson("magic", 30),
    lesson("force", 30),
    lesson("spells", 60),
    lesson("unarmed", 50),
    lesson("sword", 100),
    lesson("parry", 100),
    lesson("dodge", 100),
    lesson("gouyee", 60),
    lesson("taoism", 70),
    lesson("necromancy", 60),
];
const TEMPLE_TRAINER_LESSONS: &[NpcLesson] = &[
    lesson("literate", 10),
    lesson("magic", 60),
    lesson("force", 50),
    lesson("spells", 60),
    lesson("scratching", 20),
    lesson("unarmed", 100),
    lesson("sword", 100),
    lesson("parry", 80),
    lesson("dodge", 80),
    lesson("gouyee", 60),
    lesson("taoism", 60),
    lesson("necromancy", 60),
];

impl ObjectExchangeKind {
    pub fn money_offer(self) -> Option<u64> {
        match self {
            Self::CanyonAdviser => Some(800),
            Self::CanyonCaptain => Some(3_000),
            Self::CanyonSeller => Some(30_000),
            Self::CityWaiter => Some(1_000),
            Self::CityShangshuGuard => Some(30_000),
            Self::SnowTempleDonation => Some(1_000),
            Self::CanyonGeneral
            | Self::ChoyinSergeant
            | Self::ChoyinYoungMan
            | Self::CityChenLetter
            | Self::CityGuardToken
            | Self::CloudBHeader
            | Self::CloudBoater
            | Self::CloudGangster
            | Self::CloudGirl
            | Self::CloudJudge
            | Self::CloudMonk
            | Self::GreenShen
            | Self::LatemoonFunlin
            | Self::LatemoonOld
            | Self::LatemoonShaowei
            | Self::ScavengerDonation
            | Self::SnowDrunk
            | Self::TeacherTuition => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcCombatChatEntry {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcAmbientChatEntry {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcCombatChat {
    pub chance: Option<i32>,
    pub chance_expression: Option<String>,
    pub entries: Vec<NpcCombatChatEntry>,
}

impl NpcCombatChat {
    pub fn runtime_chance(&self) -> u32 {
        if let Some(chance) = self.chance {
            return chance.clamp(0, 100) as u32;
        }
        self.chance_expression
            .as_deref()
            .and_then(|expression| expression.strip_prefix("random("))
            .and_then(|value| value.strip_suffix(')'))
            .and_then(|value| value.parse::<u32>().ok())
            .map_or(0, |upper| (upper / 2).min(100))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcAmbientChat {
    pub chance: Option<i32>,
    pub chance_expression: Option<String>,
    pub entries: Vec<NpcAmbientChatEntry>,
}

impl NpcAmbientChat {
    pub fn runtime_chance(&self) -> u32 {
        if let Some(chance) = self.chance {
            return chance.clamp(0, 100) as u32;
        }
        self.chance_expression
            .as_deref()
            .and_then(|expression| expression.strip_prefix("random("))
            .and_then(|value| value.strip_suffix(')'))
            .and_then(|value| value.parse::<u32>().ok())
            .map_or(0, |upper| (upper / 2).min(100))
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
            | ("city.npc.vendor", "金疮药")
            | (CHOYIN_CAKE_VENDOR_ID, "大饼")
            | (CHOYIN_CUCURBIT_SELLER_ID, "糖葫芦")
            | (CHOYIN_DUMPLING_SELLER_ID, "包子") => Some(ScriptedInquiryKind::VendorList),
            ("canyon.npc.captain", "黄石隘口") => Some(ScriptedInquiryKind::CanyonHistory),
            (CHOYIN_GIRL_ID, "游晋") => Some(ScriptedInquiryKind::ChoyinSilkBag),
            (CHOYIN_POLICE_ID, "bribery") => Some(ScriptedInquiryKind::ChoyinPoliceBribery),
            (CHOYIN_YOUNG_MAN_ID, "trouble") => Some(ScriptedInquiryKind::ChoyinYoungManTrouble),
            (GREEN_OLD_MAN_ID, "玉佩") => Some(ScriptedInquiryKind::GreenOldManJade),
            (GREEN_SHEN_ID, "玉佩") => Some(ScriptedInquiryKind::GreenShenJade),
            (GREEN_SHEN_ID, "蒙汗药") => Some(ScriptedInquiryKind::GreenShenSlumberDrug),
            (LATEMOON_GIRL_ID, "舞") => Some(ScriptedInquiryKind::LatemoonGirlDance),
            (LATEMOON_GIRL_ID, "寒谷龙舞") => {
                Some(ScriptedInquiryKind::LatemoonGirlDragonDance)
            }
            (LATEMOON_SHAOWEI_ID, "竹蜻蜓") => {
                Some(ScriptedInquiryKind::LatemoonShaoweiDragonfly)
            }
            (LATEMOON_SHINFUN_ID, "舞曲谱") => {
                Some(ScriptedInquiryKind::LatemoonShinfunDanceBook)
            }
            (LATEMOON_YUMAY_ID, "芳绫") => Some(ScriptedInquiryKind::LatemoonYumayFunlin),
            (LATEMOON_YUMAY_ID, "学舞") => Some(ScriptedInquiryKind::LatemoonYumayLearnDance),
            (CLOUD_BOATER_ID, "摆渡" | "过江") => Some(ScriptedInquiryKind::CloudBoaterCross),
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
    pub attributes: HashMap<String, i32>,
    pub resources: HashMap<String, i32>,
    pub skills: HashMap<String, i32>,
    pub mappings: HashMap<String, String>,
    pub combat_apply: HashMap<String, i32>,
    pub combat_chat: Option<NpcCombatChat>,
    #[serde(default)]
    pub ambient_chat: Option<NpcAmbientChat>,
    pub carried_items: Vec<NpcCarriedItem>,
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
            CHOYIN_SERGEANT_ID => Some(ObjectExchangeKind::ChoyinSergeant),
            CHOYIN_YOUNG_MAN_ID => Some(ObjectExchangeKind::ChoyinYoungMan),
            CITY_GUARD_ID => Some(ObjectExchangeKind::CityGuardToken),
            CITY_WAITER_ID => Some(ObjectExchangeKind::CityWaiter),
            CITY_SHANGSHU_GUARD_ID => Some(ObjectExchangeKind::CityShangshuGuard),
            GREEN_SHEN_ID => Some(ObjectExchangeKind::GreenShen),
            LATEMOON_FUNLIN_ID => Some(ObjectExchangeKind::LatemoonFunlin),
            LATEMOON_OLD_ID => Some(ObjectExchangeKind::LatemoonOld),
            LATEMOON_SHAOWEI_ID => Some(ObjectExchangeKind::LatemoonShaowei),
            CLOUD_B_HEADER_ID => Some(ObjectExchangeKind::CloudBHeader),
            CLOUD_BOATER_ID => Some(ObjectExchangeKind::CloudBoater),
            CLOUD_GANGSTER_ID => Some(ObjectExchangeKind::CloudGangster),
            CLOUD_GIRL_ID => Some(ObjectExchangeKind::CloudGirl),
            CLOUD_JUDGE_ID => Some(ObjectExchangeKind::CloudJudge),
            CLOUD_MONK_ID => Some(ObjectExchangeKind::CloudMonk),
            CITY_MASTER_CHEN_ID => Some(ObjectExchangeKind::CityChenLetter),
            "snow.npc.scavenger" => Some(ObjectExchangeKind::ScavengerDonation),
            SNOW_DRUNK_ID => Some(ObjectExchangeKind::SnowDrunk),
            SNOW_KEEPER_ID => Some(ObjectExchangeKind::SnowTempleDonation),
            SNOW_TEACHER_ID => Some(ObjectExchangeKind::TeacherTuition),
            _ => None,
        }
    }

    pub fn accepts_runtime_gifts(&self) -> bool {
        self.source_path == "adapted" || self.object_exchange_kind().is_some()
    }

    pub fn is_source_combatant(&self) -> bool {
        self.source_path != "adapted"
    }

    pub fn combat_rating(&self) -> i32 {
        let experience = self.combat_exp.unwrap_or(0).max(0) as f64;
        let experience_rating = (experience / 100.0).sqrt().round() as i32;
        let skill_rating = self.skills.values().copied().max().unwrap_or(0) / 2;
        experience_rating.max(skill_rating).clamp(1, 100)
    }

    pub fn combat_max_health(&self) -> i32 {
        let rating = self.combat_rating();
        let constitution = self.attributes.get("con").copied().unwrap_or(20);
        let source_health = ["max_gin", "eff_gin", "gin"]
            .into_iter()
            .filter_map(|key| self.resources.get(key).copied())
            .max()
            .unwrap_or(0);
        (40 + rating * 3 + (constitution - 20).max(0) * 2).max(source_health)
    }

    pub fn combat_attack(&self) -> i32 {
        let strength = self.attributes.get("str").copied().unwrap_or(20);
        let apply = ["attack", "damage"]
            .into_iter()
            .filter_map(|key| self.combat_apply.get(key))
            .sum::<i32>();
        (8 + self.combat_rating() / 2 + (strength - 20).max(0) / 3 + apply / 20).max(1)
    }

    pub fn combat_defense(&self) -> i32 {
        let agility = self.attributes.get("cps").copied().unwrap_or(20)
            + self.attributes.get("cor").copied().unwrap_or(20);
        let apply = ["defense", "dodge", "parry", "armor"]
            .into_iter()
            .filter_map(|key| self.combat_apply.get(key))
            .sum::<i32>();
        (4 + self.combat_rating() / 3 + (agility - 40).max(0) / 6 + apply / 40).max(1)
    }

    pub fn attacks_spirit(&self) -> bool {
        self.skills.get("spells").copied().unwrap_or(0) > 0
            || self.skills.get("magic").copied().unwrap_or(0) > 0
    }

    pub fn fight_policy(&self) -> Option<NpcFightPolicy> {
        Some(match self.id.as_str() {
            CHOYIN_OLD_MAN_ID => NpcFightPolicy::Allow,
            CITY_BANKER_ID | SNOW_BANKER_ID | CLOUD_GANGSTER_ID | CLOUD_THIEF_ID => {
                NpcFightPolicy::ForceLethal
            }
            SNOW_FIST_TRAINER_ID => NpcFightPolicy::RequireFaction("封山剑派"),
            TEMPLE_PROTECTOR_ID | TEMPLE_TRAINER_ID => NpcFightPolicy::RequireFaction("茅山派"),
            CHOYIN_HOTEL_GUARD_ID
            | CHOYIN_MAGISTRATE_ID
            | CHUENYU_OLD_LIU_ID
            | CHUENYU_XIAO_JUAN_ID
            | CHUENYU_XIAO_JUAN_PLACED_ID
            | "sanyen.npc.bonze"
            | "sanyen.npc.cook_bonze"
            | "sanyen.npc.cripple"
            | "sanyen.npc.greeting"
            | "sanyen.npc.little_bonze"
            | "sanyen.npc.work_bonze"
            | "snow.npc.beggar"
            | SNOW_GIRL_ID
            | SNOW_SCAVENGER_ID
            | "u.cloud.npc.beggar"
            | CLOUD_GIRL_ID
            | CLOUD_GOD_ID
            | CLOUD_MONK_ID
            | "u.cloud.npc.monk_waiter"
            | TEMPLE_OLD_TAOIST_ID
            | WATERFOG_ELITE_GUARD_ID => NpcFightPolicy::Reject,
            _ => return None,
        })
    }

    pub fn apprenticeship_policy(&self) -> Option<NpcApprenticeshipPolicy> {
        Some(match self.id.as_str() {
            CITY_MASTER_CHEN_ID => NpcApprenticeshipPolicy::DeferredLetter,
            CLOUD_B_HEADER_ID => NpcApprenticeshipPolicy::PlotGated,
            SNOW_FIST_TRAINER_ID => NpcApprenticeshipPolicy::RecognizeFaction("封山剑派"),
            SNOW_GIRL_ID => NpcApprenticeshipPolicy::SameFaction("封山剑派北宗"),
            GREEN_MASTER_ID | SNOW_MERCENARY_ID | SNOW_TEACHER_DUPLICATE_ID => {
                NpcApprenticeshipPolicy::ExcludedUnplaced
            }
            SNOW_TEACHER_ID => NpcApprenticeshipPolicy::PaidStudent,
            TEMPLE_PROTECTOR_ID | TEMPLE_TRAINER_ID => {
                NpcApprenticeshipPolicy::SameFaction("茅山派")
            }
            _ => return None,
        })
    }

    pub fn lessons(&self) -> &'static [NpcLesson] {
        match self.id.as_str() {
            CITY_MASTER_CHEN_ID => CITY_CHEN_LESSONS,
            SNOW_FIST_TRAINER_ID => FIST_TRAINER_LESSONS,
            SNOW_GIRL_ID => SNOW_GIRL_LESSONS,
            SNOW_TEACHER_ID => SNOW_TEACHER_LESSONS,
            TEMPLE_PROTECTOR_ID => TEMPLE_PROTECTOR_LESSONS,
            TEMPLE_TRAINER_ID => TEMPLE_TRAINER_LESSONS,
            _ => &[],
        }
    }

    pub fn teaching_intelligence(&self) -> Option<i32> {
        Some(match self.id.as_str() {
            CITY_MASTER_CHEN_ID => 30,
            SNOW_FIST_TRAINER_ID => 14,
            SNOW_GIRL_ID => 27,
            SNOW_TEACHER_ID => 26,
            TEMPLE_PROTECTOR_ID | TEMPLE_TRAINER_ID => 30,
            _ => return None,
        })
    }
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    source_commit: String,
    npcs: Vec<NpcDefinition>,
}

#[derive(Deserialize)]
struct AmbientChatCatalog {
    schema_version: u32,
    source_commit: String,
    npcs: Vec<AmbientChatCatalogEntry>,
}

#[derive(Deserialize)]
struct AmbientChatCatalogEntry {
    id: NpcId,
    source_path: String,
    ambient_chat: NpcAmbientChat,
}

pub struct NpcRepository {
    definitions: HashMap<NpcId, NpcDefinition>,
    source_ids: HashMap<String, NpcId>,
    ambient_chats: HashMap<NpcId, NpcAmbientChat>,
    source_count: usize,
    source_commit: String,
}

impl NpcRepository {
    fn load() -> Self {
        let catalogs = [
            (M4_CATALOG_JSON, "M4", 73usize),
            (M5_CATALOG_JSON, "M5", 49usize),
            (M6_CATALOG_JSON, "M6", 57usize),
            (M7_CATALOG_JSON, "M7", 85usize),
        ];

        let mut definitions = HashMap::new();
        let mut source_ids = HashMap::new();
        let mut source_count = 0;
        for (json, milestone, expected_count) in catalogs {
            let catalog: Catalog = serde_json::from_str(json)
                .unwrap_or_else(|error| panic!("invalid {milestone} NPC catalog: {error}"));
            assert_eq!(catalog.schema_version, 4, "unsupported NPC catalog schema");
            assert_eq!(catalog.source_commit, SOURCE_COMMIT, "NPC source drift");
            assert_eq!(
                catalog.npcs.len(),
                expected_count,
                "unexpected {milestone} NPC count"
            );
            source_count += catalog.npcs.len();

            for npc in catalog.npcs {
                for good in &npc.vendor_goods {
                    assert!(
                        items().contains(&good.item_id),
                        "NPC {} references missing item {}",
                        npc.id.as_str(),
                        good.item_id.as_str()
                    );
                }
                for carried in &npc.carried_items {
                    assert!(
                        items().contains(&carried.item_id),
                        "NPC {} carries missing item {}",
                        npc.id.as_str(),
                        carried.item_id.as_str()
                    );
                    assert!(
                        matches!(carried.state.as_str(), "carried" | "worn" | "wielded"),
                        "NPC {} has invalid carried-item state {}",
                        npc.id.as_str(),
                        carried.state
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
        }

        let ambient_catalog: AmbientChatCatalog = serde_json::from_str(NPC_AMBIENT_CATALOG_JSON)
            .unwrap_or_else(|error| panic!("invalid M8 NPC ambient catalog: {error}"));
        assert_eq!(
            ambient_catalog.schema_version, 1,
            "unsupported M8 NPC ambient catalog schema"
        );
        assert_eq!(
            ambient_catalog.source_commit, SOURCE_COMMIT,
            "M8 NPC ambient source drift"
        );
        let mut ambient_chats = HashMap::new();
        for entry in ambient_catalog.npcs {
            let definition = definitions
                .get(&entry.id)
                .unwrap_or_else(|| panic!("ambient catalog references unknown NPC {}", entry.id.as_str()));
            assert_eq!(
                definition.source_path, entry.source_path,
                "ambient catalog source mismatch for {}",
                entry.id.as_str()
            );
            assert!(
                definition
                    .behavior_flags
                    .iter()
                    .any(|flag| flag == "ambient_chat"),
                "ambient catalog NPC {} lacks ambient_chat flag",
                entry.id.as_str()
            );
            assert!(
                !entry.ambient_chat.entries.is_empty(),
                "ambient catalog NPC {} has no entries",
                entry.id.as_str()
            );
            assert!(
                ambient_chats.insert(entry.id, entry.ambient_chat).is_none(),
                "duplicate ambient catalog NPC"
            );
        }

        for npc in adapted_npcs() {
            assert!(definitions.insert(npc.id.clone(), npc).is_none());
        }

        Self {
            definitions,
            source_ids,
            ambient_chats,
            source_count,
            source_commit: SOURCE_COMMIT.to_string(),
        }
    }

    pub fn definition(&self, id: &NpcId) -> Option<&NpcDefinition> {
        self.definitions.get(id)
    }

    pub fn id_for_source(&self, source_path: &str) -> Option<&NpcId> {
        self.source_ids.get(source_path)
    }

    pub fn ambient_chat(&self, id: &NpcId) -> Option<&NpcAmbientChat> {
        self.ambient_chats.get(id).or_else(|| {
            self.definitions
                .get(id)
                .and_then(|definition| definition.ambient_chat.as_ref())
        })
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
        adapted(
            XIAO_JUAN_ID,
            "小娟",
            "刘老农的独生女，刚从囚禁她的人手中脱身。",
            vec![],
        ),
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
        attributes: HashMap::new(),
        resources: HashMap::new(),
        skills: HashMap::new(),
        mappings: HashMap::new(),
        combat_apply: HashMap::new(),
        combat_chat: None,
        ambient_chat: None,
        carried_items: Vec::new(),
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

    fn is_m4_source(npc: &NpcDefinition) -> bool {
        matches!(npc.area.as_str(), "city" | "snow" | "temple" | "canyon")
    }

    #[test]
    fn fixed_m4_catalog_and_vendor_references_are_complete() {
        assert_eq!(npcs().source_npc_count(), 264);
        assert_eq!(npcs().source_commit(), SOURCE_COMMIT);
        let catalog: serde_json::Value = serde_json::from_str(M4_CATALOG_JSON).unwrap();
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
    }

    #[test]
    fn m8_ambient_chat_catalog_is_source_matched_and_runtime_ready() {
        assert_eq!(npcs().ambient_chats.len(), 70);
        let cake_vendor = npcs()
            .ambient_chat(&NpcId::from(CHOYIN_CAKE_VENDOR_ID))
            .unwrap();
        assert_eq!(cake_vendor.runtime_chance(), 13);
        assert_eq!(cake_vendor.entries.len(), 3);
        assert!(cake_vendor.entries.iter().all(|entry| entry.kind == "text"));

        for (npc_id, expected_chance, expected_entries) in [
            ("choyin.npc.cucurbit_seller", 15, 1),
            ("choyin.npc.dumpling_seller", 15, 1),
            ("choyin.npc.lboy", 2, 2),
            ("oldpine.npc.wolf_dog", 15, 1),
            ("chuenyu.npc.chuenyu", 50, 1),
            ("chuenyu.npc.wolfdog", 15, 1),
            ("green.npc.kid4", 10, 3),
            ("green.npc.woman2", 7, 4),
        ] {
            let npc_id = NpcId::from(npc_id);
            let chat = npcs().ambient_chat(&npc_id).unwrap();
            assert_eq!(chat.runtime_chance(), expected_chance, "{}", npc_id.as_str());
            assert_eq!(chat.entries.len(), expected_entries, "{}", npc_id.as_str());
            assert!(
                chat.entries.iter().all(|entry| entry.kind == "text"),
                "{}",
                npc_id.as_str()
            );
            assert!(
                npcs().definition(&npc_id).unwrap().placement_count > 0,
                "{}",
                npc_id.as_str()
            );
        }

        let cloud_worker = npcs()
            .ambient_chat(&NpcId::from("u.cloud.npc.worker"))
            .unwrap();
        assert_eq!(cloud_worker.runtime_chance(), 10);
        assert_eq!(cloud_worker.entries.len(), 1);
        assert_eq!(cloud_worker.entries[0].kind, "movement");
    }

    #[test]
    fn fixed_m5_catalog_profiles_and_references_are_runtime_ready() {
        let catalog: serde_json::Value = serde_json::from_str(M5_CATALOG_JSON).unwrap();
        assert_eq!(catalog["summary"]["total"], 49);
        assert_eq!(catalog["summary"]["placed"], 46);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 29);
        assert_eq!(catalog["summary"]["carried_item_entries"], 53);

        let guard = npcs().definition(&NpcId::from("obj.npc.garrison")).unwrap();
        assert_eq!(guard.name, "县城官兵");
        assert_eq!(guard.placement_count, 2);
        assert_eq!(guard.combat_exp, Some(100_000));
        assert_eq!(guard.skills["sword"], 70);
        assert_eq!(guard.combat_apply["attack"], 70);
        assert_eq!(guard.combat_chat.as_ref().unwrap().entries.len(), 2);
        assert_eq!(guard.carried_items.len(), 2);

        for (npc_id, expected_chance) in [
            ("oldpine.npc.fat_bandit", 10),
            ("oldpine.npc.bandit_chief", 10),
            ("oldpine.npc.wolf_dog", 15),
        ] {
            assert_eq!(
                npcs()
                    .definition(&NpcId::from(npc_id))
                    .unwrap()
                    .combat_chat
                    .as_ref()
                    .unwrap()
                    .runtime_chance(),
                expected_chance
            );
        }
    }

    #[test]
    fn fixed_m6_catalog_static_runtime_baseline_is_complete() {
        let catalog: serde_json::Value = serde_json::from_str(M6_CATALOG_JSON).unwrap();
        assert_eq!(catalog["scope"], "m6-npcs");
        assert_eq!(catalog["summary"]["total"], 57);
        assert_eq!(catalog["summary"]["placed"], 48);
        assert_eq!(catalog["summary"]["vendors"], 1);
        assert_eq!(catalog["summary"]["vendor_goods"], 6);
        assert_eq!(catalog["summary"]["static_inquiries"], 10);
        assert_eq!(catalog["summary"]["scripted_inquiries"], 7);
        assert_eq!(catalog["summary"]["combat_profiles"], 53);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 33);
        assert_eq!(catalog["summary"]["carried_item_entries"], 81);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 81);

        let flower_seller = npcs()
            .definition(&NpcId::from("chuenyu.npc.flower_seller"))
            .unwrap();
        assert_eq!(flower_seller.placement_count, 1);
        assert_eq!(flower_seller.vendor_goods.len(), 6);
        assert_eq!(flower_seller.carried_items.len(), 3);
        assert!(
            npcs()
                .definition(&NpcId::from("waterfog.npc.elite_guard"))
                .unwrap()
                .combat_chat
                .as_ref()
                .is_some_and(|chat| chat.entries.len() == 4)
        );
    }

    #[test]
    fn fixed_m7_catalog_static_runtime_baseline_is_complete() {
        let catalog: serde_json::Value = serde_json::from_str(M7_CATALOG_JSON).unwrap();
        assert_eq!(catalog["scope"], "m7-npcs");
        assert_eq!(catalog["summary"]["total"], 85);
        assert_eq!(catalog["summary"]["placed"], 65);
        assert_eq!(catalog["summary"]["vendors"], 8);
        assert_eq!(catalog["summary"]["vendor_goods"], 29);
        assert_eq!(catalog["summary"]["static_inquiries"], 56);
        assert_eq!(catalog["summary"]["scripted_inquiries"], 11);
        assert_eq!(catalog["summary"]["combat_profiles"], 85);
        assert_eq!(catalog["summary"]["combat_chat_entries"], 27);
        assert_eq!(catalog["summary"]["carried_item_entries"], 122);
        assert_eq!(catalog["warnings"].as_array().unwrap().len(), 183);

        let header = npcs()
            .definition(&NpcId::from("u.cloud.npc.b_header"))
            .unwrap();
        assert_eq!(header.placement_count, 1);
        assert_eq!(header.inquiries.len(), 5);
        assert_eq!(
            npcs()
                .definition(&NpcId::from("u.cloud.npc.butcher"))
                .unwrap()
                .vendor_goods
                .len(),
            5
        );
        assert_eq!(
            npcs()
                .id_for_source("mudlib/d/snow/npc/beggar.c")
                .unwrap()
                .as_str(),
            "snow.npc.beggar"
        );
        assert_eq!(
            npcs()
                .id_for_source("mudlib/obj/npc/garrison.c")
                .unwrap()
                .as_str(),
            "obj.npc.garrison"
        );
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
        assert_eq!(NpcId::from(XIAO_JUAN_ID).name(), "小娟");
    }

    #[test]
    fn source_combat_profiles_and_chat_are_runtime_ready() {
        let banker = npcs().definition(&NpcId::from(CITY_BANKER_ID)).unwrap();
        assert_eq!(banker.attributes["str"], 22);
        assert_eq!(banker.resources["max_force"], 1_000);
        assert_eq!(banker.skills["necromancy"], 100);
        assert_eq!(banker.mappings["spells"], "necromancy");
        assert_eq!(banker.combat_rating(), 50);
        assert_eq!(banker.combat_max_health(), 198);
        assert_eq!(banker.combat_attack(), 33);
        assert!(banker.attacks_spirit());
        let chat = banker.combat_chat.as_ref().unwrap();
        assert_eq!(chat.runtime_chance(), 40);
        assert_eq!(chat.entries.len(), 9);
        assert!(chat.entries.iter().any(|entry| entry.kind == "text"));
        assert!(chat.entries.iter().any(|entry| entry.kind == "spell"));

        let dynamic_chance = npcs()
            .definition(&NpcId::from("city.shangshu.npc.huyuan1"))
            .unwrap()
            .combat_chat
            .as_ref()
            .unwrap();
        assert_eq!(
            dynamic_chance.chance_expression.as_deref(),
            Some("random(40)")
        );
        assert_eq!(dynamic_chance.runtime_chance(), 20);

        let chats: Vec<_> = npcs()
            .definitions
            .values()
            .filter(|npc| is_m4_source(npc))
            .filter_map(|npc| npc.combat_chat.as_ref())
            .collect();
        assert_eq!(chats.len(), 25);
        assert_eq!(
            chats.iter().map(|chat| chat.entries.len()).sum::<usize>(),
            87
        );
        assert!(chats.iter().all(|chat| !chat.entries.is_empty()));

        let carried: Vec<_> = npcs()
            .definitions
            .values()
            .filter(|npc| is_m4_source(npc))
            .flat_map(|npc| &npc.carried_items)
            .collect();
        assert_eq!(carried.len(), 93);
        assert_eq!(
            carried.iter().filter(|item| item.state == "worn").count(),
            56
        );
        assert_eq!(
            carried
                .iter()
                .filter(|item| item.state == "wielded")
                .count(),
            32
        );
        assert!(carried.iter().all(|item| items().contains(&item.item_id)));
    }

    #[test]
    fn every_source_fight_gate_has_an_explicit_runtime_policy() {
        let gated: Vec<_> = npcs()
            .definitions
            .values()
            .filter(|npc| is_m4_source(npc))
            .filter(|npc| npc.behavior_flags.iter().any(|flag| flag == "fight_gate"))
            .collect();
        assert_eq!(gated.len(), 9);
        assert_eq!(
            gated.iter().filter(|npc| npc.placement_count > 0).count(),
            8
        );
        assert!(gated.iter().all(|npc| npc.fight_policy().is_some()));
        assert_eq!(
            npcs()
                .definition(&NpcId::from(CITY_BANKER_ID))
                .unwrap()
                .fight_policy(),
            Some(NpcFightPolicy::ForceLethal)
        );
        assert_eq!(
            npcs()
                .definition(&NpcId::from(SNOW_FIST_TRAINER_ID))
                .unwrap()
                .fight_policy(),
            Some(NpcFightPolicy::RequireFaction("封山剑派"))
        );
        assert_eq!(
            npcs()
                .definition(&NpcId::from("snow.npc.beggar"))
                .unwrap()
                .fight_policy(),
            Some(NpcFightPolicy::Reject)
        );
    }

    #[test]
    fn every_m5_source_fight_gate_has_an_explicit_runtime_policy() {
        let gated: Vec<_> = npcs()
            .definitions
            .values()
            .filter(|npc| npc.area == "choyin")
            .filter(|npc| npc.behavior_flags.iter().any(|flag| flag == "fight_gate"))
            .collect();
        assert_eq!(gated.len(), 3);
        assert!(gated.iter().all(|npc| npc.placement_count > 0));
        assert!(gated.iter().all(|npc| npc.fight_policy().is_some()));
        assert_eq!(
            npcs()
                .definition(&NpcId::from(CHOYIN_OLD_MAN_ID))
                .unwrap()
                .fight_policy(),
            Some(NpcFightPolicy::Allow)
        );
        assert_eq!(
            npcs()
                .definition(&NpcId::from(CHOYIN_MAGISTRATE_ID))
                .unwrap()
                .fight_policy(),
            Some(NpcFightPolicy::Reject)
        );
    }

    #[test]
    fn every_source_apprenticeship_flag_has_an_explicit_disposition() {
        let candidates: Vec<_> = npcs()
            .definitions
            .values()
            .filter(|npc| is_m4_source(npc))
            .filter(|npc| {
                npc.behavior_flags
                    .iter()
                    .any(|flag| flag == "apprenticeship")
            })
            .collect();
        assert_eq!(candidates.len(), 8);
        assert_eq!(
            candidates
                .iter()
                .filter(|npc| npc.placement_count > 0)
                .count(),
            6
        );
        assert!(
            candidates
                .iter()
                .all(|npc| npc.apprenticeship_policy().is_some())
        );

        let runtime_instructors: Vec<_> = candidates
            .iter()
            .filter(|npc| npc.placement_count > 0 && !npc.lessons().is_empty())
            .collect();
        assert_eq!(runtime_instructors.len(), 6);
        assert_eq!(
            runtime_instructors
                .iter()
                .map(|npc| npc.lessons().len())
                .sum::<usize>(),
            46
        );
        assert!(runtime_instructors.iter().all(|npc| {
            npc.teaching_intelligence().is_some()
                && npc
                    .lessons()
                    .iter()
                    .all(|lesson| crate::skills::skills().by_id(lesson.skill).is_some())
        }));
        assert_eq!(
            npcs()
                .definition(&NpcId::from(CITY_MASTER_CHEN_ID))
                .unwrap()
                .apprenticeship_policy(),
            Some(NpcApprenticeshipPolicy::DeferredLetter)
        );
        assert_eq!(
            npcs()
                .definition(&NpcId::from(SNOW_TEACHER_ID))
                .unwrap()
                .apprenticeship_policy(),
            Some(NpcApprenticeshipPolicy::PaidStudent)
        );
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
            "choyin.npc.cake_vendor",
            "choyin.npc.cucurbit_seller",
            "choyin.npc.dumpling_seller",
            "choyin.npc.girl",
            "choyin.npc.yamen_po",
            "choyin.npc.youngman",
            "city.npc.caker",
            "city.npc.dumpling_seller",
            "city.npc.vendor",
            GREEN_OLD_MAN_ID,
            GREEN_SHEN_ID,
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

        assert_eq!(handled.len(), 19);
        assert_eq!(
            handled
                .iter()
                .filter(|(_, _, kind)| *kind == ScriptedInquiryKind::VendorList)
                .count(),
            6
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
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == CHOYIN_GIRL_ID && *topic == "游晋" && *kind == ScriptedInquiryKind::ChoyinSilkBag
        }));
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == CHOYIN_YOUNG_MAN_ID
                && *topic == "trouble"
                && *kind == ScriptedInquiryKind::ChoyinYoungManTrouble
        }));
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == GREEN_OLD_MAN_ID
                && *topic == "玉佩"
                && *kind == ScriptedInquiryKind::GreenOldManJade
        }));
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == GREEN_SHEN_ID && *topic == "玉佩" && *kind == ScriptedInquiryKind::GreenShenJade
        }));
        assert!(handled.iter().any(|(id, topic, kind)| {
            *id == GREEN_SHEN_ID
                && *topic == "蒙汗药"
                && *kind == ScriptedInquiryKind::GreenShenSlumberDrug
        }));
    }

    #[test]
    fn m6_fight_apprenticeship_and_exchange_policies_are_explicit() {
        for id in [
            CHUENYU_OLD_LIU_ID,
            CHUENYU_XIAO_JUAN_ID,
            CHUENYU_XIAO_JUAN_PLACED_ID,
            "sanyen.npc.bonze",
            "sanyen.npc.cook_bonze",
            "sanyen.npc.cripple",
            "sanyen.npc.greeting",
            "sanyen.npc.little_bonze",
            "sanyen.npc.work_bonze",
            WATERFOG_ELITE_GUARD_ID,
        ] {
            assert_eq!(
                npcs().definition(&NpcId::from(id)).unwrap().fight_policy(),
                Some(NpcFightPolicy::Reject),
                "{id}"
            );
        }
        assert_eq!(
            npcs()
                .definition(&NpcId::from(GREEN_MASTER_ID))
                .unwrap()
                .apprenticeship_policy(),
            Some(NpcApprenticeshipPolicy::ExcludedUnplaced)
        );
        assert_eq!(
            npcs()
                .definition(&NpcId::from(SNOW_DRUNK_ID))
                .unwrap()
                .object_exchange_kind(),
            Some(ObjectExchangeKind::SnowDrunk)
        );
        assert_eq!(
            npcs()
                .definition(&NpcId::from(GREEN_SHEN_ID))
                .unwrap()
                .object_exchange_kind(),
            Some(ObjectExchangeKind::GreenShen)
        );
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
                CHOYIN_SERGEANT_ID,
                CHOYIN_YOUNG_MAN_ID,
                CITY_GUARD_ID,
                CITY_MASTER_CHEN_ID,
                CITY_SHANGSHU_GUARD_ID,
                CITY_WAITER_ID,
                CLOUD_B_HEADER_ID,
                CLOUD_BOATER_ID,
                CLOUD_GANGSTER_ID,
                CLOUD_GIRL_ID,
                CLOUD_JUDGE_ID,
                CLOUD_MONK_ID,
                GREEN_SHEN_ID,
                LATEMOON_FUNLIN_ID,
                LATEMOON_OLD_ID,
                LATEMOON_SHAOWEI_ID,
                "snow.npc.scavenger",
                SNOW_DRUNK_ID,
                SNOW_KEEPER_ID,
                SNOW_TEACHER_ID,
            ])
        );
    }
}
