use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    content::{self, world},
    items::{self, EquipmentSlot, ItemId, ItemInstance, LegacyItemKind, items},
    npcs::{
        FARM_WOMAN_ID, FISHER_ID, FLOWER_GIRL_ID, MELONER_ID, NpcId, OLD_LIU_ID,
        ObjectExchangeKind, SNOW_GUARD_ID, SNOW_TEACHER_ID, ScriptedInquiryKind, TEA_SELLER_ID,
        TEMPLE_MASTER_ID, TRADER_ID, npcs,
    },
    skills::{
        self, DODGE_ID, FORCE_ID, LIUH_KEN_ID, MAGIC_ID, MOVE_ID, PARRY_ID, PYROBAT_STEPS_ID,
        SIX_CHAOS_SWORD_ID, SPELLS_ID, SWORD_ID, SkillId, TechniqueKind, UNARMED_ID, skills,
    },
};

const LOG_LIMIT: usize = 80;
const SAVE_VERSION: u32 = 7;
const DEFAULT_FOOD_CAPACITY: i32 = 200;
const DEFAULT_WATER_CAPACITY: i32 = 200;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocationId(String);

impl LocationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LocationId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyKind {
    Bandit,
    Wolf,
    TempleDisciple,
    Rat,
    IceDragon,
    Meloner,
    BloodHandLiuSan,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    #[default]
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStage {
    Unasked,
    FindJuan,
    ReturnHome,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionKind {
    Bandaged,
    SnakePoison,
    Poison,
    Drunk,
    Slumber,
    AstralVision,
}

impl ConditionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Bandaged => "包扎",
            Self::SnakePoison => "蛇毒",
            Self::Poison => "中毒",
            Self::Drunk => "醉酒",
            Self::Slumber => "蒙汗药",
            Self::AstralVision => "灵视",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionState {
    pub kind: ConditionKind,
    pub duration: u32,
    pub potency: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorKind {
    LiuGarden,
    LordManor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionKind {
    PaddleToLake,
    PaddleToShore,
    DiveIntoLake,
    RevealGrassPath,
    OpenDoor(DoorKind),
    CloseDoor(DoorKind),
    InspectTablet,
    InspectBookshelf,
    PullBook(u8),
    PickMelon,
    SettleMelonDebt,
    SwearCanyonSecret,
    ClimbCanyonChain,
    ClimbCityWall,
    JumpIntoCityManor,
    JumpOutsideCityWall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Move {
        direction: String,
        target: LocationId,
    },
    Flee {
        direction: String,
        target: LocationId,
    },
    Interact(InteractionKind),
    Talk(NpcId),
    AskNpc {
        npc: NpcId,
        topic: String,
    },
    BecomeApprentice(String),
    LearnSkill {
        skill: SkillId,
        teacher: String,
    },
    LearnFromNpc {
        skill: SkillId,
        npc: NpcId,
    },
    MapSkill {
        usage: SkillId,
        skill: SkillId,
    },
    Train(SkillId),
    PracticeSkill(SkillId),
    StudyItem(u64),
    Cultivate(CultivationKind),
    UseTechnique(TechniqueKind),
    Rest,
    Fight(EnemyKind),
    Kill(EnemyKind),
    BuyItem {
        item_id: ItemId,
        npc: NpcId,
    },
    OfferMoney {
        amount: u64,
        npc: NpcId,
    },
    SellItem(u64),
    GiveItem {
        instance_id: u64,
        npc: NpcId,
    },
    PickUpItem(u64),
    DropItem(u64),
    EquipItem(u64),
    UnequipItem(EquipmentSlot),
    ConsumeItem(u64),
    ApplyItem(u64),
    MixIntoLiquid {
        powder_instance_id: u64,
        liquid_instance_id: u64,
    },
    Surrender,
}

impl DoorKind {
    fn name(self) -> &'static str {
        match self {
            Self::LiuGarden => "刘家木门",
            Self::LordManor => "田老财家大门",
        }
    }
}

impl InteractionKind {
    fn label(self, game: &Game) -> String {
        match self {
            Self::PaddleToLake => "登船划向湖心".into(),
            Self::PaddleToShore => "划船返回湖畔".into(),
            Self::DiveIntoLake => "潜入湖底".into(),
            Self::RevealGrassPath => "拨开茂密茅草".into(),
            Self::OpenDoor(door) => format!("打开{}", door.name()),
            Self::CloseDoor(door) => format!("关上{}", door.name()),
            Self::InspectTablet => "查看路口旧牌".into(),
            Self::InspectBookshelf => "查看西墙书架".into(),
            Self::PullBook(number) => format!("抽动第{number}本石书"),
            Self::PickMelon => "摘一个熟西瓜".into(),
            Self::SettleMelonDebt => "向瓜农赔付瓜钱".into(),
            Self::SwearCanyonSecret => "面对山壁立誓".into(),
            Self::ClimbCanyonChain if game.location.as_str() == content::CANYON_FOOT => {
                "沿铁索向上攀爬".into()
            }
            Self::ClimbCanyonChain => "沿铁索向下攀爬".into(),
            Self::ClimbCityWall => "爬上尚书府院墙".into(),
            Self::JumpIntoCityManor => "跳入尚书府废屋".into(),
            Self::JumpOutsideCityWall => "跳回京师东街".into(),
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::PaddleToLake | Self::PaddleToShore => {
                "使用岸边木船渡过湖面，靠岸后木船仍会留在身边。"
            }
            Self::DiveIntoLake => "潜入白光闪动的深水，一旦进入岩洞便无法原路返回。",
            Self::RevealGrassPath => "拨开挡路茅草，隐藏路径只会短暂保持畅通。",
            Self::OpenDoor(_) | Self::CloseDoor(_) => "门的开关状态会同时作用于相连房间。",
            Self::InspectTablet | Self::InspectBookshelf => "仔细查看场景中可交互的物件。",
            Self::PullBook(_) => "石书机关来自原版未完成谜题，错误顺序会重置机关。",
            Self::PickMelon => "能否找到熟瓜取决于感知；瓜农可能发现你的行为。",
            Self::SettleMelonDebt => "支付原价 60 文瓜钱，瓜农便会让开道路。",
            Self::SwearCanyonSecret => "使用军师提供的口令进入黄石峡黑市，口令只能使用一次。",
            Self::ClimbCanyonChain => "沿原版铁索连接黄石隘口与雪亭镇官道，会消耗精、气、神。",
            Self::ClimbCityWall | Self::JumpIntoCityManor | Self::JumpOutsideCityWall => {
                "沿原版尚书府院墙路径移动。"
            }
        }
    }
}

impl Action {
    pub fn label(&self, game: &Game) -> String {
        match self {
            Self::Move { direction, target } => {
                let name = world()
                    .location(target)
                    .map_or("未知区域", |location| location.name.as_str());
                format!("{} · {}", direction_name(direction), name)
            }
            Self::Flee { direction, target } => {
                let name = world()
                    .location(target)
                    .map_or("未知区域", |location| location.name.as_str());
                format!("逃往{} · {}", direction_name(direction), name)
            }
            Self::Interact(interaction) => interaction.label(game),
            Self::Talk(npc) => format!("与{}交谈", npc.name()),
            Self::AskNpc { npc, topic } => {
                format!("向{}询问{}", npc.name(), inquiry_topic_name(topic))
            }
            Self::BecomeApprentice(teacher) => {
                let teacher = skills().teacher(teacher).expect("teacher must exist");
                format!("拜{}为师", teacher.name)
            }
            Self::LearnSkill { skill, teacher } => {
                let teacher = skills().teacher(teacher).expect("teacher must exist");
                format!("向{}请教{}", teacher.name, skill.name())
            }
            Self::LearnFromNpc { skill, npc } => {
                format!("向{}请教{}", npc.name(), skill.name())
            }
            Self::MapSkill { usage, skill } => {
                format!("将{}用于{}", skill.name(), usage.name())
            }
            Self::Train(skill) => {
                if game.activity == Activity::Training(skill.clone()) {
                    format!("停止修炼{}", skill.name())
                } else {
                    format!("修炼{}", skill.name())
                }
            }
            Self::PracticeSkill(skill) => format!("练习{}", skill.name()),
            Self::StudyItem(instance_id) => {
                format!("研读{}", game.inventory_item_name(*instance_id))
            }
            Self::Cultivate(kind) => kind.label().into(),
            Self::UseTechnique(technique) => format!("施展{}", technique.name()),
            Self::Rest => {
                if game.activity == Activity::Resting {
                    "结束休息".into()
                } else {
                    "休息片刻".into()
                }
            }
            Self::Fight(enemy) => {
                if *enemy == EnemyKind::Bandit && game.quest == QuestStage::FindJuan {
                    "循声营救娟儿".into()
                } else {
                    format!("与{}比试", enemy.name())
                }
            }
            Self::Kill(enemy) => format!("与{}性命相搏", enemy.name()),
            Self::BuyItem { item_id, npc } => {
                let definition = items()
                    .definition(item_id)
                    .expect("shop item must exist in catalog");
                let price = npcs()
                    .definition(npc)
                    .and_then(|seller| seller.price_for(item_id))
                    .expect("available shop item must have a price");
                format!(
                    "向{}购买{} · {}",
                    npc.name(),
                    definition.display_name(),
                    format_money(price)
                )
            }
            Self::OfferMoney { amount, npc } => {
                format!("给{}{}", npc.name(), format_money(*amount))
            }
            Self::SellItem(instance_id) => {
                format!("出售{}", game.inventory_item_name(*instance_id))
            }
            Self::GiveItem { instance_id, npc } => format!(
                "把{}交给{}",
                game.inventory_item_name(*instance_id),
                npc.name()
            ),
            Self::PickUpItem(instance_id) => {
                format!("拾取{}", game.ground_item_name(*instance_id))
            }
            Self::DropItem(instance_id) => {
                format!("丢下{}", game.inventory_item_name(*instance_id))
            }
            Self::EquipItem(instance_id) => {
                format!("装备{}", game.inventory_item_name(*instance_id))
            }
            Self::UnequipItem(slot) => format!("卸下{}", slot.name()),
            Self::ConsumeItem(instance_id) => {
                let Some(item) = game.player.item(*instance_id) else {
                    return "使用未知物品".into();
                };
                let verb = if item.definition().category == items::ItemCategory::Liquid {
                    "饮用"
                } else {
                    "食用"
                };
                format!("{verb}{}", game.inventory_item_name(*instance_id))
            }
            Self::ApplyItem(instance_id) => {
                let verb = game.player.item(*instance_id).map_or("使用", |item| {
                    match item.item_id.as_str() {
                        items::BANDAGE_ID => "包扎",
                        items::WOUND_MEDICINE_ID => "敷用",
                        items::SNAKE_MEDICINE_ID => "服用",
                        _ => "使用",
                    }
                });
                format!("{verb}{}", game.inventory_item_name(*instance_id))
            }
            Self::MixIntoLiquid {
                powder_instance_id,
                liquid_instance_id,
            } => format!(
                "把{}倒入{}",
                game.inventory_item_name(*powder_instance_id),
                game.inventory_item_name(*liquid_instance_id)
            ),
            Self::Surrender => "认输并退出战斗".into(),
        }
    }

    pub fn detail(&self) -> &'static str {
        match self {
            Self::Move { .. } => "移动会结束当前的修炼或休息。",
            Self::Flee { .. } => "脱离当前战斗并移动，临阵退却会损失少量评价。",
            Self::Interact(interaction) => interaction.detail(),
            Self::Talk(_) => "交谈可能带来线索、奖励或新的武学见闻。",
            Self::AskNpc { .. } => "按固定源人物的询问主题追问；仅开放已审计的文本或脚本回复。",
            Self::BecomeApprentice(_) => "加入师门后才能向掌门请教本门武学。",
            Self::LearnSkill { .. } | Self::LearnFromNpc { .. } => {
                "请教消耗神和潜能，造诣不能超过师父。"
            }
            Self::MapSkill { .. } => "把已学特殊技能映射到对应基础用途。",
            Self::Train(_) => "时间会自动推进，持续积累基础熟练度并消耗精力。",
            Self::PracticeSkill(_) => "按原版规则消耗气、神或内力练习已映射武学。",
            Self::StudyItem(_) => "研读秘笈需要读书识字，并受秘笈记载上限约束。",
            Self::Cultivate(_) => "把精、气或神转化为内力、灵力或法力。",
            Self::UseTechnique(_) => "绝招按原脚本消耗内力、法力、灵力或精气神。",
            Self::Rest => "逐步恢复精、气、神，全部恢复后自动结束。",
            Self::Fight(_) => "点到为止的比试；可以认输或从出口离开。",
            Self::Kill(_) => "死斗会造成伤势、杀气和通缉，不能认输。",
            Self::BuyItem { .. } => "按原物品价值付款；钱、银、金会自动换算。",
            Self::OfferMoney { .. } => "将现有货币按原版钱币对象价值交给当前人物。",
            Self::SellItem(_) => "商人按物品原价值的一半回收，损坏物品折价。",
            Self::GiveItem { .. } => "把未装备的物品赠予当前 NPC。",
            Self::PickUpItem(_) => "拾取地面物品；超过负重上限时无法拿起。",
            Self::DropItem(_) => "把未装备的物品留在当前位置。",
            Self::EquipItem(_) | Self::UnequipItem(_) => {
                "装备会提供武器伤害或护甲防御，并在战斗中损耗耐久。"
            }
            Self::ConsumeItem(_) => "食物和饮水按原物品份量逐次消耗，可恢复饱食或饮水。",
            Self::ApplyItem(_) => "药物与绷带不能在战斗中使用，其状态会随时间更新。",
            Self::MixIntoLiquid { .. } => "药粉会溶入尚有内容的酒水，饮用后触发对应药效。",
            Self::Surrender => "立即结束比试，评价会受到少量影响。",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CultivationKind {
    Exercise,
    Meditate,
    Respirate,
}

impl CultivationKind {
    fn label(self) -> &'static str {
        match self {
            Self::Exercise => "运气练功",
            Self::Meditate => "静坐冥思",
            Self::Respirate => "打坐修行",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    Resting,
    Training(SkillId),
    Fighting(CombatState),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatMode {
    #[default]
    Spar,
    Lethal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatState {
    pub enemy: EnemyKind,
    pub health: i32,
    pub max_health: i32,
    pub rounds: u32,
    #[serde(default)]
    pub mode: CombatMode,
    #[serde(default)]
    pub attack_bonus: i32,
    #[serde(default)]
    pub dodge_bonus: i32,
    #[serde(default)]
    pub enemy_busy_rounds: u8,
    #[serde(default)]
    pub technique_cooldown: u8,
    #[serde(default)]
    pub power_up_active: bool,
    #[serde(default)]
    pub fake_fault_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatResource {
    Essence,
    Qi,
    Spirit,
}

impl CombatResource {
    fn name(self) -> &'static str {
        match self {
            Self::Essence => "精",
            Self::Qi => "气",
            Self::Spirit => "神",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub kind: SkillId,
    pub level: u32,
    pub progress: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMapping {
    pub usage: SkillId,
    pub skill: SkillId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquippedItem {
    pub slot: EquipmentSlot,
    pub instance_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub essence: i32,
    pub max_essence: i32,
    pub qi: i32,
    pub max_qi: i32,
    pub spirit: i32,
    pub max_spirit: i32,
    pub strength: u32,
    pub perception: u32,
    #[serde(default = "default_intelligence")]
    pub intelligence: u32,
    #[serde(default = "default_spirituality")]
    pub spirituality: u32,
    #[serde(default = "default_courage")]
    pub courage: u32,
    #[serde(default = "default_constitution")]
    pub constitution: u32,
    #[serde(default = "default_courage")]
    pub composure: u32,
    #[serde(default)]
    pub gender: Gender,
    pub reputation: i32,
    pub insight: u32,
    #[serde(default = "default_combat_experience")]
    pub combat_experience: u64,
    #[serde(default = "default_potential")]
    pub potential: u32,
    #[serde(default)]
    pub learned_points: u32,
    #[serde(default)]
    pub bellicosity: i32,
    #[serde(default)]
    pub wanted: u32,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default)]
    pub teacher: Option<String>,
    #[serde(default = "default_force")]
    pub force: i32,
    #[serde(default = "default_max_force")]
    pub max_force: i32,
    #[serde(default)]
    pub mana: i32,
    #[serde(default)]
    pub max_mana: i32,
    #[serde(default)]
    pub atman: i32,
    #[serde(default)]
    pub max_atman: i32,
    pub silver: u32,
    #[serde(default)]
    pub coins: u32,
    #[serde(default)]
    pub gold: u32,
    #[serde(default)]
    pub banknotes: u32,
    #[serde(default = "default_food_capacity")]
    pub food: i32,
    #[serde(default = "default_food_capacity")]
    pub max_food: i32,
    #[serde(default = "default_water_capacity")]
    pub water: i32,
    #[serde(default = "default_water_capacity")]
    pub max_water: i32,
    #[serde(default)]
    pub conditions: Vec<ConditionState>,
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub skill_mappings: Vec<SkillMapping>,
    pub inventory: Vec<ItemInstance>,
    #[serde(default)]
    pub equipment: Vec<EquippedItem>,
    #[serde(default, rename = "weapon", skip_serializing)]
    pub(crate) legacy_weapon: Option<LegacyItemKind>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            essence: 100,
            max_essence: 100,
            qi: 80,
            max_qi: 80,
            spirit: 70,
            max_spirit: 70,
            strength: 12,
            perception: 11,
            intelligence: 12,
            spirituality: 20,
            courage: 20,
            constitution: 12,
            composure: 20,
            gender: Gender::Male,
            reputation: 0,
            insight: 0,
            combat_experience: 5_000,
            potential: 120,
            learned_points: 0,
            bellicosity: 0,
            wanted: 0,
            faction: None,
            teacher: None,
            force: 50,
            max_force: 100,
            mana: 0,
            max_mana: 0,
            atman: 0,
            max_atman: 0,
            silver: 24,
            coins: 0,
            gold: 0,
            banknotes: 0,
            food: DEFAULT_FOOD_CAPACITY,
            max_food: DEFAULT_FOOD_CAPACITY,
            water: DEFAULT_WATER_CAPACITY,
            max_water: DEFAULT_WATER_CAPACITY,
            conditions: vec![],
            skills: vec![
                Skill::new(UNARMED_ID, 8),
                Skill::new(SWORD_ID, 3),
                Skill::new(DODGE_ID, 6),
                Skill::new(MOVE_ID, 6),
                Skill::new(FORCE_ID, 5),
                Skill::new(PARRY_ID, 4),
                Skill::new(LIUH_KEN_ID, 8),
                Skill::new(SIX_CHAOS_SWORD_ID, 3),
                Skill::new(PYROBAT_STEPS_ID, 6),
            ],
            skill_mappings: default_skill_mappings(),
            inventory: vec![
                ItemInstance::new(1, ItemId::from(items::CLOTH_ID), 1),
                ItemInstance::new(2, ItemId::from(items::DRY_RATIONS_ID), 3),
            ],
            equipment: vec![EquippedItem {
                slot: EquipmentSlot::Torso,
                instance_id: 1,
            }],
            legacy_weapon: None,
        }
    }
}

fn default_intelligence() -> u32 {
    12
}

fn default_spirituality() -> u32 {
    20
}

fn default_courage() -> u32 {
    20
}

fn default_constitution() -> u32 {
    12
}

fn default_combat_experience() -> u64 {
    5_000
}

fn default_potential() -> u32 {
    120
}

fn default_force() -> i32 {
    50
}

fn default_max_force() -> i32 {
    100
}

fn default_food_capacity() -> i32 {
    DEFAULT_FOOD_CAPACITY
}

fn default_water_capacity() -> i32 {
    DEFAULT_WATER_CAPACITY
}

fn default_skill_mappings() -> Vec<SkillMapping> {
    vec![
        SkillMapping {
            usage: SkillId::from(UNARMED_ID),
            skill: SkillId::from(LIUH_KEN_ID),
        },
        SkillMapping {
            usage: SkillId::from(SWORD_ID),
            skill: SkillId::from(SIX_CHAOS_SWORD_ID),
        },
        SkillMapping {
            usage: SkillId::from(PARRY_ID),
            skill: SkillId::from(SIX_CHAOS_SWORD_ID),
        },
        SkillMapping {
            usage: SkillId::from(DODGE_ID),
            skill: SkillId::from(PYROBAT_STEPS_ID),
        },
        SkillMapping {
            usage: SkillId::from(MOVE_ID),
            skill: SkillId::from(PYROBAT_STEPS_ID),
        },
    ]
}

impl Skill {
    fn new(kind: &str, level: u32) -> Self {
        Self {
            kind: SkillId::from(kind),
            level,
            progress: 0,
        }
    }

    pub fn required_progress(&self) -> u32 {
        (self.level + 1).saturating_pow(2)
    }
}

impl Player {
    pub fn skill(&self, kind: &SkillId) -> &Skill {
        self.skills
            .iter()
            .find(|skill| &skill.kind == kind)
            .expect("requested skill must be learned")
    }

    pub fn skill_by_id(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|skill| skill.kind.as_str() == id)
    }

    pub fn skill_level(&self, id: &str) -> u32 {
        self.skill_by_id(id).map_or(0, |skill| skill.level)
    }

    pub fn mapped_skill(&self, usage: &str) -> Option<&SkillId> {
        self.skill_mappings
            .iter()
            .find(|mapping| mapping.usage.as_str() == usage)
            .map(|mapping| &mapping.skill)
    }

    pub fn effective_skill(&self, usage: &str) -> u32 {
        let basic = self.skill_level(usage) / 2;
        basic
            + self
                .mapped_skill(usage)
                .map_or(0, |mapped| self.skill_level(mapped.as_str()))
    }

    fn skill_mut(&mut self, kind: &SkillId) -> &mut Skill {
        self.skills
            .iter_mut()
            .find(|skill| &skill.kind == kind)
            .expect("requested skill must be learned")
    }

    fn ensure_skill(&mut self, kind: SkillId) {
        if self.skill_by_id(kind.as_str()).is_none() {
            self.skills.push(Skill {
                kind,
                level: 0,
                progress: 0,
            });
        }
    }

    pub fn is_full_health(&self) -> bool {
        self.essence >= self.max_essence && self.qi >= self.max_qi && self.spirit >= self.max_spirit
    }

    pub fn item(&self, instance_id: u64) -> Option<&ItemInstance> {
        self.inventory
            .iter()
            .find(|item| item.instance_id == instance_id)
    }

    fn item_mut(&mut self, instance_id: u64) -> Option<&mut ItemInstance> {
        self.inventory
            .iter_mut()
            .find(|item| item.instance_id == instance_id)
    }

    pub fn equipped(&self, slot: EquipmentSlot) -> Option<&ItemInstance> {
        let instance_id = self
            .equipment
            .iter()
            .find(|item| item.slot == slot)?
            .instance_id;
        self.item(instance_id)
    }

    pub fn is_equipped(&self, instance_id: u64) -> bool {
        self.equipment
            .iter()
            .any(|item| item.instance_id == instance_id)
    }

    pub fn has_item(&self, item_id: &ItemId) -> bool {
        self.inventory.iter().any(|item| &item.item_id == item_id)
    }

    pub fn carried_weight(&self) -> u32 {
        self.inventory.iter().map(ItemInstance::total_weight).sum()
    }

    pub fn carry_capacity(&self) -> u32 {
        self.strength.saturating_mul(5_000)
    }

    pub fn money_value(&self) -> u64 {
        self.coins as u64
            + self.silver as u64 * 100
            + self.gold as u64 * 10_000
            + self.banknotes as u64 * 100_000
    }

    pub fn money_text(&self) -> String {
        format_money(self.money_value())
    }

    fn set_money_value(&mut self, mut value: u64) {
        self.banknotes = (value / 100_000).min(u32::MAX as u64) as u32;
        value %= 100_000;
        self.gold = (value / 10_000) as u32;
        value %= 10_000;
        self.silver = (value / 100) as u32;
        self.coins = (value % 100) as u32;
    }

    fn pay_money(&mut self, amount: u64) -> bool {
        let total = self.money_value();
        if total < amount {
            return false;
        }
        self.set_money_value(total - amount);
        true
    }

    fn add_money(&mut self, amount: u64) {
        self.set_money_value(self.money_value().saturating_add(amount));
    }

    pub fn condition(&self, kind: ConditionKind) -> Option<&ConditionState> {
        self.conditions
            .iter()
            .find(|condition| condition.kind == kind)
    }

    fn set_condition(&mut self, kind: ConditionKind, duration: u32, potency: i32) {
        if duration == 0 {
            self.conditions.retain(|condition| condition.kind != kind);
            return;
        }
        if let Some(condition) = self
            .conditions
            .iter_mut()
            .find(|condition| condition.kind == kind)
        {
            condition.duration = duration;
            condition.potency = potency;
        } else {
            self.conditions.push(ConditionState {
                kind,
                duration,
                potency,
            });
        }
    }

    pub fn conditions_text(&self) -> String {
        if self.conditions.is_empty() {
            return "无".into();
        }
        self.conditions
            .iter()
            .map(|condition| format!("{}({})", condition.kind.name(), condition.duration))
            .collect::<Vec<_>>()
            .join("、")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub version: u32,
    pub player: Player,
    pub location: LocationId,
    pub quest: QuestStage,
    pub activity: Activity,
    pub elapsed_minutes: u64,
    pub logs: Vec<String>,
    #[serde(default)]
    hidden_grass_path_ticks: u8,
    #[serde(default)]
    garden_door_open: bool,
    #[serde(default)]
    manor_door_open: bool,
    #[serde(default)]
    bookshelf_examined: bool,
    #[serde(default)]
    book_puzzle_step: u8,
    #[serde(default)]
    book_puzzle_completed: bool,
    #[serde(default)]
    melon_debt: bool,
    #[serde(default)]
    snow_teacher_paid: bool,
    #[serde(default)]
    snow_guard_revealed: bool,
    #[serde(default)]
    snow_guard_defeated: bool,
    #[serde(default)]
    canyon_secret_clue: bool,
    #[serde(default)]
    canyon_camp_access: bool,
    #[serde(default)]
    canyon_fake_seal_bought: bool,
    #[serde(default)]
    canyon_general_rejected_fake: bool,
    #[serde(default)]
    canyon_general_rewarded: bool,
    #[serde(default)]
    city_inn_access: bool,
    #[serde(default)]
    city_manor_pass: bool,
    #[serde(default)]
    pub ground_items: HashMap<LocationId, Vec<ItemInstance>>,
    #[serde(default)]
    next_item_instance_id: u64,
    rng_state: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            version: SAVE_VERSION,
            player: Player::default(),
            location: LocationId::from(content::LIU_HOME),
            quest: QuestStage::Unasked,
            activity: Activity::Idle,
            elapsed_minutes: 8 * 60,
            logs: vec![
                "山风推开木窗。你在刘家小房醒来，决定独自上路。".into(),
                "使用方向键选择行动，按 Enter 执行。".into(),
            ],
            hidden_grass_path_ticks: 0,
            garden_door_open: false,
            manor_door_open: false,
            bookshelf_examined: false,
            book_puzzle_step: 0,
            book_puzzle_completed: false,
            melon_debt: false,
            snow_teacher_paid: false,
            snow_guard_revealed: false,
            snow_guard_defeated: false,
            canyon_secret_clue: false,
            canyon_camp_access: false,
            canyon_fake_seal_bought: false,
            canyon_general_rejected_fake: false,
            canyon_general_rewarded: false,
            city_inn_access: false,
            city_manor_pass: false,
            ground_items: HashMap::new(),
            next_item_instance_id: 3,
            rng_state: 0x4d59_5df4_d0f3_3173,
        }
    }

    pub fn current_location(&self) -> &'static Location {
        world()
            .location(&self.location)
            .expect("saved location must exist in the embedded world")
    }

    pub fn available_actions(&self) -> Vec<Action> {
        let current = self.current_location();
        if let Activity::Fighting(combat) = &self.activity {
            let mut actions = if combat.technique_cooldown == 0 {
                self.technique_actions(true)
            } else {
                Vec::new()
            };
            if !(current.id.as_str() == content::MELON_FARM && self.melon_debt) {
                for exit in &current.exits {
                    if self.exit_is_available(current, exit) {
                        actions.push(Action::Flee {
                            direction: exit.direction.clone(),
                            target: exit.target.clone(),
                        });
                    }
                }
            }
            if combat.mode == CombatMode::Spar {
                actions.push(Action::Surrender);
            }
            return actions;
        }

        let mut actions = Vec::new();

        if current.id.as_str() == content::MELON_FARM && self.melon_debt {
            return vec![
                Action::Interact(InteractionKind::SettleMelonDebt),
                Action::Fight(EnemyKind::Meloner),
            ];
        }

        for exit in &current.exits {
            if self.exit_is_available(current, exit) {
                actions.push(Action::Move {
                    direction: exit.direction.clone(),
                    target: exit.target.clone(),
                });
            }
        }

        match current.id.as_str() {
            content::LAKESIDE => actions.push(Action::Interact(InteractionKind::PaddleToLake)),
            content::LAKE => {
                actions.push(Action::Interact(InteractionKind::PaddleToShore));
                actions.push(Action::Interact(InteractionKind::DiveIntoLake));
            }
            content::ROAD6 if self.hidden_grass_path_ticks == 0 => {
                actions.push(Action::Interact(InteractionKind::RevealGrassPath));
            }
            content::LIU_HOME | content::GARDEN => {
                actions.push(Action::Interact(self.door_action(DoorKind::LiuGarden)));
            }
            content::LORD_HOUSE1 | content::ROAD9 => {
                actions.push(Action::Interact(self.door_action(DoorKind::LordManor)));
            }
            content::ROAD3 => actions.push(Action::Interact(InteractionKind::InspectTablet)),
            content::LORD_HOUSE3 => {
                actions.push(Action::Interact(InteractionKind::InspectBookshelf));
                if self.bookshelf_examined && !self.book_puzzle_completed {
                    for number in [3, 6, 9, 11] {
                        actions.push(Action::Interact(InteractionKind::PullBook(number)));
                    }
                }
            }
            content::MELON_FARM => {
                actions.push(Action::Interact(InteractionKind::PickMelon));
            }
            content::CANYON_SECRET_WALL if self.canyon_secret_clue => {
                actions.push(Action::Interact(InteractionKind::SwearCanyonSecret));
            }
            content::CANYON_FOOT | content::CANYON_ROAD => {
                actions.push(Action::Interact(InteractionKind::ClimbCanyonChain));
            }
            content::CITY_STREET3 => {
                actions.push(Action::Interact(InteractionKind::ClimbCityWall));
            }
            content::CITY_WALL => {
                actions.push(Action::Interact(InteractionKind::JumpIntoCityManor));
                actions.push(Action::Interact(InteractionKind::JumpOutsideCityWall));
            }
            _ => {}
        }

        for npc in &current.npcs {
            if !self.npc_is_present(npc) {
                continue;
            }
            actions.push(Action::Talk(npc.clone()));
            let definition = npcs()
                .definition(npc)
                .expect("room NPC must exist in the repository");
            for inquiry in definition.inquiries.iter().filter(|inquiry| {
                inquiry.is_runtime_available() || inquiry.scripted_runtime_kind(npc).is_some()
            }) {
                actions.push(Action::AskNpc {
                    npc: npc.clone(),
                    topic: inquiry.topic.clone(),
                });
            }
            for good in &definition.vendor_goods {
                actions.push(Action::BuyItem {
                    item_id: good.item_id.clone(),
                    npc: npc.clone(),
                });
            }
            if let Some(kind) = definition.object_exchange_kind()
                && let Some(amount) = kind.money_offer()
                && self.money_offer_is_available(kind)
            {
                actions.push(Action::OfferMoney {
                    amount,
                    npc: npc.clone(),
                });
            }
        }
        if self.snow_teacher_paid
            && self.npc_is_present(&NpcId::from(SNOW_TEACHER_ID))
            && current
                .npcs
                .iter()
                .any(|npc| npc.as_str() == SNOW_TEACHER_ID)
            && self.player.skill_level("literate") < 60
        {
            actions.push(Action::LearnFromNpc {
                skill: SkillId::from("literate"),
                npc: NpcId::from(SNOW_TEACHER_ID),
            });
        }
        for teacher_id in teachers_at_location(current.id.as_str()) {
            if self.player.teacher.as_deref() == Some(*teacher_id) {
                let teacher = skills()
                    .teacher(teacher_id)
                    .expect("teacher catalog entry exists");
                let mut teachable: Vec<_> = teacher.skills.iter().collect();
                teachable.sort_by(|left, right| left.0.cmp(right.0));
                for (skill_id, master_level) in teachable {
                    if self.player.skill_level(skill_id) < (*master_level).max(0) as u32 {
                        actions.push(Action::LearnSkill {
                            skill: SkillId::from(skill_id.as_str()),
                            teacher: (*teacher_id).to_string(),
                        });
                    }
                }
            } else {
                actions.push(Action::BecomeApprentice((*teacher_id).to_string()));
            }
        }
        if let Some(skill) = &current.training {
            actions.push(Action::Train(skill.clone()));
        }
        if current.can_rest {
            actions.push(Action::Rest);
            actions.extend(self.skill_development_actions());
            if self.player.mapped_skill(FORCE_ID).is_some() {
                actions.push(Action::Cultivate(CultivationKind::Exercise));
            }
            if self.player.skill_level(SPELLS_ID) > 0 {
                actions.push(Action::Cultivate(CultivationKind::Meditate));
            }
            if self.player.skill_level(MAGIC_ID) > 0 {
                actions.push(Action::Cultivate(CultivationKind::Respirate));
            }
        }
        if let Some(enemy) = current.enemy {
            actions.push(Action::Fight(enemy));
            actions.push(Action::Kill(enemy));
        }
        actions.extend(self.technique_actions(false));

        if let Some(ground) = self.ground_items.get(&self.location) {
            for item in ground {
                actions.push(Action::PickUpItem(item.instance_id));
            }
        }
        for item in &self.player.inventory {
            let definition = item.definition();
            if item.has_uses_left()
                && (definition.food_supply.is_some()
                    || definition.category == items::ItemCategory::Liquid)
            {
                actions.push(Action::ConsumeItem(item.instance_id));
            }
            if item.has_uses_left()
                && matches!(
                    item.item_id.as_str(),
                    items::BANDAGE_ID | items::WOUND_MEDICINE_ID | items::SNAKE_MEDICINE_ID
                )
            {
                actions.push(Action::ApplyItem(item.instance_id));
            }
            if definition.study_skill.is_some() {
                actions.push(Action::StudyItem(item.instance_id));
            }
            if definition.equipment_slot().is_some()
                && !item.is_broken()
                && !self.player.is_equipped(item.instance_id)
            {
                actions.push(Action::EquipItem(item.instance_id));
            }
            if !self.player.is_equipped(item.instance_id) {
                if current.npcs.iter().any(|npc| npc.as_str() == TRADER_ID) && item.unit_value() > 0
                {
                    actions.push(Action::SellItem(item.instance_id));
                }
                for npc in &current.npcs {
                    let accepts_gifts = npcs()
                        .definition(npc)
                        .is_some_and(|definition| definition.accepts_runtime_gifts());
                    if self.npc_is_present(npc) && accepts_gifts {
                        actions.push(Action::GiveItem {
                            instance_id: item.instance_id,
                            npc: npc.clone(),
                        });
                    }
                }
                actions.push(Action::DropItem(item.instance_id));
            }
        }
        let powders: Vec<_> = self
            .player
            .inventory
            .iter()
            .filter(|item| {
                matches!(
                    item.item_id.as_str(),
                    items::SLUMBER_DRUG_ID | items::POISON_DUST_ID
                )
            })
            .map(|item| item.instance_id)
            .collect();
        let liquids: Vec<_> = self
            .player
            .inventory
            .iter()
            .filter(|item| {
                item.definition().category == items::ItemCategory::Liquid && item.has_uses_left()
            })
            .map(|item| item.instance_id)
            .collect();
        for powder_instance_id in powders {
            for &liquid_instance_id in &liquids {
                actions.push(Action::MixIntoLiquid {
                    powder_instance_id,
                    liquid_instance_id,
                });
            }
        }
        for equipped in &self.player.equipment {
            actions.push(Action::UnequipItem(equipped.slot));
        }

        actions
    }

    fn npc_is_present(&self, npc: &NpcId) -> bool {
        npc.as_str() != SNOW_GUARD_ID || !self.snow_guard_defeated
    }

    fn money_offer_is_available(&self, kind: ObjectExchangeKind) -> bool {
        match kind {
            ObjectExchangeKind::CanyonAdviser => !self.canyon_secret_clue,
            ObjectExchangeKind::CanyonCaptain => !self.canyon_camp_access,
            ObjectExchangeKind::CanyonSeller => !self.canyon_fake_seal_bought,
            ObjectExchangeKind::CityWaiter => !self.city_inn_access,
            ObjectExchangeKind::CityShangshuGuard => !self.city_manor_pass,
            ObjectExchangeKind::CanyonGeneral
            | ObjectExchangeKind::ScavengerDonation
            | ObjectExchangeKind::TeacherTuition => false,
        }
    }

    fn exit_is_available(&self, current: &Location, exit: &Exit) -> bool {
        let interaction_only = matches!(
            (current.id.as_str(), exit.target.as_str()),
            (content::LAKESIDE, content::LAKE) | (content::LAKE, content::LAKESIDE)
        );
        let dynamic_exit_closed = exit.dynamic
            && !(current.id.as_str() == content::ROAD6 && self.hidden_grass_path_ticks > 0);
        let closed_door = door_for_transition(&current.id, &exit.target)
            .is_some_and(|door| !self.is_door_open(door));
        let access_denied = match (current.id.as_str(), exit.target.as_str()) {
            (content::CANYON_CAMP7, content::CANYON_CAMP8) => !self.canyon_camp_access,
            (content::CITY_INN, content::CITY_INN_UPSTAIRS) => !self.city_inn_access,
            (content::CITY_MANOR_GATE, content::CITY_MANOR_YARD) => !self.city_manor_pass,
            _ => false,
        };
        world().contains(&exit.target)
            && !interaction_only
            && !dynamic_exit_closed
            && !closed_door
            && !access_denied
    }

    fn skill_development_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut practiced = Vec::<SkillId>::new();
        for mapping in &self.player.skill_mappings {
            if self.player.skill_by_id(mapping.skill.as_str()).is_some()
                && !practiced.contains(&mapping.skill)
            {
                practiced.push(mapping.skill.clone());
                actions.push(Action::PracticeSkill(mapping.skill.clone()));
            }
        }

        let mut learned: Vec<_> = self.player.skills.iter().collect();
        learned.sort_by(|left, right| left.kind.cmp(&right.kind));
        for learned_skill in learned {
            let Some(definition) = skills().definition(&learned_skill.kind) else {
                continue;
            };
            for usage in &definition.usages {
                if self.player.skill_level(usage) == 0
                    || usage == learned_skill.kind.as_str()
                    || self.player.mapped_skill(usage) == Some(&learned_skill.kind)
                {
                    continue;
                }
                actions.push(Action::MapSkill {
                    usage: SkillId::from(usage.as_str()),
                    skill: learned_skill.kind.clone(),
                });
            }
        }
        actions
    }

    fn technique_actions(&self, in_combat: bool) -> Vec<Action> {
        TechniqueKind::ALL
            .into_iter()
            .filter(|technique| {
                technique.combat_only() == in_combat
                    || matches!(
                        technique,
                        TechniqueKind::RecoverQi
                            | TechniqueKind::RefreshSpirit
                            | TechniqueKind::RegenerateEssence
                    )
            })
            .filter(|technique| self.technique_mapping_is_active(*technique))
            .map(Action::UseTechnique)
            .collect()
    }

    fn technique_mapping_is_active(&self, technique: TechniqueKind) -> bool {
        if self.player.skill_level(technique.skill_id()) == 0 {
            return false;
        }
        technique.required_usage().is_none_or(|usage| {
            self.player.mapped_skill(usage).map(SkillId::as_str) == Some(technique.skill_id())
        })
    }

    fn is_door_open(&self, door: DoorKind) -> bool {
        match door {
            DoorKind::LiuGarden => self.garden_door_open,
            DoorKind::LordManor => self.manor_door_open,
        }
    }

    fn door_action(&self, door: DoorKind) -> InteractionKind {
        if self.is_door_open(door) {
            InteractionKind::CloseDoor(door)
        } else {
            InteractionKind::OpenDoor(door)
        }
    }

    pub fn perform(&mut self, action: Action) {
        if !self.available_actions().contains(&action) {
            self.push_log("眼下无法执行这个行动。".into());
            return;
        }

        match action {
            Action::Move { target, .. } => self.move_to(target),
            Action::Flee { target, .. } => self.flee_to(target),
            Action::Interact(interaction) => self.interact(interaction),
            Action::Talk(npc) => self.talk(npc),
            Action::AskNpc { npc, topic } => self.ask_npc(npc, &topic),
            Action::BecomeApprentice(teacher) => self.become_apprentice(teacher),
            Action::LearnSkill { skill, teacher } => self.learn_skill(skill, teacher),
            Action::LearnFromNpc { skill, npc } => self.learn_from_npc(skill, npc),
            Action::MapSkill { usage, skill } => self.map_skill(usage, skill),
            Action::Train(skill) => self.toggle_training(skill),
            Action::PracticeSkill(skill) => self.practice_skill(skill),
            Action::StudyItem(instance_id) => self.study_item(instance_id),
            Action::Cultivate(kind) => self.cultivate(kind),
            Action::UseTechnique(technique) => self.use_technique(technique),
            Action::Rest => self.toggle_rest(),
            Action::Fight(enemy) => self.start_combat(enemy, CombatMode::Spar),
            Action::Kill(enemy) => self.start_combat(enemy, CombatMode::Lethal),
            Action::BuyItem { item_id, npc } => self.buy_item(item_id, npc),
            Action::OfferMoney { amount, npc } => self.offer_money(amount, npc),
            Action::SellItem(instance_id) => self.sell_item(instance_id),
            Action::GiveItem { instance_id, npc } => self.give_item_to_npc(instance_id, npc),
            Action::PickUpItem(instance_id) => self.pick_up_item(instance_id),
            Action::DropItem(instance_id) => self.drop_item(instance_id),
            Action::EquipItem(instance_id) => self.equip_item(instance_id),
            Action::UnequipItem(slot) => self.unequip_item(slot),
            Action::ConsumeItem(instance_id) => self.consume_item(instance_id),
            Action::ApplyItem(instance_id) => self.apply_item(instance_id),
            Action::MixIntoLiquid {
                powder_instance_id,
                liquid_instance_id,
            } => self.mix_into_liquid(powder_instance_id, liquid_instance_id),
            Action::Surrender => self.surrender(),
        }
    }

    pub fn tick(&mut self) {
        self.elapsed_minutes += 10;
        self.player.food = self.player.food.saturating_sub(1);
        self.player.water = self.player.water.saturating_sub(1);
        if self.hidden_grass_path_ticks > 0 {
            self.hidden_grass_path_ticks -= 1;
            if self.hidden_grass_path_ticks == 0 && self.location.as_str() == content::ROAD6 {
                self.push_log("茅草重新合拢，西面的隐秘小路消失了。".into());
            }
        }
        match self.activity.clone() {
            Activity::Idle => self.recover(1, 1, 1),
            Activity::Resting => {
                self.recover(6, 5, 5);
                if self.player.is_full_health() {
                    self.activity = Activity::Idle;
                    self.push_log("你已精神饱满，结束了休息。".into());
                }
            }
            Activity::Training(skill) => self.training_tick(skill),
            Activity::Fighting(combat) => self.combat_tick(combat),
        }
        self.update_conditions();
    }

    pub fn time_text(&self) -> String {
        let day = self.elapsed_minutes / (24 * 60) + 1;
        let minutes = self.elapsed_minutes % (24 * 60);
        format!("第{day}日 {:02}:{:02}", minutes / 60, minutes % 60)
    }

    pub fn activity_text(&self) -> String {
        match &self.activity {
            Activity::Idle => "整装待发".into(),
            Activity::Resting => "正在休息".into(),
            Activity::Training(skill) => format!("修炼{}中", skill.name()),
            Activity::Fighting(combat) => match combat.mode {
                CombatMode::Spar => format!("与{}比试", combat.enemy.name()),
                CombatMode::Lethal => format!("与{}死斗", combat.enemy.name()),
            },
        }
    }

    pub fn quest_title(&self) -> &'static str {
        match self.quest {
            QuestStage::Unasked => "山村旧事",
            QuestStage::FindJuan => "寻找娟儿",
            QuestStage::ReturnHome => "平安归来",
            QuestStage::Complete => "山村旧事 · 已完成",
        }
    }

    pub fn quest_objective(&self) -> &'static str {
        match self.quest {
            QuestStage::Unasked => "刘老农似乎有心事。去刘家小房问问他。",
            QuestStage::FindJuan => "娟儿在松林附近失踪。前往松林寻找她。",
            QuestStage::ReturnHome => "娟儿已经脱险。回刘家小房向刘老农报平安。",
            QuestStage::Complete => "刘家父女已经离开山村。你可以继续游历和修炼。",
        }
    }

    pub fn inventory_lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "负重 {:.1}/{:.1} 斤",
            self.player.carried_weight() as f32 / 500.0,
            self.player.carry_capacity() as f32 / 500.0
        )];
        lines.extend(self.player.inventory.iter().map(|item| {
            let definition = item.definition();
            let quantity = if item.quantity > 1 {
                format!(" ×{}", item.quantity)
            } else {
                String::new()
            };
            let equipped = self
                .player
                .equipment
                .iter()
                .find(|equipped| equipped.instance_id == item.instance_id)
                .map_or_else(String::new, |equipped| {
                    format!(" [{}]", equipped.slot.name())
                });
            let durability = item.durability.map_or_else(String::new, |durability| {
                format!(
                    " 耐久 {durability}/{}",
                    definition.max_durability().unwrap_or(durability)
                )
            });
            let uses = item.remaining_uses.map_or_else(String::new, |remaining| {
                if definition.category == items::ItemCategory::Liquid {
                    format!(" 剩余{remaining}口")
                } else if item.item_id.as_str() == items::BANDAGE_ID {
                    format!(" 可用{remaining}次")
                } else {
                    format!(" 剩余{remaining}份")
                }
            });
            format!(
                "• {}{}{}{}{}",
                item.display_name(),
                quantity,
                equipped,
                durability,
                uses
            )
        }));
        lines
    }

    fn inventory_item_name(&self, instance_id: u64) -> String {
        self.player
            .item(instance_id)
            .map(|item| {
                let quantity = if item.quantity > 1 {
                    format!(" ×{}", item.quantity)
                } else {
                    String::new()
                };
                format!("{}{}", item.display_name(), quantity)
            })
            .unwrap_or_else(|| "未知物品".into())
    }

    fn ground_item_name(&self, instance_id: u64) -> String {
        self.ground_items
            .get(&self.location)
            .and_then(|ground| ground.iter().find(|item| item.instance_id == instance_id))
            .map(|item| item.display_name().to_string())
            .unwrap_or_else(|| "未知物品".into())
    }

    fn allocate_item_instance_id(&mut self) -> u64 {
        let inventory_max = self
            .player
            .inventory
            .iter()
            .map(|item| item.instance_id)
            .max()
            .unwrap_or(0);
        let ground_max = self
            .ground_items
            .values()
            .flatten()
            .map(|item| item.instance_id)
            .max()
            .unwrap_or(0);
        self.next_item_instance_id = self
            .next_item_instance_id
            .max(inventory_max.max(ground_max).saturating_add(1))
            .max(1);
        let instance_id = self.next_item_instance_id;
        self.next_item_instance_id = self.next_item_instance_id.saturating_add(1);
        instance_id
    }

    fn add_inventory_item(&mut self, item_id: ItemId, quantity: u32) -> u64 {
        let quantity = quantity.max(1);
        let definition = items()
            .definition(&item_id)
            .expect("gameplay item ID must exist in the item catalog");
        if definition.stackable()
            && let Some(existing) = self
                .player
                .inventory
                .iter_mut()
                .find(|item| item.item_id == item_id)
        {
            existing.quantity = existing.quantity.saturating_add(quantity);
            return existing.instance_id;
        }

        let mut first_id = 0;
        let instances = if definition.stackable() { 1 } else { quantity };
        for index in 0..instances {
            let instance_id = self.allocate_item_instance_id();
            if index == 0 {
                first_id = instance_id;
            }
            self.player.inventory.push(ItemInstance::new(
                instance_id,
                item_id.clone(),
                if definition.stackable() { quantity } else { 1 },
            ));
        }
        first_id
    }

    fn offer_money(&mut self, amount: u64, npc: NpcId) {
        if !self.current_location().npcs.contains(&npc) || !self.npc_is_present(&npc) {
            return;
        }
        let Some(kind) = npcs()
            .definition(&npc)
            .and_then(|definition| definition.object_exchange_kind())
        else {
            return;
        };
        if kind.money_offer() != Some(amount) || !self.money_offer_is_available(kind) {
            return;
        }
        if !self.player.pay_money(amount) {
            self.push_log(format!("你拿不出{}。", format_money(amount)));
            return;
        }

        match kind {
            ObjectExchangeKind::CanyonAdviser => {
                self.canyon_secret_clue = true;
                self.push_log(
                    "军师低声道：到西北面的山壁前立誓，说你爱安尼席洛特尔，暗门自会开启。".into(),
                );
            }
            ObjectExchangeKind::CanyonCaptain => {
                self.canyon_camp_access = true;
                self.push_log("张德成收下礼金，准你进入东面的军营重地。".into());
            }
            ObjectExchangeKind::CanyonSeller => {
                self.canyon_fake_seal_bought = true;
                self.add_inventory_item(ItemId::from("canyon.npc.obj.fake_seal"), 1);
                self.push_log("黑市商人收下三百两银子，递给你一颗将军印鉴。".into());
            }
            ObjectExchangeKind::CityWaiter => {
                self.city_inn_access = true;
                self.push_log("店小二眉开眼笑地收下银子，请你上楼入座。".into());
            }
            ObjectExchangeKind::CityShangshuGuard => {
                self.city_manor_pass = true;
                self.push_log("尚书府护院掂了掂银子，侧身让出东面的院门。".into());
            }
            ObjectExchangeKind::CanyonGeneral
            | ObjectExchangeKind::ScavengerDonation
            | ObjectExchangeKind::TeacherTuition => {}
        }
    }

    fn buy_item(&mut self, item_id: ItemId, npc: NpcId) {
        if !self.current_location().npcs.contains(&npc) {
            return;
        }
        let Some(price) = npcs()
            .definition(&npc)
            .and_then(|seller| seller.price_for(&item_id))
        else {
            self.push_log("这里没有出售这件物品。".into());
            return;
        };
        let definition = items()
            .definition(&item_id)
            .expect("shop item must exist in catalog");
        if self
            .player
            .carried_weight()
            .saturating_add(definition.unit_weight())
            > self.player.carry_capacity()
        {
            self.push_log("你的行囊太重，无法再买下这件物品。".into());
            return;
        }
        if !self.player.pay_money(price) {
            self.push_log(format!(
                "购买{}需要{}，你的钱不够。",
                definition.display_name(),
                format_money(price)
            ));
            return;
        }
        let name = definition.display_name().to_string();
        self.add_inventory_item(item_id, 1);
        self.push_log(format!("你花费{}买下一{name}。", format_money(price)));
    }

    fn sell_item(&mut self, instance_id: u64) {
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == instance_id)
        else {
            return;
        };
        if self.player.is_equipped(instance_id) {
            self.push_log("必须先卸下这件物品。".into());
            return;
        }
        let item = &self.player.inventory[index];
        let definition = item.definition();
        let mut value = item.unit_value().max(0) as u64 * item.quantity as u64;
        if let (Some(durability), Some(max_durability)) =
            (item.durability, definition.max_durability())
        {
            value = value.saturating_mul(durability as u64) / max_durability as u64;
        }
        let price = (value / 2).max(1);
        let name = self.inventory_item_name(instance_id);
        self.player.inventory.remove(index);
        self.player.add_money(price);
        self.push_log(format!("商人以{}收下了{name}。", format_money(price)));
    }

    fn give_item_to_npc(&mut self, instance_id: u64, npc: NpcId) {
        if !self.current_location().npcs.contains(&npc)
            || !self.npc_is_present(&npc)
            || self.player.is_equipped(instance_id)
        {
            return;
        }
        let Some(definition) = npcs().definition(&npc) else {
            return;
        };
        if !definition.accepts_runtime_gifts() {
            return;
        }
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == instance_id)
        else {
            return;
        };
        let item = &self.player.inventory[index];
        let name = item.display_name().to_string();
        let value = item.unit_value().max(0) as u64 * item.quantity as u64;

        match definition.object_exchange_kind() {
            Some(ObjectExchangeKind::TeacherTuition) if !self.snow_teacher_paid && value < 500 => {
                self.push_log("魏无极说道：你的诚意不够，这东西还是拿回去吧。".into());
            }
            Some(ObjectExchangeKind::TeacherTuition) => {
                self.player.inventory.remove(index);
                if self.snow_teacher_paid {
                    self.push_log("魏无极收下礼物，勉励你继续用功。".into());
                } else {
                    self.snow_teacher_paid = true;
                    self.push_log(
                        "魏无极点头收下学费：从今天起，你随时可以来请教读书识字。".into(),
                    );
                }
            }
            Some(ObjectExchangeKind::ScavengerDonation) => {
                self.player.inventory.remove(index);
                self.push_log(format!("收破烂的笑着收下{name}，连声道谢。"));
            }
            Some(ObjectExchangeKind::CanyonAdviser) => {
                self.player.inventory.remove(index);
                if self.canyon_secret_clue {
                    self.push_log("军师收下礼物，提醒你别忘了山壁前的誓言。".into());
                } else if value >= 800 {
                    self.canyon_secret_clue = true;
                    self.push_log(
                        "军师低声道：到西北面的山壁前立誓，说你爱安尼席洛特尔，暗门自会开启。"
                            .into(),
                    );
                } else {
                    self.push_log("军师收起礼物，冷冷道：这点诚意还不够换一句口令。".into());
                }
            }
            Some(ObjectExchangeKind::CanyonCaptain) => {
                self.player.inventory.remove(index);
                if self.canyon_camp_access {
                    self.push_log("张德成收下礼物，仍准你在军营内通行。".into());
                } else if value >= 3_000 {
                    self.canyon_camp_access = true;
                    self.push_log("张德成收下礼物，准你进入东面的军营重地。".into());
                } else {
                    self.push_log("张德成没收礼物，喝道：军营重地岂容你随便进出！".into());
                }
            }
            Some(ObjectExchangeKind::CanyonGeneral)
                if item.item_id.as_str() == "canyon.npc.obj.fake_seal" =>
            {
                self.canyon_general_rejected_fake = true;
                self.push_log("镇国大将军识破假印鉴，将东西掷还给你，命人把你赶出军营。".into());
                self.move_to(LocationId::from(content::CANYON_CAMP2));
            }
            Some(ObjectExchangeKind::CanyonGeneral)
                if item.item_id.as_str() == "canyon.npc.obj.seal" =>
            {
                self.player.inventory.remove(index);
                if self.canyon_general_rewarded {
                    self.push_log("镇国大将军收回印鉴，向你点了点头。".into());
                } else {
                    self.canyon_general_rewarded = true;
                    self.add_inventory_item(ItemId::from("canyon.npc.obj.old_sword"), 1);
                    self.push_log("镇国大将军验明真印鉴，将一柄可供研习的古剑交给你。".into());
                }
            }
            Some(ObjectExchangeKind::CanyonGeneral) => {
                self.player.inventory.remove(index);
                self.push_log(format!("镇国大将军收下{name}，向你道谢。"));
            }
            Some(ObjectExchangeKind::CanyonSeller)
                if item.item_id.as_str() == "canyon.npc.obj.fake_seal"
                    && self.canyon_general_rejected_fake =>
            {
                self.player.inventory.remove(index);
                self.canyon_general_rejected_fake = false;
                self.add_inventory_item(ItemId::from("canyon.npc.obj.seal"), 1);
                self.push_log("黑市商人收回假印鉴，悄悄替你换了一颗泛着黄光的真印鉴。".into());
            }
            Some(ObjectExchangeKind::CanyonSeller) => {
                self.push_log("黑市商人摆摆手，不肯收下这件东西。".into());
            }
            Some(ObjectExchangeKind::CityWaiter) => {
                let item = self.player.inventory.remove(index);
                if self.city_inn_access {
                    self.push_log(format!("店小二收下{name}，请你自便。"));
                } else if value >= 1_000 {
                    self.city_inn_access = true;
                    self.push_log("店小二眉开眼笑地收下礼物，请你上楼入座。".into());
                } else {
                    self.merge_ground_item(item);
                    self.push_log(format!("店小二收下{name}又随手丢在地上，仍不肯让你上楼。"));
                }
            }
            Some(ObjectExchangeKind::CityShangshuGuard) => {
                self.player.inventory.remove(index);
                if value >= 30_000 {
                    self.city_manor_pass = true;
                    self.push_log("尚书府护院收下厚礼，侧身让出东面的院门。".into());
                } else {
                    self.push_log("尚书府护院收下礼物，却仍挡在院门前。".into());
                }
            }
            None if definition.source_path == "adapted" => {
                self.player.inventory.remove(index);
                if value >= 100 {
                    self.player.reputation += 1;
                    self.push_log(format!("你把{name}交给{}。评价 +1。", npc.name()));
                } else {
                    self.push_log(format!("你把{name}交给{}。", npc.name()));
                }
            }
            None => {}
        }
    }

    fn place_item_on_ground(&mut self, item_id: ItemId, quantity: u32) {
        let instance_id = self.allocate_item_instance_id();
        let item = ItemInstance::new(instance_id, item_id, quantity);
        self.merge_ground_item(item);
    }

    fn merge_ground_item(&mut self, item: ItemInstance) {
        let ground = self.ground_items.entry(self.location.clone()).or_default();
        if item.definition().stackable()
            && let Some(existing) = ground
                .iter_mut()
                .find(|existing| existing.item_id == item.item_id)
        {
            existing.quantity = existing.quantity.saturating_add(item.quantity);
        } else {
            ground.push(item);
        }
    }

    fn pick_up_item(&mut self, instance_id: u64) {
        let Some(item) = self
            .ground_items
            .get(&self.location)
            .and_then(|ground| ground.iter().find(|item| item.instance_id == instance_id))
            .cloned()
        else {
            self.push_log("那件物品已经不在这里了。".into());
            return;
        };
        if self
            .player
            .carried_weight()
            .saturating_add(item.total_weight())
            > self.player.carry_capacity()
        {
            self.push_log(format!("{}太重了，你目前无法拿起。", item.display_name()));
            return;
        }

        let name = item.display_name().to_string();
        let ground = self
            .ground_items
            .get_mut(&self.location)
            .expect("ground item location exists");
        let index = ground
            .iter()
            .position(|candidate| candidate.instance_id == instance_id)
            .expect("ground item instance exists");
        let item = ground.remove(index);
        if item.definition().stackable()
            && let Some(existing) = self
                .player
                .inventory
                .iter_mut()
                .find(|existing| existing.item_id == item.item_id)
        {
            existing.quantity = existing.quantity.saturating_add(item.quantity);
        } else {
            self.player.inventory.push(item);
        }
        if ground.is_empty() {
            self.ground_items.remove(&self.location);
        }
        self.push_log(format!("你拾起了{name}。"));
    }

    fn drop_item(&mut self, instance_id: u64) {
        if self.player.is_equipped(instance_id) {
            self.push_log("必须先卸下这件物品。".into());
            return;
        }
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == instance_id)
        else {
            self.push_log("行囊中没有这件物品。".into());
            return;
        };
        let item = self.player.inventory.remove(index);
        let name = item.display_name().to_string();
        self.merge_ground_item(item);
        self.push_log(format!("你把{name}放在了地上。"));
    }

    fn equip_item(&mut self, instance_id: u64) {
        let Some(item) = self.player.item(instance_id) else {
            self.push_log("行囊中没有这件物品。".into());
            return;
        };
        let Some(slot) = item.definition().equipment_slot() else {
            self.push_log("这件物品不能装备。".into());
            return;
        };
        if item.is_broken() {
            self.push_log("这件物品已经损坏，无法装备。".into());
            return;
        }
        let name = item.display_name().to_string();
        self.player
            .equipment
            .retain(|equipped| equipped.slot != slot && equipped.instance_id != instance_id);
        self.player
            .equipment
            .push(EquippedItem { slot, instance_id });
        self.push_log(format!("你装备了{name}。"));
    }

    fn unequip_item(&mut self, slot: EquipmentSlot) {
        let Some(index) = self
            .player
            .equipment
            .iter()
            .position(|equipped| equipped.slot == slot)
        else {
            return;
        };
        let equipped = self.player.equipment.remove(index);
        let name = self.inventory_item_name(equipped.instance_id);
        self.push_log(format!("你卸下了{name}。"));
    }

    fn remove_one_item(&mut self, item_id: &ItemId) -> bool {
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| &item.item_id == item_id)
        else {
            return false;
        };
        if self.player.inventory[index].quantity > 1 {
            self.player.inventory[index].quantity -= 1;
        } else {
            let instance_id = self.player.inventory[index].instance_id;
            self.player
                .equipment
                .retain(|equipped| equipped.instance_id != instance_id);
            self.player.inventory.remove(index);
        }
        true
    }

    fn consume_item(&mut self, instance_id: u64) {
        let Some(item) = self.player.item(instance_id) else {
            return;
        };
        let definition = item.definition();
        let name = item.display_name().to_string();
        let food_supply = definition.food_supply.unwrap_or(0).max(0);
        let water_supply = definition.water_supply.unwrap_or(0).max(0);
        let is_liquid = definition.category == items::ItemCategory::Liquid;
        let is_alcohol = definition.liquid_type.as_deref() == Some("alcohol");
        let drunk_apply = definition.drunk_apply.unwrap_or(0).max(0) as u32;
        let slumber_effect = item.slumber_effect;
        let final_food_use = !is_liquid
            && item.quantity == 1
            && item.remaining_uses.is_some_and(|remaining| remaining <= 1);
        let residual = final_food_use
            .then(|| consumed_residual(item.item_id.as_str()))
            .flatten();

        if is_liquid {
            if self.player.water >= self.player.max_water {
                self.push_log("你已经喝得太多，再也灌不下一口。".into());
                return;
            }
            self.player.water = (self.player.water + 30).min(self.player.max_water);
            if is_alcohol && drunk_apply > 0 {
                let current = self
                    .player
                    .condition(ConditionKind::Drunk)
                    .map_or(0, |condition| condition.duration);
                self.player.set_condition(
                    ConditionKind::Drunk,
                    current.saturating_add(drunk_apply),
                    0,
                );
            }
            if slumber_effect > 0 {
                let current = self
                    .player
                    .condition(ConditionKind::Slumber)
                    .map_or(0, |condition| condition.duration);
                self.player.set_condition(
                    ConditionKind::Slumber,
                    current.saturating_add(slumber_effect),
                    0,
                );
            }
            self.spend_item_use(instance_id, true);
            let liquid = definition.liquid_name.as_deref().unwrap_or("饮水");
            self.push_log(format!("你从{name}中喝了几口{liquid}。"));
            return;
        }

        if self.player.food >= self.player.max_food {
            self.push_log("你已经吃得太饱，再也塞不下东西。".into());
            return;
        }
        if water_supply > 0 && self.player.water >= self.player.max_water {
            self.push_log("你肚中水气太满，暂时吃不下这份食物。".into());
            return;
        }
        self.player.food = (self.player.food + food_supply).min(self.player.max_food);
        if water_supply > 0 {
            self.player.water = (self.player.water + water_supply).min(self.player.max_water);
        }
        if let Some((residual_name, residual_weight)) = residual {
            self.spend_item_use(instance_id, true);
            if let Some(item) = self.player.item_mut(instance_id) {
                item.transformed_name = Some(residual_name.into());
                item.transformed_weight = Some(residual_weight);
                item.transformed_value = Some(0);
            }
            self.push_log(format!("你吃完了{name}，只剩下{residual_name}。"));
        } else {
            self.spend_item_use(instance_id, false);
            self.push_log(format!("你吃了几口{name}。"));
        }
    }

    fn mix_into_liquid(&mut self, powder_instance_id: u64, liquid_instance_id: u64) {
        let Some(powder) = self.player.item(powder_instance_id) else {
            return;
        };
        if !matches!(
            powder.item_id.as_str(),
            items::SLUMBER_DRUG_ID | items::POISON_DUST_ID
        ) {
            return;
        }
        let powder_name = powder.display_name().to_string();
        let Some(liquid) = self.player.item(liquid_instance_id) else {
            return;
        };
        if liquid.definition().category != items::ItemCategory::Liquid || !liquid.has_uses_left() {
            self.push_log("容器里没有可供溶解药粉的酒水。".into());
            return;
        }
        let liquid_name = liquid.display_name().to_string();
        if let Some(liquid) = self.player.item_mut(liquid_instance_id) {
            liquid.slumber_effect = liquid.slumber_effect.saturating_add(100);
        }
        self.spend_item_use(powder_instance_id, false);
        self.push_log(format!("你把{powder_name}倒进{liquid_name}，摇匀了药粉。"));
    }

    fn apply_item(&mut self, instance_id: u64) {
        let Some(item) = self.player.item(instance_id) else {
            return;
        };
        let item_id = item.item_id.clone();
        let name = item.display_name().to_string();
        match item_id.as_str() {
            items::BANDAGE_ID => {
                if self.player.essence >= self.player.max_essence {
                    self.push_log("你没有需要包扎的外伤。".into());
                    return;
                }
                self.player.set_condition(ConditionKind::Bandaged, 40, 3);
                self.spend_item_use(instance_id, true);
                self.push_log(format!("你用{name}裹好伤口，伤势开始缓慢恢复。"));
            }
            items::WOUND_MEDICINE_ID => {
                if self.player.essence >= self.player.max_essence {
                    self.push_log("你没有需要敷药的外伤。".into());
                    return;
                }
                self.player.essence = (self.player.essence + 20).min(self.player.max_essence);
                self.spend_item_use(instance_id, false);
                self.push_log(format!("你敷上{name}，精恢复 20。"));
            }
            items::SNAKE_MEDICINE_ID => {
                let Some(duration) = self
                    .player
                    .condition(ConditionKind::SnakePoison)
                    .map(|condition| condition.duration)
                else {
                    self.push_log("你并没有中蛇毒。".into());
                    return;
                };
                self.player.set_condition(
                    ConditionKind::SnakePoison,
                    duration.saturating_sub(1),
                    10,
                );
                self.spend_item_use(instance_id, false);
                if duration > 1 {
                    self.push_log(format!("你服下{name}，但蛇毒尚未完全清除。"));
                } else {
                    self.push_log(format!("你服下{name}，终于清除了体内蛇毒。"));
                }
            }
            _ => self.push_log("这件物品没有可用的药效。".into()),
        }
    }

    fn spend_item_use(&mut self, instance_id: u64, keep_when_empty: bool) {
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == instance_id)
        else {
            return;
        };
        let item = &mut self.player.inventory[index];
        if item.remaining_uses.is_none() && item.quantity > 1 {
            item.quantity -= 1;
            return;
        }
        if let Some(remaining) = item.remaining_uses.as_mut() {
            *remaining = remaining.saturating_sub(1);
            if *remaining > 0 || keep_when_empty {
                return;
            }
        }
        self.player
            .equipment
            .retain(|equipped| equipped.instance_id != instance_id);
        self.player.inventory.remove(index);
    }

    fn move_to(&mut self, target: LocationId) {
        self.activity = Activity::Idle;
        if self.location.as_str() == content::CITY_MANOR_GATE {
            self.city_manor_pass = false;
        }
        self.location = target;
        let place = self.current_location();
        self.push_log(format!("你来到{}。{}", place.name, place.arrival));
    }

    fn flee_to(&mut self, target: LocationId) {
        let Activity::Fighting(combat) = self.activity.clone() else {
            return;
        };
        let loss = if combat.mode == CombatMode::Lethal {
            10
        } else {
            5
        };
        let actual_loss = self.player.reputation.max(0).min(loss);
        self.player.reputation -= actual_loss;
        self.move_to(target);
        self.push_log(format!(
            "你脱离{}的追击，临阵退却使评价降低{}点。",
            combat.enemy.name(),
            actual_loss
        ));
    }

    fn interact(&mut self, interaction: InteractionKind) {
        self.activity = Activity::Idle;
        match interaction {
            InteractionKind::PaddleToLake => {
                self.location = LocationId::from(content::LAKE);
                self.push_log("你解开一艘木船，摇桨离岸，片刻后到了玉螺湖心。".into());
            }
            InteractionKind::PaddleToShore => {
                self.location = LocationId::from(content::LAKESIDE);
                self.push_log("你调转木船向东划去，重新靠上玉螺湖畔。".into());
            }
            InteractionKind::DiveIntoLake => {
                self.location = LocationId::from(content::LAKE_BOTTOM);
                self.push_log(
                    "你深吸一口气潜入湖中，循着白光游进水下岩洞；入口随即结上薄冰。".into(),
                );
            }
            InteractionKind::RevealGrassPath => {
                self.hidden_grass_path_ticks = 5;
                self.push_log("你用力拨开茅草，西面露出一条通往山谷的隐秘小路。".into());
            }
            InteractionKind::OpenDoor(door) => self.set_door_open(door, true),
            InteractionKind::CloseDoor(door) => self.set_door_open(door, false),
            InteractionKind::InspectTablet => {
                self.push_log("你擦去路牌上的尘土，勉强辨出两个模糊的字：傅家坡。".into());
            }
            InteractionKind::InspectBookshelf => {
                self.bookshelf_examined = true;
                if self.book_puzzle_completed {
                    self.push_log("十二本石书已经归位，第六本后面的残缺机关不再响应。".into());
                } else {
                    self.push_log(
                        "书架上共有十二本石书；第九、三、十一和六本的边缘有明显磨痕。".into(),
                    );
                }
            }
            InteractionKind::PullBook(number) => self.pull_book(number),
            InteractionKind::PickMelon => self.pick_melon(),
            InteractionKind::SettleMelonDebt => self.settle_melon_debt(),
            InteractionKind::SwearCanyonSecret => {
                self.canyon_secret_clue = false;
                self.move_to(LocationId::from(content::CANYON_BLACK_MARKET));
                self.push_log("山壁无声滑开一道暗门，你依照军师的口令进入黑市。".into());
            }
            InteractionKind::ClimbCanyonChain => self.climb_canyon_chain(),
            InteractionKind::ClimbCityWall => {
                self.move_to(LocationId::from(content::CITY_WALL));
                self.push_log("你借墙砖缝隙攀上尚书府院墙。".into());
            }
            InteractionKind::JumpIntoCityManor => {
                self.move_to(LocationId::from(content::CITY_MANOR_RUIN));
                self.push_log("你看准院内无人，纵身跳进尚书府的废屋。".into());
            }
            InteractionKind::JumpOutsideCityWall => {
                self.move_to(LocationId::from(content::CITY_STREET3));
                self.push_log("你翻下外墙，落回京师东街。".into());
            }
        }
    }

    fn climb_canyon_chain(&mut self) {
        let climbing_up = self.location.as_str() == content::CANYON_FOOT;
        let (essence_cost, qi_cost, spirit_cost, target) = if climbing_up {
            (30, 40, 20, content::CANYON_ROAD)
        } else {
            (20, 30, 10, content::CANYON_FOOT)
        };
        let exhausted = self.player.essence < essence_cost
            || self.player.qi < qi_cost
            || self.player.spirit < spirit_cost;
        self.player.essence = (self.player.essence - essence_cost).max(0);
        self.player.qi = (self.player.qi - qi_cost).max(0);
        self.player.spirit = (self.player.spirit - spirit_cost).max(0);
        self.move_to(LocationId::from(target));
        if exhausted {
            self.push_log("你在铁索中途体力不支，勉强挪到另一端，已经精疲力竭。".into());
        } else if climbing_up {
            self.push_log("你沿铁索攀上山壁，踏上通往雪亭镇的碎石小路。".into());
        } else {
            self.push_log("你沿铁索稳稳下到黄石隘口。".into());
        }
    }

    fn set_door_open(&mut self, door: DoorKind, open: bool) {
        match door {
            DoorKind::LiuGarden => self.garden_door_open = open,
            DoorKind::LordManor => self.manor_door_open = open,
        }
        let action = if open { "打开" } else { "关上" };
        self.push_log(format!("你{action}了{}。", door.name()));
    }

    fn pull_book(&mut self, number: u8) {
        const SEQUENCE: [u8; 4] = [9, 3, 11, 6];
        if self.book_puzzle_step as usize >= SEQUENCE.len() {
            self.book_puzzle_step = 0;
        }
        let expected = SEQUENCE[self.book_puzzle_step as usize];
        if number == expected {
            self.book_puzzle_step += 1;
            if self.book_puzzle_step as usize == SEQUENCE.len() {
                self.book_puzzle_step = 0;
                self.book_puzzle_completed = true;
                self.player.insight += 3;
                self.push_log(
                    "第六本石书后传来一声空响，机关却没有连接任何暗门。原作中的这套机关也止于此处。领悟 +3。"
                        .into(),
                );
            } else {
                self.push_log(format!(
                    "第{number}本石书向外滑出半寸，书架深处传来一声轻响。"
                ));
            }
        } else {
            self.book_puzzle_step = u8::from(number == SEQUENCE[0]);
            self.push_log(format!(
                "第{number}本石书纹丝不动，先前触发的机括也复位了。"
            ));
        }
    }

    fn pick_melon(&mut self) {
        let perception = self.player.perception;
        let found = perception >= 30 || self.random(perception.max(1)) > self.random(30);
        if !found {
            self.push_log("你在瓜地里找了半天，仍没找到熟透的西瓜。".into());
            return;
        }

        self.add_inventory_item(ItemId::from(items::WATER_MELON_ID), 1);
        self.push_log("你找到一个熟透的大西瓜，刚把它抱进怀中。".into());
        let noticed = perception >= 30 || self.random(perception.max(1)) > self.random(10);
        if noticed {
            self.melon_debt = true;
            let enemy = EnemyKind::Meloner;
            self.activity = Activity::Fighting(CombatState {
                enemy,
                health: enemy.max_health(),
                max_health: enemy.max_health(),
                rounds: 0,
                mode: CombatMode::Spar,
                attack_bonus: 0,
                dodge_bonus: 0,
                enemy_busy_rounds: 0,
                technique_cooldown: 0,
                power_up_active: false,
                fake_fault_active: false,
            });
            self.push_log("瓜农发现你摘瓜，冲进瓜地拦住去路：还我瓜钱，否则别想走！".into());
        } else {
            self.push_log("瓜农似乎正在瓜棚里打瞌睡，没有发现你的动作。".into());
        }
    }

    fn settle_melon_debt(&mut self) {
        const MELON_PRICE: u64 = 60;
        if self.player.pay_money(MELON_PRICE) {
            self.melon_debt = false;
            self.push_log("你付了 60 文瓜钱。瓜农收钱后让开道路，返回了瓜棚。".into());
        } else if self.remove_one_item(&ItemId::from(items::WATER_MELON_ID)) {
            self.melon_debt = false;
            self.push_log("你拿不出瓜钱，只得归还西瓜。瓜农这才让开道路。".into());
        } else {
            self.push_log("你既没有足够碎银，也拿不出西瓜，瓜农仍挡着去路。".into());
        }
    }

    fn talk(&mut self, npc: NpcId) {
        self.activity = Activity::Idle;
        match npc.as_str() {
            OLD_LIU_ID => match self.quest {
                QuestStage::Unasked => {
                    self.quest = QuestStage::FindJuan;
                    self.push_log(
                        "刘老农：小女娟儿进了南边松林，至今未归，少侠可否帮我寻她？".into(),
                    );
                    self.push_log("获得线索：前往松林寻找娟儿。".into());
                }
                QuestStage::FindJuan => {
                    self.push_log("刘老农：松林野兽不少，还常有山贼出没，千万小心。".into());
                }
                QuestStage::ReturnHome => {
                    self.quest = QuestStage::Complete;
                    self.player.reputation += 20;
                    self.player.insight += 30;
                    let sword = self.add_inventory_item(ItemId::from(items::HENGBING_SWORD_ID), 1);
                    self.add_inventory_item(ItemId::from(items::PARRY_MANUAL_ID), 1);
                    self.equip_item(sword);
                    self.gain_skill_progress(SkillId::from(PARRY_ID), 40);
                    self.push_log(
                        "刘老农：多谢搭救小女。这口銮鱼衡冰与过招要旨便赠予少侠。".into(),
                    );
                    self.push_log("任务完成：评价 +20，领悟 +30，已装备銮鱼衡冰。".into());
                }
                QuestStage::Complete => {
                    self.push_log("空屋桌上压着一张字条：救命之恩，刘某没齿难忘。".into());
                }
            },
            TEA_SELLER_ID => {
                self.recover(8, 8, 5);
                if self.quest == QuestStage::FindJuan {
                    self.push_log(
                        "茶摊老板：刚才有几个凶汉往松林去了，像是还押着一位姑娘。".into(),
                    );
                } else {
                    self.push_log("茶摊老板斟上一碗粗茶。你听了些山南海北的见闻。".into());
                    self.player.insight += 2;
                }
            }
            TEMPLE_MASTER_ID => {
                let progress = 8 + self.player.perception / 3;
                self.gain_skill_progress(SkillId::from(FORCE_ID), progress);
                self.push_log("玄智和尚点出你吐纳中的滞涩之处，内息运转顺畅了许多。".into());
            }
            FISHER_ID => {
                self.push_log("渔夫压低声音：湖里近来常有白光，已经没人敢下水捕鱼了。".into());
            }
            FLOWER_GIRL_ID => {
                self.push_log("采花妞：要找小娟的话，得去黑松山那边问问。".into());
            }
            FARM_WOMAN_ID => {
                self.push_log("农妇叹道：我那七岁的孩子也不见了，听说黑松山常抓小孩子。".into());
            }
            MELONER_ID => {
                self.push_log("瓜农警惕地看着你：想吃瓜就去镇上买，可别打瓜田的主意。".into());
            }
            TRADER_ID => {
                self.push_log("关外商人拱手道：北边道路不太平，带足干粮再上路。".into());
            }
            _ => {
                let definition = npcs()
                    .definition(&npc)
                    .expect("current NPC must exist in the repository");
                if definition.vendor_goods.is_empty() {
                    self.push_log(format!("{}向你点头致意。", definition.name));
                } else {
                    self.push_log(format!("{}招呼你查看店里的货品。", definition.name));
                }
            }
        }
    }

    fn ask_npc(&mut self, npc: NpcId, topic: &str) {
        if !self.current_location().npcs.contains(&npc) || !self.npc_is_present(&npc) {
            return;
        }
        let Some(definition) = npcs().definition(&npc) else {
            return;
        };
        let Some(inquiry) = definition
            .inquiries
            .iter()
            .find(|inquiry| inquiry.topic == topic)
        else {
            return;
        };
        let scripted_kind = inquiry.scripted_runtime_kind(&npc);
        if !inquiry.is_runtime_available() && scripted_kind.is_none() {
            return;
        }

        self.activity = Activity::Idle;
        if let Some(response) = inquiry.response.as_deref() {
            self.push_log(format!("{}说道：{response}", definition.name));
            return;
        }

        match scripted_kind.expect("available scripted inquiry must have a runtime handler") {
            ScriptedInquiryKind::VendorList => {
                let goods = definition
                    .vendor_goods
                    .iter()
                    .map(|good| {
                        let item = items()
                            .definition(&good.item_id)
                            .expect("vendor item must exist");
                        let price = definition
                            .price_for(&good.item_id)
                            .expect("vendor item must have a price");
                        format!("{} {}", item.display_name(), format_money(price))
                    })
                    .collect::<Vec<_>>()
                    .join("、");
                self.push_log(format!("{}摆出货单：{goods}。", definition.name));
            }
            ScriptedInquiryKind::CanyonHistory => {
                for line in [
                    "二十多年前，先帝将大将军们调来这里驻守隘口。",
                    "京城也因隘口得到巩固而安全下来。",
                    "事隔多年，先帝已逝，换上的是一位我们不甚认识的小皇帝。",
                    "看来他只忙着找剑，也没有什么大作为，大家不太服他。",
                    "皇帝见到大将军也要敬畏三分。",
                    "总之，将军和军师都很厉害，你可要好好熟悉这里。",
                    "知道了，就快缴钱。",
                ] {
                    self.push_log(format!("大队长说道：{line}"));
                }
            }
            ScriptedInquiryKind::HerbalistAdvice => {
                let ratio = self.player.qi.max(0) * 100 / self.player.max_qi.max(1);
                if ratio >= 100 {
                    self.push_log(
                        "杨掌柜说道：这位少侠，您看起来气色很好，不像有受伤的样子。".into(),
                    );
                } else if ratio >= 95 {
                    self.push_log(
                        "杨掌柜说道：哦……我看看……只是些皮肉小伤，您买包金疮药回去敷敷就没事了。"
                            .into(),
                    );
                } else {
                    self.push_log("杨掌柜替你把脉良久，却没有开出药方。".into());
                }
            }
            ScriptedInquiryKind::TeacherTuition => {
                for line in [
                    "读书识字是做人的第一步。",
                    "学好了读书识字的本领，胜过百万家财。",
                    "学问虽是金钱买不到的，你若有心，只要五两银子的学费。",
                ] {
                    self.push_log(format!("魏无极说道：{line}"));
                }
            }
            ScriptedInquiryKind::SnowGuardReveal => self.reveal_snow_guard(),
        }
    }

    fn reveal_snow_guard(&mut self) {
        if self.player.combat_experience < 20_000 {
            self.push_log("刘安禄移开目光，不肯回答这个问题。".into());
            return;
        }
        if !self.snow_guard_revealed && self.random(10) < 5 {
            self.push_log("刘安禄迟疑片刻，说道：我……不知道！".into());
            return;
        }

        if self.snow_guard_revealed {
            self.push_log("刘安禄喝道：你既然知道了我的身份，今日休想活命！".into());
        } else {
            self.snow_guard_revealed = true;
            self.push_log(
                "刘安禄眼中突然放出异样光芒，承认自己正是野羊山寨二寨主血手刘三。".into(),
            );
            self.push_log("他使开单刀，招数沉猛狠辣，主动向你攻来。".into());
        }
        self.begin_combat(EnemyKind::BloodHandLiuSan, CombatMode::Lethal);
    }

    fn become_apprentice(&mut self, teacher_id: String) {
        let teacher = skills()
            .teacher(&teacher_id)
            .expect("available teacher must exist");
        if let Some(reason) = self.apprenticeship_rejection(&teacher_id) {
            self.push_log(format!("{}摇头道：{reason}", teacher.name));
            return;
        }

        self.player.teacher = Some(teacher_id);
        self.player.faction = teacher.faction.clone();
        if teacher.id == "fighter" {
            self.push_log("你立誓恪守天邪派门规，萧辟尘这才点头收你入门。".into());
        } else {
            self.push_log(format!(
                "{}将你收入{}门下。",
                teacher.name,
                teacher.faction.as_deref().unwrap_or("师门")
            ));
        }
    }

    fn apprenticeship_rejection(&self, teacher_id: &str) -> Option<&'static str> {
        match teacher_id {
            "assassin" | "beggar" | "ronin" if self.player.money_value() >= 100 => {
                Some("本门只收身无余财之人，你还是回去享福吧。")
            }
            "juechen" if self.player.spirituality < 24 => {
                Some("入我派者需有慧根，你的资质尚且不宜。")
            }
            "juechen" if self.player.combat_experience < 100_000 => {
                Some("你尚缺江湖历练，不宜投入绝尘门下。")
            }
            "ninja" if self.player.perception < 25 => {
                Some("入我派者需人品文采俱佳，你的资质尚且不宜。")
            }
            "ninja" if self.player.skill_level("literate") < 50 => {
                Some("你的文学修养尚不足以入门。")
            }
            "scholar" => Some("你还需先走一趟东边桃林，方可谈入门之事。"),
            "swordsman" if self.player.courage < 20 || self.player.composure < 20 => {
                Some("学剑之人必须胆大心细，你的心性尚需磨炼。")
            }
            _ => None,
        }
    }

    fn learn_skill(&mut self, skill_id: SkillId, teacher_id: String) {
        if self.player.teacher.as_deref() != Some(teacher_id.as_str()) {
            self.push_log("对方并不是你的师父，不愿传授本门绝学。".into());
            return;
        }
        let teacher = skills()
            .teacher(&teacher_id)
            .expect("available teacher must exist");
        let Some(master_level) = teacher.skills.get(skill_id.as_str()).copied() else {
            self.push_log("这项技能必须另寻高人请教。".into());
            return;
        };
        self.learn_skill_from_teacher(
            skill_id,
            &teacher.name,
            teacher.intelligence.unwrap_or(30).max(1),
            master_level.max(0) as u32,
        );
    }

    fn learn_from_npc(&mut self, skill_id: SkillId, npc: NpcId) {
        if npc.as_str() != SNOW_TEACHER_ID
            || skill_id.as_str() != "literate"
            || !self.current_location().npcs.contains(&npc)
            || !self.snow_teacher_paid
        {
            self.push_log("魏无极说道：我不记得收过你这个学生。".into());
            return;
        }
        self.learn_skill_from_teacher(skill_id, "魏无极", 26, 60);
    }

    fn learn_skill_from_teacher(
        &mut self,
        skill_id: SkillId,
        teacher_name: &str,
        teacher_intelligence: i32,
        master_level: u32,
    ) {
        let current_level = self.player.skill_level(skill_id.as_str());
        if current_level >= master_level {
            self.push_log(format!("你的{}造诣已经不输师父。", skill_id.name()));
            return;
        }
        let definition = skills()
            .definition(&skill_id)
            .expect("teacher skill must exist in catalog");
        if let Err(reason) = self.validate_skill_requirements(definition) {
            self.push_log(reason);
            return;
        }
        if self.player.learned_points >= self.player.potential {
            self.push_log("你的潜能已经发挥到极限，暂时无法继续请教。".into());
            return;
        }

        let is_new = self.player.skill_by_id(skill_id.as_str()).is_none();
        let base_cost =
            150 / teacher_intelligence.max(1) + 150 / self.player.intelligence.max(1) as i32;
        let spirit_cost = if is_new { base_cost * 2 } else { base_cost }.max(1);
        if self.player.spirit <= spirit_cost {
            self.push_log("你今天太累，无法领会师父的讲解。".into());
            return;
        }
        self.player.spirit -= spirit_cost;
        self.player.ensure_skill(skill_id.clone());

        if definition.skill_type == "martial"
            && (current_level as u64).pow(3) / 10 > self.player.combat_experience
        {
            self.push_log(format!(
                "也许是实战经验不足，你对{teacher_name}的回答总是无法领会。"
            ));
            return;
        }

        let experience_term =
            self.player.combat_experience / (1_000 + self.player.combat_experience / 1_000);
        let upper = self
            .player
            .intelligence
            .saturating_add(experience_term.min(u32::MAX as u64) as u32)
            .max(1);
        let gain = self.random(upper);
        self.player.learned_points += 1;
        self.gain_skill_progress(skill_id.clone(), gain);
        self.push_log(format!(
            "你向{teacher_name}请教{}，消耗{}点神和1点潜能。",
            skill_id.name(),
            spirit_cost
        ));
    }

    fn map_skill(&mut self, usage: SkillId, skill_id: SkillId) {
        let Some(definition) = skills().definition(&skill_id) else {
            self.push_log("没有这种特殊技能。".into());
            return;
        };
        if self.player.skill_level(usage.as_str()) == 0
            || self.player.skill_level(skill_id.as_str()) == 0
            || !definition.supports_usage(usage.as_str())
        {
            self.push_log("这项技能不能用于指定的基础用途。".into());
            return;
        }
        self.player
            .skill_mappings
            .retain(|mapping| mapping.usage != usage);
        self.player.skill_mappings.push(SkillMapping {
            usage: usage.clone(),
            skill: skill_id.clone(),
        });
        match usage.as_str() {
            FORCE_ID => self.player.force = 0,
            MAGIC_ID => self.player.atman = 0,
            SPELLS_ID => self.player.mana = 0,
            _ => {}
        }
        self.push_log(format!(
            "你决定以{}作为{}用途。",
            skill_id.name(),
            usage.name()
        ));
    }

    fn practice_skill(&mut self, skill_id: SkillId) {
        let usages: Vec<_> = self
            .player
            .skill_mappings
            .iter()
            .filter(|mapping| mapping.skill == skill_id)
            .map(|mapping| mapping.usage.clone())
            .collect();
        if usages.is_empty() || self.player.skill_level(skill_id.as_str()) == 0 {
            self.push_log("你只能练习已经映射的特殊技能。".into());
            return;
        }
        let definition = skills()
            .definition(&skill_id)
            .expect("mapped skill must exist in catalog");
        if definition.practice.is_none()
            || definition
                .practice
                .as_deref()
                .is_some_and(|body| body.trim_start().starts_with("{ return notify_fail"))
        {
            self.push_log(format!("{}只能通过请教或实际运用提高。", skill_id.name()));
            return;
        }
        if let Err(reason) = self.validate_skill_requirements(definition) {
            self.push_log(reason);
            return;
        }
        if let Some(reason) = self.practice_context_rejection(definition) {
            self.push_log(reason);
            return;
        }
        let Some(cost) = practice_cost(skill_id.as_str()) else {
            self.push_log("这项技能目前无法自行练习。".into());
            return;
        };
        if self.player.essence < cost.essence
            || self.player.qi < cost.qi
            || self.player.spirit < cost.spirit
            || self.player.force < cost.force
            || self.player.mana < cost.mana
        {
            self.push_log("你的精、气、神或内力不足，无法继续练习。".into());
            return;
        }
        self.player.essence -= cost.essence;
        self.player.qi -= cost.qi;
        self.player.spirit -= cost.spirit;
        self.player.force = self.player.force - cost.force + cost.force_gain;
        self.player.mana -= cost.mana;

        let basic_level = usages
            .iter()
            .map(|usage| self.player.skill_level(usage.as_str()))
            .max()
            .unwrap_or(0);
        let gain = basic_level / 5 + 1;
        self.gain_skill_progress_capped(skill_id.clone(), gain, basic_level);
        self.push_log(format!("你反复练习{}，熟练度有所增长。", skill_id.name()));
    }

    fn study_item(&mut self, instance_id: u64) {
        let Some(item) = self.player.item(instance_id) else {
            return;
        };
        let definition = item.definition();
        let Some(study_skill) = definition.study_skill.clone() else {
            self.push_log("你无法从这件物品中学到什么。".into());
            return;
        };
        let item_name = item.display_name().to_string();
        let exp_required = definition.study_exp_required.unwrap_or(0).max(0) as u64;
        let base_cost = definition.study_spirit_cost.unwrap_or(20).max(1);
        let difficulty = definition.study_difficulty.unwrap_or(20);
        let max_level = definition.study_max_level.unwrap_or(0).max(0) as u32;
        let skill_id = SkillId::from(study_skill.as_str());

        let literate = self.player.skill_level("literate");
        if literate == 0 {
            self.push_log("你是个文盲，必须先学习读书识字。".into());
            return;
        }
        if self.player.combat_experience < exp_required {
            self.push_log("你的实战经验不足，再怎么研读也无法领会。".into());
            return;
        }
        let Some(skill_definition) = skills().definition(&skill_id) else {
            self.push_log("这本旧书所载武学在原版技能目录中已经失传。".into());
            return;
        };
        if let Err(reason) = self.validate_skill_requirements(skill_definition) {
            self.push_log(reason);
            return;
        }
        if self.player.skill_level(skill_id.as_str()) > max_level {
            self.push_log("书中所述对你而言已经太浅，无法再有所得。".into());
            return;
        }
        let cost =
            (base_cost + base_cost * (difficulty - self.player.intelligence as i32) / 20).max(1);
        if self.player.spirit < cost {
            self.push_log("你现在过于疲倦，无法专心研读。".into());
            return;
        }
        self.player.spirit -= cost;
        self.player.ensure_skill(skill_id.clone());
        self.gain_skill_progress(skill_id.clone(), literate / 5 + 1);
        self.push_log(format!(
            "你研读{item_name}中有关{}的记载，似乎有些心得。",
            skill_id.name()
        ));
    }

    fn cultivate(&mut self, kind: CultivationKind) {
        const COST: i32 = 30;
        match kind {
            CultivationKind::Exercise => {
                if self.player.mapped_skill(FORCE_ID).is_none() {
                    self.push_log("你必须先选定一种内功心法。".into());
                    return;
                }
                if self.player.qi < COST
                    || self.player.essence * 100 / self.player.max_essence.max(1) < 70
                    || self.player.spirit * 100 / self.player.max_spirit.max(1) < 70
                {
                    self.push_log("你的精、气或神不足，无法运气练功。".into());
                    return;
                }
                self.player.qi -= COST;
                let gain = COST
                    * (self.player.skill_level(FORCE_ID) as i32 + self.player.strength as i32)
                    / 300;
                self.player.force += gain.max(0);
                let cap = (self.player.skill_level(FORCE_ID)
                    + self.player.effective_skill(FORCE_ID) / 5)
                    * 10;
                if self.player.force > self.player.max_force * 2 {
                    if self.player.max_force < cap as i32 {
                        self.player.max_force += 1;
                        self.push_log("你的内力修为提高了。".into());
                    }
                    self.player.force = self.player.max_force;
                } else {
                    self.push_log(format!("你行功一周天，积蓄了{}点内力。", gain.max(0)));
                }
            }
            CultivationKind::Meditate => {
                if self.player.spirit < COST
                    || self.player.qi * 100 / self.player.max_qi.max(1) < 70
                    || self.player.essence * 100 / self.player.max_essence.max(1) < 70
                {
                    self.push_log("你的精、气或神不足，无法静坐冥思。".into());
                    return;
                }
                self.player.spirit -= COST;
                let gain = COST
                    * (self.player.skill_level(SPELLS_ID) as i32 + self.player.spirituality as i32)
                    / 300;
                self.player.mana += gain.max(0);
                if self.player.mana > self.player.max_mana * 2 {
                    let cap = self.player.skill_level(SPELLS_ID) as i32 * 10;
                    if self.player.max_mana < cap {
                        self.player.max_mana += 1;
                        self.push_log("你的法力修为提高了。".into());
                    }
                    self.player.mana = self.player.max_mana;
                } else {
                    self.push_log(format!("你冥思片刻，凝聚了{}点法力。", gain.max(0)));
                }
            }
            CultivationKind::Respirate => {
                if self.player.essence < COST
                    || self.player.qi * 100 / self.player.max_qi.max(1) < 70
                    || self.player.spirit * 100 / self.player.max_spirit.max(1) < 70
                {
                    self.push_log("你的精、气或神不足，无法打坐修行。".into());
                    return;
                }
                self.player.essence -= COST;
                let gain = COST
                    * (self.player.skill_level(MAGIC_ID) as i32 + self.player.spirituality as i32)
                    / 300;
                self.player.atman += gain.max(0);
                if self.player.atman > self.player.max_atman * 2 {
                    let cap = self.player.skill_level(MAGIC_ID) as i32 * 10;
                    if self.player.max_atman < cap {
                        self.player.max_atman += 1;
                        self.push_log("你的灵力修为提高了。".into());
                    }
                    self.player.atman = self.player.max_atman;
                } else {
                    self.push_log(format!("你打坐片刻，凝聚了{}点灵力。", gain.max(0)));
                }
            }
        }
    }

    fn use_technique(&mut self, technique: TechniqueKind) {
        let level = self.player.skill_level(technique.skill_id());
        if level == 0 {
            self.push_log("你尚未学会这项绝招所属的武学。".into());
            return;
        }
        if !self.technique_mapping_is_active(technique) {
            self.push_log("你必须先把这门武学用于对应的基础用途。".into());
            return;
        }

        if !technique.combat_only() {
            match technique {
                TechniqueKind::RecoverQi
                | TechniqueKind::RefreshSpirit
                | TechniqueKind::RegenerateEssence => {
                    let (current, maximum) = match technique {
                        TechniqueKind::RecoverQi => (self.player.qi, self.player.max_qi),
                        TechniqueKind::RefreshSpirit => {
                            (self.player.spirit, self.player.max_spirit)
                        }
                        TechniqueKind::RegenerateEssence => {
                            (self.player.essence, self.player.max_essence)
                        }
                        _ => unreachable!(),
                    };
                    if current >= maximum {
                        self.push_log("这项状态已经恢复到上限。".into());
                        return;
                    }
                    if !self.spend_technique_cost(0, 0, 20, 0, 0) {
                        return;
                    }
                    let healed =
                        (self.player.skill_level(FORCE_ID) as i32 / 3 + 10).min(maximum - current);
                    match technique {
                        TechniqueKind::RecoverQi => self.player.qi += healed,
                        TechniqueKind::RefreshSpirit => self.player.spirit += healed,
                        TechniqueKind::RegenerateEssence => self.player.essence += healed,
                        _ => unreachable!(),
                    }
                    self.push_log(format!(
                        "你运转内功施展{}，恢复{healed}点。",
                        technique.name()
                    ));
                }
                TechniqueKind::VoidSense => {
                    if self
                        .player
                        .potential
                        .saturating_sub(self.player.learned_points)
                        >= 500
                    {
                        self.push_log("你的潜能尚未充分发挥，无法从虚空禅定中再有所得。".into());
                        return;
                    }
                    if !self.spend_technique_cost(30, 0, 0, 0, 75) {
                        return;
                    }
                    let gain = self.random(self.player.intelligence.max(1)) + 1;
                    self.player.potential = self.player.potential.saturating_add(gain);
                    self.push_log(format!("你入无相禅定，潜能增加{gain}点。"));
                }
                TechniqueKind::LotusHeal
                | TechniqueKind::FonxanHeal
                | TechniqueKind::GouyeeHeal => {
                    if self.player.essence >= self.player.max_essence {
                        self.push_log("你现在并未受伤，无需运功疗伤。".into());
                        return;
                    }
                    if self.player.essence < self.player.max_essence / 2 {
                        self.push_log("你已伤重过半，贸然运功只会更加危险。".into());
                        return;
                    }
                    if self.player.force - self.player.max_force < 50 {
                        self.push_log("你的内力没有超出修为五十点，无法运功疗伤。".into());
                        return;
                    }
                    if !self.spend_technique_cost(0, 0, 50, 0, 0) {
                        return;
                    }
                    let healed = 10 + level as i32 / 5;
                    self.player.essence =
                        (self.player.essence + healed).min(self.player.max_essence);
                    self.push_log(format!(
                        "你运转{}，恢复了{healed}点精。",
                        technique.skill_id()
                    ));
                }
                TechniqueKind::Concentrate => {
                    if !self.spend_technique_cost(0, 10, 30, 0, 0) {
                        return;
                    }
                    let gain = 10 + level as i32 / 5;
                    self.player.mana = (self.player.mana + gain).min(self.player.max_mana * 2);
                    self.push_log(format!("你凝聚心神，将内力化为{gain}点法力。"));
                }
                TechniqueKind::AstralVision => {
                    if !self.spend_technique_cost(0, 5, 0, 30, 0) {
                        return;
                    }
                    self.player.set_condition(
                        ConditionKind::AstralVision,
                        (5 + level / 10).max(5),
                        level as i32,
                    );
                    self.push_log("你开启灵视，四周生命的气息变得清晰可辨。".into());
                }
                _ => {
                    self.push_log("这项绝招只能在战斗中施展。".into());
                }
            }
            return;
        }

        let Activity::Fighting(mut combat) = self.activity.clone() else {
            self.push_log("这项绝招只能在战斗中施展。".into());
            return;
        };
        let mut damage = 0;
        match technique {
            TechniqueKind::ChillGaze => {
                if !self.spend_technique_cost(0, 20, 50, 0, 0) {
                    return;
                }
                damage = 10 + level as i32 / 3 + self.player.max_force / 20;
            }
            TechniqueKind::PowerUp => {
                if combat.power_up_active {
                    self.push_log("你已经在催动天邪神功。".into());
                    return;
                }
                if !self.spend_technique_cost(0, 0, 100, 0, 0) {
                    return;
                }
                let bonus = (level as i32 / 3).max(1);
                combat.attack_bonus += bonus;
                combat.dodge_bonus += bonus;
                combat.power_up_active = true;
                self.player.bellicosity += 100 + level as i32 / 2;
                self.push_log(format!(
                    "你催动天邪神功，攻防气势各提高{bonus}点，杀气随之上升。"
                ));
            }
            TechniqueKind::PowerFade => {
                if !self.spend_technique_cost(0, 100, 100, 0, 0) {
                    return;
                }
                let reduction = 100 + level as i32 / 3;
                self.player.bellicosity = (self.player.bellicosity - reduction).max(0);
                combat.attack_bonus -= (level as i32 / 6).max(1);
                self.push_log(format!("你逆转天邪真气，化去{reduction}点杀气。"));
            }
            TechniqueKind::Roar => {
                if !self.spend_technique_cost(10, 0, 150, 0, 0) {
                    return;
                }
                damage = 15 + level as i32 / 2 + self.player.max_force / 10;
            }
            TechniqueKind::Hasten => {
                let attacks = (2 + level / 30).clamp(2, 7) as i32;
                let cost = attacks * 10;
                if self.player.essence < 70
                    || self.player.force - self.player.max_force < 70
                    || !self.spend_technique_cost(cost, 0, cost, 0, 0)
                {
                    if self.player.essence < 70 || self.player.force - self.player.max_force < 70 {
                        self.push_log("你的精或额外内力不足以催动步玄连环。".into());
                    }
                    return;
                }
                damage = attacks * (4 + level as i32 / 12);
                self.push_log(format!("你身随乐律连攻{attacks}式。"));
            }
            TechniqueKind::Counterattack => {
                combat.enemy_busy_rounds = (1 + level / 50).min(3) as u8;
                self.push_log("你借势封住对手后招，准备迎隙反击。".into());
            }
            TechniqueKind::FakeFault => {
                if combat.fake_fault_active {
                    self.push_log("对手已经见过这个破绽，不会再次上当。".into());
                    return;
                }
                let bonus = (level as i32 / 3).max(1);
                combat.attack_bonus += bonus;
                combat.dodge_bonus += bonus / 2;
                combat.fake_fault_active = true;
                self.push_log("你故意卖出破绽，引得对手门户大开。".into());
            }
            TechniqueKind::SwordJab => {
                let attacks = (1 + level / 40).min(3) as i32;
                let cost = attacks * 10;
                if !self.spend_technique_cost(cost, 0, 0, 0, 0) {
                    return;
                }
                damage = attacks * (5 + level as i32 / 15);
            }
            TechniqueKind::DrainerBolt => {
                if !self.spend_technique_cost(0, 20, 0, 25, 0) {
                    return;
                }
                damage = 10 + level as i32 / 3;
                self.player.essence =
                    (self.player.essence + damage / 2).min(self.player.max_essence);
                combat.mode = CombatMode::Lethal;
            }
            TechniqueKind::FeebleBolt | TechniqueKind::NetherBolt => {
                if !self.spend_technique_cost(0, 10, 0, 25, 0) {
                    return;
                }
                damage = 10 + level as i32 / 3;
                combat.mode = CombatMode::Lethal;
            }
            _ => {
                self.push_log("这项绝招不能在当前战斗中施展。".into());
                return;
            }
        }

        combat.technique_cooldown = 1;
        if damage > 0 {
            combat.health -= damage;
            self.push_log(format!(
                "你施展{}命中{}，造成{damage}点伤势。",
                technique.name(),
                combat.enemy.name()
            ));
        }
        if combat.health <= 0 {
            self.win_combat(combat);
        } else if self.player.essence <= 0 || self.player.spirit <= 0 {
            self.lose_combat(combat);
        } else {
            self.activity = Activity::Fighting(combat);
        }
    }

    fn spend_technique_cost(
        &mut self,
        essence: i32,
        spirit: i32,
        force: i32,
        mana: i32,
        atman: i32,
    ) -> bool {
        if self.player.essence < essence
            || self.player.spirit < spirit
            || self.player.force < force
            || self.player.mana < mana
            || self.player.atman < atman
        {
            self.push_log("你的精、神、内力、法力或灵力不足。".into());
            return false;
        }
        self.player.essence -= essence;
        self.player.spirit -= spirit;
        self.player.force -= force;
        self.player.mana -= mana;
        self.player.atman -= atman;
        true
    }

    fn validate_skill_requirements(
        &self,
        definition: &skills::SkillDefinition,
    ) -> Result<(), String> {
        let id = definition.id.as_str();
        let level = self.player.skill_level(id);
        let weapon = self
            .player
            .equipped(EquipmentSlot::Weapon)
            .and_then(|item| item.definition().weapon_skill());
        let body = definition.valid_learn.as_deref().unwrap_or("");
        if body.contains("query_temp(\"weapon\") ||")
            && !body.contains("skill_type")
            && weapon.is_some()
        {
            return Err(format!("练习{}必须空手。", definition.name()));
        }
        if let Some(required) = required_weapon_usage(body)
            && weapon != Some(required)
        {
            return Err(format!(
                "你必须先装备合适的{}兵器。",
                SkillId::from(required).name()
            ));
        }

        let rejected = match id {
            "buddhism" | "taoism" if self.player.bellicosity > 100 => {
                "你的杀气太重，无法修炼这门正法。"
            }
            "celestial" if self.player.bellicosity < level as i32 * 50 => {
                "你的杀气不够，无法领悟更高深的天邪神功。"
            }
            "celestrike"
                if self.player.skill_level("celestial") < 20 || self.player.max_force < 100 =>
            {
                "你的天邪神功或内力修为不足。"
            }
            "chaos-steps" | "deisword" | "fall-steps" | "notraces"
                if self.player.max_force < 50 =>
            {
                "你的内力不足，无法修炼这门武功。"
            }
            "cloudstaff" | "jingang-staff"
                if self.player.strength as i32 + self.player.max_force / 10 < 50 =>
            {
                "你的膂力与内力尚不足以驾驭这门杖法。"
            }
            "essencemagic"
                if self.player.skill_level("buddhism") < 10
                    || self.player.skill_level("buddhism") <= level =>
            {
                "你的大乘佛法修为不够高深。"
            }
            "fonxansword"
                if self.player.max_force < 50
                    || self.player.mapped_skill(FORCE_ID).map(SkillId::as_str)
                        != Some("fonxanforce") =>
            {
                "封山剑法必须配合足够的封山派内功。"
            }
            "gouyee" if self.player.max_mana < level as i32 * 5 => {
                "你的法力不够，无法提升谷衣心法。"
            }
            "linbo-steps" if self.player.skill_level("literate") < 60 => {
                "你的文学素养不够，无法修炼凌波微步。"
            }
            "lotusforce" if self.player.skill_level("buddhism") < level => {
                "你的大乘佛法修为不足以领会莲华心法。"
            }
            "magic-array" if self.player.skill_level("tao-mystery") <= level => {
                "你的小天魔道修为不足以领悟奇门遁甲。"
            }
            "mysterrier"
                if self.player.mapped_skill(FORCE_ID).map(SkillId::as_str) != Some("mystforce")
                    || self.player.skill_level("music") < level / 2 =>
            {
                "步玄七诀必须配合步玄心法与足够的音律修为。"
            }
            "mystsword"
                if self.player.skill_level("mystforce") < 30 || self.player.max_force < 100 =>
            {
                "你的步玄心法或内力火候还不够。"
            }
            "necromancy" if self.player.skill_level("taoism") < level / 2 => {
                "你的天师正道修为不足以驾驭茅山道术。"
            }
            "nine-moon" if self.player.gender != Gender::Female => "九阴赤炼剑法只有女子可以修炼。",
            "nine-moon"
                if self.player.max_force < 50
                    || self.player.mapped_skill(FORCE_ID).map(SkillId::as_str)
                        != Some("nine-moon-force") =>
            {
                "九阴赤炼剑法必须配合足够的九阴心经。"
            }
            "scratching" if self.player.max_force < 80 => "你的内力不足以修炼天师剑法。",
            "six-chaos-sword" if self.player.max_force < 100 => "你的内力不足以修炼六阴追魂剑法。",
            "snowshade-sword"
                if self.player.max_force < 50
                    || self.player.mapped_skill(FORCE_ID).map(SkillId::as_str)
                        != Some("snowshade-force") =>
            {
                "雪影剑法必须配合足够的雪影心法。"
            }
            "snowwhip" if self.player.max_force < 150 => "你的内力不足以修炼寒雪鞭法。",
            "spicyclaw" | "ts-fist" if self.player.max_force < 80 => {
                "你的内力太弱，无法修炼这门拳掌。"
            }
            "stormdance" if self.player.gender != Gender::Female => "七宝天岚舞只有女子可以修炼。",
            "stormdance" if self.player.spirituality < 20 => "你的灵性不足以修炼七宝天岚舞。",
            "tenderzhi" if self.player.gender != Gender::Female => "柔虹指只有女子可以修炼。",
            "wu-shun" if self.player.skill_level("literate") < level => {
                "你的文学素养不足以提升小无相功。"
            }
            _ => return Ok(()),
        };
        Err(rejected.into())
    }

    fn practice_context_rejection(&self, definition: &skills::SkillDefinition) -> Option<String> {
        let body = definition.practice.as_deref().unwrap_or("");
        if let Some(required) = required_weapon_usage(body) {
            let actual = self
                .player
                .equipped(EquipmentSlot::Weapon)
                .and_then(|item| item.definition().weapon_skill());
            if actual != Some(required) {
                return Some(format!(
                    "练习{}必须装备合适的{}兵器。",
                    definition.name(),
                    SkillId::from(required).name()
                ));
            }
        }
        if definition.id.as_str() == "serpentforce"
            && !matches!(
                self.location.as_str(),
                content::LAKE | content::LAKE_BOTTOM | content::LAKESIDE | "village.lakebottom2"
            )
        {
            return Some("伏蛟功只能在有水的地方练习。".into());
        }
        None
    }

    fn toggle_training(&mut self, skill: SkillId) {
        if self.activity == Activity::Training(skill.clone()) {
            self.activity = Activity::Idle;
            self.push_log(format!("你收势调息，结束了{}修炼。", skill.name()));
            return;
        }

        self.activity = Activity::Training(skill.clone());
        self.push_log(format!("你静下心来，开始修炼{}。", skill.name()));
    }

    fn toggle_rest(&mut self) {
        if self.activity == Activity::Resting {
            self.activity = Activity::Idle;
            self.push_log("你起身整理衣装。".into());
        } else if self.player.is_full_health() {
            self.push_log("你现在精神充足，无需休息。".into());
        } else {
            self.activity = Activity::Resting;
            self.push_log("你寻了个安稳位置坐下休息。".into());
        }
    }

    fn training_tick(&mut self, skill: SkillId) {
        if self.player.essence <= 8 || self.player.spirit <= 5 {
            self.activity = Activity::Idle;
            self.push_log("你已十分疲惫，只得暂停修炼。".into());
            return;
        }

        self.player.essence -= 2;
        self.player.spirit -= 1;
        let gain = 2 + self.player.perception / 5;
        self.gain_skill_progress(skill, gain);
    }

    fn start_combat(&mut self, enemy: EnemyKind, mode: CombatMode) {
        if self.player.essence < 20 || self.player.qi < 15 || self.player.spirit < 10 {
            self.push_log("你当前状态太差，无法贸然出手。".into());
            return;
        }
        self.begin_combat(enemy, mode);
    }

    fn begin_combat(&mut self, enemy: EnemyKind, mode: CombatMode) {
        let max_health = enemy.max_health();
        self.activity = Activity::Fighting(CombatState {
            enemy,
            health: max_health,
            max_health,
            rounds: 0,
            mode,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        });
        match mode {
            CombatMode::Spar => {
                self.push_log(format!("你向{}抱拳示意，双方点到为止。", enemy.name()));
            }
            CombatMode::Lethal => {
                self.push_log(format!("你向{}喝道：今日性命相搏！", enemy.name()));
            }
        }
    }

    fn surrender(&mut self) {
        let Activity::Fighting(combat) = self.activity.clone() else {
            return;
        };
        if combat.mode == CombatMode::Lethal {
            self.push_log(format!("{}不接受求饶，死斗仍在继续。", combat.enemy.name()));
            return;
        }
        self.activity = Activity::Idle;
        let loss = self.player.reputation.clamp(0, 50);
        self.player.reputation -= loss;
        self.push_log(format!(
            "你跳出战圈，向{}认输。评价降低{}点。",
            combat.enemy.name(),
            loss
        ));
    }

    fn combat_tick(&mut self, mut combat: CombatState) {
        combat.rounds += 1;
        combat.technique_cooldown = combat.technique_cooldown.saturating_sub(1);
        let (has_weapon, weapon_bonus, usage) =
            self.player
                .equipped(EquipmentSlot::Weapon)
                .map_or((false, 0, UNARMED_ID), |item| {
                    (
                        true,
                        item.definition().weapon_damage.unwrap_or(0).max(0) / 10,
                        item.definition().weapon_skill().unwrap_or(SWORD_ID),
                    )
                });
        let style_id = self
            .player
            .mapped_skill(usage)
            .cloned()
            .unwrap_or_else(|| SkillId::from(usage));
        let skill_level = self.player.effective_skill(usage);
        let action = skills().definition(&style_id).and_then(|definition| {
            (!definition.actions.is_empty()).then(|| {
                let index = self.random(definition.actions.len() as u32) as usize;
                definition.actions[index].clone()
            })
        });
        let action_name = action
            .as_ref()
            .map_or("寻常一式", skills::SkillActionDefinition::display_name);
        let limb = ["头部", "胸口", "左臂", "右臂", "腰间", "腿部"][self.random(6) as usize];
        let hit_chance = (65 + skill_level as i32 * 2 + combat.attack_bonus
            - combat.enemy.defense() * 2)
            .clamp(20, 95) as u32;
        self.player.qi = (self.player.qi - 2).max(0);
        self.gain_skill_progress(SkillId::from(usage), 2);

        if self.random(100) < hit_chance {
            let base_damage =
                (self.player.strength as i32 / 2 + skill_level as i32 / 2 + weapon_bonus
                    - combat.enemy.defense() / 2)
                    .max(2);
            let damage_percent = action
                .as_ref()
                .and_then(|action| action.damage)
                .unwrap_or(0);
            let force_percent = action.as_ref().and_then(|action| action.force).unwrap_or(0);
            let mut damage = (base_damage
                + base_damage * damage_percent / 100
                + self.player.strength as i32 * force_percent / 200)
                .max(2);
            let mut hook_message = None;
            if self.player.mapped_skill(FORCE_ID).map(SkillId::as_str) == Some("iceforce") {
                let ice_level = self.player.skill_level("iceforce").max(1);
                if self.random(ice_level) > damage as u32 {
                    damage += (damage / 2).max(1);
                    hook_message = Some("阴寒劲力透体而入");
                }
            }
            if matches!(style_id.as_str(), "spicyclaw" | "ts-fist")
                && damage_percent >= 100
                && self.random((damage_percent / 2).max(1) as u32)
                    > combat.enemy.attack().max(0) as u32
            {
                let threshold = if style_id.as_str() == "spicyclaw" {
                    100
                } else {
                    80
                };
                damage += ((damage_percent - threshold) / 2).max(0);
                hook_message = Some("掌力迸发，传出骨节爆响");
            }
            combat.health -= damage;
            self.push_log(format!(
                "第{}合：你使出「{}」击中{}的{}，造成{}点伤势。",
                combat.rounds,
                action_name,
                combat.enemy.name(),
                limb,
                damage
            ));
            if let Some(message) = hook_message {
                self.push_log(format!("{message}。"));
            }
        } else {
            self.push_log(format!(
                "第{}合：你使出「{}」，却被{}闪开。",
                combat.rounds,
                action_name,
                combat.enemy.name()
            ));
        }

        if has_weapon {
            self.degrade_equipment(EquipmentSlot::Weapon);
        }
        if combat.health <= 0 {
            self.win_combat(combat);
            return;
        }

        let dodge_level = self.player.effective_skill(DODGE_ID);
        let parry_level = self.player.effective_skill(PARRY_ID);
        let enemy_attack = combat.enemy.attack() + self.random(6) as i32;
        if combat.enemy_busy_rounds > 0 {
            combat.enemy_busy_rounds -= 1;
            self.push_log(format!("{}尚未稳住身形，来不及反击。", combat.enemy.name()));
            self.activity = Activity::Fighting(combat);
            return;
        }

        let dodge_chance =
            (15 + dodge_level as i32 * 3 + combat.dodge_bonus - enemy_attack).clamp(5, 75) as u32;
        let parry_chance = (10 + parry_level as i32 * 2 - enemy_attack / 2).clamp(5, 45) as u32;
        let defense_roll = self.random(100);
        if defense_roll < dodge_chance {
            self.gain_skill_progress(SkillId::from(DODGE_ID), 1);
            self.push_log(format!("{}反击，你施展身法从容避开。", combat.enemy.name()));
            self.activity = Activity::Fighting(combat);
            return;
        }
        if defense_roll < dodge_chance + parry_chance {
            self.gain_skill_progress(SkillId::from(PARRY_ID), 1);
            self.push_log(format!("{}反击，被你稳稳架开。", combat.enemy.name()));
            self.activity = Activity::Fighting(combat);
            return;
        }

        let armor_bonus: i32 = self
            .player
            .equipment
            .iter()
            .filter(|equipped| equipped.slot != EquipmentSlot::Weapon)
            .filter_map(|equipped| self.player.item(equipped.instance_id))
            .map(|item| item.definition().armor.unwrap_or(0).max(0) / 10)
            .sum();
        let received = (enemy_attack - armor_bonus).max(2);
        let resource = combat.enemy.damage_resource();
        match resource {
            CombatResource::Essence => self.player.essence -= received,
            CombatResource::Qi => self.player.qi -= received,
            CombatResource::Spirit => self.player.spirit -= received,
        }
        let armor_slots: Vec<_> = self
            .player
            .equipment
            .iter()
            .filter(|equipped| equipped.slot != EquipmentSlot::Weapon)
            .map(|equipped| equipped.slot)
            .collect();
        for slot in armor_slots {
            self.degrade_equipment(slot);
        }
        self.push_log(format!(
            "{}反击命中你的{}，你损失{}点{}。",
            combat.enemy.name(),
            limb,
            received,
            resource.name()
        ));

        if self.player.essence <= 0 || self.player.qi <= 0 || self.player.spirit <= 0 {
            self.lose_combat(combat);
        } else {
            self.activity = Activity::Fighting(combat);
        }
    }

    fn degrade_equipment(&mut self, slot: EquipmentSlot) {
        let Some(instance_id) = self
            .player
            .equipment
            .iter()
            .find(|equipped| equipped.slot == slot)
            .map(|equipped| equipped.instance_id)
        else {
            return;
        };
        let broken_name = {
            let Some(item) = self.player.item_mut(instance_id) else {
                return;
            };
            let Some(durability) = item.durability.as_mut() else {
                return;
            };
            *durability = durability.saturating_sub(1);
            (*durability == 0).then(|| item.display_name().to_string())
        };
        if let Some(name) = broken_name {
            self.player
                .equipment
                .retain(|equipped| equipped.instance_id != instance_id);
            self.push_log(format!("{name}已经损坏，自动卸下。"));
        }
    }

    fn win_combat(&mut self, combat: CombatState) {
        self.activity = Activity::Idle;
        let enemy = combat.enemy;
        let insight = enemy.insight_reward();
        self.player.insight += insight;
        self.player.potential = self.player.potential.saturating_add(insight / 2 + 1);
        self.player.combat_experience = self
            .player
            .combat_experience
            .saturating_add(enemy.max_health().max(0) as u64 * combat.rounds.max(1) as u64);
        self.player.reputation += enemy.reputation_reward();
        match combat.mode {
            CombatMode::Spar => self.push_log(format!(
                "{}失去战力，抱拳认输。领悟 +{}。",
                enemy.name(),
                insight
            )),
            CombatMode::Lethal => {
                self.player.bellicosity += 1;
                let wanted = enemy.wanted_reward();
                self.player.wanted = self.player.wanted.saturating_add(wanted);
                self.push_log(format!(
                    "你在死斗中击杀{}。领悟 +{}，杀气 +1，通缉 +{}。",
                    enemy.name(),
                    insight,
                    wanted
                ));
            }
        }

        match enemy {
            EnemyKind::Bandit if self.quest == QuestStage::FindJuan => {
                self.quest = QuestStage::ReturnHome;
                self.player.add_money(1_200);
                self.push_log("你赶走山贼，在林间找到了受惊的娟儿，并护送她离开松林。".into());
                self.push_log("任务更新：回刘家小房报平安。银子 +12 两。".into());
            }
            EnemyKind::Wolf => {
                let pelt_id = ItemId::from(items::WOLF_PELT_ID);
                let already_dropped = self
                    .ground_items
                    .get(&self.location)
                    .is_some_and(|ground| ground.iter().any(|item| item.item_id == pelt_id));
                if !self.player.has_item(&pelt_id) && !already_dropped {
                    self.place_item_on_ground(pelt_id, 1);
                    self.push_log("一张完整的狼皮掉落在地上。".into());
                }
            }
            EnemyKind::Meloner => {
                self.melon_debt = false;
                self.push_log("瓜农吃了亏，只得退回瓜棚；你强占西瓜的事也损害了评价。".into());
            }
            EnemyKind::BloodHandLiuSan => {
                self.snow_guard_defeated = true;
                let manual = ItemId::from("snow.npc.obj.blade_book");
                self.place_item_on_ground(manual, 1);
                self.push_log("血手刘三倒下后，一本残破刀谱落在地上。".into());
            }
            _ => {}
        }
    }

    fn lose_combat(&mut self, combat: CombatState) {
        self.activity = Activity::Idle;
        self.player.essence = (self.player.max_essence / 2).max(1);
        self.player.qi = (self.player.max_qi / 2).max(1);
        self.player.spirit = (self.player.max_spirit / 2).max(1);
        match combat.mode {
            CombatMode::Spar => self.push_log(format!(
                "你在与{}的比试中昏迷，许久后才醒来。",
                combat.enemy.name()
            )),
            CombatMode::Lethal => {
                self.location = LocationId::from(content::LIU_HOME);
                let lost = self.player.money_value().min(500);
                self.player
                    .set_money_value(self.player.money_value() - lost);
                self.push_log(format!(
                    "你被{}重创。死亡机制将在 M7 接入；当前由路人送回刘家小房，并遗失{}。",
                    combat.enemy.name(),
                    format_money(lost)
                ));
            }
        }
    }

    fn recover(&mut self, essence: i32, qi: i32, spirit: i32) {
        if self.player.water <= 0 {
            return;
        }
        self.player.essence = (self.player.essence + essence).min(self.player.max_essence);
        if self.player.food > 0 {
            self.player.qi = (self.player.qi + qi).min(self.player.max_qi);
            self.player.spirit = (self.player.spirit + spirit).min(self.player.max_spirit);
        }
    }

    fn update_conditions(&mut self) {
        let mut active = Vec::new();
        let mut collapse_reason = None;
        for mut condition in std::mem::take(&mut self.player.conditions) {
            match condition.kind {
                ConditionKind::Bandaged => {
                    self.player.essence = (self.player.essence + condition.potency.max(1))
                        .min(self.player.max_essence);
                }
                ConditionKind::SnakePoison => {
                    self.player.essence -= condition.potency.max(10);
                    self.player.spirit -= 10;
                    self.push_log("蛇毒发作，你的精与神受到损伤。".into());
                }
                ConditionKind::Poison => {
                    let damage = condition.potency.max(1);
                    self.player.essence -= damage;
                    self.push_log(format!("毒性发作，你损失 {damage} 点精。"));
                }
                ConditionKind::Drunk => {
                    let limit = (self.player.strength + self.player.max_qi.max(0) as u32 / 50)
                        .saturating_mul(2)
                        .max(1);
                    if condition.duration > limit {
                        collapse_reason = Some("酒力");
                        condition.duration = 1;
                    } else if condition.duration > limit / 2 {
                        self.player.spirit -= 10;
                        self.push_log("你醉得脚步虚浮，神损失 10。".into());
                    } else if condition.duration > limit / 4 {
                        self.player.spirit -= 3;
                        self.player.essence =
                            (self.player.essence + 15).min(self.player.max_essence);
                        self.player.qi = (self.player.qi + 10).min(self.player.max_qi);
                    }
                }
                ConditionKind::Slumber => {
                    let limit = self.player.strength.saturating_mul(2).max(1);
                    if condition.duration > limit {
                        collapse_reason = Some("蒙汗药力");
                        condition.duration = 1;
                    }
                }
                ConditionKind::AstralVision => {}
            }
            condition.duration = condition.duration.saturating_sub(1);
            if condition.duration > 0 {
                active.push(condition);
            }
        }
        self.player.conditions = active;

        if self.player.essence <= 0 || self.player.spirit <= 0 {
            collapse_reason.get_or_insert("伤势");
        }
        if let Some(reason) = collapse_reason {
            self.activity = Activity::Idle;
            self.player.essence = (self.player.max_essence / 2).max(1);
            self.player.qi = (self.player.max_qi / 2).max(1);
            self.player.spirit = (self.player.max_spirit / 2).max(1);
            self.push_log(format!("你因{reason}昏迷，许久后才苏醒过来。"));
        }
    }

    fn gain_skill_progress(&mut self, kind: SkillId, amount: u32) {
        self.gain_skill_progress_capped(kind, amount, u32::MAX);
    }

    fn gain_skill_progress_capped(&mut self, kind: SkillId, amount: u32, cap: u32) {
        let name = kind.name().to_string();
        let mut gained_levels = Vec::new();
        self.player.ensure_skill(kind.clone());
        {
            let skill = self.player.skill_mut(&kind);
            skill.progress = skill.progress.saturating_add(amount);
            while skill.level < cap && skill.progress >= skill.required_progress() {
                skill.progress -= skill.required_progress();
                skill.level += 1;
                gained_levels.push(skill.level);
            }
            if skill.level >= cap {
                skill.progress = skill
                    .progress
                    .min(skill.required_progress().saturating_sub(1));
            }
        }
        for level in gained_levels {
            self.push_log(format!("你的{name}提升到{level}层。"));
            self.apply_skill_level_hook(&kind, level);
        }
    }

    fn apply_skill_level_hook(&mut self, skill: &SkillId, level: u32) {
        match skill.as_str() {
            "celestial" if level % 10 == 9 && self.player.composure < level / 4 => {
                self.player.composure += 2;
                self.push_log("苦练天邪神功使你的定力提高了。".into());
            }
            FORCE_ID if level % 10 == 9 && self.player.constitution < level / 4 => {
                self.player.constitution += 2;
                self.push_log("内功修炼有成使你的体质改善了。".into());
            }
            "literate" if level % 10 == 9 && self.player.intelligence < level / 4 => {
                self.player.intelligence += 2;
                self.push_log("勤学苦读使你的悟性提高了。".into());
            }
            "music" if level % 10 == 9 && self.player.spirituality < level / 4 => {
                self.player.spirituality += 2;
                self.push_log("音律修为使你的灵性提高了。".into());
            }
            "stormdance" if level % 10 == 9 && self.player.perception < level / 4 => {
                self.player.perception += 2;
                self.push_log("勤练舞技使你的容貌气质提高了。".into());
            }
            UNARMED_ID if level % 10 == 9 && self.player.strength < level / 4 => {
                self.player.strength += 2;
                self.push_log("勤练拳脚使你的膂力提高了。".into());
            }
            "nine-moon" => {
                // The source queries the absent `nine-moon-sword` ID, so its modulo branch is always 0.
                self.player.bellicosity += 2_000;
                self.push_log("九阴之气冲上心头，你的杀气陡增。".into());
            }
            SIX_CHAOS_SWORD_ID => {
                self.player.bellicosity += if level.is_multiple_of(10) { 1_000 } else { 100 };
                self.push_log("六阴剑意激起一股恶气，你的杀气上升。".into());
            }
            "tao-mystery" => {
                self.player.bellicosity += 100;
            }
            _ => {}
        }
    }

    fn random(&mut self, upper: u32) -> u32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.rng_state >> 32) as u32) % upper
    }

    pub(crate) fn migrate_v1_location_ids(&mut self) {
        let migrated = match self.location.as_str() {
            "LiuHome" => content::LIU_HOME,
            "Garden" => content::GARDEN,
            "VillageRoad" => "village.road5",
            "WheatField" => content::FIELD,
            "Lakeside" => content::LAKESIDE,
            "PineForest" => content::PINE_FOREST,
            "SnowTown" => content::SNOW_TOWN,
            "MountainPath" => content::MOUNTAIN_PATH,
            "TempleYard" => content::TEMPLE_YARD,
            current => current,
        };
        self.location = LocationId::from(migrated);
    }

    pub(crate) fn migrate_legacy_items(&mut self) {
        self.player.equipment.clear();
        for (index, item) in self.player.inventory.iter_mut().enumerate() {
            item.instance_id = index as u64 + 1;
            if item.durability.is_none() {
                item.durability = item.definition().max_durability();
            }
            if item.remaining_uses.is_none() {
                item.remaining_uses = item.definition().initial_uses();
            }
        }

        if let Some(cloth) = self
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == items::CLOTH_ID)
        {
            self.player.equipment.push(EquippedItem {
                slot: EquipmentSlot::Torso,
                instance_id: cloth.instance_id,
            });
        }
        if let Some(legacy_weapon) = self.player.legacy_weapon.take() {
            let item_id = legacy_weapon.item_id();
            if let Some(weapon) = self
                .player
                .inventory
                .iter()
                .find(|item| item.item_id == item_id)
            {
                self.player.equipment.push(EquippedItem {
                    slot: EquipmentSlot::Weapon,
                    instance_id: weapon.instance_id,
                });
            }
        }
        self.next_item_instance_id = self.player.inventory.len() as u64 + 1;
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v3_statuses(&mut self) {
        for item in self
            .player
            .inventory
            .iter_mut()
            .chain(self.ground_items.values_mut().flatten())
        {
            if item.remaining_uses.is_none() {
                item.remaining_uses = item.definition().initial_uses();
            }
        }
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v4_skills(&mut self) {
        let legacy_levels: Vec<_> = self
            .player
            .skills
            .iter()
            .map(|skill| (skill.kind.clone(), skill.level, skill.progress))
            .collect();
        for (kind, level, progress) in legacy_levels {
            let usages: &[&str] = match kind.as_str() {
                LIUH_KEN_ID => &[UNARMED_ID],
                SIX_CHAOS_SWORD_ID => &[SWORD_ID],
                PYROBAT_STEPS_ID => &[DODGE_ID, MOVE_ID],
                _ => &[],
            };
            for usage in usages {
                if self.player.skill_by_id(usage).is_none() {
                    self.player.skills.push(Skill {
                        kind: SkillId::from(*usage),
                        level,
                        progress,
                    });
                }
            }
        }
        for (id, level) in [(FORCE_ID, 5), (PARRY_ID, 4)] {
            if self.player.skill_by_id(id).is_none() {
                self.player.skills.push(Skill::new(id, level));
            }
        }
        if self.player.skill_mappings.is_empty() {
            self.player.skill_mappings = default_skill_mappings();
        }
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v5_npc_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v6_m4_access(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub fn push_log(&mut self, message: String) {
        self.logs.push(message);
        if self.logs.len() > LOG_LIMIT {
            self.logs.drain(0..self.logs.len() - LOG_LIMIT);
        }
    }
}

impl EnemyKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Bandit => "松林山贼",
            Self::Wolf => "灰背野狼",
            Self::TempleDisciple => "护院武僧",
            Self::Rat => "大老鼠",
            Self::IceDragon => "白鳞冰龙",
            Self::Meloner => "愤怒的瓜农",
            Self::BloodHandLiuSan => "血手刘三",
        }
    }

    fn max_health(self) -> i32 {
        match self {
            Self::Bandit => 65,
            Self::Wolf => 48,
            Self::TempleDisciple => 95,
            Self::Rat => 22,
            Self::IceDragon => 420,
            Self::Meloner => 90,
            Self::BloodHandLiuSan => 150,
        }
    }

    fn attack(self) -> i32 {
        match self {
            Self::Bandit => 13,
            Self::Wolf => 11,
            Self::TempleDisciple => 17,
            Self::Rat => 7,
            Self::IceDragon => 42,
            Self::Meloner => 16,
            Self::BloodHandLiuSan => 22,
        }
    }

    fn defense(self) -> i32 {
        match self {
            Self::Bandit => 7,
            Self::Wolf => 5,
            Self::TempleDisciple => 12,
            Self::Rat => 2,
            Self::IceDragon => 30,
            Self::Meloner => 11,
            Self::BloodHandLiuSan => 15,
        }
    }

    fn insight_reward(self) -> u32 {
        match self {
            Self::Bandit => 15,
            Self::Wolf => 8,
            Self::TempleDisciple => 24,
            Self::Rat => 3,
            Self::IceDragon => 80,
            Self::Meloner => 8,
            Self::BloodHandLiuSan => 30,
        }
    }

    fn reputation_reward(self) -> i32 {
        match self {
            Self::Bandit => 4,
            Self::Wolf => 1,
            Self::TempleDisciple => 2,
            Self::Rat => 0,
            Self::IceDragon => 12,
            Self::Meloner => -8,
            Self::BloodHandLiuSan => 8,
        }
    }

    fn damage_resource(self) -> CombatResource {
        match self {
            Self::TempleDisciple => CombatResource::Qi,
            Self::IceDragon => CombatResource::Spirit,
            _ => CombatResource::Essence,
        }
    }

    fn wanted_reward(self) -> u32 {
        match self {
            Self::TempleDisciple | Self::Meloner => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exit {
    pub direction: String,
    pub target: LocationId,
    pub source_target: Option<String>,
    pub internal: bool,
    pub dynamic: bool,
}

impl Exit {
    pub fn adapter(direction: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            direction: direction.into(),
            target: LocationId::new(target),
            source_target: None,
            internal: false,
            dynamic: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub zone: String,
    pub description: String,
    pub arrival: String,
    pub exits: Vec<Exit>,
    pub npcs: Vec<NpcId>,
    pub training: Option<SkillId>,
    pub can_rest: bool,
    pub enemy: Option<EnemyKind>,
    pub source_path: Option<String>,
    pub object_sources: Vec<String>,
    pub behavior_flags: Vec<String>,
}

impl Location {
    #[allow(clippy::too_many_arguments)]
    pub fn adapted(
        id: impl Into<String>,
        name: impl Into<String>,
        zone: impl Into<String>,
        description: impl Into<String>,
        arrival: impl Into<String>,
        exits: Vec<Exit>,
        npc: Option<NpcId>,
        training: Option<SkillId>,
        can_rest: bool,
        enemy: Option<EnemyKind>,
    ) -> Self {
        Self {
            id: LocationId::new(id),
            name: name.into(),
            zone: zone.into(),
            description: description.into(),
            arrival: arrival.into(),
            exits,
            npcs: npc.into_iter().collect(),
            training,
            can_rest,
            enemy,
            source_path: None,
            object_sources: Vec::new(),
            behavior_flags: Vec::new(),
        }
    }
}

fn door_for_transition(source: &LocationId, target: &LocationId) -> Option<DoorKind> {
    match (source.as_str(), target.as_str()) {
        (content::LIU_HOME, content::GARDEN) | (content::GARDEN, content::LIU_HOME) => {
            Some(DoorKind::LiuGarden)
        }
        (content::LORD_HOUSE1, content::ROAD9) | (content::ROAD9, content::LORD_HOUSE1) => {
            Some(DoorKind::LordManor)
        }
        _ => None,
    }
}

const SNOW_TOWN_TEACHERS: [&str; 10] = [
    "assassin",
    "beggar",
    "dancer",
    "fighter",
    "juechen",
    "lama",
    "ninja",
    "ronin",
    "scholar",
    "swordsman",
];
const TEMPLE_TEACHERS: [&str; 1] = ["bonze"];

fn teachers_at_location(location: &str) -> &'static [&'static str] {
    match location {
        content::SNOW_TOWN => &SNOW_TOWN_TEACHERS,
        content::TEMPLE_YARD => &TEMPLE_TEACHERS,
        _ => &[],
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PracticeCost {
    essence: i32,
    qi: i32,
    spirit: i32,
    force: i32,
    mana: i32,
    force_gain: i32,
}

fn practice_cost(skill: &str) -> Option<PracticeCost> {
    let mut cost = PracticeCost::default();
    match skill {
        "bloodystrike" | "celestrike" => {
            cost.qi = 30;
            cost.force = 5;
        }
        "chaos-steps" | "deisword" | "fall-steps" | "fonxansword" | "notraces"
        | "snowshade-sword" => {
            cost.qi = 30;
            cost.force = 3;
        }
        "cloudstaff" | "jingang-staff" => cost.qi = 60,
        "linbo-steps" => {
            cost.essence = 10;
            cost.spirit = 10;
            cost.force_gain = 3;
        }
        "liuh-ken" | "meihua-shou" | "pyrobat-steps" => cost.qi = 30,
        "mysterrier" => {
            cost.qi = 20;
            cost.spirit = 20;
        }
        "mystsword" | "nine-moon" | "scratching" | "six-chaos-sword" | "snowwhip" => {
            cost.qi = 30;
            cost.force = 5;
        }
        "necromancy" => {
            cost.spirit = 30;
            cost.mana = 10;
        }
        "serpentforce" => {
            cost.qi = 30;
            cost.force = 10;
        }
        "shortsong-blade" | "spring-blade" => cost.qi = 40,
        "spicyclaw" | "ts-fist" => {
            cost.qi = 25;
            cost.force = 3;
        }
        "stormdance" => cost.spirit = 30,
        "tenderzhi" => {
            cost.spirit = 30;
            cost.force = 10;
        }
        _ => return None,
    }
    Some(cost)
}

fn required_weapon_usage(body: &str) -> Option<&'static str> {
    [
        "axe", "blade", "dagger", "fork", "hammer", "staff", "sword", "throwing", "whip",
    ]
    .into_iter()
    .find(|usage| {
        body.contains(&format!("skill_type\") != \"{usage}\""))
            || body.contains(&format!("skill_type\")!= \"{usage}\""))
    })
}

fn consumed_residual(item_id: &str) -> Option<(&'static str, u32)> {
    match item_id {
        "canyon.npc.obj.chicken_leg" | "obj.example.chicken_leg" => {
            Some(("啃得精光的鸡腿骨头", 150))
        }
        "chuenyu.npc.obj.pigmeat" | "chuenyu.obj.pigmeat" => Some(("山猪骨头", 250)),
        "u.cloud.obj.meat.beef" => Some(("牛肋骨", 200)),
        "u.cloud.obj.meat.dog_m" => Some(("狗骨头", 250)),
        "u.cloud.obj.meat.hind" => Some(("牛腿骨", 300)),
        items::WATER_MELON_ID => Some(("西瓜皮", 150)),
        _ => None,
    }
}

fn format_money(mut value: u64) -> String {
    if value == 0 {
        return "身无分文".into();
    }
    let mut parts = Vec::new();
    let banknotes = value / 100_000;
    if banknotes > 0 {
        parts.push(format!("{banknotes}张千两银票"));
        value %= 100_000;
    }
    let gold = value / 10_000;
    if gold > 0 {
        parts.push(format!("{gold}两黄金"));
        value %= 10_000;
    }
    let silver = value / 100;
    if silver > 0 {
        parts.push(format!("{silver}两银子"));
        value %= 100;
    }
    if value > 0 {
        parts.push(format!("{value}文钱"));
    }
    parts.join(" ")
}

fn inquiry_topic_name(topic: &str) -> &str {
    match topic {
        "name" => "姓名",
        "here" => "此地",
        "home" => "家乡",
        "out" => "出城",
        "employment" => "雇用",
        other => other,
    }
}

fn direction_name(direction: &str) -> &str {
    match direction {
        "north" => "北",
        "south" => "南",
        "east" => "东",
        "west" => "西",
        "northeast" => "东北",
        "northwest" => "西北",
        "southeast" => "东南",
        "southwest" => "西南",
        "northup" => "北上",
        "southdown" => "南下",
        "westup" => "西上",
        "eastdown" => "东下",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_changes_location_and_stops_activity() {
        let mut game = Game::new();
        game.activity = Activity::Resting;
        game.perform(Action::Interact(InteractionKind::OpenDoor(
            DoorKind::LiuGarden,
        )));
        game.perform(Action::Move {
            direction: "south".into(),
            target: LocationId::from(content::GARDEN),
        });
        assert_eq!(game.location, LocationId::from(content::GARDEN));
        assert_eq!(game.activity, Activity::Idle);
    }

    #[test]
    fn paired_doors_block_both_sides_and_share_state() {
        let mut game = Game::new();
        let can_reach =
            |game: &Game, target: &str| {
                game.available_actions().iter().any(|action| matches!(
                action,
                Action::Move { target: action_target, .. } if action_target.as_str() == target
            ))
            };

        assert!(!can_reach(&game, content::GARDEN));
        game.perform(Action::Interact(InteractionKind::OpenDoor(
            DoorKind::LiuGarden,
        )));
        assert!(can_reach(&game, content::GARDEN));
        game.location = LocationId::from(content::GARDEN);
        game.perform(Action::Interact(InteractionKind::CloseDoor(
            DoorKind::LiuGarden,
        )));
        assert!(!can_reach(&game, content::LIU_HOME));

        game.location = LocationId::from(content::ROAD9);
        assert!(!can_reach(&game, content::LORD_HOUSE1));
        game.perform(Action::Interact(InteractionKind::OpenDoor(
            DoorKind::LordManor,
        )));
        assert!(can_reach(&game, content::LORD_HOUSE1));
    }

    #[test]
    fn room_objects_and_incomplete_book_puzzle_are_interactive() {
        let mut game = Game::new();
        game.location = LocationId::from(content::ROAD3);
        game.perform(Action::Interact(InteractionKind::InspectTablet));
        assert!(game.logs.last().unwrap().contains("傅家坡"));

        game.location = LocationId::from(content::LORD_HOUSE3);
        game.perform(Action::Interact(InteractionKind::InspectBookshelf));
        for number in [9, 3, 11, 6] {
            game.perform(Action::Interact(InteractionKind::PullBook(number)));
        }
        assert!(game.book_puzzle_completed);
        assert_eq!(game.player.insight, 3);
        assert!(
            game.logs
                .last()
                .unwrap()
                .contains("原作中的这套机关也止于此处")
        );
    }

    #[test]
    fn caught_melon_thief_must_fight_or_settle_debt() {
        let mut game = Game::new();
        game.location = LocationId::from(content::MELON_FARM);
        game.player.perception = 30;
        let money_before = game.player.money_value();

        game.perform(Action::Interact(InteractionKind::PickMelon));
        assert!(game.player.has_item(&ItemId::from(items::WATER_MELON_ID)));
        assert!(game.melon_debt);
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                enemy: EnemyKind::Meloner,
                ..
            })
        ));
        assert!(
            game.available_actions()
                .iter()
                .all(|action| !matches!(action, Action::Flee { .. }))
        );

        game.perform(Action::Surrender);
        assert!(
            game.available_actions()
                .iter()
                .all(|action| !matches!(action, Action::Move { .. }))
        );
        game.perform(Action::Interact(InteractionKind::SettleMelonDebt));
        assert!(!game.melon_debt);
        assert_eq!(game.player.money_value(), money_before - 60);
        assert!(
            game.available_actions()
                .iter()
                .any(|action| matches!(action, Action::Move { .. }))
        );
    }

    #[test]
    fn lake_travel_requires_original_boat_and_dive_commands() {
        let mut game = Game::new();
        game.location = LocationId::from(content::LAKESIDE);
        assert!(!game.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::LAKE
        )));
        game.perform(Action::Interact(InteractionKind::PaddleToLake));
        assert_eq!(game.location, LocationId::from(content::LAKE));

        game.perform(Action::Interact(InteractionKind::DiveIntoLake));
        assert_eq!(game.location, LocationId::from(content::LAKE_BOTTOM));
    }

    #[test]
    fn grass_path_is_visible_only_for_five_ticks() {
        let mut game = Game::new();
        game.location = LocationId::from(content::ROAD6);
        let has_west_path = |game: &Game| {
            game.available_actions().iter().any(|action| {
                matches!(
                    action,
                    Action::Move { target, .. } if target.as_str() == "village.valley2"
                )
            })
        };

        assert!(!has_west_path(&game));
        game.perform(Action::Interact(InteractionKind::RevealGrassPath));
        assert!(has_west_path(&game));
        for _ in 0..5 {
            game.tick();
        }
        assert!(!has_west_path(&game));
    }

    #[test]
    fn money_uses_original_coin_silver_gold_exchange_rates() {
        let mut player = Player::default();
        assert_eq!(player.money_value(), 2_400);
        assert!(player.pay_money(65));
        assert_eq!(player.silver, 23);
        assert_eq!(player.coins, 35);

        player.add_money(10_000);
        assert_eq!(player.gold, 1);
        assert_eq!(player.money_value(), 12_335);
        assert_eq!(player.money_text(), "1两黄金 23两银子 35文钱");
    }

    #[test]
    fn static_source_inquiries_are_bound_to_the_current_npc() {
        let mut game = Game::new();
        game.location = LocationId::from("city.jiulou");
        let asks: Vec<_> = game
            .available_actions()
            .into_iter()
            .filter(|action| matches!(action, Action::AskNpc { .. }))
            .collect();
        assert_eq!(asks.len(), 4);
        assert!(asks.iter().any(|action| {
            action.label(&game) == "向钱掌柜询问姓名"
                && matches!(
                    action,
                    Action::AskNpc { npc, topic }
                        if npc.as_str() == "city.npc.boss" && topic == "name"
                )
        }));

        game.perform(Action::AskNpc {
            npc: NpcId::from("city.npc.boss"),
            topic: "皇上".into(),
        });
        assert_eq!(
            game.logs.last().unwrap(),
            "钱掌柜说道：小声点。偷偷地告述你，我真见过皇上的。"
        );

        game.location = LocationId::from("snow.school1");
        let guard_topics: Vec<_> = game
            .available_actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::AskNpc { npc, topic } if npc.as_str() == SNOW_GUARD_ID => Some(topic),
                _ => None,
            })
            .collect();
        assert_eq!(guard_topics, ["刘老三", "血手刘三"]);
    }

    #[test]
    fn audited_scripted_inquiries_run_vendor_story_and_injury_handlers() {
        let mut game = Game::new();
        game.location = LocationId::from("city.bridge");
        let asks: Vec<_> = game
            .available_actions()
            .into_iter()
            .filter(|action| matches!(action, Action::AskNpc { .. }))
            .collect();
        assert_eq!(asks.len(), 3);
        assert!(asks.iter().any(|action| {
            action.label(&game) == "向卖饼大叔询问大饼"
                && matches!(
                    action,
                    Action::AskNpc { npc, topic }
                        if npc.as_str() == "city.npc.caker" && topic == "大饼"
                )
        }));
        game.perform(Action::AskNpc {
            npc: NpcId::from("city.npc.caker"),
            topic: "大饼".into(),
        });
        assert_eq!(
            game.logs.last().unwrap(),
            "卖饼大叔摆出货单：雪花糕 3两银子。"
        );

        game.location = LocationId::from("canyon.camp6");
        game.perform(Action::AskNpc {
            npc: NpcId::from("canyon.npc.captain"),
            topic: "黄石隘口".into(),
        });
        assert_eq!(game.logs.last().unwrap(), "大队长说道：知道了，就快缴钱。");

        game.location = LocationId::from("snow.herbshop");
        game.player.qi = game.player.max_qi;
        game.perform(Action::AskNpc {
            npc: NpcId::from("snow.npc.herbalist"),
            topic: "治伤".into(),
        });
        assert!(game.logs.last().unwrap().contains("气色很好"));
        game.player.qi = game.player.max_qi * 95 / 100;
        game.perform(Action::AskNpc {
            npc: NpcId::from("snow.npc.herbalist"),
            topic: "疗伤".into(),
        });
        assert!(game.logs.last().unwrap().contains("皮肉小伤"));
        game.player.qi = game.player.max_qi / 2;
        game.perform(Action::AskNpc {
            npc: NpcId::from("snow.npc.herbalist"),
            topic: "开药".into(),
        });
        assert!(game.logs.last().unwrap().contains("没有开出药方"));
    }

    #[test]
    fn source_noop_and_multiplayer_inquiries_remain_unavailable() {
        let mut game = Game::new();
        for (location, expected_topics) in [
            ("snow.school1", 2),
            ("snow.school", 2),
            ("snow.postoffice", 1),
        ] {
            game.location = LocationId::from(location);
            assert_eq!(
                game.available_actions()
                    .iter()
                    .filter(|action| matches!(action, Action::AskNpc { .. }))
                    .count(),
                expected_topics
            );
        }
        game.location = LocationId::from("snow.school");
        assert!(game.available_actions().iter().all(|action| {
            !matches!(action, Action::AskNpc { topic, .. } if topic == "刘安禄")
        }));
    }

    #[test]
    fn teacher_requires_full_tuition_before_teaching_literacy() {
        let mut game = Game::new();
        game.location = LocationId::from("snow.school");
        game.perform(Action::AskNpc {
            npc: NpcId::from(SNOW_TEACHER_ID),
            topic: "学费".into(),
        });
        assert!(game.logs.last().unwrap().contains("五两银子"));

        let rations = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == items::DRY_RATIONS_ID)
            .unwrap()
            .instance_id;
        game.perform(Action::GiveItem {
            instance_id: rations,
            npc: NpcId::from(SNOW_TEACHER_ID),
        });
        assert!(game.player.item(rations).is_some());
        assert!(!game.snow_teacher_paid);

        let medicine = game.add_inventory_item(ItemId::from(items::WOUND_MEDICINE_ID), 1);
        game.perform(Action::GiveItem {
            instance_id: medicine,
            npc: NpcId::from(SNOW_TEACHER_ID),
        });
        assert!(game.player.item(medicine).is_none());
        assert!(game.snow_teacher_paid);

        game.player.spirit = 100;
        let lesson = Action::LearnFromNpc {
            skill: SkillId::from("literate"),
            npc: NpcId::from(SNOW_TEACHER_ID),
        };
        assert!(game.available_actions().contains(&lesson));
        game.perform(lesson);
        assert!(game.player.skill_by_id("literate").is_some());
        assert_eq!(game.player.learned_points, 1);
    }

    #[test]
    fn snow_guard_reveals_into_forced_combat_and_drops_blade_manual() {
        let mut game = Game::new();
        game.location = LocationId::from("snow.school1");
        let inquiry = Action::AskNpc {
            npc: NpcId::from(SNOW_GUARD_ID),
            topic: "血手刘三".into(),
        };

        game.perform(inquiry.clone());
        assert!(!game.snow_guard_revealed);
        assert!(game.logs.last().unwrap().contains("不肯回答"));

        game.player.combat_experience = 20_000;
        game.perform(inquiry);
        assert!(game.snow_guard_revealed);
        let combat = match std::mem::replace(&mut game.activity, Activity::Idle) {
            Activity::Fighting(combat) => combat,
            _ => panic!("identity reveal must start combat"),
        };
        assert_eq!(combat.enemy, EnemyKind::BloodHandLiuSan);
        assert_eq!(combat.mode, CombatMode::Lethal);
        game.win_combat(combat);

        assert!(game.snow_guard_defeated);
        assert!(
            game.ground_items[&game.location]
                .iter()
                .any(|item| item.item_id.as_str() == "snow.npc.obj.blade_book")
        );
        assert!(game.available_actions().iter().all(|action| {
            !matches!(action, Action::Talk(npc) | Action::AskNpc { npc, .. } if npc.as_str() == SNOW_GUARD_ID)
        }));
    }

    #[test]
    fn trader_buys_and_sells_catalog_items() {
        let mut game = Game::new();
        game.location = LocationId::from("village.road2");
        let before = game.player.money_value();
        let dagger_id = ItemId::from("obj.weapon.dagger");

        game.perform(Action::BuyItem {
            item_id: dagger_id.clone(),
            npc: NpcId::from(TRADER_ID),
        });
        assert!(game.player.has_item(&dagger_id));
        assert_eq!(game.player.money_value(), before - 50);

        let dagger = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id == dagger_id)
            .unwrap()
            .instance_id;
        game.perform(Action::SellItem(dagger));
        assert!(!game.player.has_item(&dagger_id));
        assert_eq!(game.player.money_value(), before - 25);
    }

    #[test]
    fn m4_multi_npc_rooms_bind_stock_and_prices_to_each_vendor() {
        let mut game = Game::new();
        game.location = LocationId::from("city.bridge");
        let buy_actions: Vec<_> = game
            .available_actions()
            .into_iter()
            .filter(|action| matches!(action, Action::BuyItem { .. }))
            .collect();
        assert_eq!(buy_actions.len(), 3);
        assert!(buy_actions.iter().any(|action| matches!(
            action,
            Action::BuyItem { item_id, npc }
                if item_id.as_str() == "city.npc.obj.cake"
                    && npc.as_str() == "city.npc.caker"
        )));
        assert!(buy_actions.iter().any(|action| matches!(
            action,
            Action::BuyItem { item_id, npc }
                if item_id.as_str() == "obj.example.dumpling"
                    && npc.as_str() == "city.npc.dumpling_seller"
        )));

        let before = game.player.money_value();
        game.perform(Action::BuyItem {
            item_id: ItemId::from("city.npc.obj.cake"),
            npc: NpcId::from("city.npc.caker"),
        });
        assert!(game.player.has_item(&ItemId::from("city.npc.obj.cake")));
        assert_eq!(game.player.money_value(), before - 300);

        game.location = LocationId::from("snow.herbshop");
        game.perform(Action::BuyItem {
            item_id: ItemId::from(items::WOUND_MEDICINE_ID),
            npc: NpcId::from("snow.npc.herbalist"),
        });
        assert!(
            game.player
                .has_item(&ItemId::from(items::WOUND_MEDICINE_ID))
        );
        assert_eq!(game.player.money_value(), before - 2_300);
    }

    #[test]
    fn items_can_be_given_to_the_current_npc() {
        let mut game = Game::new();
        let rations = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == items::DRY_RATIONS_ID)
            .unwrap()
            .instance_id;

        game.perform(Action::GiveItem {
            instance_id: rations,
            npc: NpcId::from(OLD_LIU_ID),
        });

        assert!(!game.player.has_item(&ItemId::from(items::DRY_RATIONS_ID)));
        assert!(game.logs.last().unwrap().contains("刘老农"));
    }

    #[test]
    fn stackable_items_share_an_instance_and_contribute_weight() {
        let mut game = Game::new();
        let ration_id = ItemId::from(items::DRY_RATIONS_ID);
        let original_instance = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id == ration_id)
            .unwrap()
            .instance_id;

        game.add_inventory_item(ration_id.clone(), 2);

        let rations: Vec<_> = game
            .player
            .inventory
            .iter()
            .filter(|item| item.item_id == ration_id)
            .collect();
        assert_eq!(rations.len(), 1);
        assert_eq!(rations[0].instance_id, original_instance);
        assert_eq!(rations[0].quantity, 5);
        assert_eq!(game.player.carried_weight(), 5_500);
    }

    #[test]
    fn equipment_can_be_dropped_and_picked_up_again() {
        let mut game = Game::new();
        let cloth_instance = game
            .player
            .equipped(EquipmentSlot::Torso)
            .unwrap()
            .instance_id;

        game.perform(Action::UnequipItem(EquipmentSlot::Torso));
        game.perform(Action::DropItem(cloth_instance));
        assert!(game.player.item(cloth_instance).is_none());
        assert_eq!(
            game.ground_items[&game.location][0].item_id.as_str(),
            items::CLOTH_ID
        );

        game.perform(Action::PickUpItem(cloth_instance));
        assert!(game.player.item(cloth_instance).is_some());
        assert!(!game.ground_items.contains_key(&game.location));
    }

    #[test]
    fn equipped_weapon_and_armor_lose_durability_in_combat() {
        let mut game = Game::new();
        let sword = game.add_inventory_item(ItemId::from(items::HENGBING_SWORD_ID), 1);
        game.equip_item(sword);
        game.location = LocationId::from(content::PINE_FOREST);

        game.perform(Action::Fight(EnemyKind::Bandit));
        game.tick();

        assert_eq!(game.player.item(sword).unwrap().durability, Some(99));
        assert_eq!(
            game.player
                .equipped(EquipmentSlot::Torso)
                .unwrap()
                .durability,
            Some(99)
        );
    }

    #[test]
    fn food_and_liquid_preserve_original_portions_and_supplies() {
        let mut game = Game::new();
        game.player.food = 0;
        game.player.max_food = 1_000;
        game.player.water = 0;
        game.player.max_water = 1_000;
        let rations = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == items::DRY_RATIONS_ID)
            .unwrap()
            .instance_id;

        game.perform(Action::ConsumeItem(rations));
        assert_eq!(game.player.food, 20);
        assert_eq!(game.player.item(rations).unwrap().quantity, 2);

        let melon = game.add_inventory_item(ItemId::from(items::WATER_MELON_ID), 1);
        game.perform(Action::ConsumeItem(melon));
        assert_eq!(game.player.food, 40);
        assert_eq!(game.player.water, 40);
        assert_eq!(game.player.item(melon).unwrap().remaining_uses, Some(7));
        for _ in 0..7 {
            game.perform(Action::ConsumeItem(melon));
        }
        let melon_skin = game.player.item(melon).unwrap();
        assert_eq!(melon_skin.remaining_uses, Some(0));
        assert_eq!(melon_skin.display_name(), "西瓜皮");
        assert_eq!(melon_skin.total_weight(), 150);
        assert_eq!(melon_skin.unit_value(), 0);

        let wineskin = game.add_inventory_item(ItemId::from("obj.example.wineskin"), 1);
        game.perform(Action::ConsumeItem(wineskin));
        assert_eq!(game.player.water, 350);
        assert_eq!(game.player.item(wineskin).unwrap().remaining_uses, Some(14));
        assert_eq!(
            game.player
                .condition(ConditionKind::Drunk)
                .unwrap()
                .duration,
            6
        );
    }

    #[test]
    fn powder_mixes_into_liquid_and_applies_slumber_effect() {
        let mut game = Game::new();
        game.player.water = 0;
        let powder = game.add_inventory_item(ItemId::from(items::SLUMBER_DRUG_ID), 1);
        let wineskin = game.add_inventory_item(ItemId::from("obj.example.wineskin"), 1);

        game.perform(Action::MixIntoLiquid {
            powder_instance_id: powder,
            liquid_instance_id: wineskin,
        });
        assert!(game.player.item(powder).is_none());
        assert_eq!(game.player.item(wineskin).unwrap().slumber_effect, 100);

        game.perform(Action::ConsumeItem(wineskin));
        assert_eq!(
            game.player
                .condition(ConditionKind::Slumber)
                .unwrap()
                .duration,
            100
        );
    }

    #[test]
    fn bandage_wound_medicine_and_antidote_update_persistent_status() {
        let mut game = Game::new();
        game.player.essence = 40;
        let bandage = game.add_inventory_item(ItemId::from(items::BANDAGE_ID), 1);
        game.perform(Action::ApplyItem(bandage));
        assert_eq!(game.player.item(bandage).unwrap().remaining_uses, Some(1));
        assert_eq!(
            game.player
                .condition(ConditionKind::Bandaged)
                .unwrap()
                .duration,
            40
        );
        game.tick();
        assert_eq!(game.player.essence, 44);

        let medicine = game.add_inventory_item(ItemId::from(items::WOUND_MEDICINE_ID), 1);
        game.perform(Action::ApplyItem(medicine));
        assert_eq!(game.player.essence, 64);
        assert!(game.player.item(medicine).is_none());

        game.player.set_condition(ConditionKind::SnakePoison, 2, 10);
        let antidote = game.add_inventory_item(ItemId::from(items::SNAKE_MEDICINE_ID), 1);
        game.perform(Action::ApplyItem(antidote));
        assert_eq!(
            game.player
                .condition(ConditionKind::SnakePoison)
                .unwrap()
                .duration,
            1
        );
        assert!(game.player.item(antidote).is_none());
        game.tick();
        assert!(game.player.condition(ConditionKind::SnakePoison).is_none());
    }

    #[test]
    fn thirst_stops_natural_recovery() {
        let mut game = Game::new();
        game.player.essence = 50;
        game.player.qi = 40;
        game.player.spirit = 30;
        game.player.water = 0;
        game.tick();
        assert_eq!(game.player.essence, 50);
        assert_eq!(game.player.qi, 40);
        assert_eq!(game.player.spirit, 30);
    }

    #[test]
    fn training_consumes_energy_and_builds_progress() {
        let mut game = Game::new();
        game.location = LocationId::from(content::GARDEN);
        game.perform(Action::Train(SkillId::from(FORCE_ID)));
        let before = game.player.skill_by_id(FORCE_ID).unwrap().progress;
        game.tick();
        assert!(game.player.essence < game.player.max_essence);
        assert!(game.player.skill_by_id(FORCE_ID).unwrap().progress > before);
    }

    #[test]
    fn quest_advances_through_conversation_and_rescue() {
        let mut game = Game::new();
        game.perform(Action::Talk(NpcId::from(OLD_LIU_ID)));
        assert_eq!(game.quest, QuestStage::FindJuan);

        game.location = LocationId::from(content::PINE_FOREST);
        game.player.strength = 100;
        game.perform(Action::Fight(EnemyKind::Bandit));
        while matches!(game.activity, Activity::Fighting(_)) {
            game.tick();
        }
        assert_eq!(game.quest, QuestStage::ReturnHome);

        game.location = LocationId::from(content::LIU_HOME);
        game.perform(Action::Talk(NpcId::from(OLD_LIU_ID)));
        assert_eq!(game.quest, QuestStage::Complete);
        assert_eq!(
            game.player
                .equipped(EquipmentSlot::Weapon)
                .unwrap()
                .item_id
                .as_str(),
            items::HENGBING_SWORD_ID
        );
    }

    #[test]
    fn representative_swordsman_and_bonze_builds_can_be_trained() {
        let mut swordsman = Game::new();
        swordsman.location = LocationId::from(content::SNOW_TOWN);
        swordsman.perform(Action::BecomeApprentice("swordsman".into()));
        swordsman.perform(Action::LearnSkill {
            skill: SkillId::from("fonxanforce"),
            teacher: "swordsman".into(),
        });
        swordsman
            .player
            .skill_mut(&SkillId::from("fonxanforce"))
            .level = 10;
        swordsman.perform(Action::MapSkill {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("fonxanforce"),
        });
        swordsman.player.max_force = 100;
        swordsman.player.spirit = swordsman.player.max_spirit;
        let sword = swordsman.add_inventory_item(ItemId::from("obj.weapon.longsword"), 1);
        swordsman.equip_item(sword);
        swordsman.perform(Action::LearnSkill {
            skill: SkillId::from("fonxansword"),
            teacher: "swordsman".into(),
        });
        assert!(swordsman.player.skill_by_id("fonxansword").is_some());
        assert_eq!(
            swordsman.player.mapped_skill(FORCE_ID).map(SkillId::as_str),
            Some("fonxanforce")
        );

        let mut bonze = Game::new();
        bonze.location = LocationId::from(content::TEMPLE_YARD);
        bonze.perform(Action::BecomeApprentice("bonze".into()));
        bonze.perform(Action::LearnSkill {
            skill: SkillId::from("buddhism"),
            teacher: "bonze".into(),
        });
        bonze.player.skill_mut(&SkillId::from("buddhism")).level = 20;
        bonze.player.spirit = bonze.player.max_spirit;
        bonze.perform(Action::LearnSkill {
            skill: SkillId::from("lotusforce"),
            teacher: "bonze".into(),
        });
        bonze.player.skill_mut(&SkillId::from("lotusforce")).level = 10;
        bonze.perform(Action::MapSkill {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("lotusforce"),
        });
        bonze.player.spirit = bonze.player.max_spirit;
        bonze.perform(Action::LearnSkill {
            skill: SkillId::from(MAGIC_ID),
            teacher: "bonze".into(),
        });
        bonze.player.skill_mut(&SkillId::from(MAGIC_ID)).level = 10;
        bonze.player.spirit = bonze.player.max_spirit;
        bonze.perform(Action::LearnSkill {
            skill: SkillId::from("essencemagic"),
            teacher: "bonze".into(),
        });
        assert!(bonze.player.skill_by_id("essencemagic").is_some());
        assert_eq!(bonze.player.faction.as_deref(), Some("山烟寺"));
    }

    #[test]
    fn apprenticeship_and_learning_use_teacher_limits_and_potential() {
        let mut game = Game::new();
        game.location = LocationId::from(content::SNOW_TOWN);
        let apprentice = Action::BecomeApprentice("fighter".into());
        assert!(game.available_actions().contains(&apprentice));
        game.perform(apprentice);
        assert_eq!(game.player.teacher.as_deref(), Some("fighter"));
        assert_eq!(game.player.faction.as_deref(), Some("天邪派"));

        let spirit_before = game.player.spirit;
        let learn = Action::LearnSkill {
            skill: SkillId::from("celestial"),
            teacher: "fighter".into(),
        };
        assert!(game.available_actions().contains(&learn));
        game.perform(learn);
        assert!(game.player.skill_by_id("celestial").is_some());
        assert_eq!(game.player.learned_points, 1);
        assert!(game.player.spirit < spirit_before);
    }

    #[test]
    fn practice_uses_mapped_basic_skill_and_original_resource_cost() {
        let mut game = Game::new();
        let skill_id = SkillId::from(LIUH_KEN_ID);
        let progress_before = game.player.skill(&skill_id).progress;
        let qi_before = game.player.qi;

        game.perform(Action::PracticeSkill(skill_id.clone()));

        assert_eq!(game.player.qi, qi_before - 30);
        assert!(game.player.skill(&skill_id).progress > progress_before);
        assert!(game.player.skill(&skill_id).level <= game.player.skill_level(UNARMED_ID));
    }

    #[test]
    fn catalog_book_teaches_skill_with_literacy_and_experience_gates() {
        let mut game = Game::new();
        game.player.skills.push(Skill::new("literate", 20));
        let manual = game.add_inventory_item(ItemId::from(items::PARRY_MANUAL_ID), 1);
        let progress_before = game.player.skill_by_id(PARRY_ID).unwrap().progress;
        let spirit_before = game.player.spirit;

        game.perform(Action::StudyItem(manual));

        assert!(game.player.skill_by_id(PARRY_ID).unwrap().progress > progress_before);
        assert!(game.player.spirit < spirit_before);
    }

    #[test]
    fn mapping_internal_skill_resets_and_rebuilds_force() {
        let mut game = Game::new();
        game.player.skills.push(Skill::new("celestial", 20));
        let mapping = Action::MapSkill {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("celestial"),
        };
        assert!(game.available_actions().contains(&mapping));
        game.perform(mapping);
        assert_eq!(game.player.force, 0);

        game.perform(Action::Cultivate(CultivationKind::Exercise));
        assert!(game.player.force > 0);
        assert_eq!(game.player.qi, game.player.max_qi - 30);
    }

    #[test]
    fn lethal_combat_cannot_surrender_and_creates_wanted_state() {
        let mut game = Game::new();
        game.location = LocationId::from(content::TEMPLE_YARD);
        game.player.strength = 200;
        game.perform(Action::Kill(EnemyKind::TempleDisciple));

        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Lethal,
                ..
            })
        ));
        let combat_actions = game.available_actions();
        assert!(
            combat_actions
                .iter()
                .any(|action| matches!(action, Action::Flee { .. }))
        );
        assert!(!combat_actions.contains(&Action::Surrender));

        while matches!(game.activity, Activity::Fighting(_)) {
            game.tick();
        }
        assert_eq!(game.player.bellicosity, 1);
        assert_eq!(game.player.wanted, 1);
        assert!(game.player.combat_experience > 5_000);
    }

    #[test]
    fn fleeing_ends_combat_and_moves_through_a_real_exit() {
        let mut game = Game::new();
        game.location = LocationId::from(content::TEMPLE_YARD);
        game.player.reputation = 20;
        game.perform(Action::Fight(EnemyKind::TempleDisciple));
        let flee = game
            .available_actions()
            .into_iter()
            .find(|action| matches!(action, Action::Flee { .. }))
            .unwrap();
        game.perform(flee);

        assert_eq!(game.activity, Activity::Idle);
        assert_eq!(game.location, LocationId::from(content::MOUNTAIN_PATH));
        assert_eq!(game.player.reputation, 15);
    }

    #[test]
    fn recovery_and_conversion_techniques_preserve_source_costs() {
        let mut game = Game::new();
        game.player.ensure_skill(SkillId::from("lotusforce"));
        game.player.skill_mut(&SkillId::from("lotusforce")).level = 50;
        game.player.skill_mappings.push(SkillMapping {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("lotusforce"),
        });
        game.player.essence = 60;
        game.player.force = 180;
        game.player.max_force = 100;

        game.perform(Action::UseTechnique(TechniqueKind::LotusHeal));
        assert_eq!(game.player.essence, 80);
        assert_eq!(game.player.force, 130);

        game.player.ensure_skill(SkillId::from("gouyee"));
        game.player.skill_mut(&SkillId::from("gouyee")).level = 50;
        game.player
            .skill_mappings
            .retain(|mapping| mapping.usage.as_str() != FORCE_ID);
        game.player.skill_mappings.push(SkillMapping {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("gouyee"),
        });
        game.player.mana = 0;
        game.player.max_mana = 100;
        let spirit_before = game.player.spirit;
        game.perform(Action::UseTechnique(TechniqueKind::Concentrate));
        assert_eq!(game.player.mana, 20);
        assert_eq!(game.player.force, 100);
        assert_eq!(game.player.spirit, spirit_before - 10);
    }

    #[test]
    fn base_force_recovery_uses_original_fixed_cost() {
        let mut game = Game::new();
        game.player.qi = 50;
        game.player.force = 50;

        game.perform(Action::UseTechnique(TechniqueKind::RecoverQi));

        assert_eq!(game.player.qi, 61);
        assert_eq!(game.player.force, 30);
    }

    #[test]
    fn source_skill_level_hooks_update_attributes_and_bellicosity() {
        let mut game = Game::new();
        game.player.constitution = 1;
        let force = SkillId::from(FORCE_ID);
        game.player.skill_mut(&force).level = 48;
        game.gain_skill_progress(force, 49_u32.pow(2));
        assert_eq!(game.player.constitution, 3);

        let sword = SkillId::from(SIX_CHAOS_SWORD_ID);
        game.player.skill_mut(&sword).level = 9;
        game.gain_skill_progress(sword, 10_u32.pow(2));
        assert_eq!(game.player.bellicosity, 1_000);

        game.player.ensure_skill(SkillId::from("nine-moon"));
        game.gain_skill_progress(SkillId::from("nine-moon"), 1);
        assert_eq!(game.player.bellicosity, 3_000);
    }

    #[test]
    fn combat_techniques_modify_the_active_exchange() {
        let mut game = Game::new();
        game.player.ensure_skill(SkillId::from("celestial"));
        game.player.skill_mut(&SkillId::from("celestial")).level = 60;
        game.player.skill_mappings.push(SkillMapping {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("celestial"),
        });
        game.player.force = 300;
        game.location = LocationId::from(content::PINE_FOREST);
        game.perform(Action::Fight(EnemyKind::Bandit));

        game.perform(Action::UseTechnique(TechniqueKind::PowerUp));
        let Activity::Fighting(combat) = &game.activity else {
            panic!("combat should continue");
        };
        assert_eq!(combat.attack_bonus, 20);
        assert_eq!(combat.dodge_bonus, 20);
        assert!(combat.power_up_active);
        assert_eq!(game.player.force, 200);
        assert_eq!(game.player.bellicosity, 130);

        game.perform(Action::UseTechnique(TechniqueKind::PowerUp));
        assert_eq!(game.player.force, 200);
        assert_eq!(game.player.bellicosity, 130);
    }

    #[test]
    fn unlearned_weapon_usage_is_created_by_combat_growth() {
        let mut game = Game::new();
        let dagger = game.add_inventory_item(ItemId::from("obj.weapon.dagger"), 1);
        game.equip_item(dagger);
        assert_eq!(game.player.skill_level("dagger"), 0);
        game.location = LocationId::from(content::PINE_FOREST);
        game.perform(Action::Fight(EnemyKind::Bandit));

        game.tick();

        assert!(game.player.skill_by_id("dagger").is_some());
    }

    #[test]
    fn specialized_techniques_require_an_active_skill_mapping() {
        let mut game = Game::new();
        game.player.ensure_skill(SkillId::from("celestial"));
        game.player.skill_mut(&SkillId::from("celestial")).level = 60;
        game.player.force = 300;
        game.location = LocationId::from(content::PINE_FOREST);
        game.perform(Action::Fight(EnemyKind::Bandit));

        assert!(
            !game
                .available_actions()
                .contains(&Action::UseTechnique(TechniqueKind::PowerUp))
        );
        game.perform(Action::UseTechnique(TechniqueKind::PowerUp));
        assert_eq!(game.player.force, 300);
    }

    #[test]
    fn healing_techniques_reject_mortal_wounds() {
        let mut game = Game::new();
        game.player.ensure_skill(SkillId::from("lotusforce"));
        game.player.skill_mut(&SkillId::from("lotusforce")).level = 50;
        game.player.skill_mappings.push(SkillMapping {
            usage: SkillId::from(FORCE_ID),
            skill: SkillId::from("lotusforce"),
        });
        game.player.essence = 40;
        game.player.force = 180;

        game.perform(Action::UseTechnique(TechniqueKind::LotusHeal));

        assert_eq!(game.player.essence, 40);
        assert_eq!(game.player.force, 180);
    }

    #[test]
    fn necromancy_bolts_turn_a_spar_into_lethal_combat() {
        let mut game = Game::new();
        game.player.ensure_skill(SkillId::from("necromancy"));
        game.player.skill_mut(&SkillId::from("necromancy")).level = 45;
        game.player.ensure_skill(SkillId::from(SPELLS_ID));
        game.player.skill_mappings.push(SkillMapping {
            usage: SkillId::from(SPELLS_ID),
            skill: SkillId::from("necromancy"),
        });
        game.player.mana = 100;
        game.player.max_mana = 100;
        game.location = LocationId::from(content::PINE_FOREST);
        game.perform(Action::Fight(EnemyKind::Bandit));

        game.perform(Action::UseTechnique(TechniqueKind::NetherBolt));
        let Activity::Fighting(combat) = game.activity else {
            panic!("combat should continue");
        };
        assert_eq!(combat.mode, CombatMode::Lethal);
        assert_eq!(combat.health, combat.max_health - 25);
        assert_eq!(game.player.mana, 75);
        assert!(!game.available_actions().contains(&Action::Surrender));
    }

    #[test]
    fn surrender_ends_only_nonlethal_combat() {
        let mut game = Game::new();
        game.location = LocationId::from(content::PINE_FOREST);
        game.perform(Action::Fight(EnemyKind::Bandit));
        game.perform(Action::Surrender);
        assert_eq!(game.activity, Activity::Idle);
        assert_eq!(game.player.reputation, 0);
    }

    #[test]
    fn canyon_access_and_seal_exchange_chain_reaches_the_black_market_reward() {
        let mut game = Game::new();
        game.player.set_money_value(100_000);
        game.location = LocationId::from(content::CANYON_CAMP8);
        game.perform(Action::OfferMoney {
            amount: 800,
            npc: NpcId::from(crate::npcs::CANYON_ADVISER_ID),
        });
        assert!(game.canyon_secret_clue);

        game.location = LocationId::from(content::CANYON_SECRET_WALL);
        game.perform(Action::Interact(InteractionKind::SwearCanyonSecret));
        assert_eq!(game.location.as_str(), content::CANYON_BLACK_MARKET);
        assert!(!game.canyon_secret_clue);

        game.perform(Action::OfferMoney {
            amount: 30_000,
            npc: NpcId::from(crate::npcs::CANYON_SELLER_ID),
        });
        assert!(
            game.player
                .has_item(&ItemId::from("canyon.npc.obj.fake_seal"))
        );

        game.location = LocationId::from("canyon.camp6");
        game.perform(Action::OfferMoney {
            amount: 3_000,
            npc: NpcId::from(crate::npcs::CANYON_CAPTAIN_ID),
        });
        game.location = LocationId::from(content::CANYON_CAMP7);
        assert!(game.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::CANYON_CAMP8
        )));
        game.location = LocationId::from(content::CANYON_CAMP8);

        let fake_seal = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == "canyon.npc.obj.fake_seal")
            .unwrap()
            .instance_id;
        game.perform(Action::GiveItem {
            instance_id: fake_seal,
            npc: NpcId::from(crate::npcs::CANYON_GENERAL_ID),
        });
        assert_eq!(game.location.as_str(), content::CANYON_CAMP2);
        assert!(game.player.item(fake_seal).is_some());

        game.location = LocationId::from(content::CANYON_BLACK_MARKET);
        game.perform(Action::GiveItem {
            instance_id: fake_seal,
            npc: NpcId::from(crate::npcs::CANYON_SELLER_ID),
        });
        let real_seal = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == "canyon.npc.obj.seal")
            .unwrap()
            .instance_id;
        game.location = LocationId::from(content::CANYON_CAMP8);
        game.perform(Action::GiveItem {
            instance_id: real_seal,
            npc: NpcId::from(crate::npcs::CANYON_GENERAL_ID),
        });
        assert!(
            game.player
                .has_item(&ItemId::from("canyon.npc.obj.old_sword"))
        );
        assert!(game.canyon_general_rewarded);
    }

    #[test]
    fn city_gifts_control_the_inn_and_one_use_manor_entrances() {
        let mut game = Game::new();
        game.location = LocationId::from(content::CITY_INN);
        let rations = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == items::DRY_RATIONS_ID)
            .unwrap()
            .instance_id;
        game.perform(Action::GiveItem {
            instance_id: rations,
            npc: NpcId::from(crate::npcs::CITY_WAITER_ID),
        });
        assert!(!game.city_inn_access);
        assert!(
            game.ground_items[&LocationId::from(content::CITY_INN)]
                .iter()
                .any(|item| item.item_id.as_str() == items::DRY_RATIONS_ID)
        );

        game.player.set_money_value(100_000);
        game.perform(Action::OfferMoney {
            amount: 1_000,
            npc: NpcId::from(crate::npcs::CITY_WAITER_ID),
        });
        assert!(game.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::CITY_INN_UPSTAIRS
        )));

        game.location = LocationId::from(content::CITY_MANOR_GATE);
        assert!(game.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::CITY_MANOR_YARD
        )));
        game.perform(Action::OfferMoney {
            amount: 30_000,
            npc: NpcId::from(crate::npcs::CITY_SHANGSHU_GUARD_ID),
        });
        let enter = game
            .available_actions()
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Move { target, .. } if target.as_str() == content::CITY_MANOR_YARD
                )
            })
            .unwrap();
        game.perform(enter);
        assert_eq!(game.location.as_str(), content::CITY_MANOR_YARD);
        assert!(!game.city_manor_pass);

        game.location = LocationId::from(content::CITY_MANOR_GATE);
        assert!(game.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::CITY_MANOR_YARD
        )));
    }

    #[test]
    fn canyon_chain_connects_the_village_and_snow_map_components() {
        let mut game = Game::new();
        game.location = LocationId::from(content::CANYON_FOOT);
        game.player.essence = 100;
        game.player.qi = 100;
        game.player.spirit = 100;

        assert!(
            game.available_actions()
                .contains(&Action::Interact(InteractionKind::ClimbCanyonChain))
        );
        game.perform(Action::Interact(InteractionKind::ClimbCanyonChain));
        assert_eq!(game.location.as_str(), content::CANYON_ROAD);
        assert_eq!(
            (game.player.essence, game.player.qi, game.player.spirit),
            (70, 60, 80)
        );

        game.perform(Action::Interact(InteractionKind::ClimbCanyonChain));
        assert_eq!(game.location.as_str(), content::CANYON_FOOT);
        assert_eq!(
            (game.player.essence, game.player.qi, game.player.spirit),
            (50, 30, 70)
        );
    }

    #[test]
    fn city_wall_commands_preserve_both_original_destinations() {
        let mut game = Game::new();
        game.location = LocationId::from(content::CITY_STREET3);
        game.perform(Action::Interact(InteractionKind::ClimbCityWall));
        assert_eq!(game.location.as_str(), content::CITY_WALL);

        game.perform(Action::Interact(InteractionKind::JumpIntoCityManor));
        assert_eq!(game.location.as_str(), content::CITY_MANOR_RUIN);

        game.location = LocationId::from(content::CITY_WALL);
        game.perform(Action::Interact(InteractionKind::JumpOutsideCityWall));
        assert_eq!(game.location.as_str(), content::CITY_STREET3);
    }
}
