use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Deserializer, Serialize};

const CATALOG_JSON: &str = include_str!("../migration/catalog/skills.json");

pub const AXE_ID: &str = "axe";
pub const BLADE_ID: &str = "blade";
pub const DAGGER_ID: &str = "dagger";
pub const DODGE_ID: &str = "dodge";
pub const FORCE_ID: &str = "force";
pub const FORK_ID: &str = "fork";
pub const HAMMER_ID: &str = "hammer";
pub const MAGIC_ID: &str = "magic";
pub const MOVE_ID: &str = "move";
pub const PARRY_ID: &str = "parry";
pub const SPELLS_ID: &str = "spells";
pub const STAFF_ID: &str = "staff";
pub const SWORD_ID: &str = "sword";
pub const THROWING_ID: &str = "throwing";
pub const UNARMED_ID: &str = "unarmed";
pub const WHIP_ID: &str = "whip";

pub const LIUH_KEN_ID: &str = "liuh-ken";
pub const SIX_CHAOS_SWORD_ID: &str = "six-chaos-sword";
pub const PYROBAT_STEPS_ID: &str = "pyrobat-steps";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechniqueKind {
    RecoverQi,
    RefreshSpirit,
    RegenerateEssence,
    VoidSense,
    LotusHeal,
    ChillGaze,
    PowerFade,
    PowerUp,
    Roar,
    Hasten,
    FonxanHeal,
    Counterattack,
    FakeFault,
    SwordJab,
    Concentrate,
    GouyeeHeal,
    AstralVision,
    DrainerBolt,
    FeebleBolt,
    NetherBolt,
}

impl TechniqueKind {
    pub const ALL: [Self; 20] = [
        Self::RecoverQi,
        Self::RefreshSpirit,
        Self::RegenerateEssence,
        Self::VoidSense,
        Self::LotusHeal,
        Self::ChillGaze,
        Self::PowerFade,
        Self::PowerUp,
        Self::Roar,
        Self::Hasten,
        Self::FonxanHeal,
        Self::Counterattack,
        Self::FakeFault,
        Self::SwordJab,
        Self::Concentrate,
        Self::GouyeeHeal,
        Self::AstralVision,
        Self::DrainerBolt,
        Self::FeebleBolt,
        Self::NetherBolt,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::RecoverQi => "force.recover",
            Self::RefreshSpirit => "force.refresh",
            Self::RegenerateEssence => "force.regenerate",
            Self::VoidSense => "essencemagic.void_sense",
            Self::LotusHeal => "lotusforce.heal",
            Self::ChillGaze => "iceforce.chillgaze",
            Self::PowerFade => "celestial.powerfade",
            Self::PowerUp => "celestial.powerup",
            Self::Roar => "celestial.roar",
            Self::Hasten => "mysterrier.hasten",
            Self::FonxanHeal => "fonxanforce.heal",
            Self::Counterattack => "fonxansword.counterattack",
            Self::FakeFault => "fonxansword.fakefault",
            Self::SwordJab => "fonxansword.swordjab",
            Self::Concentrate => "gouyee.concentrate",
            Self::GouyeeHeal => "gouyee.heal",
            Self::AstralVision => "necromancy.astral_vision",
            Self::DrainerBolt => "necromancy.drainerbolt",
            Self::FeebleBolt => "necromancy.feeblebolt",
            Self::NetherBolt => "necromancy.netherbolt",
        }
    }

    pub fn skill_id(self) -> &'static str {
        match self {
            Self::RecoverQi | Self::RefreshSpirit | Self::RegenerateEssence => FORCE_ID,
            Self::VoidSense => "essencemagic",
            Self::LotusHeal => "lotusforce",
            Self::ChillGaze => "iceforce",
            Self::PowerFade | Self::PowerUp | Self::Roar => "celestial",
            Self::Hasten => "mysterrier",
            Self::FonxanHeal => "fonxanforce",
            Self::Counterattack | Self::FakeFault | Self::SwordJab => "fonxansword",
            Self::Concentrate | Self::GouyeeHeal => "gouyee",
            Self::AstralVision | Self::DrainerBolt | Self::FeebleBolt | Self::NetherBolt => {
                "necromancy"
            }
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::RecoverQi => "调匀气息",
            Self::RefreshSpirit => "收摄心神",
            Self::RegenerateEssence => "恢复精力",
            Self::VoidSense => "无相禅悟",
            Self::LotusHeal | Self::FonxanHeal | Self::GouyeeHeal => "运功疗伤",
            Self::ChillGaze => "寒冰凝视",
            Self::PowerFade => "化去杀气",
            Self::PowerUp => "天邪聚力",
            Self::Roar => "天邪怒吼",
            Self::Hasten => "步玄连环",
            Self::Counterattack => "封山反击",
            Self::FakeFault => "卖个破绽",
            Self::SwordJab => "剑气突刺",
            Self::Concentrate => "凝神归法",
            Self::AstralVision => "灵视",
            Self::DrainerBolt => "吸精鬼火",
            Self::FeebleBolt => "蚀神鬼火",
            Self::NetherBolt => "冥火咒",
        }
    }

    pub fn combat_only(self) -> bool {
        matches!(
            self,
            Self::ChillGaze
                | Self::PowerFade
                | Self::PowerUp
                | Self::Roar
                | Self::Hasten
                | Self::Counterattack
                | Self::FakeFault
                | Self::SwordJab
                | Self::DrainerBolt
                | Self::FeebleBolt
                | Self::NetherBolt
        )
    }

    pub fn required_usage(self) -> Option<&'static str> {
        match self {
            Self::RecoverQi | Self::RefreshSpirit | Self::RegenerateEssence => None,
            Self::VoidSense => Some(MAGIC_ID),
            Self::LotusHeal
            | Self::ChillGaze
            | Self::PowerFade
            | Self::PowerUp
            | Self::Roar
            | Self::FonxanHeal
            | Self::Concentrate
            | Self::GouyeeHeal => Some(FORCE_ID),
            Self::Hasten => Some(DODGE_ID),
            Self::Counterattack | Self::FakeFault | Self::SwordJab => Some(SWORD_ID),
            Self::AstralVision | Self::DrainerBolt | Self::FeebleBolt | Self::NetherBolt => {
                Some(SPELLS_ID)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SkillId(String);

impl SkillId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn name(&self) -> &str {
        skills()
            .definition(self)
            .map_or(self.as_str(), SkillDefinition::name)
    }
}

impl From<&str> for SkillId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for SkillId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from(match value.as_str() {
            "Unarmed" => LIUH_KEN_ID,
            "Sword" => SIX_CHAOS_SWORD_ID,
            "Dodge" => PYROBAT_STEPS_ID,
            "Breathing" => FORCE_ID,
            "Parry" => PARRY_ID,
            current => current,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillCategory {
    Basic,
    Formation,
    Internal,
    Knowledge,
    Movement,
    Mystic,
    Unarmed,
    Utility,
    Weapon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillActionDefinition {
    pub name: Option<String>,
    pub description: Option<String>,
    pub weapon: Option<String>,
    pub damage_type: Option<String>,
    pub dodge: Option<i32>,
    pub parry: Option<i32>,
    pub force: Option<i32>,
    pub damage: Option<i32>,
}

impl SkillActionDefinition {
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .or_else(|| {
                let description = self.description.as_deref()?;
                let start = description.find('「')? + '「'.len_utf8();
                let end = description[start..].find('」')? + start;
                description.get(start..end)
            })
            .unwrap_or("寻常一式")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub source_path: String,
    pub status: String,
    pub display_name: String,
    pub category: SkillCategory,
    pub skill_type: String,
    pub usages: Vec<String>,
    pub dependencies: Vec<String>,
    pub actions: Vec<SkillActionDefinition>,
    pub valid_learn: Option<String>,
    pub practice: Option<String>,
    pub routers: Vec<String>,
    pub behavior_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherDefinition {
    pub id: String,
    pub source_path: String,
    pub status: String,
    pub name: String,
    pub faction: Option<String>,
    pub intelligence: Option<i32>,
    pub skills: HashMap<String, i32>,
    pub mappings: HashMap<String, String>,
    pub apprentice_behavior: Option<String>,
}

impl SkillDefinition {
    pub fn name(&self) -> &str {
        skill_name_override(self.id.as_str()).unwrap_or(&self.display_name)
    }

    pub fn supports_usage(&self, usage: &str) -> bool {
        self.usages.iter().any(|candidate| candidate == usage)
    }
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    source_commit: String,
    skills: Vec<SkillDefinition>,
    teachers: Vec<TeacherDefinition>,
}

static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    let catalog: Catalog =
        serde_json::from_str(CATALOG_JSON).expect("invalid embedded skill catalog");
    assert_eq!(
        catalog.schema_version, 3,
        "unsupported skill catalog schema"
    );
    assert_eq!(catalog.skills.len(), 70, "unexpected skill catalog count");
    assert_eq!(catalog.teachers.len(), 11, "unexpected teacher count");
    assert_eq!(
        catalog
            .skills
            .iter()
            .map(|skill| skill.actions.len())
            .sum::<usize>(),
        114,
        "unexpected skill action count"
    );
    catalog
});

pub struct SkillRepository {
    definitions: HashMap<SkillId, SkillDefinition>,
    teachers: HashMap<String, TeacherDefinition>,
    source_commit: String,
}

impl SkillRepository {
    fn load() -> Self {
        Self {
            definitions: CATALOG
                .skills
                .iter()
                .cloned()
                .map(|skill| (skill.id.clone(), skill))
                .collect(),
            teachers: CATALOG
                .teachers
                .iter()
                .cloned()
                .map(|teacher| (teacher.id.clone(), teacher))
                .collect(),
            source_commit: CATALOG.source_commit.clone(),
        }
    }

    pub fn definition(&self, id: &SkillId) -> Option<&SkillDefinition> {
        self.definitions.get(id)
    }

    pub fn by_id(&self, id: &str) -> Option<&SkillDefinition> {
        self.definition(&SkillId::from(id))
    }

    pub fn all(&self) -> impl Iterator<Item = &SkillDefinition> {
        self.definitions.values()
    }

    pub fn teacher(&self, id: &str) -> Option<&TeacherDefinition> {
        self.teachers.get(id)
    }

    pub fn teachers(&self) -> impl Iterator<Item = &TeacherDefinition> {
        self.teachers.values()
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn source_commit(&self) -> &str {
        &self.source_commit
    }
}

static SKILLS: LazyLock<SkillRepository> = LazyLock::new(SkillRepository::load);

pub fn skills() -> &'static SkillRepository {
    &SKILLS
}

fn skill_name_override(id: &str) -> Option<&'static str> {
    Some(match id {
        "axe" => "斧法",
        "blade" => "刀法",
        "bloodystrike" => "密宗大手印",
        "bolomiduo" => "婆萝蜜多心经",
        "buddhism" => "大乘佛法",
        "celestrike" => "天邪掌法",
        "chanting" => "诵经",
        "dagger" => "匕首",
        "deisword" => "蝶恋花剑法",
        "dodge" => "闪躲",
        "fall-steps" => "秋风步",
        "fonxanforce" => "封山派内功",
        "force" => "内功",
        "fork" => "叉法",
        "hammer" => "锤法",
        "iceforce" => "意寒功",
        "instruments" => "乐器",
        "iron-cloth" => "铁布衫",
        "jin-gang" => "金刚不坏功",
        "juechen-force" => "绝尘心法",
        "linbo-steps" => "凌波微步",
        "literate" => "读书识字",
        "magic" => "法术",
        "magic-array" => "奇门遁甲之术",
        "meihua-shou" => "一剪梅花手",
        "move" => "行动",
        "music" => "音律",
        "mystsword" => "小步玄剑",
        "nine-moon" => "九阴赤炼剑法",
        "notraces" => "踏雪无痕",
        "parry" => "招架",
        "pyrobat-steps" => "火蝠身法",
        "qidaoforce" => "棋道",
        "scratching" => "天师剑法",
        "serpentforce" => "伏蛟功",
        "shortsong-blade" => "短歌刀法",
        "snowshade-force" => "天山雪影心法",
        "snowshade-sword" => "雪影剑法",
        "snowwhip" => "寒雪鞭法",
        "spells" => "咒文",
        "spicyclaw" => "油流麻香手",
        "spider-array" => "蜘蛛阵法",
        "staff" => "杖法",
        "stealing" => "偷窃",
        "stormdance" => "七宝天岚舞",
        "sword" => "剑法",
        "tenderzhi" => "柔虹指",
        "throwing" => "暗器",
        "ts-fist" => "天山折梅手",
        "unarmed" => "拳脚",
        "whip" => "鞭法",
        "wu-shun" => "小无相功",
        "yirong" => "易容术",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    #[test]
    fn repository_contains_complete_fixed_catalog() {
        let repository = skills();
        assert_eq!(repository.len(), 70);
        assert_eq!(
            repository.source_commit(),
            "87bba6bd2249beec8424b0d6623486a0dd1f7b30"
        );
        let ids: HashSet<_> = repository.all().map(|skill| skill.id.as_str()).collect();
        assert_eq!(ids.len(), 70);
        assert_eq!(repository.teachers().count(), 11);
        let fighter = repository.teacher("fighter").unwrap();
        assert_eq!(fighter.faction.as_deref(), Some("天邪派"));
        assert_eq!(fighter.skills.get("celestial"), Some(&100));
        assert_eq!(fighter.intelligence, Some(24));
        assert_eq!(
            fighter.mappings.get(FORCE_ID).map(String::as_str),
            Some("celestial")
        );
        for required in [UNARMED_ID, LIUH_KEN_ID, SIX_CHAOS_SWORD_ID, FORCE_ID] {
            assert!(ids.contains(required));
        }
    }

    #[test]
    fn action_names_fall_back_to_descriptions() {
        let skill = skills().by_id("bloodystrike").unwrap();
        assert_eq!(skill.actions.len(), 6);
        assert_eq!(skill.actions[0].display_name(), "苦海端无涯");

        let sword = skills().by_id(SIX_CHAOS_SWORD_ID).unwrap();
        assert_eq!(sword.name(), "六阴追魂剑法");
        assert!(sword.supports_usage(SWORD_ID));
        assert!(sword.supports_usage(PARRY_ID));
    }

    #[test]
    fn technique_ledger_covers_all_callable_scripts_and_runtime_actions() {
        #[derive(Deserialize)]
        struct Ledger {
            schema_version: u32,
            source_commit: String,
            techniques: Vec<LedgerEntry>,
            behavior_dispositions: Vec<BehaviorDisposition>,
        }
        #[derive(Deserialize)]
        struct LedgerEntry {
            source_path: String,
            status: String,
            runtime_id: Option<String>,
            milestone: Option<String>,
            rationale: Option<String>,
        }
        #[derive(Deserialize)]
        struct BehaviorDisposition {
            flag: String,
            source_count: usize,
            status: String,
            milestone: Option<String>,
            rationale: Option<String>,
        }

        let ledger: Ledger =
            serde_json::from_str(include_str!("../migration/overrides/skills.json")).unwrap();
        assert_eq!(ledger.schema_version, 1);
        assert_eq!(ledger.source_commit, skills().source_commit());
        assert_eq!(ledger.techniques.len(), 33);

        let paths: HashSet<_> = ledger
            .techniques
            .iter()
            .map(|entry| entry.source_path.as_str())
            .collect();
        assert_eq!(paths.len(), 33);
        let expected_runtime: HashSet<_> = TechniqueKind::ALL
            .iter()
            .map(|technique| technique.id())
            .collect();
        let verified_runtime: HashSet<_> = ledger
            .techniques
            .iter()
            .filter(|entry| entry.status == "verified")
            .map(|entry| entry.runtime_id.as_deref().expect("verified runtime ID"))
            .collect();
        assert_eq!(verified_runtime, expected_runtime);

        for entry in &ledger.techniques {
            match entry.status.as_str() {
                "verified" => assert!(entry.runtime_id.is_some()),
                "deferred" | "alias" => {
                    assert!(entry.milestone.is_some());
                    assert!(
                        entry
                            .rationale
                            .as_deref()
                            .is_some_and(|text| !text.is_empty())
                    );
                }
                status => panic!("unknown technique status {status}"),
            }
        }

        let mut catalog_flags = HashMap::<&str, usize>::new();
        for definition in skills().all() {
            for flag in &definition.behavior_flags {
                *catalog_flags.entry(flag).or_default() += 1;
            }
        }
        let ledger_flags: HashMap<_, _> = ledger
            .behavior_dispositions
            .iter()
            .map(|entry| (entry.flag.as_str(), entry.source_count))
            .collect();
        assert_eq!(ledger_flags, catalog_flags);
        for entry in &ledger.behavior_dispositions {
            assert!(matches!(
                entry.status.as_str(),
                "verified" | "adapted" | "tracked" | "source_noop" | "deferred"
            ));
            if entry.status == "deferred" {
                assert!(entry.milestone.is_some());
            }
            if matches!(
                entry.status.as_str(),
                "adapted" | "source_noop" | "deferred"
            ) {
                assert!(
                    entry
                        .rationale
                        .as_deref()
                        .is_some_and(|text| !text.is_empty())
                );
            }
        }
    }
}
