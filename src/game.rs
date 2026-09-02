use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    content::{self, world},
    items::{
        self, EquipmentSlot, ItemId, ItemInstance, LegacyItemKind, TalismanInscription,
        TalismanKind, items,
    },
    npcs::{
        CHOYIN_GIRL_ID, CHOYIN_HOTEL_GUARD_ID, CHOYIN_LION_ID, CHOYIN_MAGISTRATE_ID,
        CHOYIN_POLICE_ID, CHOYIN_YOUNG_MAN_ID, CHUENYU_OLD_LIU_ID, CHUENYU_XIAO_JUAN_ID,
        CHUENYU_XIAO_JUAN_PLACED_ID, CITY_SHANGSHU_PATROL_ELITE_ID, CITY_SHANGSHU_PATROL_ID,
        CLOUD_B_HEADER_ID, CLOUD_GOD_ID, FARM_WOMAN_ID, FISHER_ID, FLOWER_GIRL_ID, GREEN_SHEN_ID,
        MELONER_ID, NpcApprenticeshipPolicy, NpcFightPolicy, NpcId, OLD_LIU_ID, ObjectExchangeKind,
        SNOW_FIST_TRAINER_ID, SNOW_GIRL_ID, SNOW_GUARD_ID, SNOW_SCAVENGER_ID, ScriptedInquiryKind,
        TEA_SELLER_ID, TEMPLE_MASTER_ID, TEMPLE_OLD_TAOIST_ID, TEMPLE_PROTECTOR_ID,
        TEMPLE_TRAINER_ID, TRADER_ID, WATERFOG_ELITE_GUARD_ID, XIAO_JUAN_ID, npcs,
    },
    quests,
    skills::{
        self, DODGE_ID, FORCE_ID, LIUH_KEN_ID, MAGIC_ID, MOVE_ID, PARRY_ID, PYROBAT_STEPS_ID,
        SIX_CHAOS_SWORD_ID, SPELLS_ID, SWORD_ID, SkillId, TechniqueKind, UNARMED_ID, skills,
    },
};

const LOG_LIMIT: usize = 80;
const SAVE_VERSION: u32 = 31;
const M8_WORLD_TIME_SAVE_VERSION: u32 = 26;
const M8_CORPSE_LIFECYCLE_SAVE_VERSION: u32 = 27;
const M8_NPC_POSITION_SAVE_VERSION: u32 = 28;
const M8_CHOYIN_JUSTICE_SAVE_VERSION: u32 = 29;
const M8_DYNAMIC_QUEST_SAVE_VERSION: u32 = 30;
const M8_FINALIZATION_SAVE_VERSION: u32 = 31;
const GAME_MINUTES_PER_TICK: u64 = 10;
const DAY_MINUTES: u64 = 24 * 60;
const OFFLINE_REAL_SECONDS_PER_TICK: u64 = 60;
const MAX_OFFLINE_TICKS: u64 = 8 * 60;
const CHOYIN_BRIBE_AMOUNT: u64 = 100_000;
const CORPSE_ITEM_ID: &str = "obj.corpse";
const CORPSE_FRESH_DECAY_TICKS: u64 = 120;
const CORPSE_ROTTEN_DECAY_TICKS: u64 = 120;
const CORPSE_BONE_DECAY_TICKS: u64 = 60;
const DYNAMIC_QUEST_FACTOR: u32 = 10;
const DYNAMIC_QUEST_MIN_COMBAT_EXPERIENCE: u64 = 1_000;
const STEAL_DELAY_TICKS: u8 = 3;
const ZOMBIE_HAUNT_DURATION_TICKS: u64 = 60;
const NPC_RESPAWN_DELAY_TICKS: u64 = 72;

#[derive(Debug, Clone, Copy)]
struct DayPhase {
    duration_minutes: u16,
    name: &'static str,
    transition_message: &'static str,
    outdoor_description: &'static str,
}

const DAY_PHASES: [DayPhase; 8] = [
    DayPhase {
        duration_minutes: 240,
        name: "拂晓",
        transition_message: "东方的天空中开始出现一丝微曦。",
        outdoor_description: "东方的天空已逐渐发白",
    },
    DayPhase {
        duration_minutes: 120,
        name: "日出",
        transition_message: "太阳从东方的地平线升起了。",
        outdoor_description: "太阳刚从东方的地平线升起",
    },
    DayPhase {
        duration_minutes: 180,
        name: "清晨",
        transition_message: "太阳已经高高地挂在东方的天空中。",
        outdoor_description: "太阳正高挂在东方的天空中",
    },
    DayPhase {
        duration_minutes: 180,
        name: "正午",
        transition_message: "已经是正午了，太阳从你正上方照耀著大地。",
        outdoor_description: "现在是正午时分，太阳高挂在你的头顶正上方",
    },
    DayPhase {
        duration_minutes: 180,
        name: "午后",
        transition_message: "太阳开始从西方的天空中慢慢西沉。",
        outdoor_description: "太阳正高挂在西方的天空中",
    },
    DayPhase {
        duration_minutes: 180,
        name: "黄昏",
        transition_message: "傍晚了，太阳的馀晖将西方的天空映成一片火红。",
        outdoor_description: "一轮火红的夕阳正徘徊在西方的地平线上",
    },
    DayPhase {
        duration_minutes: 120,
        name: "夜晚",
        transition_message: "夜晚降临了。",
        outdoor_description: "夜幕笼罩著大地",
    },
    DayPhase {
        duration_minutes: 240,
        name: "午夜",
        transition_message: "已经是午夜了。",
        outdoor_description: "夜幕低垂，满天繁星",
    },
];
const CHOYIN_DONATION_BOX_IDS: [&str; 2] = ["choyin.obj.denotation", "choyin.npc.obj.denotation"];
const CHOYIN_GOLDEN_ROPE_ID: &str = "choyin.obj.goldenrope";
const CHOYIN_GRASS_ID: &str = "choyin.obj.grass";
const CHOYIN_MAGIC_BOOK_ID: &str = "choyin.npc.obj.magic_book";
const CHOYIN_PEACH_CHEST_IDS: [&str; 2] = ["choyin.obj.chest", "choyin.npc.obj.chest"];
const CHOYIN_SILK_BAG_ID: &str = "choyin.npc.obj.silk_bag";
const CHOYIN_TABLET_ID: &str = "choyin.obj.tablet";
const GOATHILL_DEAD_LEECH_ID: &str = "goathill.npc.obj.dead_leech";
const GOATHILL_DEAD_LEECH_IDS: [&str; 2] = [GOATHILL_DEAD_LEECH_ID, "goathill.obj.dead_leech"];
const GOATHILL_DEAD_LEECH_DECAY_TICKS: u64 = 240;
const GOATHILL_LEECH_CORPSE_NPCS: [&str; 3] = [
    "goathill.npc.worm",
    "goathill.npc.fat_worm",
    "goathill.npc.big_worm",
];
const OLDPINE_BANDIT_CHIEF_ID: &str = "oldpine.npc.bandit_chief";
const OLDPINE_FAT_BANDIT_ID: &str = "oldpine.npc.fat_bandit";
const OLDPINE_VENOM_SNAKE_ID: &str = "oldpine.npc.venomsnake";
const CHUENYU_BOSS_ID: &str = "chuenyu.npc.chuenyu";
const CHUENYU_GUARD_ID: &str = "chuenyu.npc.guard";
const CHUENYU_GUARD_TWO_ID: &str = "chuenyu.npc.guard2";
const CHUENYU_JIADING_THREE_ID: &str = "chuenyu.npc.jiading3";
const GREEN_JADE_ID: &str = "green.obj.jade";
const GREEN_SPIDER_ID: &str = "green.npc.spider";
const GREEN_WIND_SWORD_ID: &str = "green.obj.windsword";
const SANYEN_COOK_ID: &str = "sanyen.npc.cook_bonze";
const SANYEN_BUN_ID: &str = "sanyen.npc.obj.maintal";
const LATEMOON_DANCE_BOOK_IDS: [&str; 2] = ["latemoon.npc.obj.book", "latemoon.obj.book"];
const LATEMOON_BRACELET_IDS: [&str; 2] = ["latemoon.npc.obj.bracelet", "latemoon.obj.bracelet"];
const LATEMOON_SPECIAL_CONSUMABLE_IDS: [&str; 5] = [
    "latemoon.park.npc.obj.bean",
    "latemoon.park.npc.obj.flower",
    "latemoon.sell.bean",
    "latemoon.sell.white_pill",
    "latemoon.sell.wine",
];
const LATEMOON_SECRET_LETTER_ID: &str = "latemoon.room.npc.obj.letter";
const LATEMOON_FIRE_ID: &str = "latemoon.room.npc.obj.fire";
const LATEMOON_BAMBOO_IDS: [&str; 2] = ["latemoon.npc.obj.bamboo", "latemoon.obj.bamboo"];
const LATEMOON_DRAGONFLY_IDS: [&str; 2] = ["latemoon.npc.obj.dragonfly", "latemoon.obj.dragonfly"];
const LATEMOON_TOKEN_ID: &str = "latemoon.room.npc.obj.token";
const LATEMOON_WHIP_BOOK_ID: &str = "latemoon.room.npc.obj.whip_book";
const CLOUD_ESCORT_LETTER_ID: &str = "u.cloud.npc.obj.letter";
const CLOUD_MEAT_IDS: [&str; 3] = [
    "u.cloud.obj.meat.beef",
    "u.cloud.obj.meat.dog_m",
    "u.cloud.obj.meat.hind",
];
const CITY_EXIT_TOKEN_ID: &str = "city.obj.token";
const CITY_ALTAR: &str = "city.jitan";
const CITY_ALTAR_TUNNEL: &str = "city.midao1";
const SNOW_WEAPON_STORAGE: &str = "snow.weapon_storage";
const SNOW_SECRET_STORAGE: &str = "snow.secret_storage";
const SNOW_WORKPLACE: &str = "snow.workplace";
const CANYON_BAMBOO_BOULDER: &str = "canyon.bamboo.bamboo3";
const CANYON_BAMBOO_TRAINING_ROOM: &str = "canyon.bamboo.train";
const CANYON_SLIPCASE_ID: &str = "canyon.bamboo.obj.slipcase";
const CANYON_PARRY_BOOK_ID: &str = "canyon.bamboo.obj.parry_book";
const TEMPLE_SLIPPERY_ROAD: &str = "temple.road1";
const TEMPLE_BOOK_ROOM: &str = "temple.book_room1";
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyKind {
    Bandit,
    XiaoJuan,
    OldLiuRevenge,
    Wolf,
    TempleDisciple,
    Rat,
    IceDragon,
    Meloner,
    BloodHandLiuSan,
    Npc(NpcId),
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
    FoundJuan,
    ReturnHome,
    MurderedJuan,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionKind {
    Bandaged,
    SnakePoison,
    Poison,
    Drunk,
    Slumber,
    AstralVision,
    RosePoison,
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
            Self::RosePoison => "火玫瑰毒",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionState {
    pub kind: ConditionKind,
    pub duration: u32,
    pub potency: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum ChoyinJusticeState {
    #[default]
    Free,
    Pursuit,
    Caught,
    AwaitingJudgment,
}

impl ChoyinJusticeState {
    fn blocks_actions(self) -> bool {
        matches!(self, Self::Caught | Self::AwaitingJudgment)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicQuest {
    pub target: String,
    pub tier: u64,
    pub deadline_elapsed_minutes: u64,
    pub exp_bonus: u32,
    pub potential_bonus: u32,
    pub score_bonus: i32,
    pub factor: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuicideKind {
    Reincarnate,
    EraseSave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StealState {
    npc: NpcId,
    item_id: ItemId,
    slot: usize,
    remaining_ticks: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StolenNpcItem {
    location: LocationId,
    npc: NpcId,
    slot: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ZombieHaunt {
    zombie: NpcId,
    target: NpcId,
    expires_at_elapsed_minutes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NpcRespawn {
    location: LocationId,
    npc: NpcId,
    due_elapsed_minutes: u64,
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
    SearchCityRuinedGarden,
    TurnAltarForward,
    TurnAltarBackward,
    PressAltarButton,
    PushSnowShelf,
    WorkAtSnowWorkshop,
    MoveBambooBoulder,
    SearchBambooBookcase,
    SwearCanyonSecret,
    ClimbCanyonChain,
    ClimbCityWall,
    JumpIntoCityManor,
    JumpOutsideCityWall,
    HoldOldPineVine,
    ClimbChoyinTree,
    HoldChoyinVine,
    TouchChoyinCloudFlag,
    DrinkChoyinWell,
    LiftChoyinStoneLion,
    BuryOldPineSkeleton,
    BlowOldPineBambooPipe,
    BorrowChoyinBook,
    ReadChoyinPeachNote,
    TieChoyinCrane,
    PullChuenyuHallRope,
    ClimbChuenyuCastleWall,
    DescendChuenyuRopeBridge,
    PushChuenyuDungeonSlab,
    PushGreenBoulder,
    FillGreenWell,
    SearchGreenStream,
    OpenSanyenSteamer,
    TakeSanyenBun,
    InspectLateMoonLantern,
    TakeLateMoonCloth,
    DanceLateMoonOut,
    DanceLateMoonYuFong,
    PickLateMoonFlower,
    BatheLateMoonPool,
    PonderLateMoonRoom,
    InspectDeathShadows,
    ReincarnateDeathInn,
    InspectCloudButcherySign,
    UseLateMoonDanceBook(u64),
    PrayLateMoonBracelet(u64),
    ReadLateMoonSecretLetter(u64),
    SearchLateMoonBracelet,
    SearchLateMoonDanceBook,
    JoinCloudEscort,
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
    OpenSourceDoor {
        target: LocationId,
    },
    CloseSourceDoor {
        target: LocationId,
    },
    InspectRoomDetail(String),
    Interact(InteractionKind),
    Talk(NpcId),
    RequestDynamicQuest,
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
    AbandonSkill(SkillId),
    ConfigureCombat,
    SelfLearn(SkillId),
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
    DonateItem(u64),
    PickUpItem(u64),
    DropItem(u64),
    PutItem {
        instance_id: u64,
        container_id: u64,
        quantity: u32,
    },
    TakeFromContainer {
        instance_id: u64,
        container_id: u64,
        quantity: u32,
    },
    ScribeHaunt {
        paper_instance_id: u64,
        target: NpcId,
    },
    AttachTalisman {
        talisman_instance_id: u64,
        zombie: NpcId,
    },
    Steal {
        npc: NpcId,
        item_id: ItemId,
        slot: usize,
    },
    Suicide(SuicideKind),
    EquipItem(u64),
    UnequipItem(EquipmentSlot),
    ConsumeItem(u64),
    ApplyItem(u64),
    MixIntoLiquid {
        powder_instance_id: u64,
        liquid_instance_id: u64,
    },
    DissolveCorpse {
        dust_instance_id: u64,
        corpse_instance_id: u64,
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
            Self::SearchCityRuinedGarden => "在废园草丛中寻找".into(),
            Self::TurnAltarForward => "顺时针转动祭坛按钮".into(),
            Self::TurnAltarBackward => "逆时针转动祭坛按钮".into(),
            Self::PressAltarButton => "按下祭坛按钮".into(),
            Self::PushSnowShelf => "向左推动兵器架".into(),
            Self::WorkAtSnowWorkshop => "在谷物加工厂做工".into(),
            Self::MoveBambooBoulder => "运功推开大黄石".into(),
            Self::SearchBambooBookcase => "搜寻石制书柜".into(),
            Self::SwearCanyonSecret => "面对山壁立誓".into(),
            Self::ClimbCanyonChain if game.location.as_str() == content::CANYON_FOOT => {
                "沿铁索向上攀爬".into()
            }
            Self::ClimbCanyonChain => "沿铁索向下攀爬".into(),
            Self::ClimbCityWall => "爬上尚书府院墙".into(),
            Self::JumpIntoCityManor => "跳入尚书府废屋".into(),
            Self::JumpOutsideCityWall => "跳回京师东街".into(),
            Self::HoldOldPineVine => "抓住桥边藤蔓".into(),
            Self::ClimbChoyinTree => "攀上绝壁古树".into(),
            Self::HoldChoyinVine => "抓住绝壁藤蔓".into(),
            Self::TouchChoyinCloudFlag => "触碰云台仙幡".into(),
            Self::DrinkChoyinWell => "舀取井水饮用".into(),
            Self::LiftChoyinStoneLion => "抬动西街石狮".into(),
            Self::BuryOldPineSkeleton => "掩埋洞中骸骨".into(),
            Self::BlowOldPineBambooPipe => "吹响竹哨移开寨门巨石".into(),
            Self::BorrowChoyinBook => "趁人不备取走一本书".into(),
            Self::ReadChoyinPeachNote => "读桃枝上的字条".into(),
            Self::TieChoyinCrane => "以缚仙绳缚住仙鹤".into(),
            Self::PullChuenyuHallRope => "拉动正厅垂绳".into(),
            Self::ClimbChuenyuCastleWall => "沿藤蔓翻越城墙".into(),
            Self::DescendChuenyuRopeBridge => "抓紧铁链下到山脚".into(),
            Self::PushChuenyuDungeonSlab => "推动地牢石板".into(),
            Self::PushGreenBoulder => "运功推动绝地巨石".into(),
            Self::FillGreenWell => "用酒袋汲取井水".into(),
            Self::SearchGreenStream => "下溪寻找亮光".into(),
            Self::OpenSanyenSteamer => "打开厨房蒸笼".into(),
            Self::TakeSanyenBun => "从蒸笼取一枚馒头".into(),
            Self::InspectLateMoonLantern => "查看晚月庄门前灯笼".into(),
            Self::TakeLateMoonCloth => "从碧纱橱取一件衣裳".into(),
            Self::DanceLateMoonOut => "跳一曲「西出阳关」".into(),
            Self::DanceLateMoonYuFong => "跳一曲「有凤来仪」".into(),
            Self::PickLateMoonFlower => "摘下一朵金黄花蕊".into(),
            Self::BatheLateMoonPool => "在小花池沐浴".into(),
            Self::PonderLateMoonRoom => "在缀芳阁静修".into(),
            Self::InspectDeathShadows => "靠近壁炉旁的黑影".into(),
            Self::ReincarnateDeathInn => "向另一个自己询问回家".into(),
            Self::InspectCloudButcherySign => "查看肉铺牛骨招牌".into(),
            Self::UseLateMoonDanceBook(_) => "按舞曲谱跳「春宫怨」".into(),
            Self::PrayLateMoonBracelet(_) => "以玛瑙手镯祈求归返".into(),
            Self::ReadLateMoonSecretLetter(_) => "借火查看密函暗字".into(),
            Self::SearchLateMoonBracelet => "按线索搜索碧纱橱底层".into(),
            Self::SearchLateMoonDanceBook => "按舞姬提示搜索床榻".into(),
            Self::JoinCloudEscort => "向陈剑秋请求加入振远镖局".into(),
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
            Self::SearchCityRuinedGarden => "翻找废园草丛中一闪而过的旧物。",
            Self::TurnAltarForward | Self::TurnAltarBackward | Self::PressAltarButton => {
                "依照原版顺一逆三的次序操作祭坛按钮。"
            }
            Self::PushSnowShelf => "连续推动三次，短暂打开通往密室的阶梯。",
            Self::WorkAtSnowWorkshop => "消耗精与神各三十点，换取一两纹银。",
            Self::MoveBambooBoulder => "运足内力推开石缝，进入时石缝会立即闭合。",
            Self::SearchBambooBookcase => "书柜中的书匣与招架秘笈只能取得一次。",
            Self::SwearCanyonSecret => "使用军师提供的口令进入黄石峡黑市，口令只能使用一次。",
            Self::ClimbCanyonChain => "沿原版铁索连接黄石隘口与雪亭镇官道，会消耗精、气、神。",
            Self::ClimbCityWall | Self::JumpIntoCityManor | Self::JumpOutsideCityWall => {
                "沿原版尚书府院墙路径移动。"
            }
            Self::HoldOldPineVine => "抓住藤蔓后按轻功判断落入水潭或攀进瀑布后的通道。",
            Self::ClimbChoyinTree => "沿固定源古树路径攀上鹤室。",
            Self::HoldChoyinVine => "抓住藤蔓后按轻功判断坠入寒谷或攀近半山洞穴。",
            Self::TouchChoyinCloudFlag => "短暂打开云台下行入口，并延迟引来持续雷击。",
            Self::DrinkChoyinWell => "从南门广场的井中补充二十点饮水。",
            Self::LiftChoyinStoneLion => "力量会减少所需抬动次数，机关开启后直接落入洞穴。",
            Self::BuryOldPineSkeleton => "按灵性判定过招要旨奖励或坠下瀑布。",
            Self::BlowOldPineBambooPipe => "用来源竹哨重新绞开老松寨门口的巨石。",
            Self::BorrowChoyinBook => "随机取走一本书；离开草堂时会按源归还。",
            Self::ReadChoyinPeachNote => "按当前字条提示连续选择方向才能离开桃林。",
            Self::TieChoyinCrane => "借仙鹤登上云台，损失五十点神；缚仙绳不会消耗。",
            Self::PullChuenyuHallRope => "触发正厅翻板，跌入地牢并受到少量气伤。",
            Self::ClimbChuenyuCastleWall => "沿固定源藤蔓在城堡外墙与花园之间翻越。",
            Self::DescendChuenyuRopeBridge => "沿铁链从摇晃的铁索桥直接下到黑松山脚。",
            Self::PushChuenyuDungeonSlab => "连续推动五次，短暂打开城堡东侧与地牢之间的石板通道。",
            Self::PushGreenBoulder => {
                "需要足够内力与基本内功；推动会损伤精、气、神，并有机会逃出绝地。"
            }
            Self::FillGreenWell => "把一个酒水容器改装为十五口清水，并清除容器中已有药效。",
            Self::SearchGreenStream => {
                "通过八卦阵后可在溪中寻找一次追风剑，结果由持久化随机状态决定。"
            }
            Self::OpenSanyenSteamer => "烧饭僧仍在时会阻止开盖；无人看守时可看见馒头。",
            Self::TakeSanyenBun => "烧饭僧不在时可取走馒头，单份存档最多五枚。",
            Self::InspectLateMoonLantern => "查看目录未能结构化的彩色灯笼题字。",
            Self::TakeLateMoonCloth => "碧纱橱每份存档保留两件来源衣裳。",
            Self::DanceLateMoonOut | Self::DanceLateMoonYuFong => {
                "按来源舞步消耗神并在晚月庄密室、竹林之间移动。"
            }
            Self::PickLateMoonFlower => "西府海棠每份存档最多摘取两朵解毒花蕊。",
            Self::BatheLateMoonPool => "女性沐浴恢复少量神；男性会染上持续发作的火玫瑰毒。",
            Self::PonderLateMoonRoom => "消耗五十点神，并按灵性随机降低杀气。",
            Self::InspectDeathShadows => "查看与自己相貌相同的幽冥黑影。",
            Self::ReincarnateDeathInn => "恢复精气神并从幽冥小店返回雪亭城隍庙。",
            Self::InspectCloudButcherySign => "查看目录未能结构化的肉铺收购告示。",
            Self::UseLateMoonDanceBook(_) => "消耗五十点神，按舞曲谱来源返回对应晚月庄房间。",
            Self::PrayLateMoonBracelet(_) => "消耗五十点神，经来源传送回雪亭城隍庙。",
            Self::ReadLateMoonSecretLetter(_) => "携带火种时显出密函中的晚月庄线索，不消耗物品。",
            Self::SearchLateMoonBracelet => "凤铃确认竹蜻蜓后，可按其线索取得一只玛瑙手镯。",
            Self::SearchLateMoonDanceBook => "辛芬说出舞曲谱藏处后，可从床榻取得一本舞谱。",
            Self::JoinCloudEscort => "胆识达到二十五后，加入陈剑秋主持的振远镖局。",
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
            Self::OpenSourceDoor { target } => {
                format!("打开{}", game.source_door_name(target))
            }
            Self::CloseSourceDoor { target } => {
                format!("关上{}", game.source_door_name(target))
            }
            Self::InspectRoomDetail(key) => format!("查看{key}"),
            Self::Interact(interaction) => interaction.label(game),
            Self::Talk(npc) => format!("与{}交谈", npc.name()),
            Self::RequestDynamicQuest => {
                if game.dynamic_quest.is_some() {
                    "向朱鸿雪查看悬赏".into()
                } else {
                    "向朱鸿雪领取悬赏".into()
                }
            }
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
            Self::AbandonSkill(skill) => format!("放弃继续学习{}", skill.name()),
            Self::ConfigureCombat => "设置内力、灵力与自动逃跑".into(),
            Self::SelfLearn(skill) => format!("自行钻研{}", skill.name()),
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
                if enemy == &EnemyKind::Bandit && game.quest == QuestStage::FindJuan {
                    "循声营救娟儿".into()
                } else {
                    format!("与{}比试", enemy.name())
                }
            }
            Self::Kill(EnemyKind::XiaoJuan) => "加害小娟".into(),
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
            Self::DonateItem(instance_id) => {
                format!("把{}投入功德箱", game.inventory_item_name(*instance_id))
            }
            Self::PickUpItem(instance_id) => {
                format!("拾取{}", game.ground_item_name(*instance_id))
            }
            Self::DropItem(instance_id) => {
                format!("丢下{}", game.inventory_item_name(*instance_id))
            }
            Self::PutItem {
                instance_id,
                container_id,
                quantity,
            } => format!(
                "把{}{}放进{}",
                game.inventory_item_name(*instance_id),
                game.item_quantity_suffix(*quantity),
                game.container_item_name(*container_id)
            ),
            Self::TakeFromContainer {
                instance_id,
                container_id,
                quantity,
            } => format!(
                "从{}取出{}{}",
                game.container_item_name(*container_id),
                game.container_content_name(*container_id, *instance_id),
                game.item_quantity_suffix(*quantity)
            ),
            Self::ScribeHaunt {
                paper_instance_id,
                target,
            } => format!(
                "在{}上书写「僵尸追魂符」· {}",
                game.inventory_item_name(*paper_instance_id),
                target.name()
            ),
            Self::AttachTalisman {
                talisman_instance_id,
                zombie,
            } => format!(
                "把{}贴在{}身上",
                game.inventory_item_name(*talisman_instance_id),
                zombie.name()
            ),
            Self::Steal { npc, item_id, .. } => format!(
                "试图从{}身上偷取{}",
                npc.name(),
                items()
                    .definition(item_id)
                    .expect("steal item must exist")
                    .display_name()
            ),
            Self::Suicide(SuicideKind::Reincarnate) => "自尽后投胎重来".into(),
            Self::Suicide(SuicideKind::EraseSave) => "永久删除本地存档".into(),
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
            Self::DissolveCorpse {
                dust_instance_id,
                corpse_instance_id,
            } => format!(
                "用{}化去{}",
                game.inventory_item_name(*dust_instance_id),
                game.ground_item_name(*corpse_instance_id)
            ),
            Self::Surrender => "认输并退出战斗".into(),
        }
    }

    pub fn detail(&self) -> &'static str {
        match self {
            Self::Move { .. } => "移动会结束当前的修炼或休息。",
            Self::Flee { .. } => "脱离当前战斗并移动，临阵退却会损失少量评价。",
            Self::OpenSourceDoor { .. } | Self::CloseSourceDoor { .. } => {
                "门的开关状态会同时作用于相连房间。"
            }
            Self::InspectRoomDetail(_) => "查看固定源房间中记载的场景细节。",
            Self::Interact(interaction) => interaction.detail(),
            Self::Talk(_) => "交谈可能带来线索、奖励或新的武学见闻。",
            Self::RequestDynamicQuest => {
                "朱鸿雪按实战经验和连续完成次数发布可完成的限时杀敌悬赏；时限随世界时钟推进。"
            }
            Self::AskNpc { .. } => "按固定源人物的询问主题追问；仅开放已审计的文本或脚本回复。",
            Self::BecomeApprentice(_) => "加入师门后才能向掌门请教本门武学。",
            Self::LearnSkill { .. } | Self::LearnFromNpc { .. } => {
                "请教消耗神和潜能，造诣不能超过师父。"
            }
            Self::MapSkill { .. } => "把已学特殊技能映射到对应基础用途。",
            Self::AbandonSkill(_) => {
                "永久删除这项已学技能，不返还已消耗的潜能；日后重新学习须从零开始。原有致能映射会保留，但不再提供加成。"
            }
            Self::ConfigureCombat => {
                "分别设置每次命中的内力、武器灵力强度，以及气低于指定百分比时的自动逃跑阈值。"
            }
            Self::SelfLearn(_) => "基础武学达到四十层后可耗精自学；仍受潜能与实战经验门槛约束。",
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
            Self::DonateItem(_) => "把有价值的物品投入乔阴寺庙功德箱，并按价值与灵性消减杀气。",
            Self::PickUpItem(_) => "拾取地面物品；超过负重上限时无法拿起。",
            Self::DropItem(_) => "把未装备的物品留在当前位置。",
            Self::PutItem { .. } => "容器容量按来源负重限制计算；可把堆叠物逐个或整叠分装。",
            Self::TakeFromContainer { .. } => "从当前可见容器中取回物品，仍受自身负重限制。",
            Self::ScribeHaunt { .. } => {
                "消耗桃符纸、法力、气和神，书写带有指定追踪目标的僵尸追魂符。"
            }
            Self::AttachTalisman { .. } => {
                "仅可贴给当前受控的来源僵尸；符纸会被消耗，并在限时内让其协助追击指定目标。"
            }
            Self::Steal { .. } => "行窃需要等待三拍；失手可能被当场发现、进入死斗并增加通缉。",
            Self::Suicide(SuicideKind::Reincarnate) => {
                "留下可搜取的遗体并重置人物旅程；该操作需要二次确认。"
            }
            Self::Suicide(SuicideKind::EraseSave) => {
                "永久移除本机存档文件；需经过额外的最终确认，无法从游戏内恢复。"
            }
            Self::EquipItem(_) | Self::UnequipItem(_) => {
                "装备会提供武器伤害或护甲防御，并在战斗中损耗耐久。"
            }
            Self::ConsumeItem(_) => "食物和饮水按原物品份量逐次消耗，可恢复饱食或饮水。",
            Self::ApplyItem(_) => "药物与绷带不能在战斗中使用，其状态会随时间更新。",
            Self::MixIntoLiquid { .. } => "药粉会溶入尚有内容的酒水，饮用后触发对应药效。",
            Self::DissolveCorpse { .. } => {
                "化尸粉只能化去尚未变成骸骨的尸体；尸体外的掉落物会留在原地。"
            }
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
    Stealing(StealState),
    Fighting(CombatState),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatMode {
    #[default]
    Spar,
    Lethal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub enemy_attack_bonus: i32,
    #[serde(default)]
    pub enemy_busy_rounds: u8,
    #[serde(default)]
    pub technique_cooldown: u8,
    #[serde(default)]
    pub power_up_active: bool,
    #[serde(default)]
    pub fake_fault_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DefeatedNpcInstance {
    location: LocationId,
    npc: NpcId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SpawnedNpcInstance {
    location: LocationId,
    npc: NpcId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NpcLocationOverride {
    origin: LocationId,
    npc: NpcId,
    ordinal: usize,
    location: LocationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SourceDoorState {
    first: LocationId,
    second: LocationId,
    open: bool,
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
    #[serde(default = "default_player_name")]
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
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
    #[serde(default)]
    pub force_factor: u32,
    #[serde(default)]
    pub mana_factor: u32,
    #[serde(default)]
    pub wimpy_percent: u8,
    #[serde(default)]
    pub theft_heat: u32,
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
            name: default_player_name(),
            title: None,
            description: None,
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
            force_factor: 0,
            mana_factor: 0,
            wimpy_percent: 0,
            theft_heat: 0,
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

fn default_player_name() -> String {
    "无名客".into()
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
    last_saved_at_unix_seconds: Option<u64>,
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
    city_exit_permit: bool,
    #[serde(default)]
    city_altar_forward_turns: u8,
    #[serde(default)]
    city_altar_backward_turns: u8,
    #[serde(default)]
    city_altar_passage_ticks: u8,
    #[serde(default)]
    snow_shelf_pushes: u8,
    #[serde(default)]
    snow_storage_passage_ticks: u8,
    #[serde(default)]
    canyon_boulder_open: bool,
    #[serde(default)]
    canyon_bookcase_searched: bool,
    #[serde(default)]
    source_door_states: Vec<SourceDoorState>,
    #[serde(default)]
    source_room_items_initialized: bool,
    #[serde(default)]
    m5_source_room_items_initialized: bool,
    #[serde(default)]
    m6_source_room_items_initialized: bool,
    #[serde(default)]
    m7_source_room_items_initialized: bool,
    #[serde(default)]
    chuenyu_slab_pushes: u8,
    #[serde(default)]
    chuenyu_slab_passage_ticks: u8,
    #[serde(default)]
    chuenyu_trap_arrow_ticks: u8,
    #[serde(default)]
    green_bagua_completed: bool,
    #[serde(default)]
    green_windsword_rewarded: bool,
    #[serde(default)]
    green_elder_jade_clue: bool,
    #[serde(default)]
    green_drunk_jade_clue: bool,
    #[serde(default)]
    green_drunk_drug_clue: bool,
    #[serde(default)]
    green_jade_received: bool,
    #[serde(default)]
    green_drug_offer_unlocked: bool,
    #[serde(default)]
    sanyen_buns_taken: u8,
    #[serde(default)]
    latemoon_clothes_taken: u8,
    #[serde(default)]
    latemoon_flowers_picked: u8,
    #[serde(default)]
    death_road_steps: u8,
    #[serde(default)]
    latemoon_dragonfly_received: bool,
    #[serde(default)]
    latemoon_bracelet_clue: bool,
    #[serde(default)]
    latemoon_bracelet_received: bool,
    #[serde(default)]
    latemoon_dance_book_clue: bool,
    #[serde(default)]
    latemoon_dance_book_received: bool,
    #[serde(default)]
    latemoon_token_rewarded: bool,
    #[serde(default)]
    cloud_escort_member: bool,
    #[serde(default)]
    cloud_escort_letter_received: bool,
    #[serde(default)]
    city_chen_letter_delivered: bool,
    #[serde(default)]
    cloud_boater_paid: bool,
    #[serde(default)]
    cloud_gangster_pass: bool,
    #[serde(default)]
    cloud_girl_recognized: bool,
    #[serde(default)]
    choyin_platform_passage_ticks: u8,
    #[serde(default)]
    choyin_thunder_ticks: u8,
    #[serde(default)]
    choyin_lion_lift_count: u8,
    #[serde(default)]
    oldpine_keep_sealed: bool,
    #[serde(default)]
    choyin_taolin_steps: u8,
    #[serde(default)]
    choyin_taolin_clue: u8,
    #[serde(default)]
    choyin_scholar_trial_started: bool,
    #[serde(default)]
    choyin_scholar_trial_completed: bool,
    #[serde(default)]
    choyin_silk_bag_received: bool,
    #[serde(default)]
    choyin_silk_bag_delivered: bool,
    #[serde(default)]
    choyin_chest_rewarded: bool,
    #[serde(default)]
    choyin_justice: ChoyinJusticeState,
    #[serde(default)]
    dynamic_quest: Option<DynamicQuest>,
    #[serde(default)]
    dynamic_quest_finished: i32,
    #[serde(default)]
    pub ground_items: HashMap<LocationId, Vec<ItemInstance>>,
    #[serde(default)]
    stolen_npc_items: Vec<StolenNpcItem>,
    #[serde(default)]
    zombie_haunts: Vec<ZombieHaunt>,
    #[serde(default)]
    npc_respawns: Vec<NpcRespawn>,
    #[serde(default)]
    defeated_npcs: Vec<NpcId>,
    #[serde(default)]
    defeated_npc_instances: Vec<DefeatedNpcInstance>,
    #[serde(default)]
    spawned_npc_instances: Vec<SpawnedNpcInstance>,
    #[serde(default)]
    npc_location_overrides: Vec<NpcLocationOverride>,
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
        let mut game = Self {
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
            last_saved_at_unix_seconds: None,
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
            city_exit_permit: false,
            city_altar_forward_turns: 0,
            city_altar_backward_turns: 0,
            city_altar_passage_ticks: 0,
            snow_shelf_pushes: 0,
            snow_storage_passage_ticks: 0,
            canyon_boulder_open: false,
            canyon_bookcase_searched: false,
            source_door_states: Vec::new(),
            source_room_items_initialized: false,
            m5_source_room_items_initialized: false,
            m6_source_room_items_initialized: false,
            m7_source_room_items_initialized: false,
            chuenyu_slab_pushes: 0,
            chuenyu_slab_passage_ticks: 0,
            chuenyu_trap_arrow_ticks: 0,
            green_bagua_completed: false,
            green_windsword_rewarded: false,
            green_elder_jade_clue: false,
            green_drunk_jade_clue: false,
            green_drunk_drug_clue: false,
            green_jade_received: false,
            green_drug_offer_unlocked: false,
            sanyen_buns_taken: 0,
            latemoon_clothes_taken: 0,
            latemoon_flowers_picked: 0,
            death_road_steps: 0,
            latemoon_dragonfly_received: false,
            latemoon_bracelet_clue: false,
            latemoon_bracelet_received: false,
            latemoon_dance_book_clue: false,
            latemoon_dance_book_received: false,
            latemoon_token_rewarded: false,
            cloud_escort_member: false,
            cloud_escort_letter_received: false,
            city_chen_letter_delivered: false,
            cloud_boater_paid: false,
            cloud_gangster_pass: false,
            cloud_girl_recognized: false,
            choyin_platform_passage_ticks: 0,
            choyin_thunder_ticks: 0,
            choyin_lion_lift_count: 0,
            oldpine_keep_sealed: false,
            choyin_taolin_steps: 0,
            choyin_taolin_clue: 0,
            choyin_scholar_trial_started: false,
            choyin_scholar_trial_completed: false,
            choyin_silk_bag_received: false,
            choyin_silk_bag_delivered: false,
            choyin_chest_rewarded: false,
            choyin_justice: ChoyinJusticeState::Free,
            dynamic_quest: None,
            dynamic_quest_finished: 0,
            ground_items: HashMap::new(),
            stolen_npc_items: Vec::new(),
            zombie_haunts: Vec::new(),
            npc_respawns: Vec::new(),
            defeated_npcs: Vec::new(),
            defeated_npc_instances: Vec::new(),
            spawned_npc_instances: Vec::new(),
            npc_location_overrides: Vec::new(),
            next_item_instance_id: 3,
            rng_state: 0x4d59_5df4_d0f3_3173,
        };
        game.initialize_source_room_items();
        game.initialize_m5_source_room_items();
        game.initialize_m6_source_room_items();
        game.initialize_m7_source_room_items();
        game
    }

    pub fn current_location(&self) -> &'static Location {
        world()
            .location(&self.location)
            .expect("saved location must exist in the embedded world")
    }

    pub fn available_actions(&self) -> Vec<Action> {
        if self.choyin_justice.blocks_actions() {
            return vec![Action::OfferMoney {
                amount: CHOYIN_BRIBE_AMOUNT,
                npc: NpcId::from(CHOYIN_POLICE_ID),
            }];
        }

        let current = self.current_location();
        if matches!(self.activity, Activity::Stealing(_)) {
            return Vec::new();
        }
        if let Activity::Fighting(combat) = &self.activity {
            let mut actions = if combat.technique_cooldown == 0 {
                self.technique_actions(true)
            } else {
                Vec::new()
            };
            if !(current.id.as_str() == content::MELON_FARM && self.melon_debt) {
                for exit in &current.exits {
                    let target = self.resolved_source_exit_target(current, exit);
                    if self.exit_is_available(current, exit, &target) {
                        actions.push(Action::Flee {
                            direction: exit.direction.clone(),
                            target,
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
        actions.push(Action::ConfigureCombat);

        if current.id.as_str() == content::MELON_FARM && self.melon_debt {
            return vec![
                Action::Interact(InteractionKind::SettleMelonDebt),
                Action::Fight(EnemyKind::Meloner),
            ];
        }

        for exit in &current.exits {
            let target = self.resolved_source_exit_target(current, exit);
            if self.exit_is_available(current, exit, &target) {
                actions.push(Action::Move {
                    direction: exit.direction.clone(),
                    target,
                });
            }
            if let Some(open) = self.source_door_is_open(&current.id, &exit.target) {
                actions.push(if open {
                    Action::CloseSourceDoor {
                        target: exit.target.clone(),
                    }
                } else {
                    Action::OpenSourceDoor {
                        target: exit.target.clone(),
                    }
                });
            }
        }
        if current.id.as_str() == CITY_ALTAR_TUNNEL && self.city_altar_passage_ticks > 0 {
            actions.push(Action::Move {
                direction: "up".into(),
                target: LocationId::from(CITY_ALTAR),
            });
        }
        if current.id.as_str() == SNOW_SECRET_STORAGE && self.snow_storage_passage_ticks > 0 {
            actions.push(Action::Move {
                direction: "up".into(),
                target: LocationId::from(SNOW_WEAPON_STORAGE),
            });
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
            content::CITY_RUINED_GARDEN
                if !self.city_exit_permit
                    && !self.player.has_item(&ItemId::from(CITY_EXIT_TOKEN_ID)) =>
            {
                actions.push(Action::Interact(InteractionKind::SearchCityRuinedGarden));
            }
            CITY_ALTAR => {
                actions.push(Action::Interact(InteractionKind::TurnAltarForward));
                actions.push(Action::Interact(InteractionKind::TurnAltarBackward));
                actions.push(Action::Interact(InteractionKind::PressAltarButton));
            }
            SNOW_WEAPON_STORAGE => {
                actions.push(Action::Interact(InteractionKind::PushSnowShelf));
            }
            SNOW_WORKPLACE => {
                actions.push(Action::Interact(InteractionKind::WorkAtSnowWorkshop));
            }
            CANYON_BAMBOO_BOULDER if !self.canyon_boulder_open => {
                actions.push(Action::Interact(InteractionKind::MoveBambooBoulder));
            }
            CANYON_BAMBOO_TRAINING_ROOM if !self.canyon_bookcase_searched => {
                actions.push(Action::Interact(InteractionKind::SearchBambooBookcase));
            }
            content::CITY_WALL => {
                actions.push(Action::Interact(InteractionKind::JumpIntoCityManor));
                actions.push(Action::Interact(InteractionKind::JumpOutsideCityWall));
            }
            "oldpine.epath2" => {
                actions.push(Action::Interact(InteractionKind::HoldOldPineVine));
            }
            "choyin.guyehill" => {
                actions.push(Action::Interact(InteractionKind::ClimbChoyinTree));
                actions.push(Action::Interact(InteractionKind::HoldChoyinVine));
            }
            "choyin.platform" => {
                actions.push(Action::Interact(InteractionKind::TouchChoyinCloudFlag));
            }
            "choyin.s_street1" if self.player.water < DEFAULT_WATER_CAPACITY => {
                actions.push(Action::Interact(InteractionKind::DrinkChoyinWell));
            }
            "choyin.w_street1" => {
                actions.push(Action::Interact(InteractionKind::LiftChoyinStoneLion));
            }
            "oldpine.cave5" if self.current_ground_has("oldpine.npc.skeleton") => {
                actions.push(Action::Interact(InteractionKind::BuryOldPineSkeleton));
            }
            "oldpine.keep2"
                if self.oldpine_keep_sealed
                    && (self
                        .player
                        .has_item(&ItemId::from("oldpine.obj.bamboo_pipe"))
                        || self
                            .player
                            .has_item(&ItemId::from("oldpine.npc.obj.bamboo_pipe"))) =>
            {
                actions.push(Action::Interact(InteractionKind::BlowOldPineBambooPipe));
            }
            "choyin.club" => {
                actions.push(Action::Interact(InteractionKind::BorrowChoyinBook));
            }
            "choyin.taolin" => {
                actions.push(Action::Interact(InteractionKind::ReadChoyinPeachNote));
            }
            "choyin.craneroom" if self.player.has_item(&ItemId::from(CHOYIN_GOLDEN_ROPE_ID)) => {
                actions.push(Action::Interact(InteractionKind::TieChoyinCrane));
            }
            "chuenyu.center" => {
                actions.push(Action::Interact(InteractionKind::PullChuenyuHallRope));
            }
            "chuenyu.east_castle"
            | "chuenyu.east_garden"
            | "chuenyu.west_castle"
            | "chuenyu.west_garden" => {
                actions.push(Action::Interact(InteractionKind::ClimbChuenyuCastleWall));
            }
            "chuenyu.rope_bridge" => {
                actions.push(Action::Interact(InteractionKind::DescendChuenyuRopeBridge));
            }
            "chuenyu.tunnel4" if self.chuenyu_slab_passage_ticks == 0 => {
                actions.push(Action::Interact(InteractionKind::PushChuenyuDungeonSlab));
            }
            "green.closed" => {
                actions.push(Action::Interact(InteractionKind::PushGreenBoulder));
            }
            "green.station0"
                if self
                    .player
                    .inventory
                    .iter()
                    .any(|item| item.definition().category == items::ItemCategory::Liquid) =>
            {
                actions.push(Action::Interact(InteractionKind::FillGreenWell));
            }
            "green.water" => {
                actions.push(Action::Interact(InteractionKind::SearchGreenStream));
            }
            "sanyen.kitchen" => {
                actions.push(Action::Interact(InteractionKind::OpenSanyenSteamer));
                if !self.current_room_has_npc(SANYEN_COOK_ID) && self.sanyen_buns_taken < 5 {
                    actions.push(Action::Interact(InteractionKind::TakeSanyenBun));
                }
            }
            "latemoon.gate" => {
                actions.push(Action::Interact(InteractionKind::InspectLateMoonLantern));
            }
            "latemoon.latemoon2" => {
                if self.latemoon_clothes_taken < 2 {
                    actions.push(Action::Interact(InteractionKind::TakeLateMoonCloth));
                }
                if self.latemoon_bracelet_clue && !self.latemoon_bracelet_received {
                    actions.push(Action::Interact(InteractionKind::SearchLateMoonBracelet));
                }
            }
            "latemoon.latemoon8" => {
                actions.push(Action::Interact(InteractionKind::DanceLateMoonOut));
                actions.push(Action::Interact(InteractionKind::DanceLateMoonYuFong));
                if self.latemoon_dance_book_clue && !self.latemoon_dance_book_received {
                    actions.push(Action::Interact(InteractionKind::SearchLateMoonDanceBook));
                }
            }
            "latemoon.miroom" => {
                actions.push(Action::Interact(InteractionKind::DanceLateMoonOut));
            }
            "latemoon.park.moonc" if self.latemoon_flowers_picked < 2 => {
                actions.push(Action::Interact(InteractionKind::PickLateMoonFlower));
            }
            "latemoon.room.bathroom" => {
                actions.push(Action::Interact(InteractionKind::BatheLateMoonPool));
            }
            "latemoon.upstar.uproom3" => {
                actions.push(Action::Interact(InteractionKind::PonderLateMoonRoom));
            }
            "death.inn1" => {
                actions.push(Action::Interact(InteractionKind::InspectDeathShadows));
                actions.push(Action::Interact(InteractionKind::ReincarnateDeathInn));
            }
            "u.cloud.butchery" => {
                actions.push(Action::Interact(InteractionKind::InspectCloudButcherySign));
            }
            "u.cloud.biaoju" if !self.cloud_escort_member => {
                actions.push(Action::Interact(InteractionKind::JoinCloudEscort));
            }
            _ => {}
        }

        actions.extend(
            current
                .details
                .iter()
                .map(|detail| Action::InspectRoomDetail(detail.key.clone())),
        );

        let present_npcs = self.present_current_npcs();
        for npc in &present_npcs {
            actions.push(Action::Talk(npc.clone()));
            if npc.as_str() == CLOUD_GOD_ID {
                actions.push(Action::RequestDynamicQuest);
            }
            let definition = npcs()
                .definition(npc)
                .expect("room NPC must exist in the repository");
            for inquiry in definition.inquiries.iter().filter(|inquiry| {
                (inquiry.is_runtime_available() || inquiry.scripted_runtime_kind(npc).is_some())
                    && self.scripted_inquiry_is_available(npc, &inquiry.topic)
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
            if definition.is_source_combatant() {
                actions.push(Action::Fight(EnemyKind::Npc(npc.clone())));
                actions.push(Action::Kill(EnemyKind::Npc(npc.clone())));
            }
            if definition
                .apprenticeship_policy()
                .is_some_and(|policy| self.npc_lesson_access(policy))
            {
                for lesson in definition.lessons() {
                    if self.player.skill_level(lesson.skill) < lesson.max_level {
                        actions.push(Action::LearnFromNpc {
                            skill: SkillId::from(lesson.skill),
                            npc: npc.clone(),
                        });
                    }
                }
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
        actions.extend(self.self_learning_actions());
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
        if self.location.as_str() == content::PINE_FOREST && self.quest == QuestStage::FoundJuan {
            actions.push(Action::Kill(EnemyKind::XiaoJuan));
        }
        if let Some(enemy) = &current.enemy
            && (!matches!(enemy, EnemyKind::Bandit)
                || matches!(self.quest, QuestStage::Unasked | QuestStage::FindJuan))
        {
            actions.push(Action::Fight(enemy.clone()));
            actions.push(Action::Kill(enemy.clone()));
        }
        actions.extend(self.talisman_actions(&present_npcs));
        actions.extend(self.steal_actions(&present_npcs));
        actions.extend(self.container_actions());
        actions.extend(self.technique_actions(false));
        let mut learned_skills: Vec<_> = self.player.skills.iter().collect();
        learned_skills.sort_by(|left, right| left.kind.cmp(&right.kind));
        actions.extend(
            learned_skills
                .into_iter()
                .map(|skill| Action::AbandonSkill(skill.kind.clone())),
        );

        if let Some(ground) = self.ground_items.get(&self.location) {
            for item in ground {
                if !CHOYIN_DONATION_BOX_IDS.contains(&item.item_id.as_str()) {
                    actions.push(Action::PickUpItem(item.instance_id));
                }
            }
        }
        for item in &self.player.inventory {
            let definition = item.definition();
            if item.has_uses_left()
                && (definition.food_supply.is_some()
                    || definition.category == items::ItemCategory::Liquid
                    || item.item_id.as_str() == CHOYIN_TABLET_ID
                    || LATEMOON_SPECIAL_CONSUMABLE_IDS.contains(&item.item_id.as_str()))
            {
                actions.push(Action::ConsumeItem(item.instance_id));
            }
            if LATEMOON_DANCE_BOOK_IDS.contains(&item.item_id.as_str()) {
                actions.push(Action::Interact(InteractionKind::UseLateMoonDanceBook(
                    item.instance_id,
                )));
            }
            if LATEMOON_BRACELET_IDS.contains(&item.item_id.as_str()) {
                actions.push(Action::Interact(InteractionKind::PrayLateMoonBracelet(
                    item.instance_id,
                )));
            }
            if item.item_id.as_str() == LATEMOON_SECRET_LETTER_ID
                && self.player.has_item(&ItemId::from(LATEMOON_FIRE_ID))
            {
                actions.push(Action::Interact(InteractionKind::ReadLateMoonSecretLetter(
                    item.instance_id,
                )));
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
                let transfer_restricted = definition
                    .behavior_flags
                    .iter()
                    .any(|flag| flag == "restricted_movement");
                if !transfer_restricted
                    && present_npcs.iter().any(|npc| npc.as_str() == TRADER_ID)
                    && item.unit_value() > 0
                {
                    actions.push(Action::SellItem(item.instance_id));
                }
                for npc in &present_npcs {
                    let accepts_gifts = npcs()
                        .definition(npc)
                        .is_some_and(|definition| definition.accepts_runtime_gifts());
                    if accepts_gifts && !transfer_restricted {
                        actions.push(Action::GiveItem {
                            instance_id: item.instance_id,
                            npc: npc.clone(),
                        });
                    }
                }
                if self.location.as_str() == "choyin.altar"
                    && item.unit_value() > 0
                    && !transfer_restricted
                    && CHOYIN_DONATION_BOX_IDS
                        .iter()
                        .any(|box_id| self.current_ground_has(box_id))
                {
                    actions.push(Action::DonateItem(item.instance_id));
                }
                if !transfer_restricted {
                    actions.push(Action::DropItem(item.instance_id));
                }
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
        let corpse_instance_ids: Vec<_> = self
            .ground_items
            .get(&self.location)
            .into_iter()
            .flatten()
            .filter(|item| item.item_id.as_str() == CORPSE_ITEM_ID && item.lifecycle_stage < 2)
            .map(|item| item.instance_id)
            .collect();
        let dust_instance_ids: Vec<_> = self
            .player
            .inventory
            .iter()
            .filter(|item| item.item_id.as_str() == "obj.dust" && item.has_uses_left())
            .map(|item| item.instance_id)
            .collect();
        for dust_instance_id in dust_instance_ids {
            for &corpse_instance_id in &corpse_instance_ids {
                actions.push(Action::DissolveCorpse {
                    dust_instance_id,
                    corpse_instance_id,
                });
            }
        }
        for equipped in &self.player.equipment {
            actions.push(Action::UnequipItem(equipped.slot));
        }
        actions.push(Action::Suicide(SuicideKind::Reincarnate));
        actions.push(Action::Suicide(SuicideKind::EraseSave));

        actions
    }

    pub fn force_factor_limit(&self) -> u32 {
        let Some(mapped) = self.player.mapped_skill(FORCE_ID) else {
            return 0;
        };
        if self.player.skill_by_id(mapped.as_str()).is_some() {
            self.player.skill_level(FORCE_ID) / 2
        } else {
            0
        }
    }

    pub fn mana_factor_limit(&self) -> u32 {
        u32::try_from(self.player.atman.max(0)).unwrap_or_default() / 50 + 1
    }

    pub fn wimpy_percent(&self) -> u8 {
        self.player.wimpy_percent.min(80)
    }

    pub fn set_force_factor(&mut self, requested: u32) {
        let limit = self.force_factor_limit();
        self.player.force_factor = requested.min(limit);
        if requested > limit {
            self.push_log(format!("内力输出最多只能设为{limit}点。"));
        } else if limit == 0 {
            self.push_log("必须先致能一门仍已学会的内功。".into());
        } else if requested == 0 {
            self.push_log("你收敛内力，之后每击不再额外运劲。".into());
        } else {
            self.push_log(format!(
                "之后每次命中会运出{}点内力。",
                self.player.force_factor
            ));
        }
    }

    pub fn set_mana_factor(&mut self, requested: u32) {
        let limit = self.mana_factor_limit();
        self.player.mana_factor = requested.min(limit);
        if requested > limit {
            self.push_log(format!("当前灵力最多只能导引{limit}点强度。"));
        } else if requested == 0 {
            self.push_log("你不再以灵力灌注兵器。".into());
        } else {
            self.push_log(format!("武器灵力强度设为{}。", self.player.mana_factor));
        }
    }

    pub fn set_wimpy_percent(&mut self, requested: u8) {
        self.player.wimpy_percent = requested.min(80);
        self.push_log(format!(
            "气低于{}%时会尝试自动逃跑。",
            self.player.wimpy_percent
        ));
    }

    fn self_learning_actions(&self) -> Vec<Action> {
        const SELF_LEARNABLE: [&str; 7] = [
            DODGE_ID, FORCE_ID, SWORD_ID, "blade", "staff", PARRY_ID, UNARMED_ID,
        ];
        SELF_LEARNABLE
            .into_iter()
            .filter(|skill| self.player.skill_level(skill) >= 40)
            .map(|skill| Action::SelfLearn(SkillId::from(skill)))
            .collect()
    }

    fn item_is_transfer_restricted(item: &ItemInstance) -> bool {
        item.definition()
            .behavior_flags
            .iter()
            .any(|flag| flag == "restricted_movement")
    }

    fn current_container_item(&self, instance_id: u64) -> Option<&ItemInstance> {
        self.player
            .inventory
            .iter()
            .find_map(|item| item.find_nested(instance_id))
            .or_else(|| {
                self.ground_items
                    .get(&self.location)
                    .and_then(|ground| ground.iter().find_map(|item| item.find_nested(instance_id)))
            })
    }

    fn current_container_item_mut(&mut self, instance_id: u64) -> Option<&mut ItemInstance> {
        if let Some(item) = self
            .player
            .inventory
            .iter_mut()
            .find_map(|item| item.find_nested_mut(instance_id))
        {
            return Some(item);
        }
        self.ground_items
            .get_mut(&self.location)
            .and_then(|ground| {
                ground
                    .iter_mut()
                    .find_map(|item| item.find_nested_mut(instance_id))
            })
    }

    fn collect_containers_from(item: &ItemInstance, containers: &mut Vec<u64>) {
        if item.is_container() {
            containers.push(item.instance_id);
        }
        for child in &item.contents {
            Self::collect_containers_from(child, containers);
        }
    }

    fn current_container_ids(&self) -> Vec<u64> {
        let mut containers = Vec::new();
        for item in &self.player.inventory {
            Self::collect_containers_from(item, &mut containers);
        }
        if let Some(ground) = self.ground_items.get(&self.location) {
            for item in ground {
                Self::collect_containers_from(item, &mut containers);
            }
        }
        containers
    }

    fn container_actions(&self) -> Vec<Action> {
        let containers = self.current_container_ids();
        if containers.is_empty() {
            return Vec::new();
        }
        let mut actions = Vec::new();
        for item in &self.player.inventory {
            if self.player.is_equipped(item.instance_id) || Self::item_is_transfer_restricted(item)
            {
                continue;
            }
            for &container_id in &containers {
                if item.contains_instance_id(container_id) {
                    continue;
                }
                actions.push(Action::PutItem {
                    instance_id: item.instance_id,
                    container_id,
                    quantity: item.quantity,
                });
                if item.quantity > 1 {
                    actions.push(Action::PutItem {
                        instance_id: item.instance_id,
                        container_id,
                        quantity: 1,
                    });
                }
            }
        }
        for &container_id in &containers {
            let Some(container) = self.current_container_item(container_id) else {
                continue;
            };
            for item in &container.contents {
                actions.push(Action::TakeFromContainer {
                    instance_id: item.instance_id,
                    container_id,
                    quantity: item.quantity,
                });
                if item.quantity > 1 {
                    actions.push(Action::TakeFromContainer {
                        instance_id: item.instance_id,
                        container_id,
                        quantity: 1,
                    });
                }
            }
        }
        actions
    }

    fn is_zombie_npc(npc: &NpcId) -> bool {
        matches!(npc.as_str(), TEMPLE_PROTECTOR_ID | TEMPLE_TRAINER_ID)
    }

    fn talisman_actions(&self, present_npcs: &[NpcId]) -> Vec<Action> {
        let mut actions = Vec::new();
        let can_scribe = self.player.mapped_skill(SPELLS_ID).map(SkillId::as_str)
            == Some("necromancy")
            && self.player.mana >= 20
            && self.player.spirit >= 40
            && self.player.qi >= 1;
        if can_scribe {
            for paper in
                self.player.inventory.iter().filter(|item| {
                    item.item_id.as_str() == "obj.paper_seal" && item.talisman.is_none()
                })
            {
                for target in present_npcs.iter().filter(|npc| !Self::is_zombie_npc(npc)) {
                    actions.push(Action::ScribeHaunt {
                        paper_instance_id: paper.instance_id,
                        target: target.clone(),
                    });
                }
            }
        }
        let zombies: Vec<_> = present_npcs
            .iter()
            .filter(|npc| Self::is_zombie_npc(npc))
            .cloned()
            .collect();
        if !zombies.is_empty() {
            for talisman in self.player.inventory.iter().filter(|item| {
                item.item_id.as_str() == "obj.magic_seal"
                    && item
                        .talisman
                        .as_ref()
                        .is_some_and(|mark| mark.kind == TalismanKind::Haunt)
            }) {
                for zombie in &zombies {
                    actions.push(Action::AttachTalisman {
                        talisman_instance_id: talisman.instance_id,
                        zombie: zombie.clone(),
                    });
                }
            }
        }
        actions
    }

    fn npc_item_has_been_stolen(&self, npc: &NpcId, slot: usize) -> bool {
        self.stolen_npc_items
            .iter()
            .any(|entry| &entry.npc == npc && entry.slot == slot)
    }

    fn steal_actions(&self, present_npcs: &[NpcId]) -> Vec<Action> {
        if self
            .current_location()
            .behavior_flags
            .iter()
            .any(|flag| flag == "no_fight")
        {
            return Vec::new();
        }
        let mut actions = Vec::new();
        for npc in present_npcs {
            let Some(definition) = npcs().definition(npc) else {
                continue;
            };
            for (slot, carried) in definition.carried_items.iter().enumerate() {
                if !self.npc_item_has_been_stolen(npc, slot) {
                    actions.push(Action::Steal {
                        npc: npc.clone(),
                        item_id: carried.item_id.clone(),
                        slot,
                    });
                }
            }
        }
        actions
    }

    fn item_quantity_suffix(&self, quantity: u32) -> String {
        if quantity == 1 {
            "（一件）".into()
        } else {
            format!("（{}件）", quantity)
        }
    }

    fn container_item_name(&self, instance_id: u64) -> String {
        self.current_container_item(instance_id)
            .map(|item| item.display_name().to_string())
            .unwrap_or_else(|| "未知容器".into())
    }

    fn container_content_name(&self, container_id: u64, instance_id: u64) -> String {
        self.current_container_item(container_id)
            .and_then(|container| {
                container
                    .contents
                    .iter()
                    .find(|item| item.instance_id == instance_id)
            })
            .map(|item| item.display_name().to_string())
            .unwrap_or_else(|| "未知物品".into())
    }

    fn static_npc_location(&self, origin: &LocationId, npc: &NpcId, ordinal: usize) -> LocationId {
        self.npc_location_overrides
            .iter()
            .find(|entry| {
                entry.origin.as_str() == origin.as_str()
                    && entry.npc.as_str() == npc.as_str()
                    && entry.ordinal == ordinal
            })
            .map_or_else(|| origin.clone(), |entry| entry.location.clone())
    }

    fn static_npcs_at(&self, location: &LocationId) -> Vec<NpcId> {
        let mut slots = Vec::new();
        for source_location in world().locations() {
            for (ordinal, npc) in source_location.npcs.iter().enumerate() {
                if self
                    .static_npc_location(&source_location.id, npc, ordinal)
                    .as_str()
                    == location.as_str()
                {
                    slots.push((source_location.id.clone(), ordinal, npc.clone()));
                }
            }
        }
        slots.sort_by(|left, right| {
            left.0
                .as_str()
                .cmp(right.0.as_str())
                .then(left.1.cmp(&right.1))
        });
        slots.into_iter().map(|(_, _, npc)| npc).collect()
    }

    fn static_npc_slots_at(&self, location: &LocationId, npc: &NpcId) -> Vec<(LocationId, usize)> {
        let mut slots = Vec::new();
        for source_location in world().locations() {
            for (ordinal, candidate) in source_location.npcs.iter().enumerate() {
                if candidate.as_str() == npc.as_str()
                    && self
                        .static_npc_location(&source_location.id, candidate, ordinal)
                        .as_str()
                        == location.as_str()
                {
                    slots.push((source_location.id.clone(), ordinal));
                }
            }
        }
        slots.sort_by(|left, right| {
            left.0
                .as_str()
                .cmp(right.0.as_str())
                .then(left.1.cmp(&right.1))
        });
        slots
    }

    fn set_static_npc_location(
        &mut self,
        origin: &LocationId,
        npc: &NpcId,
        ordinal: usize,
        location: LocationId,
    ) {
        let existing = self.npc_location_overrides.iter().position(|entry| {
            entry.origin.as_str() == origin.as_str()
                && entry.npc.as_str() == npc.as_str()
                && entry.ordinal == ordinal
        });
        if location.as_str() == origin.as_str() {
            if let Some(index) = existing {
                self.npc_location_overrides.remove(index);
            }
            return;
        }
        if let Some(index) = existing {
            self.npc_location_overrides[index].location = location;
        } else {
            self.npc_location_overrides.push(NpcLocationOverride {
                origin: origin.clone(),
                npc: npc.clone(),
                ordinal,
                location,
            });
        }
    }

    fn npc_is_present_at(&self, location: &LocationId, npc: &NpcId) -> bool {
        let quest_presence = match npc.as_str() {
            OLD_LIU_ID | CHUENYU_OLD_LIU_ID => {
                !matches!(self.quest, QuestStage::Complete | QuestStage::Failed)
            }
            XIAO_JUAN_ID | CHUENYU_XIAO_JUAN_ID | CHUENYU_XIAO_JUAN_PLACED_ID => {
                self.quest == QuestStage::FoundJuan
            }
            _ => true,
        };
        let total = self
            .static_npcs_at(location)
            .iter()
            .filter(|candidate| candidate.as_str() == npc.as_str())
            .count()
            + self
                .spawned_npc_instances
                .iter()
                .filter(|entry| {
                    entry.location.as_str() == location.as_str()
                        && entry.npc.as_str() == npc.as_str()
                })
                .count();
        let defeated = self
            .defeated_npc_instances
            .iter()
            .filter(|entry| {
                entry.location.as_str() == location.as_str() && entry.npc.as_str() == npc.as_str()
            })
            .count();
        quest_presence
            && (npc.as_str() != SNOW_GUARD_ID || !self.snow_guard_defeated)
            && !self.defeated_npcs.contains(npc)
            && defeated < total
    }

    fn npc_is_present(&self, npc: &NpcId) -> bool {
        self.npc_is_present_at(&self.location, npc)
    }

    fn present_current_npcs(&self) -> Vec<NpcId> {
        let mut seen = HashSet::new();
        self.static_npcs_at(&self.location)
            .into_iter()
            .chain(
                self.spawned_npc_instances
                    .iter()
                    .filter(|entry| entry.location == self.location)
                    .map(|entry| entry.npc.clone()),
            )
            .filter(|npc| seen.insert(npc.clone()) && self.npc_is_present(npc))
            .collect()
    }

    fn current_room_has_npc(&self, npc_id: &str) -> bool {
        self.npc_is_present(&NpcId::from(npc_id))
    }

    fn available_dynamic_quest_targets(&self) -> HashSet<&'static str> {
        let mut targets = HashSet::new();
        for location in world().locations() {
            for npc in self.static_npcs_at(&location.id) {
                if self.npc_is_present_at(&location.id, &npc)
                    && npcs()
                        .definition(&npc)
                        .is_some_and(|definition| definition.is_source_combatant())
                {
                    targets.insert(npc.name());
                }
            }
        }
        for entry in &self.spawned_npc_instances {
            if self.npc_is_present_at(&entry.location, &entry.npc)
                && npcs()
                    .definition(&entry.npc)
                    .is_some_and(|definition| definition.is_source_combatant())
            {
                targets.insert(entry.npc.name());
            }
        }
        targets
    }

    fn dynamic_quest_candidates(&self, tier: u64) -> Vec<&'static quests::TaskDefinition> {
        let targets = self.available_dynamic_quest_targets();
        quests::task_definitions_for_tier(tier)
            .filter(|definition| {
                definition.kind == quests::TaskKind::Kill
                    && targets.contains(definition.target.as_str())
            })
            .collect()
    }

    fn dynamic_quest_has_expired(&self) -> bool {
        self.dynamic_quest
            .as_ref()
            .is_some_and(|quest| self.elapsed_minutes > quest.deadline_elapsed_minutes)
    }

    fn request_dynamic_quest(&mut self) {
        if self.player.combat_experience <= DYNAMIC_QUEST_MIN_COMBAT_EXPERIENCE {
            self.push_log("朱鸿雪奇怪地看着你：就凭你这点本事，也想接悬赏？".into());
            return;
        }

        if let Some(quest) = self.dynamic_quest.as_ref()
            && !self.dynamic_quest_has_expired()
        {
            let target = quest.target.clone();
            let remaining = self.dynamic_quest_remaining_seconds().unwrap_or_default();
            self.push_log(format!(
                "朱鸿雪说道：你仍在追索「{target}」，还剩约{remaining}秒。"
            ));
            return;
        }

        if let Some(expired) = self.dynamic_quest.take() {
            self.player.qi = (self.player.qi / 2 + 1).min(self.player.max_qi);
            self.dynamic_quest_finished = if self.dynamic_quest_finished <= -10 {
                self.dynamic_quest_finished.saturating_sub(1)
            } else {
                0
            };
            self.push_log(format!(
                "朱鸿雪向你一甩袍袖：未能在期限内杀死「{}」，这次算你失手。",
                expired.target
            ));
        }

        let tier =
            quests::task_tier_for(self.player.combat_experience, self.dynamic_quest_finished);
        let candidates = self.dynamic_quest_candidates(tier);
        if candidates.is_empty() {
            self.push_log("朱鸿雪沉吟道：眼下没有仍可追索的合适目标，过些时候再来。".into());
            return;
        }
        let definition = candidates[self.random(candidates.len() as u32) as usize];
        let deadline_elapsed_minutes = self.elapsed_minutes.saturating_add(
            definition
                .time_seconds
                .saturating_mul(GAME_MINUTES_PER_TICK),
        );
        self.dynamic_quest = Some(DynamicQuest {
            target: definition.target.clone(),
            tier: definition.tier,
            deadline_elapsed_minutes,
            exp_bonus: definition.exp_bonus,
            potential_bonus: definition.potential_bonus,
            score_bonus: definition.score_bonus,
            factor: DYNAMIC_QUEST_FACTOR,
        });
        self.push_log(format!(
            "朱鸿雪沉思了一会儿，说道：请在{}秒内替我杀了「{}」。",
            definition.time_seconds, definition.target
        ));
    }

    fn scripted_inquiry_is_available(&self, npc: &NpcId, topic: &str) -> bool {
        !matches!(
            (npc.as_str(), topic),
            (CHOYIN_GIRL_ID, "游晋") if self.choyin_silk_bag_received
        ) && !matches!(
            (npc.as_str(), topic),
            (CHOYIN_YOUNG_MAN_ID, "trouble") if self.choyin_silk_bag_delivered
        ) && !matches!(
            (npc.as_str(), topic),
            (GREEN_SHEN_ID, "玉佩")
                if !self.green_drunk_jade_clue || self.green_jade_received
        ) && !matches!(
            (npc.as_str(), topic),
            (GREEN_SHEN_ID, "蒙汗药")
                if !self.green_drunk_drug_clue || self.green_drug_offer_unlocked
        )
    }

    fn money_offer_is_available(&self, kind: ObjectExchangeKind) -> bool {
        match kind {
            ObjectExchangeKind::CanyonAdviser => !self.canyon_secret_clue,
            ObjectExchangeKind::CanyonCaptain => !self.canyon_camp_access,
            ObjectExchangeKind::CanyonSeller => !self.canyon_fake_seal_bought,
            ObjectExchangeKind::CityWaiter => !self.city_inn_access,
            ObjectExchangeKind::CityShangshuGuard => !self.city_manor_pass,
            ObjectExchangeKind::SnowTempleDonation => true,
            ObjectExchangeKind::CommonerDonation
            | ObjectExchangeKind::CanyonGeneral
            | ObjectExchangeKind::ChoyinSergeant
            | ObjectExchangeKind::ChoyinYoungMan
            | ObjectExchangeKind::CityGuardToken
            | ObjectExchangeKind::GreenShen
            | ObjectExchangeKind::LatemoonFunlin
            | ObjectExchangeKind::LatemoonOld
            | ObjectExchangeKind::LatemoonShaowei
            | ObjectExchangeKind::CloudBHeader
            | ObjectExchangeKind::CloudBoater
            | ObjectExchangeKind::CloudGangster
            | ObjectExchangeKind::CloudGirl
            | ObjectExchangeKind::CloudJudge
            | ObjectExchangeKind::CloudMonk
            | ObjectExchangeKind::CityChenLetter
            | ObjectExchangeKind::ScavengerDonation
            | ObjectExchangeKind::SnowDrunk
            | ObjectExchangeKind::TeacherTuition => false,
        }
    }

    fn npc_lesson_access(&self, policy: NpcApprenticeshipPolicy) -> bool {
        match policy {
            NpcApprenticeshipPolicy::RecognizeFaction(faction)
            | NpcApprenticeshipPolicy::SameFaction(faction) => {
                self.player.faction.as_deref() == Some(faction)
            }
            NpcApprenticeshipPolicy::PaidStudent => self.snow_teacher_paid,
            NpcApprenticeshipPolicy::DeferredLetter => self.city_chen_letter_delivered,
            NpcApprenticeshipPolicy::PlotGated => self.cloud_escort_member,
            NpcApprenticeshipPolicy::ExcludedUnplaced => false,
        }
    }

    fn resolved_source_exit_target(&self, current: &Location, exit: &Exit) -> LocationId {
        let Some(targets) = content::dynamic_exit_target_candidates(&current.id, &exit.target)
        else {
            return exit.target.clone();
        };
        let hash = current
            .id
            .as_str()
            .bytes()
            .chain(exit.direction.bytes())
            .fold(self.rng_state, |hash, byte| {
                hash.wrapping_mul(1_099_511_628_211)
                    .wrapping_add(u64::from(byte))
            });
        LocationId::new(targets[hash as usize % targets.len()])
    }

    fn exit_is_available(&self, current: &Location, exit: &Exit, target: &LocationId) -> bool {
        let interaction_only = matches!(
            (current.id.as_str(), exit.target.as_str()),
            (content::LAKESIDE, content::LAKE) | (content::LAKE, content::LAKESIDE)
        );
        let dynamic_exit_closed = exit.dynamic
            && !match (current.id.as_str(), exit.direction.as_str()) {
                (content::ROAD6, "west") => self.hidden_grass_path_ticks > 0,
                (CITY_ALTAR, "down") => self.city_altar_passage_ticks > 0,
                (SNOW_WEAPON_STORAGE, "down") => self.snow_storage_passage_ticks > 0,
                (CANYON_BAMBOO_BOULDER, "enter") => self.canyon_boulder_open,
                ("choyin.platform", "down") => self.choyin_platform_passage_ticks > 0,
                ("chuenyu.tunnel4", "up") | ("chuenyu.east_castle", "down") => {
                    self.chuenyu_slab_passage_ticks > 0
                }
                _ => false,
            };
        let closed_door =
            door_for_transition(&current.id, target).is_some_and(|door| !self.is_door_open(door));
        let closed_source_door = self
            .source_door_is_open(&current.id, target)
            .is_some_and(|open| !open);
        let access_denied = match (current.id.as_str(), target.as_str()) {
            (content::CANYON_CAMP7, content::CANYON_CAMP8) => !self.canyon_camp_access,
            (content::CITY_INN, content::CITY_INN_UPSTAIRS) => !self.city_inn_access,
            (content::CITY_MANOR_GATE, content::CITY_MANOR_YARD) => !self.city_manor_pass,
            (content::CITY_MANOR_ROAD_TWO, target) if target != content::CITY_MANOR_YARD => {
                !self.city_manor_pass
                    && (self.current_room_has_npc(CITY_SHANGSHU_PATROL_ID)
                        || self.current_room_has_npc(CITY_SHANGSHU_PATROL_ELITE_ID))
            }
            (content::CITY_NORTH_GATE, content::CITY_NORTH_ROAD) => !self.city_exit_permit,
            (content::TEMPLE_ROAD_TWO, TEMPLE_BOOK_ROOM) => {
                self.player.faction.as_deref() != Some("茅山派")
            }
            ("oldpine.keep2", "oldpine.keep1") | ("oldpine.keep1", "oldpine.keep2") => {
                self.oldpine_keep_sealed
            }
            ("choyin.entrance", "choyin.taolin") => !self.choyin_scholar_trial_started,
            ("green.entrance", "green.eight0") => self.player.combat_experience < 100_000,
            ("death.gateway", "death.gate") => true,
            ("u.cloud.sunhill.northriver", "u.cloud.sunhill.midriver") => !self.cloud_boater_paid,
            _ => false,
        };
        world().contains(target)
            && !interaction_only
            && !dynamic_exit_closed
            && !closed_door
            && !closed_source_door
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

    fn source_door_name(&self, target: &LocationId) -> &'static str {
        source_door_pair(&self.location, target).map_or("门", |(door, _)| door.name.as_str())
    }

    fn source_door_is_open(&self, source: &LocationId, target: &LocationId) -> Option<bool> {
        let (door, reverse) = source_door_pair(source, target)?;
        let (first, second) = canonical_location_pair(source, target);
        self.source_door_states
            .iter()
            .find(|state| state.first == first && state.second == second)
            .map(|state| state.open)
            .or(Some(!(door.initially_closed || reverse.initially_closed)))
    }

    fn inspect_room_detail(&mut self, key: &str) {
        let current = self.current_location();
        let Some(detail) = current.details.iter().find(|detail| detail.key == key) else {
            self.push_log("这里没有可细看的东西。".into());
            return;
        };
        let green_web = current.id.as_str() == "green.house3" && key == "web";
        let green_spiders_spawned = self
            .spawned_npc_instances
            .iter()
            .filter(|entry| {
                entry.location.as_str() == "green.house3" && entry.npc.as_str() == GREEN_SPIDER_ID
            })
            .count();
        let message = if green_web && green_spiders_spawned >= 3 {
            "一个很大的蜘蛛网。".into()
        } else if let Some(direction) = &detail.door_direction {
            let door = current
                .doors
                .iter()
                .find(|door| &door.direction == direction);
            let Some(door) = door else {
                self.push_log("这扇门没有留下可辨认的状态。".into());
                return;
            };
            let open = current
                .exits
                .iter()
                .find(|exit| &exit.direction == direction)
                .and_then(|exit| self.source_door_is_open(&current.id, &exit.target))
                .unwrap_or(!door.initially_closed);
            format!("{}现在{}。", door.name, if open { "开着" } else { "关着" })
        } else {
            detail
                .description
                .clone()
                .unwrap_or_else(|| "这里没有更多可辨认的细节。".into())
        };
        self.push_log(message);
        if green_web && green_spiders_spawned < 3 {
            self.spawned_npc_instances.push(SpawnedNpcInstance {
                location: self.location.clone(),
                npc: NpcId::from(GREEN_SPIDER_ID),
            });
            self.push_log("屋角阴影一动，一只硕大的蜘蛛从网后爬了出来。".into());
        }
    }

    fn set_source_door_open(&mut self, target: LocationId, open: bool) {
        let name = self.source_door_name(&target).to_string();
        let (first, second) = canonical_location_pair(&self.location, &target);
        if let Some(state) = self
            .source_door_states
            .iter_mut()
            .find(|state| state.first == first && state.second == second)
        {
            state.open = open;
        } else {
            self.source_door_states.push(SourceDoorState {
                first,
                second,
                open,
            });
        }
        let action = if open { "打开" } else { "关上" };
        self.push_log(format!("你{action}了{name}。"));
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
            Action::Move { direction, .. } if self.location.as_str() == "choyin.taolin" => {
                self.move_through_choyin_taolin(&direction);
            }
            Action::Move { direction, target }
                if self.location.as_str() == "death.road2" && direction == "north" =>
            {
                self.advance_death_road(target);
            }
            Action::Move { target, .. } => self.move_to(target),
            Action::Flee { target, .. } => self.flee_to(target),
            Action::OpenSourceDoor { target } => self.set_source_door_open(target, true),
            Action::CloseSourceDoor { target } => self.set_source_door_open(target, false),
            Action::InspectRoomDetail(key) => self.inspect_room_detail(&key),
            Action::Interact(interaction) => self.interact(interaction),
            Action::Talk(npc) => self.talk(npc),
            Action::RequestDynamicQuest => self.request_dynamic_quest(),
            Action::AskNpc { npc, topic } => self.ask_npc(npc, &topic),
            Action::BecomeApprentice(teacher) => self.become_apprentice(teacher),
            Action::LearnSkill { skill, teacher } => self.learn_skill(skill, teacher),
            Action::LearnFromNpc { skill, npc } => self.learn_from_npc(skill, npc),
            Action::MapSkill { usage, skill } => self.map_skill(usage, skill),
            Action::AbandonSkill(skill) => self.abandon_skill(skill),
            Action::ConfigureCombat => {
                self.push_log("请用战斗设置面板调整内力、灵力与自动逃跑。".into())
            }
            Action::SelfLearn(skill) => self.self_learn(skill),
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
            Action::DonateItem(instance_id) => self.donate_item(instance_id),
            Action::PickUpItem(instance_id) => self.pick_up_item(instance_id),
            Action::DropItem(instance_id) => self.drop_item(instance_id),
            Action::PutItem {
                instance_id,
                container_id,
                quantity,
            } => self.put_item(instance_id, container_id, quantity),
            Action::TakeFromContainer {
                instance_id,
                container_id,
                quantity,
            } => self.take_from_container(instance_id, container_id, quantity),
            Action::ScribeHaunt {
                paper_instance_id,
                target,
            } => self.scribe_haunt(paper_instance_id, target),
            Action::AttachTalisman {
                talisman_instance_id,
                zombie,
            } => self.attach_talisman(talisman_instance_id, zombie),
            Action::Steal { npc, item_id, slot } => self.start_steal(npc, item_id, slot),
            Action::Suicide(SuicideKind::Reincarnate) => self.reincarnate_player(),
            Action::Suicide(SuicideKind::EraseSave) => {
                self.push_log("永久删除需要在确认弹层完成。".into())
            }
            Action::EquipItem(instance_id) => self.equip_item(instance_id),
            Action::UnequipItem(slot) => self.unequip_item(slot),
            Action::ConsumeItem(instance_id) => self.consume_item(instance_id),
            Action::ApplyItem(instance_id) => self.apply_item(instance_id),
            Action::MixIntoLiquid {
                powder_instance_id,
                liquid_instance_id,
            } => self.mix_into_liquid(powder_instance_id, liquid_instance_id),
            Action::DissolveCorpse {
                dust_instance_id,
                corpse_instance_id,
            } => self.dissolve_corpse(dust_instance_id, corpse_instance_id),
            Action::Surrender => self.surrender(),
        }
    }

    pub fn tick(&mut self) {
        if self.trigger_old_liu_revenge() {
            return;
        }
        let previous_day_phase = Self::day_phase_index(self.elapsed_minutes);
        let previous_day = self.elapsed_minutes / DAY_MINUTES;
        self.elapsed_minutes += GAME_MINUTES_PER_TICK;
        if previous_day != self.elapsed_minutes / DAY_MINUTES {
            self.reset_scheduled_npc_positions();
        }
        self.expire_zombie_haunts();
        self.advance_npc_respawns();
        if self.current_location().outdoors.is_some()
            && previous_day_phase != Self::day_phase_index(self.elapsed_minutes)
        {
            self.push_log(self.day_phase().transition_message.into());
        }
        self.expire_temporary_items();
        self.player.food = self.player.food.saturating_sub(1);
        self.player.water = self.player.water.saturating_sub(1);
        if self.hidden_grass_path_ticks > 0 {
            self.hidden_grass_path_ticks -= 1;
            if self.hidden_grass_path_ticks == 0 && self.location.as_str() == content::ROAD6 {
                self.push_log("茅草重新合拢，西面的隐秘小路消失了。".into());
            }
        }
        if self.chuenyu_slab_passage_ticks > 0 {
            self.chuenyu_slab_passage_ticks -= 1;
            if self.chuenyu_slab_passage_ticks == 0
                && matches!(
                    self.location.as_str(),
                    "chuenyu.tunnel4" | "chuenyu.east_castle"
                )
            {
                self.push_log("地牢石板轰然落回原位，上下通道重新封闭。".into());
            }
        }
        if self.location.as_str() == "chuenyu.trap_castle" {
            if self.chuenyu_trap_arrow_ticks == 0 {
                self.chuenyu_trap_arrow_ticks = 2;
            } else {
                self.chuenyu_trap_arrow_ticks -= 1;
                if self.chuenyu_trap_arrow_ticks == 0 {
                    let damage = 35 + self.random(10) as i32;
                    self.player.qi = (self.player.qi - damage).max(0);
                    self.chuenyu_trap_arrow_ticks = 1;
                    self.push_log(format!("石墙圆孔射出密集羽箭，你损失{damage}点气。"));
                }
            }
        } else {
            self.chuenyu_trap_arrow_ticks = 0;
        }
        if self.city_altar_passage_ticks > 0 && self.location.as_str() != CITY_ALTAR_TUNNEL {
            self.city_altar_passage_ticks -= 1;
            if self.city_altar_passage_ticks == 0 && self.location.as_str() == CITY_ALTAR {
                self.push_log("祭坛地板轧轧合拢，向下的阶梯消失了。".into());
            }
        }
        if self.snow_storage_passage_ticks > 0 && self.location.as_str() != SNOW_SECRET_STORAGE {
            self.snow_storage_passage_ticks -= 1;
            if self.snow_storage_passage_ticks == 0 && self.location.as_str() == SNOW_WEAPON_STORAGE
            {
                self.push_log("兵器架后的地板缓缓合拢，密道阶梯再次隐没。".into());
            }
        }
        if self.choyin_platform_passage_ticks > 0 {
            self.choyin_platform_passage_ticks -= 1;
            if self.choyin_platform_passage_ticks == 0
                && self.location.as_str() == "choyin.platform"
            {
                self.push_log("一道蓝光掠过，云台下行的裂口重新合拢。".into());
            }
        }
        if self.choyin_thunder_ticks > 0 {
            if self.location.as_str() == "choyin.platform" {
                self.choyin_thunder_ticks -= 1;
                if self.choyin_thunder_ticks == 0 {
                    let damage = 35 + self.random(10) as i32;
                    self.player.essence = (self.player.essence - damage).max(0);
                    self.choyin_thunder_ticks = 1;
                    self.push_log(format!("雷霆劈落云台，你损失{damage}点精。"));
                }
            } else {
                self.choyin_thunder_ticks = 0;
            }
        }
        self.advance_choyin_justice();
        if !matches!(self.activity, Activity::Fighting(_) | Activity::Stealing(_)) {
            self.run_npc_ambient_chat();
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
            Activity::Stealing(state) => self.steal_tick(state),
            Activity::Fighting(combat) => self.combat_tick(combat),
        }
        self.update_conditions();
    }

    pub(crate) fn advance_offline_progress(
        &mut self,
        now_unix_seconds: u64,
        file_modified_unix_seconds: Option<u64>,
    ) {
        let Some(saved_at) = self.last_saved_at_unix_seconds else {
            return;
        };
        let reference =
            file_modified_unix_seconds.map_or(saved_at, |modified_at| saved_at.max(modified_at));
        self.last_saved_at_unix_seconds = Some(now_unix_seconds);

        if reference > now_unix_seconds {
            self.push_log("检测到本地时钟回拨或存档时间异常，未结算离线时间。".into());
            return;
        }

        let elapsed_seconds = now_unix_seconds - reference;
        let uncapped_ticks = elapsed_seconds / OFFLINE_REAL_SECONDS_PER_TICK;
        let ticks = uncapped_ticks.min(MAX_OFFLINE_TICKS);
        if ticks == 0 {
            return;
        }

        self.elapsed_minutes = self
            .elapsed_minutes
            .saturating_add(ticks.saturating_mul(GAME_MINUTES_PER_TICK));
        let decay = i32::try_from(ticks).expect("offline progress cap fits in i32");
        self.player.food = self.player.food.saturating_sub(decay);
        self.player.water = self.player.water.saturating_sub(decay);
        self.expire_temporary_items();

        let world_minutes = ticks * GAME_MINUTES_PER_TICK;
        if uncapped_ticks > MAX_OFFLINE_TICKS {
            self.push_log(format!(
                "离线 {} 分钟，离线推进最多结算 8 小时；世界仅推进 {} 小时 {} 分。",
                elapsed_seconds / 60,
                world_minutes / 60,
                world_minutes % 60
            ));
        } else {
            self.push_log(format!(
                "离线 {} 分钟，世界推进 {} 小时 {} 分；未结算战斗、修炼、休息或状态效果。",
                elapsed_seconds / 60,
                world_minutes / 60,
                world_minutes % 60
            ));
        }
    }

    pub fn time_text(&self) -> String {
        let day = self.elapsed_minutes / DAY_MINUTES + 1;
        let minutes = self.elapsed_minutes % DAY_MINUTES;
        format!("第{day}日 {:02}:{:02}", minutes / 60, minutes % 60)
    }

    pub fn time_period_text(&self) -> &'static str {
        self.day_phase().name
    }

    pub fn outdoor_time_description(&self) -> Option<&'static str> {
        self.current_location()
            .outdoors
            .as_ref()
            .map(|_| self.day_phase().outdoor_description)
    }

    pub fn location_description(&self) -> String {
        let location = self.current_location();
        self.outdoor_time_description().map_or_else(
            || location.description.clone(),
            |outdoor_description| {
                format!(
                    "{}\n  {outdoor_description}。",
                    location.description.trim_end()
                )
            },
        )
    }

    pub fn activity_text(&self) -> String {
        match self.choyin_justice {
            ChoyinJusticeState::Pursuit => return "巡捕缉拿中".into(),
            ChoyinJusticeState::Caught => return "被巡捕拘押".into(),
            ChoyinJusticeState::AwaitingJudgment => return "押往县衙受审".into(),
            ChoyinJusticeState::Free => {}
        }
        match &self.activity {
            Activity::Idle => "整装待发".into(),
            Activity::Resting => "正在休息".into(),
            Activity::Training(skill) => format!("修炼{}中", skill.name()),
            Activity::Stealing(state) => {
                format!("伺机偷取{}的{}", state.npc.name(), state.item_id.as_str())
            }
            Activity::Fighting(combat) => match combat.mode {
                CombatMode::Spar => format!("与{}比试", combat.enemy.name()),
                CombatMode::Lethal => format!("与{}死斗", combat.enemy.name()),
            },
        }
    }

    pub fn dynamic_quest(&self) -> Option<&DynamicQuest> {
        self.dynamic_quest.as_ref()
    }

    pub fn dynamic_quest_finished(&self) -> i32 {
        self.dynamic_quest_finished
    }

    pub fn dynamic_quest_remaining_seconds(&self) -> Option<u64> {
        let quest = self.dynamic_quest.as_ref()?;
        let remaining_minutes = quest
            .deadline_elapsed_minutes
            .saturating_sub(self.elapsed_minutes);
        Some(remaining_minutes.div_ceil(GAME_MINUTES_PER_TICK))
    }

    pub fn quest_title(&self) -> String {
        if let Some(quest) = &self.dynamic_quest {
            return if self.dynamic_quest_has_expired() {
                "朱鸿雪悬赏 · 已逾期".into()
            } else {
                format!("朱鸿雪悬赏 · 阶位{}", quest.tier)
            };
        }
        match self.quest {
            QuestStage::Unasked => "山村旧事".into(),
            QuestStage::FindJuan => "寻找娟儿".into(),
            QuestStage::FoundJuan => "救出娟儿".into(),
            QuestStage::ReturnHome => "平安归来".into(),
            QuestStage::MurderedJuan => "父女之殇".into(),
            QuestStage::Complete => "山村旧事 · 已完成".into(),
            QuestStage::Failed => "山村旧事 · 已失败".into(),
        }
    }

    pub fn quest_objective(&self) -> String {
        if let Some(quest) = &self.dynamic_quest {
            return if self.dynamic_quest_has_expired() {
                format!(
                    "未能及时杀死「{}」。返回朱鸿雪处重新领受任务。",
                    quest.target
                )
            } else {
                format!(
                    "替朱鸿雪杀死「{}」。剩余约{}秒（世界时间）。",
                    quest.target,
                    self.dynamic_quest_remaining_seconds().unwrap_or_default()
                )
            };
        }
        match self.quest {
            QuestStage::Unasked => "刘老农似乎有心事。去刘家小房问问他。".into(),
            QuestStage::FindJuan => "娟儿在松林附近失踪。前往松林寻找她。".into(),
            QuestStage::FoundJuan => "小娟已经脱险。与她交谈并护送她回家。".into(),
            QuestStage::ReturnHome => "娟儿已经脱险。回刘家小房向刘老农报平安。".into(),
            QuestStage::MurderedJuan => "你杀害了小娟。刘老农绝不会原谅此事。".into(),
            QuestStage::Complete => "刘家父女已经离开山村。你可以继续游历和修炼。".into(),
            QuestStage::Failed => "刘家父女的命运已无法挽回。".into(),
        }
    }

    pub fn identity_lines(&self) -> Vec<String> {
        let player = &self.player;
        let gender = match player.gender {
            Gender::Male => "男",
            Gender::Female => "女",
        };
        let default_title = if player.combat_experience >= 100_000 {
            "江湖前辈"
        } else if player.combat_experience >= 20_000 {
            "江湖侠客"
        } else {
            "初入江湖"
        };
        let title = player.title.as_deref().unwrap_or(default_title);
        let standing = if player.wanted > 0 {
            format!("被通缉（{}级）", player.wanted)
        } else if player.bellicosity > 0 {
            format!("杀气 {}", player.bellicosity)
        } else {
            "行止自由".into()
        };
        let mut lines = vec![
            format!("姓名：{}    性别：{}", player.name, gender),
            format!("名号：{}", title),
            format!("师门：{}", player.faction.as_deref().unwrap_or("无门无派")),
            format!("师承：{}", player.teacher.as_deref().unwrap_or("暂无")),
            format!(
                "实战经验：{}    江湖评价：{:+}",
                player.combat_experience, player.reputation
            ),
            format!("江湖状态：{}", standing),
        ];
        if let Some(description) = player
            .description
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            lines.push(String::new());
            lines.push(format!("自述：{description}"));
        }
        lines
    }

    pub fn inventory_lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "负重 {:.1}/{:.1} 斤",
            self.player.carried_weight() as f32 / 500.0,
            self.player.carry_capacity() as f32 / 500.0
        )];
        for item in &self.player.inventory {
            Self::append_inventory_lines(&mut lines, item, &self.player.equipment, 0);
        }
        lines
    }

    fn append_inventory_lines(
        lines: &mut Vec<String>,
        item: &ItemInstance,
        equipment: &[EquippedItem],
        depth: usize,
    ) {
        let definition = item.definition();
        let quantity = if item.quantity > 1 {
            format!(" ×{}", item.quantity)
        } else {
            String::new()
        };
        let equipped = equipment
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
        let talisman = item
            .talisman
            .as_ref()
            .map_or_else(String::new, |mark| match mark.kind {
                TalismanKind::Haunt => format!(" · 追魂：{}", mark.target),
            });
        let indent = "  ".repeat(depth);
        lines.push(format!(
            "{indent}• {}{}{}{}{}{}",
            item.display_name(),
            quantity,
            equipped,
            durability,
            uses,
            talisman
        ));
        if let Some(capacity) = item.container_capacity() {
            lines.push(format!(
                "{indent}  容量 {:.1}/{:.1} 斤",
                item.contents_weight() as f32 / 500.0,
                capacity as f32 / 500.0
            ));
            for child in &item.contents {
                Self::append_inventory_lines(lines, child, equipment, depth.saturating_add(1));
            }
        }
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

    fn current_ground_has(&self, item_id: &str) -> bool {
        self.ground_items
            .get(&self.location)
            .is_some_and(|items| items.iter().any(|item| item.item_id.as_str() == item_id))
    }

    fn temporary_item_lifetime_ticks(item_id: &str) -> Option<u64> {
        GOATHILL_DEAD_LEECH_IDS
            .contains(&item_id)
            .then_some(GOATHILL_DEAD_LEECH_DECAY_TICKS)
    }

    fn corpse_stage_ticks(stage: u8) -> u64 {
        match stage {
            0 => CORPSE_FRESH_DECAY_TICKS,
            1 => CORPSE_ROTTEN_DECAY_TICKS,
            _ => CORPSE_BONE_DECAY_TICKS,
        }
    }

    fn schedule_temporary_item_expiry(&self, item: &mut ItemInstance) {
        if item.expires_at_elapsed_minutes.is_some() {
            return;
        }
        if item.item_id.as_str() == CORPSE_ITEM_ID {
            item.expires_at_elapsed_minutes = Some(
                self.elapsed_minutes.saturating_add(
                    Self::corpse_stage_ticks(item.lifecycle_stage)
                        .saturating_mul(GAME_MINUTES_PER_TICK),
                ),
            );
            return;
        }
        let Some(lifetime_ticks) = Self::temporary_item_lifetime_ticks(item.item_id.as_str())
        else {
            return;
        };
        item.expires_at_elapsed_minutes = Some(
            self.elapsed_minutes
                .saturating_add(lifetime_ticks.saturating_mul(GAME_MINUTES_PER_TICK)),
        );
    }

    fn initialize_temporary_item_expirations_recursively(
        item: &mut ItemInstance,
        deadline: u64,
        corpse_deadline: u64,
    ) {
        if item.item_id.as_str() == CORPSE_ITEM_ID && item.expires_at_elapsed_minutes.is_none() {
            item.lifecycle_stage = 0;
            item.expires_at_elapsed_minutes = Some(corpse_deadline);
        }
        if item.expires_at_elapsed_minutes.is_none()
            && Self::temporary_item_lifetime_ticks(item.item_id.as_str()).is_some()
        {
            item.expires_at_elapsed_minutes = Some(deadline);
        }
        for child in &mut item.contents {
            Self::initialize_temporary_item_expirations_recursively(
                child,
                deadline,
                corpse_deadline,
            );
        }
    }

    fn initialize_m8_temporary_item_expirations(&mut self) {
        let deadline = self
            .elapsed_minutes
            .saturating_add(GOATHILL_DEAD_LEECH_DECAY_TICKS.saturating_mul(GAME_MINUTES_PER_TICK));
        let corpse_deadline = self
            .elapsed_minutes
            .saturating_add(CORPSE_FRESH_DECAY_TICKS.saturating_mul(GAME_MINUTES_PER_TICK));
        for item in self
            .player
            .inventory
            .iter_mut()
            .chain(self.ground_items.values_mut().flatten())
        {
            Self::initialize_temporary_item_expirations_recursively(
                item,
                deadline,
                corpse_deadline,
            );
        }
    }

    fn advance_temporary_item(item: &mut ItemInstance, now: u64) -> (bool, Vec<String>) {
        let mut messages = Vec::new();
        while item
            .expires_at_elapsed_minutes
            .is_some_and(|deadline| deadline <= now)
        {
            if item.item_id.as_str() != CORPSE_ITEM_ID {
                return (false, messages);
            }

            let deadline = item
                .expires_at_elapsed_minutes
                .expect("expired temporary item must have a deadline");
            let name = item.display_name().to_string();
            match item.lifecycle_stage {
                0 => {
                    item.lifecycle_stage = 1;
                    item.transformed_name = Some("腐烂的尸体".into());
                    item.expires_at_elapsed_minutes = Some(deadline.saturating_add(
                        CORPSE_ROTTEN_DECAY_TICKS.saturating_mul(GAME_MINUTES_PER_TICK),
                    ));
                    messages.push(format!("{name}开始腐烂了，发出一股难闻的恶臭。"));
                }
                1 => {
                    item.lifecycle_stage = 2;
                    item.transformed_name = Some("一具枯干的骸骨".into());
                    item.expires_at_elapsed_minutes = Some(deadline.saturating_add(
                        CORPSE_BONE_DECAY_TICKS.saturating_mul(GAME_MINUTES_PER_TICK),
                    ));
                    messages.push(format!("{name}被风吹干了，变成一具骸骨。"));
                }
                _ => {
                    messages.push(format!("一阵风吹过，把{name}化成骨灰吹散了。"));
                    return (false, messages);
                }
            }
        }
        (true, messages)
    }

    fn advance_temporary_item_tree(
        item: &mut ItemInstance,
        now: u64,
    ) -> (bool, Vec<String>, Vec<ItemInstance>) {
        let (keep, mut messages) = Self::advance_temporary_item(item, now);
        if !keep {
            return (false, messages, std::mem::take(&mut item.contents));
        }

        let mut kept_contents = Vec::new();
        for mut child in std::mem::take(&mut item.contents) {
            let (child_keep, child_messages, spilled) =
                Self::advance_temporary_item_tree(&mut child, now);
            messages.extend(child_messages);
            if child_keep {
                kept_contents.push(child);
            }
            kept_contents.extend(spilled);
        }
        item.contents = kept_contents;
        (true, messages, Vec::new())
    }

    fn expire_temporary_items(&mut self) {
        let now = self.elapsed_minutes;
        let mut expired_inventory = Vec::new();
        let mut inventory_messages = Vec::new();
        let mut inventory_spill = Vec::new();
        let mut surviving_inventory = Vec::new();
        for mut item in std::mem::take(&mut self.player.inventory) {
            let name = item.display_name().to_string();
            let was_corpse = item.item_id.as_str() == CORPSE_ITEM_ID;
            let (keep, messages, spilled) = Self::advance_temporary_item_tree(&mut item, now);
            inventory_messages.extend(messages);
            if keep {
                surviving_inventory.push(item);
            } else {
                if !was_corpse {
                    expired_inventory.push(name);
                }
                inventory_spill.extend(spilled);
            }
        }
        self.player.inventory = surviving_inventory;
        if !expired_inventory.is_empty() {
            let surviving = self
                .player
                .inventory
                .iter()
                .map(|item| item.instance_id)
                .collect::<HashSet<_>>();
            self.player
                .equipment
                .retain(|equipped| surviving.contains(&equipped.instance_id));
            self.push_log(format!(
                "行囊中的{}已经腐坏。",
                expired_inventory.join("、")
            ));
        }
        if !inventory_spill.is_empty() {
            let names = inventory_spill
                .iter()
                .map(|item| item.display_name().to_string())
                .collect::<Vec<_>>()
                .join("、");
            for item in inventory_spill {
                self.merge_ground_item(item);
            }
            self.push_log(format!("行囊中的遗体消散，遗物落在地上：{names}。"));
        }
        for message in inventory_messages {
            self.push_log(message);
        }

        let current_location = self.location.clone();
        let mut expired_here = Vec::new();
        let mut ground_messages = Vec::new();
        for (location, ground) in &mut self.ground_items {
            let mut retained = Vec::new();
            for mut item in std::mem::take(ground) {
                let name = item.display_name().to_string();
                let was_corpse = item.item_id.as_str() == CORPSE_ITEM_ID;
                let (keep, messages, spilled) = Self::advance_temporary_item_tree(&mut item, now);
                if location == &current_location {
                    ground_messages.extend(messages);
                }
                if keep {
                    retained.push(item);
                } else {
                    if !was_corpse && location == &current_location {
                        expired_here.push(name.clone());
                    }
                    if !spilled.is_empty() {
                        if location == &current_location {
                            ground_messages.push(format!("{name}消散后，遗物散落在地上。"));
                        }
                        retained.extend(spilled);
                    }
                }
            }
            *ground = retained;
        }
        self.ground_items.retain(|_, ground| !ground.is_empty());
        if !expired_here.is_empty() {
            self.push_log(format!("地上的{}已经腐坏。", expired_here.join("、")));
        }
        for message in ground_messages {
            self.push_log(message);
        }
    }

    fn initialize_source_room_items(&mut self) {
        if self.source_room_items_initialized {
            return;
        }
        self.initialize_source_room_items_for(&["village", "city", "snow", "temple", "canyon"]);
        self.source_room_items_initialized = true;
    }

    fn initialize_m5_source_room_items(&mut self) {
        if self.m5_source_room_items_initialized {
            return;
        }
        self.initialize_source_room_items_for(&["oldpine", "goathill", "choyin"]);
        self.m5_source_room_items_initialized = true;
    }

    fn initialize_m6_source_room_items(&mut self) {
        if self.m6_source_room_items_initialized {
            return;
        }
        self.initialize_source_room_items_for(&["chuenyu", "green", "sanyen", "waterfog"]);
        self.m6_source_room_items_initialized = true;
    }

    fn initialize_m7_source_room_items(&mut self) {
        if self.m7_source_room_items_initialized {
            return;
        }
        self.initialize_source_room_items_for(&[
            "latemoon",
            "death",
            "graveyard",
            "jail",
            "u.cloud",
        ]);
        self.m7_source_room_items_initialized = true;
    }

    fn initialize_source_room_items_for(&mut self, location_prefixes: &[&str]) {
        let mut placements: Vec<_> = world()
            .locations()
            .filter(|location| {
                location_prefixes
                    .iter()
                    .any(|prefix| location.id.as_str().starts_with(&format!("{prefix}.")))
            })
            .flat_map(|location| {
                location.room_items.iter().map(|placement| {
                    (
                        location.id.clone(),
                        placement.item_id.clone(),
                        placement.count,
                    )
                })
            })
            .collect();
        placements.sort_by(|left, right| {
            left.0
                .as_str()
                .cmp(right.0.as_str())
                .then_with(|| left.1.as_str().cmp(right.1.as_str()))
        });
        for (location, item_id, count) in placements {
            for _ in 0..count {
                let instance_id = self.allocate_item_instance_id();
                self.ground_items
                    .entry(location.clone())
                    .or_default()
                    .push(ItemInstance::new(instance_id, item_id.clone(), 1));
            }
        }
    }

    fn allocate_item_instance_id(&mut self) -> u64 {
        let inventory_max = self
            .player
            .inventory
            .iter()
            .map(ItemInstance::max_nested_instance_id)
            .max()
            .unwrap_or(0);
        let ground_max = self
            .ground_items
            .values()
            .flatten()
            .map(ItemInstance::max_nested_instance_id)
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
            let mut item = ItemInstance::new(
                instance_id,
                item_id.clone(),
                if definition.stackable() { quantity } else { 1 },
            );
            self.schedule_temporary_item_expiry(&mut item);
            self.player.inventory.push(item);
        }
        first_id
    }

    fn offer_money(&mut self, amount: u64, npc: NpcId) {
        if npc.as_str() == CHOYIN_POLICE_ID
            && amount == CHOYIN_BRIBE_AMOUNT
            && self.choyin_justice.blocks_actions()
        {
            self.bribe_choyin_police();
            return;
        }
        if !self.npc_is_present(&npc) {
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
            ObjectExchangeKind::SnowTempleDonation => {
                self.apply_snow_temple_donation(amount);
                self.push_log("庙祝收下香火钱：神明一定会保佑你的。".into());
            }
            ObjectExchangeKind::CommonerDonation
            | ObjectExchangeKind::CanyonGeneral
            | ObjectExchangeKind::ChoyinSergeant
            | ObjectExchangeKind::ChoyinYoungMan
            | ObjectExchangeKind::CityGuardToken
            | ObjectExchangeKind::GreenShen
            | ObjectExchangeKind::LatemoonFunlin
            | ObjectExchangeKind::LatemoonOld
            | ObjectExchangeKind::LatemoonShaowei
            | ObjectExchangeKind::CloudBHeader
            | ObjectExchangeKind::CloudBoater
            | ObjectExchangeKind::CloudGangster
            | ObjectExchangeKind::CloudGirl
            | ObjectExchangeKind::CloudJudge
            | ObjectExchangeKind::CloudMonk
            | ObjectExchangeKind::CityChenLetter
            | ObjectExchangeKind::ScavengerDonation
            | ObjectExchangeKind::SnowDrunk
            | ObjectExchangeKind::TeacherTuition => {}
        }
    }

    fn bribe_choyin_police(&mut self) {
        if !self.player.pay_money(CHOYIN_BRIBE_AMOUNT) {
            self.push_log("你拿不出十万文钱，巡捕冷笑着不肯松手。".into());
            return;
        }
        self.player.wanted = self.player.wanted.saturating_sub(3);
        self.choyin_justice = ChoyinJusticeState::Free;
        self.activity = Activity::Idle;
        self.push_log("巡捕掂了掂钱，低声道：盛情难却……我就收下了。罪名减轻三级。".into());
        if self.player.wanted == 0 {
            self.push_log("你的通缉已被撤销，巡捕松开了铁链。".into());
        } else {
            self.push_log("巡捕松开铁链离去，但你仍在通缉之列。".into());
        }
    }

    fn advance_choyin_justice(&mut self) {
        if self.player.wanted == 0 {
            self.choyin_justice = ChoyinJusticeState::Free;
            return;
        }

        match self.choyin_justice {
            ChoyinJusticeState::Free => {
                self.choyin_justice = ChoyinJusticeState::Pursuit;
                self.push_log("乔阴县衙接到你的通缉文书，巡捕正在赶来缉拿。".into());
            }
            ChoyinJusticeState::Pursuit => {
                self.activity = Activity::Idle;
                self.choyin_justice = ChoyinJusticeState::Caught;
                self.push_log("巡捕突然追至，解下铁链套在你的脖子上：还不跟我去县衙受审！".into());
            }
            ChoyinJusticeState::Caught => {
                self.activity = Activity::Idle;
                self.location = LocationId::from("choyin.yamen");
                self.choyin_justice = ChoyinJusticeState::AwaitingJudgment;
                self.push_log("巡捕押着你回到乔阴县衙大堂：老爷，人犯已带到。".into());
            }
            ChoyinJusticeState::AwaitingJudgment => self.settle_choyin_lawsuit(),
        }
    }

    fn settle_choyin_lawsuit(&mut self) {
        let level = self.player.wanted;
        if level == 0 {
            self.location = LocationId::from("choyin.court1");
            self.choyin_justice = ChoyinJusticeState::Free;
            self.push_log("程不平说道：下人办事不周，让你受委屈了。".into());
            return;
        }

        self.push_log(format!(
            "程不平喝道：来人哪！把你拖下去打{}大板！",
            level.saturating_mul(10)
        ));
        let essence_damage = (10 - self.player.max_atman / 100).max(0);
        let qi_damage = (10 - self.player.max_force / 100).max(0);
        let spirit_damage = (10 - self.player.max_mana / 100).max(0);
        let mut unconscious = false;

        for _ in 0..level {
            let collapsed = self.player.essence < essence_damage
                && self.player.qi < qi_damage
                && self.player.spirit < spirit_damage;
            self.player.essence = (self.player.essence - essence_damage).max(0);
            self.player.qi = (self.player.qi - qi_damage).max(0);
            self.player.spirit = (self.player.spirit - spirit_damage).max(0);
            if collapsed {
                self.player.wanted = self.player.wanted.saturating_sub(1);
                unconscious = true;
            }
        }

        if unconscious {
            self.player.essence = self.player.essence.max(1);
            self.player.qi = self.player.qi.max(1);
            self.player.spirit = self.player.spirit.max(1);
            self.push_log("哎，早知今日，何必当初？你被打得昏厥过去，又被差役抬了出去。".into());
        }
        self.push_log("程不平喝道：再敢犯案，定斩不饶！".into());
        self.location = LocationId::from("choyin.court1");
        self.choyin_justice = ChoyinJusticeState::Free;
        self.push_log("差役将你丢在县衙门外。".into());
    }

    fn buy_item(&mut self, item_id: ItemId, npc: NpcId) {
        if !self.npc_is_present(&npc) {
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

    fn reject_no_drop_transfer(&mut self, instance_id: u64) -> bool {
        let restricted = self.player.item(instance_id).is_some_and(|item| {
            item.definition()
                .behavior_flags
                .iter()
                .any(|flag| flag == "restricted_movement")
        });
        if restricted {
            self.push_log("这件物品与当前旅程相系，无法转手或丢弃。".into());
        }
        restricted
    }

    fn sell_item(&mut self, instance_id: u64) {
        if self.reject_no_drop_transfer(instance_id) {
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
        if !self.npc_is_present(&npc) || self.player.is_equipped(instance_id) {
            return;
        }
        if self.reject_no_drop_transfer(instance_id) {
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
            Some(ObjectExchangeKind::CommonerDonation) => {
                self.player.inventory.remove(index);
                self.push_log(format!("普通百姓收下{name}，向你道了声谢。"));
            }
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
            Some(ObjectExchangeKind::SnowDrunk)
                if item.definition().category != items::ItemCategory::Liquid
                    || item.definition().liquid_type.as_deref() != Some("alcohol")
                    || item.remaining_uses.unwrap_or(0) <= 5 =>
            {
                self.push_log("醉汉摆摆手：这点酒还不够我润喉。".into());
            }
            Some(ObjectExchangeKind::SnowDrunk) => {
                self.player.inventory.remove(index);
                if self.green_elder_jade_clue && !self.green_drunk_jade_clue {
                    self.green_drunk_jade_clue = true;
                    self.push_log("醉汉痛饮后说道：那块玉佩早卖给青石村杂货铺的沈万年了。".into());
                } else if self.green_drunk_jade_clue && !self.green_drunk_drug_clue {
                    self.green_drunk_drug_clue = true;
                    self.push_log(
                        "醉汉又灌下一壶酒：沈万年还藏着蒙汗药，只肯卖给知道门道的人。".into(),
                    );
                } else {
                    self.push_log(format!("醉汉接过{name}，仰头喝得一滴不剩。"));
                }
            }
            Some(ObjectExchangeKind::GreenShen) if !self.green_drug_offer_unlocked => {
                self.push_log("沈万年眯起眼道：我这里不收来路不明的东西。".into());
            }
            Some(ObjectExchangeKind::GreenShen) if value < 1_000 => {
                self.push_log("沈万年掂了掂礼物：要换蒙汗药，这点价值还不够。".into());
            }
            Some(ObjectExchangeKind::GreenShen) => {
                self.player.inventory.remove(index);
                self.add_inventory_item(ItemId::from(items::SLUMBER_DRUG_ID), 1);
                self.push_log(format!("沈万年收下{name}，从柜底取出一包蒙汗药交给你。"));
            }
            Some(ObjectExchangeKind::SnowTempleDonation) if value == 0 => {
                self.push_log("庙祝说道：这里不收没有价值的物品。".into());
            }
            Some(ObjectExchangeKind::SnowTempleDonation) => {
                self.player.inventory.remove(index);
                self.apply_snow_temple_donation(value);
                self.push_log(format!("庙祝收下{name}作为香火捐献：神明一定会保佑你的。"));
            }
            Some(ObjectExchangeKind::ChoyinYoungMan)
                if item.item_id.as_str() == CHOYIN_SILK_BAG_ID
                    && !self.choyin_silk_bag_delivered =>
            {
                self.player.inventory.remove(index);
                self.choyin_silk_bag_delivered = true;
                self.push_log(
                    "游晋一眼认出荷包上的鸳鸯图案，又从怀中取出一方绣着相同图案的手帕。".into(),
                );
                self.push_log("游晋说道：原来爹爹替我主张的婚事，竟然是……".into());
            }
            Some(ObjectExchangeKind::ChoyinYoungMan) => {
                self.push_log("游晋看了一眼，摇头道：这不是那位姑娘的信物。".into());
            }
            Some(ObjectExchangeKind::ChoyinSergeant)
                if CHOYIN_PEACH_CHEST_IDS.contains(&item.item_id.as_str())
                    && !self.choyin_chest_rewarded =>
            {
                self.player.inventory.remove(index);
                self.choyin_chest_rewarded = true;
                self.add_inventory_item(ItemId::from(CHOYIN_MAGIC_BOOK_ID), 1);
                self.push_log("陈显祖喜道：太好了！就是这个箱子！".into());
                self.push_log("陈显祖将一本「白杨经」交给你，作为寻回桃木箱的答谢。".into());
            }
            Some(ObjectExchangeKind::ChoyinSergeant) => {
                self.push_log("陈显祖说道：这不是我遗失的桃木箱。".into());
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
            Some(ObjectExchangeKind::CityGuardToken)
                if item.item_id.as_str() == CITY_EXIT_TOKEN_ID =>
            {
                self.player.inventory.remove(index);
                self.city_exit_permit = true;
                self.push_log("京师守城兵验过令牌，准你从北门出城；通行许可仅可使用一次。".into());
            }
            Some(ObjectExchangeKind::CityGuardToken) => {
                self.push_log("京师守城兵摇头道：这不是官府认可的出城令牌。".into());
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
            Some(ObjectExchangeKind::LatemoonShaowei)
                if LATEMOON_BAMBOO_IDS.contains(&item.item_id.as_str())
                    && !self.latemoon_dragonfly_received =>
            {
                self.player.inventory.remove(index);
                self.latemoon_dragonfly_received = true;
                self.add_inventory_item(ItemId::from("latemoon.obj.dragonfly"), 1);
                self.push_log("少庄主削开竹片，替你做成一只轻巧的竹蜻蜓。".into());
            }
            Some(ObjectExchangeKind::LatemoonShaowei) => {
                self.push_log("少庄主说道：要做竹蜻蜓，得拿一截合用的竹子来。".into());
            }
            Some(ObjectExchangeKind::LatemoonFunlin)
                if LATEMOON_DRAGONFLY_IDS.contains(&item.item_id.as_str())
                    && !self.latemoon_bracelet_clue =>
            {
                self.player.inventory.remove(index);
                self.latemoon_bracelet_clue = true;
                self.push_log("凤铃把玩竹蜻蜓许久，悄悄告诉你碧纱橱底层藏着一只手镯。".into());
            }
            Some(ObjectExchangeKind::LatemoonFunlin) => {
                self.player.inventory.remove(index);
                self.push_log(format!("凤铃收下{name}，向你甜甜一笑。"));
            }
            Some(ObjectExchangeKind::LatemoonOld)
                if item.item_id.as_str() == LATEMOON_TOKEN_ID && !self.latemoon_token_rewarded =>
            {
                self.player.inventory.remove(index);
                self.latemoon_token_rewarded = true;
                if self.player.faction.as_deref() == Some("晚月庄") && self.player.max_force < 160
                {
                    let gain = i32::try_from(self.random(10) + 1).unwrap_or(10);
                    self.player.max_force = (self.player.max_force + gain).min(160);
                    self.player.force = 0;
                    self.push_log(format!("老人以令牌替你打通经脉，最大内力增加 {gain}。"));
                } else {
                    self.add_inventory_item(ItemId::from(LATEMOON_WHIP_BOOK_ID), 1);
                    self.push_log("老人收回令牌，递给你一本可供研习的鞭法要诀。".into());
                }
            }
            Some(ObjectExchangeKind::LatemoonOld) => {
                self.push_log("老人摇头道：这不是庄中流传的旧令牌。".into());
            }
            Some(ObjectExchangeKind::CloudBHeader)
                if item.item_id.as_str() == CHOYIN_GRASS_ID
                    && self.cloud_escort_member
                    && !self.cloud_escort_letter_received =>
            {
                self.player.inventory.remove(index);
                self.cloud_escort_letter_received = true;
                self.add_inventory_item(ItemId::from(CLOUD_ESCORT_LETTER_ID), 1);
                self.push_log("陈剑秋收下忘忧草，写好一封署名荐书交给你。".into());
            }
            Some(ObjectExchangeKind::CloudBHeader) if !self.cloud_escort_member => {
                self.push_log("陈剑秋说道：非我振远镖局门下，不敢劳动你替我采药。".into());
            }
            Some(ObjectExchangeKind::CloudBHeader) => {
                self.push_log("陈剑秋说道：我只需要一株与你一路带来的忘忧草。".into());
            }
            Some(ObjectExchangeKind::CloudBoater) if self.cloud_boater_paid => {
                self.push_log("船夫说道：船资已经收过，上船便是。".into());
            }
            Some(ObjectExchangeKind::CloudBoater) if value >= 2 => {
                self.player.inventory.remove(index);
                self.cloud_boater_paid = true;
                self.push_log("船夫收下船资，解开缆绳等你登船。".into());
            }
            Some(ObjectExchangeKind::CloudBoater) => {
                self.push_log("船夫掂了掂物件：这点船资还不够。".into());
            }
            Some(ObjectExchangeKind::CloudGangster) if value >= 100_000 => {
                self.player.inventory.remove(index);
                self.cloud_gangster_pass = true;
                self.push_log("恶霸收下厚礼，答应往后不再拦你的路。".into());
            }
            Some(ObjectExchangeKind::CloudGangster) => {
                self.push_log("恶霸把东西推回来，冷笑着拔出兵刃。".into());
                self.begin_combat(EnemyKind::Npc(npc.clone()), CombatMode::Lethal);
            }
            Some(ObjectExchangeKind::CloudGirl)
                if value == 0
                    && self.player.gender == Gender::Male
                    && self.player.perception >= 25
                    && !self.cloud_girl_recognized =>
            {
                self.player.inventory.remove(index);
                self.cloud_girl_recognized = true;
                self.push_log("青云姑娘认出礼物来历，点头记下了你的姓名。".into());
            }
            Some(ObjectExchangeKind::CloudGirl) => {
                self.push_log("青云姑娘没有收下这件礼物。".into());
            }
            Some(ObjectExchangeKind::CloudJudge) if value == 0 => {
                self.push_log("判官摆手道：没有价值的东西不能作赌注。".into());
            }
            Some(ObjectExchangeKind::CloudJudge) => {
                self.player.inventory.remove(index);
                if self.random(5) == 0 {
                    self.player.add_money(value.saturating_mul(2));
                    self.push_log(format!(
                        "判官掷骰开盅，你赢得了{}。",
                        format_money(value * 2)
                    ));
                } else {
                    self.push_log(format!("判官收走{name}：这一局你输了。"));
                }
            }
            Some(ObjectExchangeKind::CloudMonk) if value == 0 => {
                self.push_log("青云僧说道：无价之物不宜充作香火。".into());
            }
            Some(ObjectExchangeKind::CloudMonk) => {
                self.player.inventory.remove(index);
                self.apply_snow_temple_donation(value);
                self.push_log(format!("青云僧收下{name}作为香火，合十称谢。"));
            }
            Some(ObjectExchangeKind::CityChenLetter)
                if item.item_id.as_str() == CLOUD_ESCORT_LETTER_ID
                    && !self.city_chen_letter_delivered =>
            {
                self.player.inventory.remove(index);
                self.city_chen_letter_delivered = true;
                self.push_log("陈天星看过陈剑秋的署名荐书，答应正式传授你武艺。".into());
            }
            Some(ObjectExchangeKind::CityChenLetter) => {
                self.push_log("陈天星说道：没有陈剑秋的署名荐书，我不能收你。".into());
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

    fn donate_item(&mut self, instance_id: u64) {
        if self.reject_no_drop_transfer(instance_id) {
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
        if self.player.is_equipped(instance_id) {
            return;
        }
        let item = &self.player.inventory[index];
        let value = item.unit_value().max(0) as u64 * item.quantity as u64;
        if value == 0 {
            self.push_log("功德箱只接受有价值的供物。".into());
            return;
        }
        let name = item.display_name().to_string();
        self.player.inventory.remove(index);
        self.apply_snow_temple_donation(value);
        self.push_log(format!("你把{name}投入功德箱，作为寺庙香火。"));
    }

    fn apply_snow_temple_donation(&mut self, value: u64) {
        if value <= 100 || self.player.bellicosity <= 0 {
            return;
        }
        let chance_upper = u32::try_from(value / 10).unwrap_or(u32::MAX).max(1);
        if self.random(chance_upper) <= self.player.spirituality {
            return;
        }
        let fortune = self.random(self.player.spirituality.max(1));
        let value_reduction = i32::try_from(value / 1_000).unwrap_or(i32::MAX);
        let reduction = i32::try_from(fortune)
            .unwrap_or(i32::MAX)
            .saturating_add(value_reduction)
            .min(self.player.bellicosity);
        self.player.bellicosity -= reduction;
    }

    fn place_item_on_ground(&mut self, item_id: ItemId, quantity: u32) {
        let instance_id = self.allocate_item_instance_id();
        let mut item = ItemInstance::new(instance_id, item_id, quantity);
        self.schedule_temporary_item_expiry(&mut item);
        self.merge_ground_item(item);
    }

    fn place_corpse_on_ground(&mut self, victim_name: &str) {
        let instance_id = self.allocate_item_instance_id();
        let mut corpse = ItemInstance::new(instance_id, ItemId::from(CORPSE_ITEM_ID), 1);
        corpse.transformed_name = Some(format!("{victim_name}的尸体"));
        self.schedule_temporary_item_expiry(&mut corpse);
        self.merge_ground_item(corpse);
        self.push_log(format!("{victim_name}倒在地上，留下了一具尸体。"));
    }

    fn drop_npc_carried_items(&mut self, npc: &NpcId) {
        let drops: Vec<_> = npcs()
            .definition(npc)
            .map(|definition| {
                definition
                    .carried_items
                    .iter()
                    .enumerate()
                    .filter(|(slot, _)| !self.npc_item_has_been_stolen(npc, *slot))
                    .map(|(_, item)| item.item_id.clone())
                    .collect()
            })
            .unwrap_or_default();
        if drops.is_empty() {
            return;
        }
        let names = drops
            .iter()
            .map(|item_id| {
                items()
                    .definition(item_id)
                    .expect("NPC carried item must exist")
                    .display_name()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("、");
        for item_id in drops {
            self.place_item_on_ground(item_id, 1);
        }
        self.push_log(format!("{}遗落：{names}。", npc.name()));
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
        if self.reject_no_drop_transfer(instance_id) {
            return;
        }
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

    fn put_item(&mut self, instance_id: u64, container_id: u64, quantity: u32) {
        if instance_id == container_id || quantity == 0 {
            return;
        }
        if self.player.is_equipped(instance_id) {
            self.push_log("必须先卸下这件物品。".into());
            return;
        }
        let Some(source_index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == instance_id)
        else {
            self.push_log("行囊中没有这件物品。".into());
            return;
        };
        let source = self.player.inventory[source_index].clone();
        if Self::item_is_transfer_restricted(&source) {
            self.push_log("这个东西还是小心保管的好，不必放在别处。".into());
            return;
        }
        if quantity > source.quantity {
            self.push_log("你没有那么多这件物品。".into());
            return;
        }
        if source.contains_instance_id(container_id) {
            self.push_log("不能把容器放进它自己或自己的内层。".into());
            return;
        }
        let Some(container) = self.current_container_item(container_id) else {
            self.push_log("这里没有这个容器。".into());
            return;
        };
        let Some(capacity) = container.container_capacity() else {
            self.push_log("这不是可以存放物品的容器。".into());
            return;
        };
        let mut transfer_weight = source.clone();
        transfer_weight.quantity = quantity;
        if container
            .contents_weight()
            .saturating_add(transfer_weight.total_weight())
            > capacity
        {
            self.push_log(format!("{}已经装不下了。", container.display_name()));
            return;
        }

        let transferred = if quantity == source.quantity {
            self.player.inventory.remove(source_index)
        } else {
            let new_instance_id = self.allocate_item_instance_id();
            self.player.inventory[source_index]
                .split_off(new_instance_id, quantity)
                .expect("partial stack split must be valid")
        };
        let name = transferred.display_name().to_string();
        let Some(container) = self.current_container_item_mut(container_id) else {
            self.push_log("容器已经不在这里了。".into());
            self.player.inventory.push(transferred);
            return;
        };
        container.contents.push(transferred);
        let container_name = container.display_name().to_string();
        self.push_log(format!("你把{name}放进{container_name}。"));
    }

    fn take_from_container(&mut self, instance_id: u64, container_id: u64, quantity: u32) {
        if quantity == 0 {
            return;
        }
        let Some(container) = self.current_container_item(container_id) else {
            return;
        };
        let Some(capacity) = container.container_capacity() else {
            return;
        };
        let _ = capacity;
        let Some(source) = container
            .contents
            .iter()
            .find(|item| item.instance_id == instance_id)
            .cloned()
        else {
            return;
        };
        if quantity > source.quantity {
            return;
        }
        let container_is_carried = self
            .player
            .inventory
            .iter()
            .any(|item| item.contains_instance_id(container_id));
        let mut transfer_weight = source.clone();
        transfer_weight.quantity = quantity;
        if !container_is_carried
            && self
                .player
                .carried_weight()
                .saturating_add(transfer_weight.total_weight())
                > self.player.carry_capacity()
        {
            self.push_log(format!("{}太重了，你目前无法拿起。", source.display_name()));
            return;
        }
        let new_instance_id =
            (quantity < source.quantity).then(|| self.allocate_item_instance_id());
        let transferred = {
            let container = self
                .current_container_item_mut(container_id)
                .expect("visible container must remain available");
            let index = container
                .contents
                .iter()
                .position(|item| item.instance_id == instance_id)
                .expect("container item must remain available");
            if quantity == source.quantity {
                container.contents.remove(index)
            } else {
                container.contents[index]
                    .split_off(new_instance_id.expect("partial item needs an ID"), quantity)
                    .expect("partial container stack split must be valid")
            }
        };
        let name = transferred.display_name().to_string();
        if transferred.definition().stackable()
            && transferred.contents.is_empty()
            && transferred.talisman.is_none()
            && let Some(existing) = self.player.inventory.iter_mut().find(|item| {
                item.item_id == transferred.item_id
                    && item.contents.is_empty()
                    && item.talisman.is_none()
            })
        {
            existing.quantity = existing.quantity.saturating_add(transferred.quantity);
        } else {
            self.player.inventory.push(transferred);
        }
        self.push_log(format!("你从容器中取出{}。", name));
    }

    fn scribe_haunt(&mut self, paper_instance_id: u64, target: NpcId) {
        if self.player.mapped_skill(SPELLS_ID).map(SkillId::as_str) != Some("necromancy") {
            self.push_log("请先致能茅山咒文后再画符。".into());
            return;
        }
        if !self.npc_is_present(&target) || Self::is_zombie_npc(&target) {
            self.push_log("你无法为这个目标书写追魂符。".into());
            return;
        }
        if self.player.mana < 20 || self.player.spirit < 40 || self.player.qi < 1 {
            self.push_log("你的法力、气或神不足，无法画符。".into());
            return;
        }
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == paper_instance_id)
        else {
            return;
        };
        if self.player.inventory[index].item_id.as_str() != "obj.paper_seal"
            || self.player.inventory[index].talisman.is_some()
        {
            return;
        }
        let seal_id = if self.player.inventory[index].quantity > 1 {
            let new_instance_id = self.allocate_item_instance_id();
            let seal = self.player.inventory[index]
                .split_off(new_instance_id, 1)
                .expect("paper seal stack must split");
            self.player.inventory.push(seal);
            new_instance_id
        } else {
            paper_instance_id
        };
        let seal = self
            .player
            .item_mut(seal_id)
            .expect("paper seal must remain in inventory");
        seal.item_id = ItemId::from("obj.magic_seal");
        seal.quantity = 1;
        seal.durability = None;
        seal.remaining_uses = None;
        seal.transformed_name = Some("僵尸追魂符".into());
        seal.talisman = Some(TalismanInscription {
            kind: TalismanKind::Haunt,
            target: target.as_str().into(),
        });
        self.player.mana -= 20;
        self.player.spirit -= 40;
        self.player.qi -= 1;
        self.push_log(format!(
            "你以血在桃符纸上写下{}的名字，画成一张僵尸追魂符。",
            target.name()
        ));
    }

    fn attach_talisman(&mut self, talisman_instance_id: u64, zombie: NpcId) {
        if !Self::is_zombie_npc(&zombie) || !self.npc_is_present(&zombie) {
            self.push_log("僵尸不在这里，无法贴符。".into());
            return;
        }
        if self.player.atman < 10 || self.player.mapped_skill(MAGIC_ID).is_none() {
            self.push_log("你必须先以灵力镇住僵尸，才能使用追魂符。".into());
            return;
        }
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.instance_id == talisman_instance_id)
        else {
            return;
        };
        let Some(mark) = self.player.inventory[index].talisman.clone() else {
            self.push_log("这张符没有可附着的法力。".into());
            return;
        };
        if mark.kind != TalismanKind::Haunt {
            return;
        }
        self.player.inventory.remove(index);
        self.player.atman -= 10;
        self.zombie_haunts.retain(|entry| entry.zombie != zombie);
        self.zombie_haunts.push(ZombieHaunt {
            zombie: zombie.clone(),
            target: NpcId::from(mark.target.as_str()),
            expires_at_elapsed_minutes: self
                .elapsed_minutes
                .saturating_add(ZOMBIE_HAUNT_DURATION_TICKS.saturating_mul(GAME_MINUTES_PER_TICK)),
        });
        self.push_log(format!(
            "你把追魂符贴在{}额前，它喃喃道：杀……死……{}……",
            zombie.name(),
            mark.target
        ));
    }

    fn start_steal(&mut self, npc: NpcId, item_id: ItemId, slot: usize) {
        if !self.npc_is_present(&npc) || self.npc_item_has_been_stolen(&npc, slot) {
            return;
        }
        let valid = npcs()
            .definition(&npc)
            .and_then(|definition| definition.carried_items.get(slot))
            .is_some_and(|item| item.item_id == item_id);
        if !valid {
            return;
        }
        self.activity = Activity::Stealing(StealState {
            npc: npc.clone(),
            item_id,
            slot,
            remaining_ticks: STEAL_DELAY_TICKS,
        });
        self.push_log(format!("你不动声色地靠近{}，等待下手的机会……", npc.name()));
    }

    fn steal_tick(&mut self, mut state: StealState) {
        if state.remaining_ticks > 1 {
            state.remaining_ticks -= 1;
            self.activity = Activity::Stealing(state);
            return;
        }
        self.activity = Activity::Idle;
        if !self.npc_is_present(&state.npc) {
            self.push_log("太可惜了，下手的目标已经离开。".into());
            return;
        }
        if self.npc_item_has_been_stolen(&state.npc, state.slot) {
            self.push_log("你想偷取的东西已经不在对方身上。".into());
            return;
        }
        let Some(definition) = npcs().definition(&state.npc) else {
            return;
        };
        let Some(carried) = definition.carried_items.get(state.slot) else {
            return;
        };
        if carried.item_id != state.item_id {
            return;
        }
        let stealing = self.player.skill_level("stealing");
        let mut sp = i32::try_from(stealing.saturating_mul(5)).unwrap_or(i32::MAX)
            + i32::try_from(self.player.courage.saturating_mul(2)).unwrap_or(i32::MAX)
            - i32::try_from(self.player.theft_heat.saturating_mul(20)).unwrap_or(i32::MAX);
        sp = sp.max(1);
        let spirit = definition
            .resources
            .get("sen")
            .or_else(|| definition.resources.get("max_sen"))
            .copied()
            .unwrap_or(20)
            .max(1);
        let weight = items()
            .definition(&state.item_id)
            .expect("NPC carried item must exist")
            .unit_weight();
        let dp = u32::try_from(spirit.saturating_mul(2))
            .unwrap_or(u32::MAX)
            .saturating_add(weight / 25)
            .max(1);
        let sp = u32::try_from(sp).unwrap_or(1).max(1);
        if self.random(sp.saturating_add(dp).max(1)) > dp {
            let item =
                ItemInstance::new(self.allocate_item_instance_id(), state.item_id.clone(), 1);
            if self
                .player
                .carried_weight()
                .saturating_add(item.total_weight())
                > self.player.carry_capacity()
            {
                self.push_log(format!(
                    "你摸到{}，却发现它太重，只得放弃。",
                    item.display_name()
                ));
                return;
            }
            let name = item.display_name().to_string();
            self.player.inventory.push(item);
            self.stolen_npc_items.push(StolenNpcItem {
                location: self.location.clone(),
                npc: state.npc.clone(),
                slot: state.slot,
            });
            if self.player.skill_by_id("stealing").is_some() {
                let progress = self.random(self.player.intelligence.max(1));
                self.gain_skill_progress(SkillId::from("stealing"), progress);
            }
            self.push_log(format!("得手了！你成功偷到一件{name}。"));
            if self.random(sp) < dp / 2 {
                self.push_log("有人似乎看见你鬼鬼祟祟地离开，却没有出声。".into());
            }
        } else if self.random(sp) > dp / 2 {
            self.push_log(format!(
                "{}不经意地一转头，你急忙将手缩回；还好没有被发现。",
                state.npc.name()
            ));
        } else {
            if self.player.skill_by_id("stealing").is_some() {
                self.gain_skill_progress(SkillId::from("stealing"), 1);
            }
            self.player.theft_heat = self.player.theft_heat.saturating_add(1);
            self.player.wanted = self.player.wanted.saturating_add(1);
            self.push_log(format!(
                "糟糕！{}当场发现你的手伸向了他的随身物件，立刻拔兵刃反击；通缉 +1。",
                state.npc.name()
            ));
            self.begin_combat(EnemyKind::Npc(state.npc), CombatMode::Lethal);
        }
    }

    fn reincarnate_player(&mut self) {
        if !matches!(self.activity, Activity::Idle | Activity::Resting) {
            self.push_log("眼下无法静下心来投胎。".into());
            return;
        }
        let old_player = std::mem::take(&mut self.player);
        let name = old_player.name.clone();
        let title = old_player.title.clone();
        let description = old_player.description.clone();
        let instance_id = self.allocate_item_instance_id();
        let mut corpse = ItemInstance::new(instance_id, ItemId::from(CORPSE_ITEM_ID), 1);
        corpse.transformed_name = Some(format!("{}的遗体", name));
        corpse.container_capacity = Some(old_player.carry_capacity());
        corpse.contents = old_player.inventory;
        self.schedule_temporary_item_expiry(&mut corpse);
        self.merge_ground_item(corpse);
        self.player = Player::default();
        self.player.name = name;
        self.player.title = title;
        self.player.description = description;
        self.location = LocationId::from(content::LIU_HOME);
        self.quest = QuestStage::Unasked;
        self.activity = Activity::Idle;
        self.dynamic_quest = None;
        self.choyin_justice = ChoyinJusticeState::Free;
        self.push_log("你留下遗体，魂魄转入新身，于刘家小房重新醒来。".into());
    }

    fn dissolve_corpse(&mut self, dust_instance_id: u64, corpse_instance_id: u64) {
        if !self
            .player
            .item(dust_instance_id)
            .is_some_and(|item| item.item_id.as_str() == "obj.dust" && item.has_uses_left())
        {
            return;
        }
        let (corpse, clear_ground) = {
            let Some(ground) = self.ground_items.get_mut(&self.location) else {
                return;
            };
            let Some(index) = ground.iter().position(|item| {
                item.instance_id == corpse_instance_id
                    && item.item_id.as_str() == CORPSE_ITEM_ID
                    && item.lifecycle_stage < 2
            }) else {
                return;
            };
            let corpse = ground.remove(index);
            let clear_ground = ground.is_empty();
            (corpse, clear_ground)
        };
        if clear_ground {
            self.ground_items.remove(&self.location);
        }
        let name = corpse.display_name().to_string();
        let contents = corpse.contents;
        let had_contents = !contents.is_empty();
        for item in contents {
            self.merge_ground_item(item);
        }
        self.spend_item_use(dust_instance_id, false);
        self.push_log(format!(
            "你把一点化尸粉撒在{name}上，尸体在嗤嗤声中化成一滩黄水。"
        ));
        if had_contents {
            self.push_log("遗体中的随身物品留在了原地。".into());
        }
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
        let item_id = item.item_id.clone();
        let name = item.display_name().to_string();
        if LATEMOON_SPECIAL_CONSUMABLE_IDS.contains(&item_id.as_str()) {
            self.consume_latemoon_special(instance_id, item_id.as_str());
            return;
        }
        if item_id.as_str() == CHOYIN_TABLET_ID {
            self.player.essence = (self.player.essence + 5).min(self.player.max_essence);
            self.player.qi = (self.player.qi + 30).min(self.player.max_qi);
            self.player.spirit = (self.player.spirit + 5).min(self.player.max_spirit);
            self.spend_item_use(instance_id, false);
            self.push_log("你吞下一粒仙丹，精恢复 5、气恢复 30、神恢复 5。".into());
            return;
        }
        let food_supply = definition.food_supply.unwrap_or(0).max(0);
        let water_supply = definition.water_supply.unwrap_or(0).max(0);
        let is_liquid = definition.category == items::ItemCategory::Liquid;
        let filled_with_water = item.filled_with_water;
        let is_alcohol = !filled_with_water && definition.liquid_type.as_deref() == Some("alcohol");
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
            let liquid = if filled_with_water {
                "清水"
            } else {
                definition.liquid_name.as_deref().unwrap_or("饮水")
            };
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
        if GOATHILL_DEAD_LEECH_IDS.contains(&item_id.as_str()) {
            self.player.force = self.player.force.saturating_add(1);
            self.player.max_mana = self.player.max_mana.saturating_add(60);
            self.push_log("岩蛭的药力化开：内力 +1，最大法力 +60。".into());
        }
        if let Some((residual_name, residual_weight)) = residual {
            self.spend_item_use(instance_id, true);
            if let Some(item) = self.player.item_mut(instance_id) {
                item.transformed_name = Some(residual_name.into());
                item.transformed_weight = Some(residual_weight);
                if !items::CHUENYU_PIGMEAT_IDS.contains(&item.item_id.as_str())
                    && !CLOUD_MEAT_IDS.contains(&item.item_id.as_str())
                {
                    item.transformed_value = Some(0);
                }
            }
            self.push_log(format!("你吃完了{name}，只剩下{residual_name}。"));
        } else {
            self.spend_item_use(instance_id, false);
            self.push_log(format!("你吃了几口{name}。"));
        }
    }

    fn consume_latemoon_special(&mut self, instance_id: u64, item_id: &str) {
        match item_id {
            "latemoon.park.npc.obj.bean" | "latemoon.sell.bean" => {
                self.player.essence = (self.player.essence + 50).min(self.player.max_essence);
                self.player.qi = (self.player.qi + 100).min(self.player.max_qi);
                self.player.spirit = (self.player.spirit + 50).min(self.player.max_spirit);
                self.push_log("你吞下一粒仙豆，精恢复 50、气恢复 100、神恢复 50。".into());
            }
            "latemoon.park.npc.obj.flower" => {
                self.player.spirit = (self.player.spirit + 50).min(self.player.max_spirit);
                let remaining = self
                    .player
                    .condition(ConditionKind::RosePoison)
                    .map_or(0, |condition| condition.duration.saturating_sub(10));
                self.player
                    .set_condition(ConditionKind::RosePoison, remaining, 10);
                self.push_log("你吞下金黄花蕊，神恢复 50，火玫瑰毒减轻十拍。".into());
            }
            "latemoon.sell.white_pill" => {
                self.player.essence = (self.player.essence + 100).min(self.player.max_essence);
                self.player.qi = (self.player.qi + 300).min(self.player.max_qi);
                self.player.spirit = (self.player.spirit + 100).min(self.player.max_spirit);
                self.push_log("你服下白凤丸，精恢复 100、气恢复 300、神恢复 100。".into());
            }
            "latemoon.sell.wine" => {
                self.player.essence = (self.player.essence - 10).max(0);
                self.player.spirit = (self.player.spirit + 20).min(self.player.max_spirit);
                self.push_log("你大口喝下女儿红，精损失 10，神恢复 20。".into());
            }
            _ => return,
        }
        self.spend_item_use(instance_id, false);
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
        let advances_oldpine_maze =
            matches!(
                self.location.as_str(),
                "oldpine.cave1"
                    | "oldpine.cave2"
                    | "oldpine.cave3"
                    | "oldpine.cave4"
                    | "oldpine.cliffdown"
                    | "oldpine.pine1"
                    | "oldpine.pine2"
                    | "oldpine.pine3"
                    | "oldpine.pine4"
                    | "oldpine.pine5"
                    | "oldpine.pine6"
                    | "oldpine.pine7"
            ) && (target.as_str().starts_with(content::OLD_PINE_CAVE_PREFIX)
                || target.as_str().starts_with(content::OLD_PINE_FOREST_PREFIX));
        if advances_oldpine_maze {
            self.random(1);
        }
        if self.location.as_str() == TEMPLE_SLIPPERY_ROAD
            && matches!(
                target.as_str(),
                content::TEMPLE_ROAD_TWO | "temple.corridor3"
            )
            && self.random(self.player.spirituality.max(1)) < 3
        {
            self.player.essence = (self.player.essence / 2).max(1);
            self.player.qi = (self.player.qi / 2).max(1);
            self.player.spirit = (self.player.spirit / 2).max(1);
            self.push_log("你一脚踩上青苔，重重滑倒在石径上，一时无力起身。".into());
            return;
        }
        let closes_boulder = self.location.as_str() == CANYON_BAMBOO_BOULDER
            && target.as_str() == CANYON_BAMBOO_TRAINING_ROOM;
        if self.location.as_str() == content::CITY_MANOR_GATE {
            self.city_manor_pass = false;
        }
        if self.location.as_str() == content::CITY_NORTH_GATE
            && target.as_str() == content::CITY_NORTH_ROAD
        {
            self.city_exit_permit = false;
        }
        if self.location.as_str() == "choyin.platform" && target.as_str() != "choyin.platform" {
            self.choyin_thunder_ticks = 0;
        }
        if self.location.as_str() == "choyin.club" && target.as_str() != "choyin.club" {
            self.return_choyin_borrowed_books();
        }
        if self.location.as_str() == "oldpine.keep2" && target.as_str() == "oldpine.keep3" {
            self.oldpine_keep_sealed = true;
            self.push_log("身后轰然一响，寨门已被巨石堵死。".into());
        }
        if self.location.as_str() == "choyin.entrance" && target.as_str() == "choyin.taolin" {
            self.choyin_taolin_steps = 3;
            self.choyin_taolin_clue = self.random(11) as u8;
        }
        if self.location.as_str() == "green.eight7" && target.as_str() == "green.stoneroom" {
            self.green_bagua_completed = true;
            self.push_log("你循乾位踏出迷雾，已记住八卦阵的生门。".into());
        }
        if self.location.as_str() == "latemoon.room.bathroom1" && self.player.gender == Gender::Male
        {
            self.player.set_condition(ConditionKind::RosePoison, 5, 10);
            self.push_log("有人隔着软帘向你洒下粉末，火玫瑰毒侵入经脉。".into());
        }
        if self.location.as_str() == "death.road2" && target.as_str() == "death.road1" {
            self.death_road_steps = 0;
        }
        if self.location.as_str() == "u.cloud.sunhill.northriver"
            && target.as_str() == "u.cloud.sunhill.midriver"
        {
            self.cloud_boater_paid = false;
            self.push_log("船夫撑篙渡你过河，这次船资已经用尽。".into());
        }
        self.location = target;
        if self.location.as_str() == "chuenyu.trap_castle" {
            self.chuenyu_trap_arrow_ticks = 2;
        } else {
            self.chuenyu_trap_arrow_ticks = 0;
        }
        let place = self.current_location();
        self.push_log(format!("你来到{}。{}", place.name, place.arrival));
        if closes_boulder {
            self.canyon_boulder_open = false;
            self.push_log("你刚穿过缝隙，大黄石便在身后急速合拢。".into());
        }
        if !self.trigger_old_liu_revenge() {
            self.trigger_m6_npc_aggression();
        }
    }

    fn trigger_m6_npc_aggression(&mut self) -> bool {
        if !matches!(self.activity, Activity::Idle) {
            return false;
        }
        let location = self.location.as_str().to_string();
        let jiading_attacks =
            location == "chuenyu.rope_bridge" && self.random(self.player.perception.max(1)) < 20;
        let npc_id = match location.as_str() {
            "chuenyu.dungeon" => CHUENYU_BOSS_ID,
            "chuenyu.west_blackge" => CHUENYU_GUARD_ID,
            "chuenyu.tortureroom" | "chuenyu.tortureroom2" => CHUENYU_GUARD_TWO_ID,
            "chuenyu.rope_bridge" if jiading_attacks => CHUENYU_JIADING_THREE_ID,
            _ => return false,
        };
        let npc = NpcId::from(npc_id);
        if !self.npc_is_present(&npc) {
            return false;
        }
        self.push_log(format!("{}发现你闯入禁地，立刻扑上来动手！", npc.name()));
        self.begin_combat(EnemyKind::Npc(npc), CombatMode::Lethal);
        true
    }

    fn trigger_old_liu_revenge(&mut self) -> bool {
        if !matches!(self.location.as_str(), content::LIU_HOME | "chuenyu.home")
            || self.quest != QuestStage::MurderedJuan
            || !matches!(self.activity, Activity::Idle)
        {
            return false;
        }
        self.quest = QuestStage::Failed;
        self.push_log("刘老农悲愤欲绝：你竟杀了我的女儿，纳命来！".into());
        self.begin_combat(EnemyKind::OldLiuRevenge, CombatMode::Lethal);
        true
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
            InteractionKind::SearchCityRuinedGarden => self.search_city_ruined_garden(),
            InteractionKind::TurnAltarForward => self.turn_altar_button(true),
            InteractionKind::TurnAltarBackward => self.turn_altar_button(false),
            InteractionKind::PressAltarButton => self.press_altar_button(),
            InteractionKind::PushSnowShelf => self.push_snow_shelf(),
            InteractionKind::WorkAtSnowWorkshop => self.work_at_snow_workshop(),
            InteractionKind::MoveBambooBoulder => self.move_bamboo_boulder(),
            InteractionKind::SearchBambooBookcase => self.search_bamboo_bookcase(),
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
            InteractionKind::HoldOldPineVine => {
                let target = if self.random(self.player.skill_level(DODGE_ID).max(1)) < 5 {
                    self.push_log("你没能抓稳藤蔓，惨叫着坠入瀑布下的水潭。".into());
                    "oldpine.waterfall"
                } else {
                    self.push_log("你抓稳藤蔓，沿着山涧慢慢攀进瀑布后的通道。".into());
                    "oldpine.passage"
                };
                self.move_to(LocationId::from(target));
            }
            InteractionKind::ClimbChoyinTree => {
                self.move_to(LocationId::from("choyin.craneroom"));
                self.push_log("你攀上绝壁古树，从枝桠间翻进鹤室。".into());
            }
            InteractionKind::HoldChoyinVine => {
                let target = if self.random(self.player.skill_level(DODGE_ID).max(1)) < 30 {
                    self.push_log("你抓空藤蔓，沿绝壁坠进寒谷。".into());
                    "choyin.hollow"
                } else {
                    self.push_log("你抓稳藤蔓，慢慢攀近半山洞穴。".into());
                    "choyin.halfhole"
                };
                self.move_to(LocationId::from(target));
            }
            InteractionKind::TouchChoyinCloudFlag => {
                self.choyin_platform_passage_ticks = 2;
                self.choyin_thunder_ticks = 3;
                self.push_log("白光一闪，云台裂开向下的入口，远处雷声开始聚拢。".into());
            }
            InteractionKind::DrinkChoyinWell => {
                self.player.water = (self.player.water + 20).min(DEFAULT_WATER_CAPACITY);
                self.push_log("你用井边的杯子舀水喝了几口。".into());
            }
            InteractionKind::LiftChoyinStoneLion => {
                self.choyin_lion_lift_count = self.choyin_lion_lift_count.saturating_add(1);
                if u32::from(self.choyin_lion_lift_count) + self.player.strength / 5 >= 10 {
                    self.choyin_lion_lift_count = 0;
                    self.move_to(LocationId::from("choyin.lionroom"));
                    self.push_log("石狮向左挪开尺许，你从露出的洞口坠入神秘洞穴。".into());
                } else {
                    self.push_log("你奋力抬动石狮，机关发出一声沉闷的摩擦。".into());
                }
            }
            InteractionKind::BuryOldPineSkeleton => self.bury_oldpine_skeleton(),
            InteractionKind::BlowOldPineBambooPipe => {
                self.oldpine_keep_sealed = false;
                self.push_log("竹哨声落，一阵轮盘绞动声后，堵门巨石被慢慢移开。".into());
            }
            InteractionKind::BorrowChoyinBook => self.borrow_choyin_book(),
            InteractionKind::ReadChoyinPeachNote => {
                self.push_log(choyin_taolin_clue(self.choyin_taolin_clue).0.into());
            }
            InteractionKind::TieChoyinCrane => {
                self.player.spirit = (self.player.spirit - 50).max(0);
                self.move_to(LocationId::from("choyin.platform"));
                self.push_log("你以缚仙绳套住仙鹤，随它扶摇直上云台；神损失 50。".into());
            }
            InteractionKind::PullChuenyuHallRope => self.pull_chuenyu_hall_rope(),
            InteractionKind::ClimbChuenyuCastleWall => self.climb_chuenyu_castle_wall(),
            InteractionKind::DescendChuenyuRopeBridge => {
                self.move_to(LocationId::from("chuenyu.base_b_m"));
                self.push_log("你抓紧铁链，顺着摇晃的铁索桥下到黑松山脚。".into());
            }
            InteractionKind::PushChuenyuDungeonSlab => self.push_chuenyu_dungeon_slab(),
            InteractionKind::PushGreenBoulder => self.push_green_boulder(),
            InteractionKind::FillGreenWell => self.fill_green_well(),
            InteractionKind::SearchGreenStream => self.search_green_stream(),
            InteractionKind::OpenSanyenSteamer => {
                if self.current_room_has_npc(SANYEN_COOK_ID) {
                    self.push_log("烧饭僧合十道：施主请勿动手动脚，妨碍贫僧煮饭。".into());
                } else {
                    self.push_log("你揭开蒸笼，热气间整齐排着一枚枚白馒头。".into());
                }
            }
            InteractionKind::TakeSanyenBun => self.take_sanyen_bun(),
            InteractionKind::InspectLateMoonLantern => {
                self.push_log("灯笼上写着：晚霞西照人影依稀，月影高挂和风婉约。".into());
            }
            InteractionKind::TakeLateMoonCloth => self.take_latemoon_cloth(),
            InteractionKind::DanceLateMoonOut => self.dance_latemoon(false),
            InteractionKind::DanceLateMoonYuFong => self.dance_latemoon(true),
            InteractionKind::PickLateMoonFlower => self.pick_latemoon_flower(),
            InteractionKind::BatheLateMoonPool => self.bathe_latemoon_pool(),
            InteractionKind::PonderLateMoonRoom => self.ponder_latemoon_room(),
            InteractionKind::InspectDeathShadows => {
                self.push_log(
                    "四个披黑斗篷的人围在炉边，其中一人回头时竟与你长得一模一样。".into(),
                );
            }
            InteractionKind::ReincarnateDeathInn => self.reincarnate_from_death_inn(),
            InteractionKind::InspectCloudButcherySign => {
                self.push_log("牛骨招牌写着：本店即将收购死狗。".into());
            }
            InteractionKind::UseLateMoonDanceBook(instance_id) => {
                self.use_latemoon_dance_book(instance_id);
            }
            InteractionKind::PrayLateMoonBracelet(instance_id) => {
                self.pray_latemoon_bracelet(instance_id);
            }
            InteractionKind::ReadLateMoonSecretLetter(instance_id) => {
                self.read_latemoon_secret_letter(instance_id);
            }
            InteractionKind::SearchLateMoonBracelet => self.search_latemoon_bracelet(),
            InteractionKind::SearchLateMoonDanceBook => self.search_latemoon_dance_book(),
            InteractionKind::JoinCloudEscort => self.join_cloud_escort(),
        }
    }

    fn search_latemoon_bracelet(&mut self) {
        if !self.latemoon_bracelet_clue || self.latemoon_bracelet_received {
            return;
        }
        self.latemoon_bracelet_received = true;
        self.add_inventory_item(ItemId::from("latemoon.obj.bracelet"), 1);
        self.push_log("你依凤灵的提示在碧纱橱底摸索，找到一串玛瑙手镯。".into());
    }

    fn search_latemoon_dance_book(&mut self) {
        if !self.latemoon_dance_book_clue || self.latemoon_dance_book_received {
            return;
        }
        self.latemoon_dance_book_received = true;
        self.add_inventory_item(ItemId::from("latemoon.obj.book"), 1);
        self.push_log("你依沈芳的提示掀开床褥，找到一本泛黄的舞曲谱。".into());
    }

    fn join_cloud_escort(&mut self) {
        if self.cloud_escort_member {
            return;
        }
        if self.player.courage < 25 {
            self.push_log("陈剑秋摇头道：走镖之人须有胆识，你还需历练。".into());
            return;
        }
        self.cloud_escort_member = true;
        self.player.faction = Some("振远镖局".into());
        self.player.teacher = Some(CLOUD_B_HEADER_ID.into());
        self.push_log("陈剑秋见你胆识过人，将你收入振远镖局门下。".into());
    }

    fn advance_death_road(&mut self, target: LocationId) {
        self.death_road_steps = self.death_road_steps.saturating_add(1);
        if self.death_road_steps >= 5 {
            self.death_road_steps = 0;
            self.move_to(target);
        } else {
            self.push_log("你走了许久，四周浓雾与灯笼竟丝毫没有变化。".into());
        }
    }

    fn take_latemoon_cloth(&mut self) {
        if self.latemoon_clothes_taken >= 2 {
            return;
        }
        self.latemoon_clothes_taken += 1;
        self.add_inventory_item(ItemId::from("latemoon.obj.skirt"), 1);
        self.push_log("你从碧纱橱中取出一件轻软衣裳。".into());
    }

    fn dance_latemoon(&mut self, yu_fong: bool) {
        let required = match self.player.gender {
            Gender::Male => 100,
            Gender::Female => 50,
        };
        if self.player.spirit < required {
            self.push_log("你的神不足，无法专注踏出这套舞步。".into());
            return;
        }
        let (target, cost, name) = match (self.location.as_str(), yu_fong, self.player.gender) {
            ("latemoon.latemoon8", true, Gender::Male) => ("latemoon.miroom", 100, "有凤来仪"),
            ("latemoon.latemoon8", true, Gender::Female) => ("latemoon.miroom", 80, "有凤来仪"),
            ("latemoon.latemoon8", false, Gender::Male) => ("latemoon.bamboo", 50, "西出阳关"),
            ("latemoon.latemoon8", false, Gender::Female) => ("latemoon.bamboo", 30, "西出阳关"),
            ("latemoon.miroom", false, Gender::Male) => ("latemoon.bamboo", 80, "西出阳关"),
            ("latemoon.miroom", false, Gender::Female) => ("latemoon.bamboo", 50, "西出阳关"),
            _ => return,
        };
        self.player.spirit = (self.player.spirit - cost).max(0);
        self.move_to(LocationId::from(target));
        self.push_log(format!("你踏出一曲「{name}」，身影随曲声移入另一处。"));
    }

    fn pick_latemoon_flower(&mut self) {
        if self.latemoon_flowers_picked >= 2 {
            return;
        }
        self.latemoon_flowers_picked += 1;
        self.add_inventory_item(ItemId::from("latemoon.park.npc.obj.flower"), 1);
        self.push_log("你从西府海棠旁摘下一朵不起眼的金黄花蕊。".into());
    }

    fn bathe_latemoon_pool(&mut self) {
        match self.player.gender {
            Gender::Male => {
                self.player.set_condition(ConditionKind::RosePoison, 15, 10);
                self.push_log("池水沁凉得异常，火玫瑰毒已侵入经脉。".into());
            }
            Gender::Female => {
                let restored = 5 + self.random(5) as i32;
                self.player.essence = (self.player.essence - 10).max(0);
                self.player.spirit = (self.player.spirit + restored).min(self.player.max_spirit);
                self.push_log(format!("你在花池中沐浴，精损失 10，神恢复 {restored}。"));
            }
        }
    }

    fn ponder_latemoon_room(&mut self) {
        self.player.spirit = (self.player.spirit - 50).max(0);
        let reduction = (self.random(self.player.spirituality.max(1)) + 7) as i32;
        self.player.bellicosity = (self.player.bellicosity - reduction).max(0);
        self.push_log(format!("你合掌静修，神损失 50，杀气降低 {reduction}。"));
    }

    fn reincarnate_from_death_inn(&mut self) {
        self.player.essence = self.player.max_essence;
        self.player.qi = self.player.max_qi;
        self.player.spirit = self.player.max_spirit;
        self.player.conditions.clear();
        self.move_to(LocationId::from("snow.temple"));
        self.push_log("另一个自己与你相撞，眼前一黑；再睁眼时已回到雪亭城隍庙。".into());
    }

    fn use_latemoon_dance_book(&mut self, instance_id: u64) {
        let Some(item) = self.player.item(instance_id) else {
            return;
        };
        if !LATEMOON_DANCE_BOOK_IDS.contains(&item.item_id.as_str()) {
            return;
        }
        if self.player.spirit < 50 {
            self.push_log("你的神不足，无法按舞谱踏完整套舞步。".into());
            return;
        }
        let target = if item.item_id.as_str() == "latemoon.npc.obj.book" {
            "latemoon.latemoon8"
        } else {
            "latemoon.latemoon1"
        };
        self.player.spirit = (self.player.spirit - 50).max(0);
        self.move_to(LocationId::from(target));
        self.push_log("你按舞曲谱踏出「春宫怨」，身影随曲声消失。".into());
    }

    fn pray_latemoon_bracelet(&mut self, instance_id: u64) {
        let Some(item) = self.player.item(instance_id) else {
            return;
        };
        if !LATEMOON_BRACELET_IDS.contains(&item.item_id.as_str()) {
            return;
        }
        if self.player.spirit < 50 {
            self.push_log("你的神不足，无法借手镯祈求归返。".into());
            return;
        }
        self.player.spirit -= 50;
        self.move_to(LocationId::from("snow.temple"));
        self.push_log("玛瑙手镯嗡嗡作响，你在烟雾中回到雪亭城隍庙。".into());
    }

    fn read_latemoon_secret_letter(&mut self, instance_id: u64) {
        if self
            .player
            .item(instance_id)
            .is_none_or(|item| item.item_id.as_str() != LATEMOON_SECRET_LETTER_ID)
            || !self.player.has_item(&ItemId::from(LATEMOON_FIRE_ID))
        {
            return;
        }
        self.push_log("火光烘出密函暗字：晚月庄密室藏有舞谱与玛瑙手镯，小花池也另有玄机。".into());
    }

    fn pull_chuenyu_hall_rope(&mut self) {
        let damage = 5 + self.random(10) as i32;
        self.player.qi = (self.player.qi - damage).max(0);
        self.move_to(LocationId::from("chuenyu.tunnel1"));
        self.push_log(format!("垂绳触动翻板，你跌进地牢，损失{damage}点气。"));
    }

    fn climb_chuenyu_castle_wall(&mut self) {
        let target = match self.location.as_str() {
            "chuenyu.east_castle" => "chuenyu.east_garden",
            "chuenyu.east_garden" => "chuenyu.east_castle",
            "chuenyu.west_castle" => "chuenyu.west_garden",
            "chuenyu.west_garden" => "chuenyu.west_castle",
            _ => return,
        };
        self.move_to(LocationId::from(target));
        self.push_log("你抓住藤蔓翻过墙头，轻巧地落到另一侧。".into());
    }

    fn push_chuenyu_dungeon_slab(&mut self) {
        self.chuenyu_slab_pushes = self.chuenyu_slab_pushes.saturating_add(1);
        if self.chuenyu_slab_pushes >= 5 {
            self.chuenyu_slab_pushes = 0;
            self.chuenyu_slab_passage_ticks = 3;
            self.push_log("石板终于斜立起来，露出通往城堡东侧的窄缝。".into());
        } else {
            self.push_log(format!(
                "你用力推动石板，石板渐渐松动（{}/5）。",
                self.chuenyu_slab_pushes
            ));
        }
    }

    fn push_green_boulder(&mut self) {
        if self.player.force < 560
            || self.player.max_force < 560
            || self.player.skill_level(FORCE_ID) < 40
        {
            self.push_log("你运起内力推向巨石，却仍然出力不足。".into());
            return;
        }
        self.player.essence = (self.player.essence - 20).max(0);
        self.player.qi = (self.player.qi - 60).max(0);
        self.player.spirit = (self.player.spirit - 20).max(0);
        if self.random(3) == 0 {
            self.move_to(LocationId::from("green.entrance"));
            self.push_log("巨石滚开片刻，你从后方小洞钻出，身后洞口随即封闭。".into());
        } else {
            self.push_log("巨石挪动了少许，又在风中滚回原位。".into());
        }
    }

    fn fill_green_well(&mut self) {
        let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| item.definition().category == items::ItemCategory::Liquid)
        else {
            return;
        };
        let name = self.player.inventory[index].display_name().to_string();
        let item = &mut self.player.inventory[index];
        item.remaining_uses = Some(15);
        item.filled_with_water = true;
        item.slumber_effect = 0;
        self.push_log(format!("你倒净{name}，从井里装入十五口清水。"));
    }

    fn search_green_stream(&mut self) {
        if self.green_windsword_rewarded {
            self.push_log("溪底只剩被水磨亮的碎石，再没有先前那道剑光。".into());
            return;
        }
        if !self.green_bagua_completed {
            self.push_log("你在溪中摸索许久，除了冰凉卵石外一无所获。".into());
            return;
        }
        if self.random(2) == 1 {
            self.add_inventory_item(ItemId::from(GREEN_WIND_SWORD_ID), 1);
            self.green_windsword_rewarded = true;
            self.green_bagua_completed = false;
            self.push_log("你从溪底抽出一把淡青长剑，正是追风剑。".into());
        } else {
            self.green_bagua_completed = false;
            self.push_log("亮光在指间一闪即逝，你最终仍是一无所获。".into());
        }
    }

    fn take_sanyen_bun(&mut self) {
        if self.current_room_has_npc(SANYEN_COOK_ID) || self.sanyen_buns_taken >= 5 {
            return;
        }
        self.add_inventory_item(ItemId::from(SANYEN_BUN_ID), 1);
        self.sanyen_buns_taken += 1;
        self.push_log("你从蒸笼里取出一枚热乎乎的馒头。".into());
    }

    fn bury_oldpine_skeleton(&mut self) {
        let Some(ground) = self.ground_items.get_mut(&self.location) else {
            return;
        };
        let Some(index) = ground
            .iter()
            .position(|item| item.item_id.as_str() == "oldpine.npc.skeleton")
        else {
            return;
        };
        ground.remove(index);
        if ground.is_empty() {
            self.ground_items.remove(&self.location);
        }
        self.push_log("你小心翼翼地掩埋了南危水的骸骨。".into());
        let fortune = self.random(self.player.spirituality.saturating_add(10).max(1));
        if fortune > 25 {
            let instance_id = self.allocate_item_instance_id();
            self.ground_items
                .entry(self.location.clone())
                .or_default()
                .push(ItemInstance::new(
                    instance_id,
                    ItemId::from("oldpine.npc.obj.parrybook"),
                    1,
                ));
            self.push_log("洞顶喀喇一响，一本《过招要旨》坠落在地。".into());
        } else {
            if fortune > 20 {
                self.push_log("洞顶纷纷扬扬飘下几张残破纸片。".into());
            }
            self.move_to(LocationId::from("oldpine.waterfall"));
            self.push_log("山洞轰然震动，你立足不稳，摔进瀑布下方。".into());
        }
    }

    fn borrow_choyin_book(&mut self) {
        let item_id = if self.random(3) < 2 {
            "choyin.npc.obj.book1"
        } else {
            "choyin.npc.obj.book2"
        };
        self.add_inventory_item(ItemId::from(item_id), 1);
        self.push_log("你趁人不备，从矮几上取了一本旧书藏入怀中。".into());
    }

    fn return_choyin_borrowed_books(&mut self) {
        let before = self.player.inventory.len();
        self.player.inventory.retain(|item| {
            !matches!(
                item.item_id.as_str(),
                "choyin.npc.obj.book1" | "choyin.npc.obj.book2"
            )
        });
        if self.player.inventory.len() != before {
            self.push_log("离开草堂前，你把借来的书卷放回矮几。".into());
        }
    }

    fn move_through_choyin_taolin(&mut self, direction: &str) {
        if self.choyin_taolin_steps == 0 {
            self.choyin_taolin_steps = 3;
        }
        let expected = choyin_taolin_clue(self.choyin_taolin_clue).1;
        if direction == expected {
            if self.choyin_taolin_steps <= 1 {
                self.choyin_taolin_steps = 0;
                self.choyin_taolin_clue = self.random(11) as u8;
                self.choyin_scholar_trial_completed = true;
                self.move_to(LocationId::from("choyin.entrance"));
                self.push_log("你循着最后一张字条走出桃林，骆云舟的考验已经完成。".into());
                return;
            }
            self.choyin_taolin_steps -= 1;
        } else {
            self.choyin_taolin_steps = self.choyin_taolin_steps.saturating_add(3);
        }
        self.choyin_taolin_clue = self.random(11) as u8;
        self.push_log("桃枝在身后合拢，你仍置身纵横交错的桃林中。".into());
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
            let max_health = enemy.max_health();
            self.activity = Activity::Fighting(CombatState {
                enemy,
                health: max_health,
                max_health,
                rounds: 0,
                mode: CombatMode::Spar,
                attack_bonus: 0,
                dodge_bonus: 0,
                enemy_attack_bonus: 0,
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

    fn search_city_ruined_garden(&mut self) {
        let found = self.player.spirituality >= 35
            || self.random(self.player.spirituality.saturating_add(30).max(1)) >= 35;
        if found {
            self.add_inventory_item(ItemId::from(CITY_EXIT_TOKEN_ID), 1);
            self.push_log("你拨开一片乱草，找到一支遗失多年的出城令牌。".into());
        } else {
            self.push_log("你在荒草和残垣间翻找许久，仍旧一无所获。".into());
        }
    }

    fn turn_altar_button(&mut self, forward: bool) {
        if forward {
            self.city_altar_forward_turns = self.city_altar_forward_turns.saturating_add(1);
            self.push_log("你将祭坛按钮顺时针转了一圈，机关深处毫无动静。".into());
        } else {
            self.city_altar_backward_turns = self.city_altar_backward_turns.saturating_add(1);
            self.push_log("你将祭坛按钮逆时针转了一圈，石台下传来轻微摩擦声。".into());
        }
    }

    fn press_altar_button(&mut self) {
        let combination_matches =
            self.city_altar_forward_turns == 1 && self.city_altar_backward_turns == 3;
        self.city_altar_forward_turns = 0;
        self.city_altar_backward_turns = 0;
        if combination_matches && self.city_altar_passage_ticks == 0 {
            self.city_altar_passage_ticks = 3;
            self.push_log("地板轧轧移开，祭坛下露出一段向下的阶梯。".into());
        } else if self.city_altar_passage_ticks > 0 {
            self.push_log("祭坛下的阶梯已经敞开。".into());
        } else {
            self.push_log("按钮沉下又弹回，先前转动的机括随之复位。".into());
        }
    }

    fn push_snow_shelf(&mut self) {
        if self.snow_storage_passage_ticks > 0 {
            self.push_log("架子后通往密室的阶梯仍然敞开。".into());
            return;
        }
        self.snow_shelf_pushes = self.snow_shelf_pushes.saturating_add(1);
        if self.snow_shelf_pushes == 3 {
            self.snow_shelf_pushes = 0;
            self.snow_storage_passage_ticks = 10;
            self.push_log("架子第三次弹回原位，地板随即移开，露出向下的密道。".into());
        } else {
            self.push_log("你将兵器架往左推去；喀的一声，它又弹回原位。".into());
        }
    }

    fn work_at_snow_workshop(&mut self) {
        if self.player.essence < 30 || self.player.spirit < 30 {
            self.push_log("你的精神太差，现在不能继续做工。".into());
            return;
        }
        self.player.essence -= 30;
        self.player.spirit -= 30;
        self.player.silver = self.player.silver.saturating_add(1);
        self.push_log("你辛苦做完一轮谷物脱壳，老板付给你一两纹银。".into());
    }

    fn move_bamboo_boulder(&mut self) {
        if self.player.force < 560
            || self.player.max_force < 560
            || self.player.skill_level(FORCE_ID) < 40
        {
            self.push_log("你运劲推了推大黄石，内力火候还远远不够。".into());
            return;
        }
        self.player.essence = (self.player.essence - 20).max(0);
        self.player.qi = (self.player.qi - 60).max(0);
        self.player.spirit = (self.player.spirit - 20).max(0);
        self.canyon_boulder_open = true;
        self.push_log("你运足内力将大黄石缓缓推向左侧，露出一道仅容一人通过的缝隙。".into());
    }

    fn search_bamboo_bookcase(&mut self) {
        self.canyon_bookcase_searched = true;
        self.add_inventory_item(ItemId::from(CANYON_SLIPCASE_ID), 1);
        self.add_inventory_item(ItemId::from(CANYON_PARRY_BOOK_ID), 1);
        self.push_log("你从石制书柜中找到一只铜书匣，匣内藏着《悟疾风劲竹书》。".into());
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
            OLD_LIU_ID | CHUENYU_OLD_LIU_ID => match self.quest {
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
                QuestStage::FoundJuan => {
                    self.push_log("刘老农：还没有娟儿的消息么？老汉实在放心不下。".into());
                }
                QuestStage::ReturnHome => {
                    self.quest = QuestStage::Complete;
                    self.add_inventory_item(ItemId::from(items::HENGBING_SWORD_ID), 1);
                    self.add_inventory_item(ItemId::from(items::PARRY_MANUAL_ID), 1);
                    self.push_log(
                        "刘老农：多谢搭救小女。这口銮鱼衡冰与过招要旨便赠予少侠。".into(),
                    );
                    self.push_log("刘老农领着娟儿匆匆离去。你获得銮鱼衡冰与过招要旨。".into());
                }
                QuestStage::MurderedJuan => {
                    self.trigger_old_liu_revenge();
                }
                QuestStage::Complete | QuestStage::Failed => {}
            },
            XIAO_JUAN_ID | CHUENYU_XIAO_JUAN_ID | CHUENYU_XIAO_JUAN_PLACED_ID
                if self.quest == QuestStage::FoundJuan =>
            {
                self.quest = QuestStage::ReturnHome;
                self.push_log("小娟挣脱绳索后跟在你身边，请你带她回刘家小房。".into());
                self.push_log("任务更新：护送小娟回家。".into());
            }
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
        if !self.npc_is_present(&npc) {
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
            ScriptedInquiryKind::ChoyinPoliceBribery => {
                self.push_log(
                    "巡捕说道：说哪里话来，县太爷清贫廉正，我们作手下的岂能辱没他的名声？收起你的钱吧！"
                        .into(),
                );
            }
            ScriptedInquiryKind::ChoyinSilkBag => {
                self.choyin_silk_bag_received = true;
                self.add_inventory_item(ItemId::from(CHOYIN_SILK_BAG_ID), 1);
                self.push_log(
                    "官家小姐低声道：小女子有一事相求，请您将这个交给游公子。她把一个紫罗鸳鸯荷包递给你。"
                        .into(),
                );
            }
            ScriptedInquiryKind::ChoyinYoungManTrouble => {
                for line in [
                    "实不相瞒，在下仰慕前面曲桥上赏莲的姑娘已久……",
                    "但是一直不知那位姑娘芳名……",
                    "唉……",
                ] {
                    self.push_log(format!("贵公子说道：{line}"));
                }
            }
            ScriptedInquiryKind::GreenOldManJade => {
                self.green_elder_jade_clue = true;
                for line in [
                    "这块玉佩原是村里故人留下的物件。",
                    "后来辗转落到雪亭一个醉汉手中，你带好酒去问，他也许肯说。",
                ] {
                    self.push_log(format!("村长说道：{line}"));
                }
            }
            ScriptedInquiryKind::GreenShenJade => {
                self.green_jade_received = true;
                self.add_inventory_item(ItemId::from(GREEN_JADE_ID), 1);
                self.push_log("沈万年从柜中取出一块玉佩：既然你知道来历，就拿去吧。".into());
            }
            ScriptedInquiryKind::GreenShenSlumberDrug => {
                self.green_drug_offer_unlocked = true;
                self.push_log(
                    "沈万年压低声音：蒙汗药可以给你，但得拿价值一千文以上的东西来换。".into(),
                );
            }
            ScriptedInquiryKind::LatemoonGirlDance => {
                self.push_log("小姑娘笑道：晚月庄的舞步要合着月色与风声，半点也急不得。".into());
            }
            ScriptedInquiryKind::LatemoonGirlDragonDance => {
                self.push_log("小姑娘说道：寒谷龙舞刚柔相济，是庄中代代相传的舞法。".into());
            }
            ScriptedInquiryKind::LatemoonShaoweiDragonfly => {
                self.push_log("少庄主说道：寻一截好竹子来，我可以替你削成竹蜻蜓。".into());
            }
            ScriptedInquiryKind::LatemoonShinfunDanceBook => {
                self.latemoon_dance_book_clue = true;
                self.push_log("辛芬低声道：舞曲谱藏在西厢床榻内侧，你仔细摸索便能找到。".into());
            }
            ScriptedInquiryKind::LatemoonYumayFunlin => {
                self.push_log("玉梅说道：凤铃最喜欢精巧玩意，拿竹蜻蜓给她看准没错。".into());
            }
            ScriptedInquiryKind::LatemoonYumayLearnDance => {
                self.push_log("玉梅笑道：先去拜见蓝庄主，入门之后才好认真学舞。".into());
            }
            ScriptedInquiryKind::CloudBoaterCross => {
                self.push_log("船夫说道：拿一件值钱物事作船资，我便送你渡过北河。".into());
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
        if teacher_id == "scholar" && !self.choyin_scholar_trial_completed {
            self.choyin_scholar_trial_started = true;
            self.push_log("骆云舟说道：你还是先走一趟东边的桃林吧。".into());
            return;
        }
        if let Some(reason) = self.apprenticeship_rejection(&teacher_id) {
            self.push_log(format!("{}摇头道：{reason}", teacher.name));
            return;
        }

        self.player.teacher = Some(teacher_id);
        self.player.faction = teacher.faction.clone();
        if teacher.id == "scholar" {
            self.choyin_scholar_trial_started = false;
            self.push_log("骆云舟点头认可你走出桃林，将你收入步玄派门下。".into());
        } else if teacher.id == "fighter" {
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
            "swordsman" if self.player.courage < 20 || self.player.composure < 20 => {
                Some("学剑之人必须胆大心细，你的心性尚需磨炼。")
            }
            "dancer" if self.player.gender != Gender::Female => {
                Some("晚月庄只收女弟子，你与本门舞学无缘。")
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
        if !self.npc_is_present(&npc) {
            self.push_log("这里没有可以指点你这项技能的人。".into());
            return;
        }
        let Some(definition) = npcs().definition(&npc) else {
            self.push_log("这里没有可以指点你这项技能的人。".into());
            return;
        };
        let Some(policy) = definition.apprenticeship_policy() else {
            self.push_log(format!("{}不愿传授武艺。", definition.name));
            return;
        };
        let Some(lesson) = definition
            .lessons()
            .iter()
            .find(|lesson| lesson.skill == skill_id.as_str())
        else {
            self.push_log("这项技能必须另寻高人请教。".into());
            return;
        };
        if !self.npc_lesson_access(policy) {
            let rejection = match policy {
                NpcApprenticeshipPolicy::RecognizeFaction(faction)
                | NpcApprenticeshipPolicy::SameFaction(faction) => {
                    format!(
                        "{}说道：你并非{faction}同门，我不能传你本门武艺。",
                        definition.name
                    )
                }
                NpcApprenticeshipPolicy::PaidStudent => {
                    "魏无极说道：咦？我不记得收过你这个学生啊。".into()
                }
                NpcApprenticeshipPolicy::DeferredLetter => {
                    "陈天星问道：可有剑秋署名的信物？".into()
                }
                NpcApprenticeshipPolicy::PlotGated => {
                    "陈剑秋说道：先入振远镖局，再谈本门功夫。".into()
                }
                NpcApprenticeshipPolicy::ExcludedUnplaced => {
                    format!("{}眼下不在此处授业。", definition.name)
                }
            };
            self.push_log(rejection);
            return;
        }

        let teacher_name = definition.name.clone();
        let intelligence = definition
            .teaching_intelligence()
            .expect("runtime instructor must have intelligence");
        self.learn_skill_from_teacher(skill_id, &teacher_name, intelligence, lesson.max_level);
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

    fn abandon_skill(&mut self, skill_id: SkillId) {
        let Some(index) = self
            .player
            .skills
            .iter()
            .position(|skill| skill.kind == skill_id)
        else {
            self.push_log("你并没有学过这项技能。".into());
            return;
        };

        if self.activity == Activity::Training(skill_id.clone()) {
            self.activity = Activity::Idle;
        }
        self.player.skills.remove(index);
        self.push_log(format!("你决定放弃继续学习{}。", skill_id.name()));
    }

    fn self_learn(&mut self, skill_id: SkillId) {
        if !matches!(
            skill_id.as_str(),
            DODGE_ID | FORCE_ID | SWORD_ID | "blade" | "staff" | PARRY_ID | UNARMED_ID
        ) {
            self.push_log("这项技能不能通过自学取得进步。".into());
            return;
        }
        let level = self.player.skill_level(skill_id.as_str());
        if level < 40 {
            self.push_log(format!("你得先具备{}的入门造诣才行。", skill_id.name()));
            return;
        }
        if self.player.learned_points >= self.player.potential {
            self.push_log("你的潜能已经发挥到极限，无法再靠自学成长。".into());
            return;
        }
        let intelligence = self.player.intelligence.max(1);
        let full_cost = 300 / i32::try_from(intelligence).unwrap_or(1).max(1);
        self.push_log(format!("你开始钻研有关{}的问题。", skill_id.name()));
        if self.player.essence > full_cost {
            if u64::from(level).pow(3) / 10 > self.player.combat_experience {
                self.push_log("也许是缺乏实战经验，结果一无所获。".into());
            } else {
                let upper = intelligence.saturating_add(level).max(1);
                let gain = self.random(upper);
                self.player.learned_points = self.player.learned_points.saturating_add(1);
                self.gain_skill_progress(skill_id.clone(), gain);
                self.push_log("你苦思冥想，似乎有些心得。".into());
            }
            self.player.essence -= full_cost;
        } else {
            self.player.essence = 0;
            self.push_log("你今天太累，结果什么也没有学到。".into());
        }
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

    fn start_combat(&mut self, enemy: EnemyKind, mut mode: CombatMode) {
        if self.player.essence < 20 || self.player.qi < 15 || self.player.spirit < 10 {
            self.push_log("你当前状态太差，无法贸然出手。".into());
            return;
        }

        if let EnemyKind::Npc(npc) = &enemy
            && mode == CombatMode::Lethal
            && npc.as_str() == CHOYIN_HOTEL_GUARD_ID
        {
            let total = self
                .current_location()
                .npcs
                .iter()
                .filter(|candidate| *candidate == npc)
                .count();
            let defeated = self
                .defeated_npc_instances
                .iter()
                .filter(|entry| entry.location == self.location && &entry.npc == npc)
                .count();
            if total.saturating_sub(defeated) > 1 {
                self.player.wanted = self.player.wanted.saturating_add(1);
                self.push_log("酒楼守卫高喊有强人打劫，同伴已报官缉拿。".into());
            }
        }

        if let EnemyKind::Npc(npc) = &enemy
            && mode == CombatMode::Spar
            && let Some(policy) = npcs().definition(npc).and_then(|npc| npc.fight_policy())
        {
            match policy {
                NpcFightPolicy::Allow => {}
                NpcFightPolicy::ForceLethal => {
                    mode = CombatMode::Lethal;
                    self.push_log(format!(
                        "{}说道：咦……要打就真打，光是较量多没意思？",
                        npc.name()
                    ));
                }
                NpcFightPolicy::RequireFaction(faction)
                    if self.player.faction.as_deref() != Some(faction) =>
                {
                    self.push_log(npc_fight_rejection(npc, &self.player));
                    return;
                }
                NpcFightPolicy::RequireFaction(_) => {
                    self.push_log(format!("{}点头道：进招吧。", npc.name()));
                }
                NpcFightPolicy::Reject => {
                    self.push_log(npc_fight_rejection(npc, &self.player));
                    return;
                }
            }
        }
        self.begin_combat(enemy, mode);
    }

    fn begin_combat(&mut self, enemy: EnemyKind, mode: CombatMode) {
        let max_health = enemy.max_health();
        let enemy_name = enemy.name();
        let elite_guard_powerup = matches!(
            &enemy,
            EnemyKind::Npc(npc)
                if mode == CombatMode::Lethal && npc.as_str() == WATERFOG_ELITE_GUARD_ID
        );
        let enemy_attack_bonus = if elite_guard_powerup {
            (enemy.attack() / 10).clamp(2, 30)
        } else {
            0
        };
        let forced_by_npc = matches!(
            &enemy,
            EnemyKind::Npc(npc)
                if npcs().definition(npc).and_then(|npc| npc.fight_policy())
                    == Some(NpcFightPolicy::ForceLethal)
        );
        self.activity = Activity::Fighting(CombatState {
            enemy,
            health: max_health,
            max_health,
            rounds: 0,
            mode,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        });
        if elite_guard_powerup {
            self.push_log(format!("{enemy_name}怒喝一声运起内功，攻势骤然增强。"));
        }
        match mode {
            CombatMode::Spar => {
                self.push_log(format!("你向{enemy_name}抱拳示意，双方点到为止。"));
            }
            CombatMode::Lethal if forced_by_npc => {
                self.push_log(format!("{enemy_name}忽然出手，点到为止的比试变成了死斗。"));
            }
            CombatMode::Lethal => {
                self.push_log(format!("你向{enemy_name}喝道：今日性命相搏！"));
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

    fn try_wimpy_flee(&mut self, combat: &CombatState) -> bool {
        let percent = self.wimpy_percent();
        if percent == 0 || self.location.as_str() == content::MELON_FARM && self.melon_debt {
            return false;
        }
        let current_qi = i64::from(self.player.qi.max(0));
        let maximum_qi = i64::from(self.player.max_qi.max(1));
        if current_qi * 100 > maximum_qi * i64::from(percent) {
            return false;
        }
        let destinations: Vec<_> = {
            let current = self.current_location();
            current
                .exits
                .iter()
                .filter_map(|exit| {
                    let target = self.resolved_source_exit_target(current, exit);
                    self.exit_is_available(current, exit, &target)
                        .then_some(target)
                })
                .collect()
        };
        let destination = match &combat.enemy {
            EnemyKind::Npc(npc) => destinations
                .iter()
                .find(|target| !self.npc_is_present_at(target, npc))
                .cloned(),
            _ => None,
        }
        .or_else(|| destinations.first().cloned());
        let Some(destination) = destination else {
            return false;
        };
        self.push_log(format!(
            "你的气低于自动逃跑阈值，仓促寻找退路离开{}。",
            combat.enemy.name()
        ));
        self.flee_to(destination);
        true
    }

    fn combat_tick(&mut self, mut combat: CombatState) {
        if self.try_wimpy_flee(&combat) {
            return;
        }
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
            let mut force_spent = 0;
            let force_factor = self.player.force_factor.min(self.force_factor_limit());
            let force_cost = i32::try_from(force_factor).unwrap_or(i32::MAX);
            if force_cost > 0 && self.player.force >= force_cost {
                let bonus = (force_cost / 2)
                    .saturating_add(
                        i32::try_from(self.player.effective_skill(FORCE_ID) / 10)
                            .unwrap_or(i32::MAX),
                    )
                    .max(1);
                self.player.force -= force_cost;
                damage = damage.saturating_add(bonus);
                force_spent = force_cost;
            }
            let mut mana_spent = 0;
            let mana_factor = self.player.mana_factor.min(self.mana_factor_limit());
            let mana_cost = i32::try_from(mana_factor).unwrap_or(i32::MAX);
            let magic_is_mapped = self
                .player
                .mapped_skill(MAGIC_ID)
                .is_some_and(|skill| self.player.skill_by_id(skill.as_str()).is_some());
            if has_weapon && mana_cost > 0 && magic_is_mapped && self.player.atman >= mana_cost {
                let bonus = mana_cost
                    .saturating_add(
                        i32::try_from(self.player.effective_skill(MAGIC_ID) / 12)
                            .unwrap_or(i32::MAX),
                    )
                    .max(1);
                self.player.atman -= mana_cost;
                damage = damage.saturating_add(bonus);
                mana_spent = mana_cost;
            }
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
            let zombie_assistance = match &combat.enemy {
                EnemyKind::Npc(target) => self.zombie_haunt_assistance(target),
                _ => Vec::new(),
            };
            for (_, bonus) in &zombie_assistance {
                damage = damage.saturating_add(*bonus);
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
            if force_spent > 0 {
                self.push_log(format!("你运出{}点内力，劲力随招式透出。", force_spent));
            }
            if mana_spent > 0 {
                self.push_log(format!("你以{}点灵力灌注兵器，锋芒一盛。", mana_spent));
            }
            for (zombie, bonus) in zombie_assistance {
                self.push_log(format!(
                    "{}受追魂符驱使，隔空协击造成{}点伤势。",
                    zombie.name(),
                    bonus
                ));
            }
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
        if self.run_npc_combat_chat(&mut combat) {
            return;
        }

        let enemy_attack =
            combat.enemy.attack() + combat.enemy_attack_bonus + self.random(6) as i32;
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
        self.apply_npc_hit_hook(
            &combat.enemy,
            enemy_attack.max(1) as u32,
            armor_bonus.max(0) as u32,
        );

        if self.player.essence <= 0 || self.player.qi <= 0 || self.player.spirit <= 0 {
            self.lose_combat(combat);
        } else if !self.try_wimpy_flee(&combat) {
            self.activity = Activity::Fighting(combat);
        }
    }

    fn run_npc_combat_chat(&mut self, combat: &mut CombatState) -> bool {
        let EnemyKind::Npc(npc) = combat.enemy.clone() else {
            return false;
        };
        let Some(chat) = npcs()
            .definition(&npc)
            .and_then(|definition| definition.combat_chat.clone())
        else {
            return false;
        };
        if chat.entries.is_empty() || self.random(100) >= chat.runtime_chance() {
            return false;
        }
        let entry = chat.entries[self.random(chat.entries.len() as u32) as usize].clone();
        match entry.kind.as_str() {
            "text" => self.push_log(entry.value),
            "spell" if entry.value.contains("invocation") => {
                let bonus = (combat.enemy.attack() / 8).max(2);
                combat.enemy_attack_bonus = (combat.enemy_attack_bonus + bonus).min(30);
                self.push_log(format!(
                    "{}施展招魂咒，攻势增强了{}点。",
                    combat.enemy.name(),
                    bonus
                ));
            }
            "spell" => {
                let (resource, spell) = if entry.value.contains("drainerbolt") {
                    (CombatResource::Spirit, "吸魂阴雷")
                } else if entry.value.contains("feeblebolt") {
                    (CombatResource::Qi, "虚弱阴雷")
                } else {
                    (CombatResource::Essence, "冥界阴雷")
                };
                return self.npc_special_hit(combat, resource, spell);
            }
            "force" if entry.value.contains("powerup") => {
                let bonus = (combat.enemy.attack() / 10).max(2);
                combat.enemy_attack_bonus = (combat.enemy_attack_bonus + bonus).min(30);
                self.push_log(format!(
                    "{}运起内功，攻势增强了{}点。",
                    combat.enemy.name(),
                    bonus
                ));
            }
            "force" if entry.value.contains("recover") || entry.value.contains("heal") => {
                let recovered = (combat.max_health / 5).max(1);
                let before = combat.health;
                combat.health = (combat.health + recovered).min(combat.max_health);
                self.push_log(format!(
                    "{}运功疗伤，恢复了{}点战力。",
                    combat.enemy.name(),
                    combat.health - before
                ));
            }
            "force" if entry.value.contains("powerfade") => {
                combat.enemy_attack_bonus = 0;
                self.push_log(format!("{}收敛内息，攻势恢复如常。", combat.enemy.name()));
            }
            "perform" => {
                return self.npc_special_hit(combat, CombatResource::Essence, "回剑反击");
            }
            "command" if entry.value.contains("surrender") => {
                self.activity = Activity::Idle;
                self.push_log(format!("{}跳出战圈，连声认输。", combat.enemy.name()));
                return true;
            }
            "command" if entry.value.contains("corpse") => {
                self.push_log(format!(
                    "{}掐诀试图役使尸体，但战圈内没有可用尸身。",
                    combat.enemy.name()
                ));
            }
            "movement" => self.push_log(format!(
                "{}试图冲出战圈，却被你拦住去路。",
                combat.enemy.name()
            )),
            "callback" => self.run_npc_combat_callback(combat, &npc, &entry.value),
            _ => {}
        }
        false
    }

    fn npc_can_roam_safely(npc: &NpcId) -> bool {
        matches!(
            npc.as_str(),
            "city.npc.trader"
                | "snow.npc.crazy_dog"
                | "snow.npc.trav_blade"
                | "snow.npc.traveller"
                | "snow.npc.woodcutter"
                | "choyin.npc.ghost"
                | "choyin.npc.scholar"
                | "obj.npc.garrison"
                | "green.npc.kid2"
                | "green.npc.kid3"
                | "latemoon.npc.butterfly"
                | "latemoon.park.npc.bird"
                | "latemoon.park.npc.dodo1"
                | "latemoon.park.npc.rabbit"
                | "latemoon.room.npc.killer"
                | "latemoon.upstar.npc.bird"
        )
    }

    fn run_npc_random_movement(&mut self, npc: &NpcId) -> Option<LocationId> {
        if !Self::npc_can_roam_safely(npc) || !self.npc_is_present(npc) {
            return None;
        }
        let source = self.location.clone();
        let current = self.current_location();
        let targets: Vec<_> = current
            .exits
            .iter()
            .filter(|exit| {
                exit.internal
                    && !exit.dynamic
                    && !current
                        .doors
                        .iter()
                        .any(|door| door.direction == exit.direction)
            })
            .filter_map(|exit| {
                let target = exit.target.clone();
                let target_location = world().location(&target)?;
                (target_location.zone.as_str() == current.zone.as_str()
                    && self.exit_is_available(current, exit, &target))
                .then(|| (exit.direction.clone(), target))
            })
            .collect();
        if targets.is_empty() {
            return None;
        }
        let slots = self.static_npc_slots_at(&source, npc);
        if slots.is_empty() {
            return None;
        }
        let (direction, target) = targets[self.random(targets.len() as u32) as usize].clone();
        let (origin, ordinal) = slots[self.random(slots.len() as u32) as usize].clone();
        self.set_static_npc_location(&origin, npc, ordinal, target.clone());
        self.push_log(format!("{}向{}离开了。", npc.name(), direction));
        Some(target)
    }

    fn reset_scheduled_npc_positions(&mut self) {
        let defeated_locations: HashSet<_> = self
            .defeated_npc_instances
            .iter()
            .map(|entry| (entry.location.clone(), entry.npc.clone()))
            .collect();
        let returning = self
            .npc_location_overrides
            .iter()
            .filter(|entry| {
                Self::npc_can_roam_safely(&entry.npc)
                    && !defeated_locations.contains(&(entry.location.clone(), entry.npc.clone()))
            })
            .count();
        self.npc_location_overrides.retain(|entry| {
            !Self::npc_can_roam_safely(&entry.npc)
                || defeated_locations.contains(&(entry.location.clone(), entry.npc.clone()))
        });
        if returning > 0 && self.current_location().outdoors.is_some() {
            self.push_log("拂晓将至，游荡的行人与小兽陆续回到原处。".into());
        }
    }

    fn expire_zombie_haunts(&mut self) {
        let mut active = Vec::new();
        let mut expired = Vec::new();
        for haunt in std::mem::take(&mut self.zombie_haunts) {
            if haunt.expires_at_elapsed_minutes <= self.elapsed_minutes {
                expired.push(haunt);
            } else {
                active.push(haunt);
            }
        }
        self.zombie_haunts = active;
        for haunt in expired {
            if self.npc_is_present(&haunt.zombie) || self.npc_is_present(&haunt.target) {
                self.push_log(format!(
                    "{}额前的追魂符失去灵光，追击随之停歇。",
                    haunt.zombie.name()
                ));
            }
        }
    }

    fn advance_npc_respawns(&mut self) {
        let mut pending = Vec::new();
        for respawn in std::mem::take(&mut self.npc_respawns) {
            if respawn.due_elapsed_minutes > self.elapsed_minutes {
                pending.push(respawn);
                continue;
            }
            if !Self::npc_can_roam_safely(&respawn.npc) {
                continue;
            }
            let Some(index) = self
                .defeated_npc_instances
                .iter()
                .position(|entry| entry.location == respawn.location && entry.npc == respawn.npc)
            else {
                continue;
            };
            self.defeated_npc_instances.remove(index);
            self.stolen_npc_items
                .retain(|entry| entry.npc != respawn.npc);
            self.npc_location_overrides
                .retain(|entry| entry.npc != respawn.npc || entry.location != respawn.location);
            if self.location == respawn.location {
                self.push_log(format!("{}重新出现在附近。", respawn.npc.name()));
            }
        }
        self.npc_respawns = pending;
    }

    fn zombie_haunt_assistance(&self, target: &NpcId) -> Vec<(NpcId, i32)> {
        self.zombie_haunts
            .iter()
            .filter(|haunt| {
                haunt.target == *target
                    && haunt.expires_at_elapsed_minutes > self.elapsed_minutes
                    && self.npc_is_present(&haunt.zombie)
            })
            .map(|haunt| {
                let bonus = npcs()
                    .definition(&haunt.zombie)
                    .map_or(3, |definition| (definition.combat_attack() / 8).max(3));
                (haunt.zombie.clone(), bonus)
            })
            .collect()
    }

    fn day_phase_index(elapsed_minutes: u64) -> usize {
        let mut remaining = elapsed_minutes % DAY_MINUTES;
        for (index, phase) in DAY_PHASES.iter().enumerate() {
            if remaining < u64::from(phase.duration_minutes) {
                return index;
            }
            remaining -= u64::from(phase.duration_minutes);
        }
        unreachable!("day phases must cover a full day")
    }

    fn day_phase(&self) -> &'static DayPhase {
        &DAY_PHASES[Self::day_phase_index(self.elapsed_minutes)]
    }

    fn run_npc_ambient_chat(&mut self) {
        let present_npcs = self.present_current_npcs();
        for npc in present_npcs {
            let Some(chat) = npcs().ambient_chat(&npc).cloned() else {
                continue;
            };
            if chat.entries.is_empty() || self.random(100) >= chat.runtime_chance() {
                continue;
            }
            let entry = chat.entries[self.random(chat.entries.len() as u32) as usize].clone();
            match entry.kind.as_str() {
                "text" => self.push_log(entry.value),
                "movement" if !entry.value.trim_start().starts_with("//") => {
                    self.run_npc_random_movement(&npc);
                }
                _ => {}
            }
        }
    }

    fn run_npc_combat_callback(&mut self, combat: &mut CombatState, npc: &NpcId, callback: &str) {
        if npc.as_str() == OLDPINE_FAT_BANDIT_ID && callback.contains("call_for_help") {
            if self.spawned_npc_instances.iter().any(|entry| {
                entry.location == self.location && entry.npc.as_str() == OLDPINE_BANDIT_CHIEF_ID
            }) {
                return;
            }
            self.spawned_npc_instances.push(SpawnedNpcInstance {
                location: self.location.clone(),
                npc: NpcId::from(OLDPINE_BANDIT_CHIEF_ID),
            });
            self.push_log("矮胖土匪高声呼救，一名土匪老大提刀赶到战圈旁。".into());
            return;
        }

        let bonus = match (npc.as_str(), callback) {
            ("green.npc.oldman", value) if value.contains("ask_for_help") => {
                if self.current_room_has_npc("green.npc.oldwoman") {
                    self.push_log("老头高声呼喊，老妇从旁夹攻，使他的攻势更紧。".into());
                    5
                } else {
                    self.push_log("老头高声呼救，却没有人赶来。".into());
                    0
                }
            }
            ("green.npc.oldwoman", value) if value.contains("ask_for_help") => {
                if self.current_room_has_npc("green.npc.oldman") {
                    self.push_log("老妇招呼老伴相助，两人一前一后逼近。".into());
                    5
                } else {
                    self.push_log("老妇呼喊老伴，却没有回应。".into());
                    0
                }
            }
            ("green.npc.oldman", value) if value.contains("wield_something") => {
                self.push_log("老头抽出随身短刃，攻势陡然凌厉。".into());
                4
            }
            ("green.npc.woman1", value) if value.contains("wield_weapon") => {
                self.push_log("农妇抄起一把短刀护在身前。".into());
                4
            }
            ("green.npc.woman1", value) if value.contains("converse_one") => {
                self.push_log("农妇一边招架，一边厉声质问你为何欺负村中妇孺。".into());
                0
            }
            _ => 0,
        };
        combat.enemy_attack_bonus = (combat.enemy_attack_bonus + bonus).min(30);
    }

    fn apply_npc_hit_hook(&mut self, enemy: &EnemyKind, damage: u32, armor: u32) {
        let EnemyKind::Npc(npc) = enemy else {
            return;
        };
        let current_duration = self
            .player
            .condition(ConditionKind::SnakePoison)
            .map_or(0, |condition| condition.duration);
        if npc.as_str() == OLDPINE_VENOM_SNAKE_ID
            && current_duration < 10
            && self.random(damage.max(1)) > armor
        {
            self.player
                .set_condition(ConditionKind::SnakePoison, 20, 10);
            self.push_log("你觉得被金银花蛇咬中的地方一阵麻痒。".into());
        }
    }

    fn npc_special_hit(
        &mut self,
        combat: &CombatState,
        resource: CombatResource,
        action: &str,
    ) -> bool {
        let damage = (combat.enemy.attack() / 3).max(3);
        match resource {
            CombatResource::Essence => self.player.essence -= damage,
            CombatResource::Qi => self.player.qi -= damage,
            CombatResource::Spirit => self.player.spirit -= damage,
        }
        self.push_log(format!(
            "{}施展{action}，你损失{}点{}。",
            combat.enemy.name(),
            damage,
            resource.name()
        ));
        if self.player.essence <= 0 || self.player.qi <= 0 || self.player.spirit <= 0 {
            self.lose_combat(combat.clone());
            true
        } else {
            false
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

        if combat.mode == CombatMode::Lethal
            && !matches!(
                &enemy,
                EnemyKind::Npc(npc) if GOATHILL_LEECH_CORPSE_NPCS.contains(&npc.as_str())
            )
        {
            self.place_corpse_on_ground(enemy.name());
        }

        self.complete_dynamic_quest(&enemy, combat.mode);

        match enemy {
            EnemyKind::Bandit if self.quest == QuestStage::FindJuan => {
                self.quest = QuestStage::FoundJuan;
                self.push_log("你赶走山贼，在林间找到了被绑住的小娟。".into());
                self.push_log("小娟一边挣脱绳索，一边向你呼救。".into());
            }
            EnemyKind::XiaoJuan => {
                self.quest = QuestStage::MurderedJuan;
                self.push_log("小娟倒在林中。这个消息迟早会传到刘老农耳中。".into());
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
            EnemyKind::Npc(npc) if combat.mode == CombatMode::Lethal => {
                if npc.as_str() == CHUENYU_BOSS_ID && self.quest == QuestStage::FindJuan {
                    self.quest = QuestStage::FoundJuan;
                    self.push_log("你击败绮云庄主，在地牢深处找到了被囚禁的小娟。".into());
                }
                if matches!(
                    npc.as_str(),
                    CHUENYU_XIAO_JUAN_ID | CHUENYU_XIAO_JUAN_PLACED_ID
                ) {
                    self.quest = QuestStage::MurderedJuan;
                    self.push_log("小娟倒在地牢中，这个消息迟早会传到刘老农耳中。".into());
                }
                self.defeated_npc_instances.push(DefeatedNpcInstance {
                    location: self.location.clone(),
                    npc: npc.clone(),
                });
                if Self::npc_can_roam_safely(&npc) {
                    self.npc_respawns.push(NpcRespawn {
                        location: self.location.clone(),
                        npc: npc.clone(),
                        due_elapsed_minutes: self.elapsed_minutes.saturating_add(
                            NPC_RESPAWN_DELAY_TICKS.saturating_mul(GAME_MINUTES_PER_TICK),
                        ),
                    });
                }
                if npc.as_str() == CHOYIN_LION_ID {
                    self.place_item_on_ground(ItemId::from(CHOYIN_GRASS_ID), 1);
                    self.push_log("护草神兽倒下，一棵晶莹的忘忧草落在地上。".into());
                }
                if npc.as_str() == CHOYIN_POLICE_ID {
                    self.player.wanted = self.player.wanted.saturating_add(5);
                    self.push_log("杀害公差使你的罪名加重，通缉额外 +5。".into());
                }
                if GOATHILL_LEECH_CORPSE_NPCS.contains(&npc.as_str()) {
                    self.place_item_on_ground(ItemId::from(GOATHILL_DEAD_LEECH_ID), 1);
                    self.push_log("岩蛭死后蜷成一团，一条死岩蛭留在地上。".into());
                }
                self.drop_npc_carried_items(&npc);
            }
            EnemyKind::Npc(_) => {}
            _ => {}
        }
    }

    fn complete_dynamic_quest(&mut self, enemy: &EnemyKind, mode: CombatMode) {
        if mode != CombatMode::Lethal {
            return;
        }
        let EnemyKind::Npc(npc) = enemy else {
            return;
        };
        let Some(quest) = self.dynamic_quest.clone() else {
            return;
        };
        if npc.name() != quest.target {
            return;
        }
        if self.elapsed_minutes > quest.deadline_elapsed_minutes {
            self.push_log(format!(
                "「{}」已经倒下，但朱鸿雪的悬赏期限已过，未获报酬。",
                quest.target
            ));
            return;
        }

        let experience = self.dynamic_quest_reward(quest.exp_bonus, quest.factor);
        let potential = self.dynamic_quest_reward(quest.potential_bonus, quest.factor);
        let score = self.dynamic_quest_reward(
            u32::try_from(quest.score_bonus)
                .expect("source dynamic quest score reward must be non-negative"),
            quest.factor,
        ) as i32;
        self.player.combat_experience = self
            .player
            .combat_experience
            .saturating_add(experience as u64);
        let available_potential = self
            .player
            .potential
            .saturating_sub(self.player.learned_points);
        self.player.potential = self
            .player
            .learned_points
            .saturating_add(available_potential.saturating_add(potential).min(100));
        let score_delta = if self.player.reputation < 0 {
            -score
        } else {
            score
        };
        self.player.reputation = self.player.reputation.saturating_add(score_delta);
        self.dynamic_quest = None;
        self.dynamic_quest_finished = if self.dynamic_quest_finished > 9 {
            0
        } else if self.dynamic_quest_finished < -10 {
            1
        } else {
            self.dynamic_quest_finished.saturating_add(1)
        };
        self.push_log(format!(
            "朱鸿雪的悬赏完成：实战经验 +{experience}，潜能 +{potential}，综合评价 {score_delta:+}。"
        ));
    }

    fn dynamic_quest_reward(&mut self, bonus: u32, factor: u32) -> u32 {
        let half = bonus / 2;
        let rolled = half.saturating_add(self.random(half.max(1)));
        rolled.saturating_mul(factor) / DYNAMIC_QUEST_FACTOR
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
                ConditionKind::RosePoison => {
                    self.player.spirit -= 20;
                    self.player.qi -= 10;
                    self.push_log("火玫瑰毒发作，你的神与气受到损伤。".into());
                }
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

    pub(crate) fn migrate_v7_m4_combat(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v8_old_liu_plot(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v9_m4_npc_combat(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v10_city_exit_permit(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v11_m4_room_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v12_source_doors(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v13_source_room_items(&mut self) {
        self.initialize_source_room_items();
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v14_m5_source_room_items(&mut self) {
        self.initialize_m5_source_room_items();
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v15_m5_choyin_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v16_m5_room_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v17_m5_npc_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v18_m5_regional_gameplay(&mut self) {
        self.player
            .inventory
            .retain(|item| !CHOYIN_DONATION_BOX_IDS.contains(&item.item_id.as_str()));
        for ground in self.ground_items.values_mut() {
            ground.retain(|item| !CHOYIN_DONATION_BOX_IDS.contains(&item.item_id.as_str()));
        }
        self.ground_items.retain(|_, ground| !ground.is_empty());
        let instance_id = self.allocate_item_instance_id();
        self.ground_items
            .entry(LocationId::from("choyin.altar"))
            .or_default()
            .push(ItemInstance::new(
                instance_id,
                ItemId::from("choyin.obj.denotation"),
                1,
            ));
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v19_m6_source_room_items(&mut self) {
        self.initialize_m6_source_room_items();
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v20_m6_room_events(&mut self) {
        if self.location.as_str() == "chuenyu.trap_castle" {
            self.chuenyu_trap_arrow_ticks = 2;
        }
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v21_m6_npc_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v22_m7_source_room_items(&mut self) {
        self.initialize_m7_source_room_items();
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v23_m7_room_and_item_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v24_m7_npc_events(&mut self) {
        self.version = SAVE_VERSION;
    }

    pub(crate) fn migrate_v25_m8_world_time(&mut self) {
        self.last_saved_at_unix_seconds = None;
        self.initialize_m8_temporary_item_expirations();
        self.version = M8_WORLD_TIME_SAVE_VERSION;
    }

    pub(crate) fn migrate_v26_m8_corpse_lifecycle(&mut self) {
        self.initialize_m8_temporary_item_expirations();
        self.version = M8_CORPSE_LIFECYCLE_SAVE_VERSION;
    }

    pub(crate) fn migrate_v27_m8_npc_positions(&mut self) {
        self.npc_location_overrides.clear();
        self.version = M8_NPC_POSITION_SAVE_VERSION;
    }

    pub(crate) fn migrate_v28_m8_choyin_justice(&mut self) {
        self.choyin_justice = ChoyinJusticeState::Free;
        self.version = M8_CHOYIN_JUSTICE_SAVE_VERSION;
    }

    pub(crate) fn migrate_v29_m8_dynamic_quest(&mut self) {
        self.dynamic_quest = None;
        self.dynamic_quest_finished = 0;
        self.version = M8_DYNAMIC_QUEST_SAVE_VERSION;
    }

    pub(crate) fn migrate_v30_m8_finalization(&mut self) {
        self.player.force_factor = self.player.force_factor.min(self.force_factor_limit());
        self.player.mana_factor = self.player.mana_factor.min(self.mana_factor_limit());
        self.player.wimpy_percent = self.player.wimpy_percent.min(80);
        self.stolen_npc_items.retain(|entry| {
            world().contains(&entry.location) && npcs().definition(&entry.npc).is_some()
        });
        let now = self.elapsed_minutes;
        self.zombie_haunts.retain(|entry| {
            npcs().definition(&entry.zombie).is_some()
                && npcs().definition(&entry.target).is_some()
                && entry.expires_at_elapsed_minutes > now
        });
        self.npc_respawns.retain(|entry| {
            world().contains(&entry.location)
                && Self::npc_can_roam_safely(&entry.npc)
                && entry.due_elapsed_minutes > now
        });
        self.initialize_m8_temporary_item_expirations();
        self.version = M8_FINALIZATION_SAVE_VERSION;
    }

    pub fn push_log(&mut self, message: String) {
        self.logs.push(message);
        if self.logs.len() > LOG_LIMIT {
            self.logs.drain(0..self.logs.len() - LOG_LIMIT);
        }
    }
}

impl EnemyKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bandit => "松林山贼",
            Self::XiaoJuan => "小娟",
            Self::OldLiuRevenge => "悲愤欲绝的刘老农",
            Self::Wolf => "灰背野狼",
            Self::TempleDisciple => "护院武僧",
            Self::Rat => "大老鼠",
            Self::IceDragon => "白鳞冰龙",
            Self::Meloner => "愤怒的瓜农",
            Self::BloodHandLiuSan => "血手刘三",
            Self::Npc(npc) => npc.name(),
        }
    }

    fn max_health(&self) -> i32 {
        match self {
            Self::Bandit => 65,
            Self::XiaoJuan => 1,
            Self::OldLiuRevenge => 500,
            Self::Wolf => 48,
            Self::TempleDisciple => 95,
            Self::Rat => 22,
            Self::IceDragon => 420,
            Self::Meloner => 90,
            Self::BloodHandLiuSan => 150,
            Self::Npc(npc) => npcs()
                .definition(npc)
                .map_or(43, |definition| definition.combat_max_health()),
        }
    }

    fn attack(&self) -> i32 {
        match self {
            Self::Bandit => 13,
            Self::XiaoJuan => 1,
            Self::OldLiuRevenge => 70,
            Self::Wolf => 11,
            Self::TempleDisciple => 17,
            Self::Rat => 7,
            Self::IceDragon => 42,
            Self::Meloner => 16,
            Self::BloodHandLiuSan => 22,
            Self::Npc(npc) => npcs()
                .definition(npc)
                .map_or(8, |definition| definition.combat_attack()),
        }
    }

    fn defense(&self) -> i32 {
        match self {
            Self::Bandit => 7,
            Self::XiaoJuan => 0,
            Self::OldLiuRevenge => 45,
            Self::Wolf => 5,
            Self::TempleDisciple => 12,
            Self::Rat => 2,
            Self::IceDragon => 30,
            Self::Meloner => 11,
            Self::BloodHandLiuSan => 15,
            Self::Npc(npc) => npcs()
                .definition(npc)
                .map_or(4, |definition| definition.combat_defense()),
        }
    }

    fn insight_reward(&self) -> u32 {
        match self {
            Self::Bandit => 15,
            Self::XiaoJuan => 0,
            Self::OldLiuRevenge => 80,
            Self::Wolf => 8,
            Self::TempleDisciple => 24,
            Self::Rat => 3,
            Self::IceDragon => 80,
            Self::Meloner => 8,
            Self::BloodHandLiuSan => 30,
            Self::Npc(npc) => npcs().definition(npc).map_or(2, |definition| {
                (definition.combat_rating() / 2).max(2) as u32
            }),
        }
    }

    fn reputation_reward(&self) -> i32 {
        match self {
            Self::Bandit => 4,
            Self::XiaoJuan => -500,
            Self::OldLiuRevenge => -20,
            Self::Wolf => 1,
            Self::TempleDisciple => 2,
            Self::Rat | Self::Npc(_) => 0,
            Self::IceDragon => 12,
            Self::Meloner => -8,
            Self::BloodHandLiuSan => 8,
        }
    }

    fn damage_resource(&self) -> CombatResource {
        match self {
            Self::TempleDisciple => CombatResource::Qi,
            Self::IceDragon => CombatResource::Spirit,
            Self::Npc(npc)
                if npcs()
                    .definition(npc)
                    .is_some_and(|definition| definition.attacks_spirit()) =>
            {
                CombatResource::Spirit
            }
            _ => CombatResource::Essence,
        }
    }

    fn wanted_reward(&self) -> u32 {
        match self {
            Self::XiaoJuan => 5,
            Self::OldLiuRevenge => 3,
            Self::TempleDisciple | Self::Meloner | Self::Npc(_) => 1,
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
pub struct SourceDoor {
    pub direction: String,
    pub name: String,
    pub reverse_direction: String,
    pub initially_closed: bool,
}

#[derive(Debug, Clone)]
pub struct RoomDetail {
    pub key: String,
    pub description: Option<String>,
    pub door_direction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RoomItemPlacement {
    pub item_id: ItemId,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub zone: String,
    pub description: String,
    pub arrival: String,
    pub outdoors: Option<String>,
    pub exits: Vec<Exit>,
    pub doors: Vec<SourceDoor>,
    pub details: Vec<RoomDetail>,
    pub npcs: Vec<NpcId>,
    pub room_items: Vec<RoomItemPlacement>,
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
            outdoors: None,
            exits,
            doors: Vec::new(),
            details: Vec::new(),
            npcs: npc.into_iter().collect(),
            room_items: Vec::new(),
            training,
            can_rest,
            enemy,
            source_path: None,
            object_sources: Vec::new(),
            behavior_flags: Vec::new(),
        }
    }
}

fn choyin_taolin_clue(index: u8) -> (&'static str, &'static str) {
    const CLUES: [(&str, &str); 11] = [
        ("欲将愁心附明月，随君直到夜郎--", "west"),
        ("问君能有几多愁，恰似一江春水向--流", "east"),
        ("自笑堂堂汉使，得似洋洋河水，依旧只流--", "east"),
        ("--朝四百八十寺，多少楼台烟雨中", "south"),
        ("孔雀--飞，五里一徘徊", "southeast"),
        ("帘卷--风，人比黄花瘦", "west"),
        ("醉别--楼醒不记，春梦秋云，聚散真容易", "west"),
        ("春草绿色，春水碧波，送君--浦，伤之如何", "south"),
        ("--望，射天狼", "northwest"),
        ("--风卷地白草折，胡天八月即飞雪", "north"),
        ("青山横--郭，白水绕东城", "north"),
    ];
    CLUES[usize::from(index) % CLUES.len()]
}

fn npc_fight_rejection(npc: &NpcId, player: &Player) -> String {
    match npc.as_str() {
        SNOW_FIST_TRAINER_ID => "李火狮说道：馆主吩咐过，不许和来这里的客人过招。".into(),
        SNOW_GIRL_ID if player.faction.as_deref() == Some("封山剑派") => match player.gender {
            Gender::Female => "柳绘心说道：师姐，别整天想着练功，我们去花园摘花玩吧。".into(),
            Gender::Male => "柳绘心说道：我才不要，你去找李教头练吧！".into(),
        },
        SNOW_GIRL_ID => "柳绘心说道：爹爹说过，不能跟江湖人物比武过招。".into(),
        "snow.npc.beggar" | SNOW_SCAVENGER_ID => {
            format!("{}说道：少侠饶命！小的这就离开！", npc.name())
        }
        TEMPLE_OLD_TAOIST_ID => "老道士说道：无量寿佛！贫道年迈力衰，怎是施主的对手。".into(),
        TEMPLE_PROTECTOR_ID | TEMPLE_TRAINER_ID => "对方说道：茅山派不和别派的人过招。".into(),
        CHOYIN_HOTEL_GUARD_ID => "酒楼守卫说道：掌柜的有交代，不准任何人在这里打架！".into(),
        CHOYIN_MAGISTRATE_ID => "程不平说道：这是衙门，快回去吧。".into(),
        _ => format!("{}不愿与你比试。", npc.name()),
    }
}

fn canonical_location_pair(source: &LocationId, target: &LocationId) -> (LocationId, LocationId) {
    if source.as_str() <= target.as_str() {
        (source.clone(), target.clone())
    } else {
        (target.clone(), source.clone())
    }
}

fn source_door_pair(
    source: &LocationId,
    target: &LocationId,
) -> Option<(&'static SourceDoor, &'static SourceDoor)> {
    if door_for_transition(source, target).is_some() {
        return None;
    }
    let source_location = world().location(source)?;
    let source_exit = source_location
        .exits
        .iter()
        .find(|exit| &exit.target == target)?;
    let door = source_location
        .doors
        .iter()
        .find(|door| door.direction == source_exit.direction)?;
    let target_location = world().location(target)?;
    let target_exit = target_location
        .exits
        .iter()
        .find(|exit| &exit.target == source && exit.direction == door.reverse_direction)?;
    let reverse = target_location.doors.iter().find(|candidate| {
        candidate.direction == target_exit.direction
            && candidate.reverse_direction == door.direction
    })?;
    Some((door, reverse))
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

const SNOW_TOWN_TEACHERS: [&str; 8] = [
    "assassin",
    "beggar",
    "fighter",
    "juechen",
    "lama",
    "ninja",
    "ronin",
    "swordsman",
];
const TEMPLE_TEACHERS: [&str; 1] = ["bonze"];
const CHOYIN_ENTRANCE_TEACHERS: [&str; 1] = ["scholar"];
const LATEMOON_TEACHERS: [&str; 1] = ["dancer"];

fn teachers_at_location(location: &str) -> &'static [&'static str] {
    match location {
        content::SNOW_TOWN => &SNOW_TOWN_TEACHERS,
        content::TEMPLE_YARD => &TEMPLE_TEACHERS,
        "choyin.entrance" => &CHOYIN_ENTRANCE_TEACHERS,
        "latemoon.latemoon1" => &LATEMOON_TEACHERS,
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
    use std::collections::HashSet;

    use crate::npcs::{
        CHOYIN_OLD_MAN_ID, CLOUD_BOATER_ID, CLOUD_GANGSTER_ID, CLOUD_GIRL_ID, CLOUD_JUDGE_ID,
        CLOUD_MONK_ID, LATEMOON_FUNLIN_ID, LATEMOON_OLD_ID, LATEMOON_SHAOWEI_ID,
        LATEMOON_SHINFUN_ID, SNOW_TEACHER_ID,
    };

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
    fn m4_source_doors_pair_all_valid_endpoints_and_persist_shared_state() {
        const PAIRS: [(&str, &str); 14] = [
            ("city.shangshu.gate", "city.shangshu.yuan"),
            ("city.shangshu.huayuan", "city.shangshu.neizhai"),
            ("city.shangshu.kefang", "city.shangshu.road2"),
            ("city.shangshu.road1", "city.shangshu.xiaowu"),
            ("snow.e_room", "snow.inn_2f"),
            ("snow.hockshop", "snow.mstreet3"),
            ("snow.inn_2f", "snow.n_room"),
            ("snow.inn_2f", "snow.w_room"),
            ("snow.school1", "snow.school2"),
            ("temple.corridor3", "temple.restroom2"),
            ("temple.corridor4", "temple.trainroom"),
            ("temple.corridor5", "temple.temple2"),
            ("temple.corridor7", "temple.restroom1"),
            ("temple.square", "temple.temple1"),
        ];

        let has_door_action = |game: &Game, target: &str| {
            game.available_actions().iter().any(|action| match action {
                Action::OpenSourceDoor {
                    target: action_target,
                }
                | Action::CloseSourceDoor {
                    target: action_target,
                } => action_target.as_str() == target,
                _ => false,
            })
        };
        let mut game = Game::new();
        for (first, second) in PAIRS {
            game.location = LocationId::from(first);
            assert!(
                has_door_action(&game, second),
                "missing door {first} -> {second}"
            );
            game.location = LocationId::from(second);
            assert!(
                has_door_action(&game, first),
                "missing door {second} -> {first}"
            );
        }

        game.location = LocationId::from("city.shangshu.road1");
        assert!(game.available_actions().contains(&Action::OpenSourceDoor {
            target: LocationId::from("city.shangshu.xiaowu")
        }));
        game.perform(Action::OpenSourceDoor {
            target: LocationId::from("city.shangshu.xiaowu"),
        });
        assert!(game.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "city.shangshu.xiaowu"
        )));

        game.location = LocationId::from("city.shangshu.xiaowu");
        game.perform(Action::CloseSourceDoor {
            target: LocationId::from("city.shangshu.road1"),
        });
        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(
            restored
                .available_actions()
                .contains(&Action::OpenSourceDoor {
                    target: LocationId::from("city.shangshu.road1")
                })
        );
        assert!(restored.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "city.shangshu.road1"
        )));

        game.location = LocationId::from("snow.inn");
        assert!(game.available_actions().iter().all(|action| !matches!(
            action,
            Action::OpenSourceDoor { target } | Action::CloseSourceDoor { target }
                if target.as_str() == "wiz.entrance"
        )));
    }

    #[test]
    fn fixed_source_room_items_spawn_once_and_remain_pickable() {
        let mut game = Game::new();
        assert_eq!(game.ground_items.values().flatten().count(), 38);
        assert_eq!(
            game.ground_items
                .iter()
                .filter(|(location, _)| {
                    ["oldpine", "goathill", "choyin"]
                        .iter()
                        .any(|area| location.as_str().starts_with(&format!("{area}.")))
                })
                .flat_map(|(_, items)| items)
                .count(),
            10
        );
        let mut instance_ids = game
            .ground_items
            .values()
            .flatten()
            .map(|item| item.instance_id)
            .collect::<Vec<_>>();
        instance_ids.sort_unstable();
        instance_ids.dedup();
        assert_eq!(instance_ids.len(), 38);

        game.location = LocationId::from("snow.temple");
        let paper = game.ground_items[&game.location]
            .iter()
            .find(|item| item.item_id.as_str() == "obj.paper_seal")
            .unwrap()
            .instance_id;
        game.perform(Action::PickUpItem(paper));
        assert_eq!(
            game.ground_items[&game.location]
                .iter()
                .filter(|item| item.item_id.as_str() == "obj.paper_seal")
                .count(),
            1
        );
        assert!(
            game.player
                .inventory
                .iter()
                .any(|item| item.item_id.as_str() == "obj.paper_seal")
        );

        assert_eq!(
            game.ground_items
                .iter()
                .filter(|(location, _)| {
                    ["chuenyu", "green", "sanyen", "waterfog"]
                        .iter()
                        .any(|area| location.as_str().starts_with(&format!("{area}.")))
                })
                .flat_map(|(_, items)| items)
                .count(),
            11
        );

        assert_eq!(
            game.ground_items
                .iter()
                .filter(|(location, _)| {
                    ["latemoon", "death", "graveyard", "jail", "u.cloud"]
                        .iter()
                        .any(|prefix| location.as_str().starts_with(&format!("{prefix}.")))
                })
                .flat_map(|(_, items)| items)
                .count(),
            12
        );

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert_eq!(restored.ground_items.values().flatten().count(), 37);
    }

    #[test]
    fn m4_source_room_details_are_actionable_and_doors_report_live_state() {
        let mut game = Game::new();
        let mut rooms = 0;
        let mut details = 0;
        for catalog in [
            include_str!("../migration/catalog/city.json"),
            include_str!("../migration/catalog/snow.json"),
            include_str!("../migration/catalog/temple.json"),
            include_str!("../migration/catalog/canyon.json"),
        ] {
            let catalog: serde_json::Value = serde_json::from_str(catalog).unwrap();
            for room in catalog["rooms"].as_array().unwrap().iter().filter(|room| {
                room["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|flag| flag == "item_interaction")
            }) {
                let location = world()
                    .location(&LocationId::from(room["id"].as_str().unwrap()))
                    .unwrap();
                rooms += 1;
                details += location.details.len();
                game.location = location.id.clone();
                let actions = game.available_actions();
                assert_eq!(
                    actions
                        .iter()
                        .filter(|action| matches!(action, Action::InspectRoomDetail(_)))
                        .count(),
                    location.details.len(),
                    "{}",
                    location.id.as_str()
                );
            }
        }
        assert_eq!(rooms, 32);
        assert_eq!(details, 42);

        game.location = LocationId::from("city.bank");
        game.perform(Action::InspectRoomDetail("sign".into()));
        assert!(game.logs.last().unwrap().contains("convert"));

        game.location = LocationId::from("city.shangshu.kefang");
        game.perform(Action::InspectRoomDetail("book".into()));
        assert!(game.logs.last().unwrap().contains("诗词集"));

        game.location = LocationId::from("snow.school1");
        game.perform(Action::InspectRoomDetail("door".into()));
        assert!(game.logs.last().unwrap().contains("关着"));
        game.perform(Action::OpenSourceDoor {
            target: LocationId::from("snow.school2"),
        });
        game.perform(Action::InspectRoomDetail("door".into()));
        assert!(game.logs.last().unwrap().contains("开着"));
        game.perform(Action::CloseSourceDoor {
            target: LocationId::from("snow.school2"),
        });
        game.perform(Action::InspectRoomDetail("door".into()));
        assert!(game.logs.last().unwrap().contains("关着"));
    }

    #[test]
    fn m5_source_doors_and_room_details_use_the_generic_runtime() {
        let mut game = Game::new();
        game.location = LocationId::from("choyin.yamen");
        assert!(game.available_actions().contains(&Action::OpenSourceDoor {
            target: LocationId::from("choyin.yamen_yard"),
        }));
        assert!(game.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "choyin.yamen_yard"
        )));
        game.perform(Action::OpenSourceDoor {
            target: LocationId::from("choyin.yamen_yard"),
        });
        assert!(game.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "choyin.yamen_yard"
        )));
        game.location = LocationId::from("choyin.yamen_yard");
        game.perform(Action::CloseSourceDoor {
            target: LocationId::from("choyin.yamen"),
        });
        assert!(game.available_actions().contains(&Action::OpenSourceDoor {
            target: LocationId::from("choyin.yamen"),
        }));

        for room_id in ["choyin.club", "choyin.fence"] {
            game.location = LocationId::from(room_id);
            assert!(game.available_actions().iter().all(|action| !matches!(
                action,
                Action::OpenSourceDoor { .. } | Action::CloseSourceDoor { .. }
            )));
        }

        let mut rooms = 0;
        let mut details = 0;
        for catalog in [
            include_str!("../migration/catalog/oldpine.json"),
            include_str!("../migration/catalog/goathill.json"),
            include_str!("../migration/catalog/choyin.json"),
        ] {
            let catalog: serde_json::Value = serde_json::from_str(catalog).unwrap();
            for room in catalog["rooms"].as_array().unwrap().iter().filter(|room| {
                room["behavior_flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|flag| flag == "item_interaction")
            }) {
                let location = world()
                    .location(&LocationId::from(room["id"].as_str().unwrap()))
                    .unwrap();
                rooms += 1;
                details += location.details.len();
                game.location = location.id.clone();
                assert_eq!(
                    game.available_actions()
                        .iter()
                        .filter(|action| matches!(action, Action::InspectRoomDetail(_)))
                        .count(),
                    location.details.len(),
                    "{}",
                    location.id.as_str()
                );
            }
        }
        assert_eq!(rooms, 19);
        assert_eq!(details, 25);

        game.location = LocationId::from("oldpine.cave5");
        game.perform(Action::InspectRoomDetail("wall".into()));
        assert!(game.logs.last().unwrap().contains("水烟阁"));
        game.location = LocationId::from("choyin.taolin");
        game.perform(Action::InspectRoomDetail("note".into()));
        assert!(game.logs.last().unwrap().contains("路径"));
    }

    #[test]
    fn m5_oldpine_scripted_routes_and_random_mazes_are_single_player_safe() {
        let game = Game::new();
        for (source, targets) in [
            ("oldpine.clearing", &["oldpine.tree1"][..]),
            (
                "oldpine.cliff1",
                &["oldpine.cliffside", "oldpine.riverbank1"][..],
            ),
            (
                "oldpine.cliff2",
                &["oldpine.cliffdown", "oldpine.epath3"][..],
            ),
            ("oldpine.cliffdown", &["oldpine.cliff2"][..]),
            ("oldpine.path3", &["oldpine.stone"][..]),
            ("oldpine.riverbank1", &["oldpine.cliff1"][..]),
            ("oldpine.stone", &["oldpine.cave1"][..]),
        ] {
            let mut at_source = game.clone();
            at_source.location = LocationId::from(source);
            let actions = at_source.available_actions();
            for target in targets {
                assert!(
                    actions.iter().any(|action| matches!(
                        action,
                        Action::Move { target: action_target, .. }
                            if action_target.as_str() == *target
                    )),
                    "{source} -> {target}"
                );
            }
        }

        let mut cave = Game::new();
        cave.location = LocationId::from("oldpine.cave3");
        let actions = cave.available_actions();
        assert_eq!(
            actions
                .iter()
                .filter(|action| matches!(action, Action::Move { .. }))
                .count(),
            4
        );
        assert!(actions.iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::OLD_PINE_CAVE_PREFIX
        )));
        assert!(
            actions
                .iter()
                .filter_map(|action| match action {
                    Action::Move { target, .. } => Some(target.as_str()),
                    _ => None,
                })
                .all(|target| matches!(
                    target,
                    "oldpine.cave1" | "oldpine.cave2" | "oldpine.cave3" | "oldpine.cave4"
                ))
        );
        let restored: Game = serde_json::from_str(&serde_json::to_string(&cave).unwrap()).unwrap();
        assert_eq!(restored.available_actions(), actions);
        let maze_move = actions
            .into_iter()
            .find(|action| matches!(action, Action::Move { .. }))
            .unwrap();
        let rng_before = cave.rng_state;
        cave.perform(maze_move);
        assert_ne!(cave.rng_state, rng_before);
        assert!(
            cave.location
                .as_str()
                .starts_with(content::OLD_PINE_CAVE_PREFIX)
        );

        let mut pine = Game::new();
        pine.location = LocationId::from("oldpine.pine3");
        let pine_actions = pine.available_actions();
        assert_eq!(
            pine_actions
                .iter()
                .filter(|action| matches!(action, Action::Move { .. }))
                .count(),
            4
        );
        assert!(pine_actions.iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::OLD_PINE_FOREST_PREFIX
        )));

        let mut vine = Game::new();
        vine.location = LocationId::from("oldpine.epath2");
        vine.perform(Action::Interact(InteractionKind::HoldOldPineVine));
        assert_eq!(vine.location.as_str(), "oldpine.waterfall");
    }

    #[test]
    fn m5_choyin_cliff_platform_well_and_lion_events_are_actionable() {
        let mut cliff = Game::new();
        cliff.location = LocationId::from("choyin.guyehill");
        cliff.perform(Action::Interact(InteractionKind::ClimbChoyinTree));
        assert_eq!(cliff.location.as_str(), "choyin.craneroom");
        cliff.location = LocationId::from("choyin.guyehill");
        cliff.perform(Action::Interact(InteractionKind::HoldChoyinVine));
        assert_eq!(cliff.location.as_str(), "choyin.hollow");

        let mut platform = Game::new();
        platform.location = LocationId::from("choyin.platform");
        assert!(platform.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "choyin.stove"
        )));
        platform.perform(Action::Interact(InteractionKind::TouchChoyinCloudFlag));
        let down = platform
            .available_actions()
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Move { target, .. } if target.as_str() == "choyin.stove"
                )
            })
            .unwrap();
        let restored: Game =
            serde_json::from_str(&serde_json::to_string(&platform).unwrap()).unwrap();
        assert!(restored.available_actions().contains(&down));
        let essence_before = platform.player.essence;
        platform.tick();
        platform.tick();
        assert!(platform.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "choyin.stove"
        )));
        platform.tick();
        assert!(platform.player.essence < essence_before);

        let mut escape = Game::new();
        escape.location = LocationId::from("choyin.platform");
        escape.perform(Action::Interact(InteractionKind::TouchChoyinCloudFlag));
        let down = escape
            .available_actions()
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Move { target, .. } if target.as_str() == "choyin.stove"
                )
            })
            .unwrap();
        escape.perform(down);
        assert_eq!(escape.location.as_str(), "choyin.stove");
        assert!(escape.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "choyin.tongbhill"
        )));

        let mut well = Game::new();
        well.location = LocationId::from("choyin.s_street1");
        well.player.water = 10;
        well.perform(Action::Interact(InteractionKind::DrinkChoyinWell));
        assert_eq!(well.player.water, 30);

        let mut lion = Game::new();
        lion.location = LocationId::from("choyin.w_street1");
        lion.player.strength = 50;
        lion.perform(Action::Interact(InteractionKind::LiftChoyinStoneLion));
        assert_eq!(lion.location.as_str(), "choyin.lionroom");
    }

    #[test]
    fn m5_oldpine_and_choyin_room_events_preserve_source_state_machines() {
        let mut burial = Game::new();
        burial.location = LocationId::from("oldpine.cave5");
        burial.player.spirituality = 0;
        assert!(burial.current_ground_has("oldpine.npc.skeleton"));
        burial.perform(Action::Interact(InteractionKind::BuryOldPineSkeleton));
        assert!(!burial.current_ground_has("oldpine.npc.skeleton"));
        assert_eq!(burial.location.as_str(), "oldpine.waterfall");

        let mut keep = Game::new();
        keep.location = LocationId::from("oldpine.keep2");
        let east = keep
            .available_actions()
            .into_iter()
            .find(|action| matches!(action, Action::Move { direction, .. } if direction == "east"))
            .unwrap();
        keep.perform(east);
        keep.location = LocationId::from("oldpine.keep2");
        assert!(keep.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { direction, .. } if direction == "west"
        )));
        keep.add_inventory_item(ItemId::from("oldpine.obj.bamboo_pipe"), 1);
        keep.perform(Action::Interact(InteractionKind::BlowOldPineBambooPipe));
        assert!(keep.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { direction, .. } if direction == "west"
        )));

        let mut books = Game::new();
        books.location = LocationId::from("choyin.club");
        books.perform(Action::Interact(InteractionKind::BorrowChoyinBook));
        assert!(books.player.inventory.iter().any(|item| matches!(
            item.item_id.as_str(),
            "choyin.npc.obj.book1" | "choyin.npc.obj.book2"
        )));
        books.move_to(LocationId::from("choyin.fence"));
        assert!(books.player.inventory.iter().all(|item| !matches!(
            item.item_id.as_str(),
            "choyin.npc.obj.book1" | "choyin.npc.obj.book2"
        )));

        let mut peach = Game::new();
        peach.location = LocationId::from("choyin.entrance");
        assert!(peach.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { direction, .. } if direction == "east"
        )));
        let trial = Action::BecomeApprentice("scholar".into());
        assert!(peach.available_actions().contains(&trial));
        peach.perform(trial.clone());
        assert!(peach.choyin_scholar_trial_started);
        assert!(peach.player.teacher.is_none());
        let enter = peach
            .available_actions()
            .into_iter()
            .find(|action| matches!(action, Action::Move { direction, .. } if direction == "east"))
            .unwrap();
        peach.perform(enter);
        assert_eq!(peach.choyin_taolin_steps, 3);
        peach.perform(Action::Interact(InteractionKind::ReadChoyinPeachNote));
        assert!(peach.logs.last().unwrap().contains("--"));
        for _ in 0..3 {
            let direction = choyin_taolin_clue(peach.choyin_taolin_clue).1;
            let step = peach
                .available_actions()
                .into_iter()
                .find(|action| {
                    matches!(action, Action::Move { direction: candidate, .. } if candidate == direction)
                })
                .unwrap();
            peach.perform(step);
        }
        assert_eq!(peach.location.as_str(), "choyin.entrance");
        assert_eq!(peach.choyin_taolin_steps, 0);
        assert!(peach.choyin_scholar_trial_completed);
        peach.perform(trial);
        assert_eq!(peach.player.teacher.as_deref(), Some("scholar"));
        assert_eq!(peach.player.faction.as_deref(), Some("步玄派"));
        assert!(!peach.choyin_scholar_trial_started);
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
    fn m5_scripted_inquiries_and_exchanges_preserve_source_rewards() {
        let mut game = Game::new();

        game.location = LocationId::from("choyin.n_street1");
        game.perform(Action::AskNpc {
            npc: NpcId::from("choyin.npc.cake_vendor"),
            topic: "大饼".into(),
        });
        assert!(game.logs.last().unwrap().contains("摆出货单"));

        game.location = LocationId::from("choyin.yamen");
        game.perform(Action::AskNpc {
            npc: NpcId::from(CHOYIN_POLICE_ID),
            topic: "bribery".into(),
        });
        assert!(game.logs.last().unwrap().contains("收起你的钱"));

        game.location = LocationId::from("choyin.bridge4");
        let ask_for_bag = Action::AskNpc {
            npc: NpcId::from(CHOYIN_GIRL_ID),
            topic: "游晋".into(),
        };
        assert!(game.available_actions().contains(&ask_for_bag));
        game.perform(ask_for_bag.clone());
        assert!(game.player.has_item(&ItemId::from(CHOYIN_SILK_BAG_ID)));
        assert!(!game.available_actions().contains(&ask_for_bag));

        game.location = LocationId::from("choyin.hotel2");
        let ask_about_trouble = Action::AskNpc {
            npc: NpcId::from(CHOYIN_YOUNG_MAN_ID),
            topic: "trouble".into(),
        };
        game.perform(ask_about_trouble.clone());
        assert!(game.logs.last().unwrap().contains("唉"));
        let bag = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == CHOYIN_SILK_BAG_ID)
            .unwrap()
            .instance_id;
        game.perform(Action::GiveItem {
            instance_id: bag,
            npc: NpcId::from(CHOYIN_YOUNG_MAN_ID),
        });
        assert!(game.player.item(bag).is_none());
        assert!(game.choyin_silk_bag_delivered);
        assert!(!game.available_actions().contains(&ask_about_trouble));

        game.location = LocationId::from("choyin.tomb3");
        let chest = game.ground_items[&game.location]
            .iter()
            .find(|item| CHOYIN_PEACH_CHEST_IDS.contains(&item.item_id.as_str()))
            .unwrap()
            .instance_id;
        game.perform(Action::PickUpItem(chest));
        game.location = LocationId::from("choyin.hotel1");
        game.perform(Action::GiveItem {
            instance_id: chest,
            npc: NpcId::from("choyin.npc.sergeant"),
        });
        assert!(game.player.item(chest).is_none());
        assert!(game.choyin_chest_rewarded);
        assert!(game.player.has_item(&ItemId::from(CHOYIN_MAGIC_BOOK_ID)));
    }

    #[test]
    fn m5_rope_and_tablet_actions_preserve_choyin_item_scripts() {
        assert_eq!(
            items()
                .definition(&ItemId::from("choyin.npc.obj.book1"))
                .unwrap()
                .display_name(),
            "「笑傲江湖」"
        );

        let mut altar = Game::new();
        altar.location = LocationId::from("choyin.altar");
        let box_instance = altar.ground_items[&altar.location]
            .iter()
            .find(|item| CHOYIN_DONATION_BOX_IDS.contains(&item.item_id.as_str()))
            .unwrap()
            .instance_id;
        assert!(
            !altar
                .available_actions()
                .contains(&Action::PickUpItem(box_instance))
        );
        let offering = altar
            .player
            .inventory
            .iter()
            .find(|item| item.unit_value() > 0 && !altar.player.is_equipped(item.instance_id))
            .unwrap()
            .instance_id;
        let donate = Action::DonateItem(offering);
        assert!(altar.available_actions().contains(&donate));
        altar.perform(donate);
        assert!(altar.player.item(offering).is_none());
        assert!(altar.current_ground_has("choyin.obj.denotation"));

        let mut rope = Game::new();
        rope.location = LocationId::from("choyin.halfhole");
        let rope_instance = rope.ground_items[&rope.location]
            .iter()
            .find(|item| item.item_id.as_str() == CHOYIN_GOLDEN_ROPE_ID)
            .unwrap()
            .instance_id;
        rope.perform(Action::PickUpItem(rope_instance));

        rope.location = LocationId::from("village.road2");
        let sell = Action::SellItem(rope_instance);
        assert!(!rope.available_actions().contains(&sell));
        rope.perform(sell);
        assert!(rope.player.item(rope_instance).is_some());

        rope.location = LocationId::from("snow.school");
        let give = Action::GiveItem {
            instance_id: rope_instance,
            npc: NpcId::from(SNOW_TEACHER_ID),
        };
        assert!(!rope.available_actions().contains(&give));
        rope.perform(give);
        assert!(rope.player.item(rope_instance).is_some());

        rope.location = LocationId::from("choyin.altar");
        let donate = Action::DonateItem(rope_instance);
        assert!(!rope.available_actions().contains(&donate));
        rope.perform(donate);
        assert!(rope.player.item(rope_instance).is_some());

        rope.location = LocationId::from("choyin.craneroom");
        let drop = Action::DropItem(rope_instance);
        assert!(!rope.available_actions().contains(&drop));
        rope.perform(drop);
        assert!(rope.player.item(rope_instance).is_some());
        let spirit_before = rope.player.spirit;
        rope.perform(Action::Interact(InteractionKind::TieChoyinCrane));
        assert_eq!(rope.location.as_str(), "choyin.platform");
        assert_eq!(rope.player.spirit, (spirit_before - 50).max(0));
        assert!(rope.player.has_item(&ItemId::from(CHOYIN_GOLDEN_ROPE_ID)));

        let mut tablet = Game::new();
        tablet.location = LocationId::from("choyin.stove");
        let tablet_instance = tablet.ground_items[&tablet.location]
            .iter()
            .find(|item| item.item_id.as_str() == CHOYIN_TABLET_ID)
            .unwrap()
            .instance_id;
        tablet.perform(Action::PickUpItem(tablet_instance));
        tablet.player.essence = 40;
        tablet.player.qi = 30;
        tablet.player.spirit = 20;
        let consume = Action::ConsumeItem(tablet_instance);
        assert!(tablet.available_actions().contains(&consume));
        tablet.perform(consume);
        assert_eq!(tablet.player.essence, 45);
        assert_eq!(tablet.player.qi, 60);
        assert_eq!(tablet.player.spirit, 25);
        assert!(tablet.player.item(tablet_instance).is_none());
    }

    #[test]
    fn m6_source_doors_details_and_spider_web_use_generic_runtime() {
        let catalogs: [serde_json::Value; 4] = [
            serde_json::from_str(include_str!("../migration/catalog/chuenyu.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/green.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/sanyen.json")).unwrap(),
            serde_json::from_str(include_str!("../migration/catalog/waterfog.json")).unwrap(),
        ];
        let mut door_entries = 0;
        let mut detail_rooms = 0;
        let mut details = 0;
        for catalog in catalogs {
            for room in catalog["rooms"].as_array().unwrap() {
                let flags = room["behavior_flags"].as_array().unwrap();
                let mut game = Game::new();
                game.location = LocationId::from(room["id"].as_str().unwrap());
                let location = game.current_location();
                if flags.iter().any(|flag| flag == "door") {
                    door_entries += location.doors.len();
                    let actions = game.available_actions();
                    for door in &location.doors {
                        let target = &location
                            .exits
                            .iter()
                            .find(|exit| exit.direction == door.direction)
                            .unwrap()
                            .target;
                        assert!(actions.iter().any(|action| matches!(
                            action,
                            Action::OpenSourceDoor { target: action_target }
                                | Action::CloseSourceDoor { target: action_target }
                                if action_target == target
                        )));
                    }
                }
                if flags.iter().any(|flag| flag == "item_interaction") {
                    detail_rooms += 1;
                    details += location.details.len();
                    assert_eq!(
                        game.available_actions()
                            .iter()
                            .filter(|action| matches!(action, Action::InspectRoomDetail(_)))
                            .count(),
                        location.details.len()
                    );
                }
            }
        }
        assert_eq!(door_entries, 16);
        assert_eq!(detail_rooms, 22);
        assert_eq!(details, 38);

        let mut web = Game::new();
        web.location = LocationId::from("green.house3");
        for _ in 0..4 {
            web.perform(Action::InspectRoomDetail("web".into()));
        }
        assert_eq!(
            web.spawned_npc_instances
                .iter()
                .filter(|entry| entry.location.as_str() == "green.house3"
                    && entry.npc.as_str() == GREEN_SPIDER_ID)
                .count(),
            3
        );
        assert!(
            web.available_actions()
                .contains(&Action::Kill(EnemyKind::Npc(NpcId::from(GREEN_SPIDER_ID))))
        );
    }

    #[test]
    fn m6_room_events_items_and_rewards_preserve_source_state_machines() {
        let mut hall = Game::new();
        hall.location = LocationId::from("chuenyu.center");
        hall.rng_state = 0;
        let qi_before = hall.player.qi;
        hall.perform(Action::Interact(InteractionKind::PullChuenyuHallRope));
        assert_eq!(hall.location.as_str(), "chuenyu.tunnel1");
        assert_eq!(hall.player.qi, qi_before - 9);

        for (source, target) in [
            ("chuenyu.east_castle", "chuenyu.east_garden"),
            ("chuenyu.east_garden", "chuenyu.east_castle"),
            ("chuenyu.west_castle", "chuenyu.west_garden"),
            ("chuenyu.west_garden", "chuenyu.west_castle"),
        ] {
            hall.location = LocationId::from(source);
            hall.perform(Action::Interact(InteractionKind::ClimbChuenyuCastleWall));
            assert_eq!(hall.location.as_str(), target);
        }
        hall.location = LocationId::from("chuenyu.rope_bridge");
        hall.perform(Action::Interact(InteractionKind::DescendChuenyuRopeBridge));
        assert_eq!(hall.location.as_str(), "chuenyu.base_b_m");

        hall.location = LocationId::from("chuenyu.tunnel4");
        for _ in 0..5 {
            hall.perform(Action::Interact(InteractionKind::PushChuenyuDungeonSlab));
        }
        assert_eq!(hall.chuenyu_slab_passage_ticks, 3);
        let climb = hall
            .available_actions()
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Move { target, .. } if target.as_str() == "chuenyu.east_castle"
                )
            })
            .unwrap();
        hall.perform(climb);
        assert_eq!(hall.location.as_str(), "chuenyu.east_castle");
        assert!(hall.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "chuenyu.tunnel4"
        )));
        for _ in 0..3 {
            hall.tick();
        }
        assert_eq!(hall.chuenyu_slab_passage_ticks, 0);

        hall.player.qi = 200;
        hall.player.max_qi = 200;
        hall.move_to(LocationId::from("chuenyu.trap_castle"));
        let before_arrows = hall.player.qi;
        hall.tick();
        assert_eq!(hall.player.qi, before_arrows);
        hall.tick();
        assert!(hall.player.qi < before_arrows);

        let mut green = Game::new();
        green.location = LocationId::from("green.entrance");
        green.player.combat_experience = 99_999;
        assert!(!green.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "green.eight0"
        )));
        green.player.combat_experience = 100_000;
        assert!(green.available_actions().iter().any(|action| matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "green.eight0"
        )));
        green.location = LocationId::from("green.eight7");
        green.perform(Action::Move {
            direction: "south".into(),
            target: LocationId::from("green.stoneroom"),
        });
        assert!(green.green_bagua_completed);
        green.location = LocationId::from("green.water");
        green.rng_state = 2;
        green.perform(Action::Interact(InteractionKind::SearchGreenStream));
        assert!(green.green_windsword_rewarded);
        assert!(green.player.has_item(&ItemId::from(GREEN_WIND_SWORD_ID)));

        green.location = LocationId::from("green.closed");
        green.perform(Action::Interact(InteractionKind::PushGreenBoulder));
        assert_eq!(green.location.as_str(), "green.closed");
        green.player.force = 560;
        green.player.max_force = 560;
        green
            .player
            .skills
            .iter_mut()
            .find(|skill| skill.kind.as_str() == FORCE_ID)
            .unwrap()
            .level = 40;
        green.rng_state = 2;
        green.perform(Action::Interact(InteractionKind::PushGreenBoulder));
        assert_eq!(green.location.as_str(), "green.entrance");

        green.location = LocationId::from("green.station0");
        let wineskin = green.add_inventory_item(ItemId::from("chuenyu.obj.qiwine"), 1);
        green.perform(Action::Interact(InteractionKind::FillGreenWell));
        let filled = green.player.item(wineskin).unwrap();
        assert_eq!(filled.remaining_uses, Some(15));
        assert!(filled.filled_with_water);
        green.player.water = 0;
        green.perform(Action::ConsumeItem(wineskin));
        assert_eq!(
            green.player.item(wineskin).unwrap().remaining_uses,
            Some(14)
        );
        assert!(green.player.condition(ConditionKind::Drunk).is_none());

        let mut kitchen = Game::new();
        kitchen.location = LocationId::from("sanyen.kitchen");
        assert!(
            !kitchen
                .available_actions()
                .contains(&Action::Interact(InteractionKind::TakeSanyenBun))
        );
        kitchen.win_combat(CombatState {
            enemy: EnemyKind::Npc(NpcId::from(SANYEN_COOK_ID)),
            health: 0,
            max_health: 1,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        });
        for _ in 0..5 {
            kitchen.perform(Action::Interact(InteractionKind::TakeSanyenBun));
        }
        assert_eq!(kitchen.sanyen_buns_taken, 5);
        assert_eq!(
            kitchen
                .player
                .inventory
                .iter()
                .filter(|item| item.item_id.as_str() == SANYEN_BUN_ID)
                .count(),
            5
        );
        assert!(
            !kitchen
                .available_actions()
                .contains(&Action::Interact(InteractionKind::TakeSanyenBun))
        );

        let pigmeat = kitchen.add_inventory_item(ItemId::from("chuenyu.obj.pigmeat"), 1);
        kitchen.player.food = 0;
        kitchen.player.max_food = 10_000;
        for _ in 0..6 {
            kitchen.perform(Action::ConsumeItem(pigmeat));
        }
        let bone = kitchen.player.item(pigmeat).unwrap();
        assert_eq!(bone.display_name(), "山猪骨头");
        assert_eq!(bone.total_weight(), 250);
        assert_eq!(bone.unit_value(), 60);
        assert_eq!(bone.remaining_uses, Some(0));
        assert_eq!(bone.definition().weapon_skill(), Some("hammer"));
        kitchen.perform(Action::EquipItem(pigmeat));
        assert!(kitchen.player.is_equipped(pigmeat));

        let restored: Game =
            serde_json::from_str(&serde_json::to_string(&kitchen).unwrap()).unwrap();
        assert_eq!(restored.sanyen_buns_taken, 5);
        let restored_bone = restored.player.item(pigmeat).unwrap();
        assert_eq!(restored_bone.display_name(), "山猪骨头");
        assert_eq!(restored_bone.unit_value(), 60);
    }

    #[test]
    fn m8_offline_time_is_capped_passive_and_rejects_clock_rollback() {
        let mut game = Game::new();
        let start_minutes = game.elapsed_minutes;
        let start_food = game.player.food;
        let start_water = game.player.water;
        let skill = SkillId::from(FORCE_ID);
        let progress_before = game.player.skill_by_id(FORCE_ID).unwrap().progress;
        game.activity = Activity::Training(skill.clone());
        game.player.set_condition(ConditionKind::Bandaged, 40, 3);
        game.last_saved_at_unix_seconds = Some(1_000);

        game.advance_offline_progress(1_000 + 12 * 60 * 60, Some(1_000));

        assert_eq!(
            game.elapsed_minutes,
            start_minutes + MAX_OFFLINE_TICKS * GAME_MINUTES_PER_TICK
        );
        assert_eq!(
            game.player.food,
            start_food.saturating_sub(MAX_OFFLINE_TICKS as i32)
        );
        assert_eq!(
            game.player.water,
            start_water.saturating_sub(MAX_OFFLINE_TICKS as i32)
        );
        assert_eq!(game.activity, Activity::Training(skill));
        assert_eq!(
            game.player.skill_by_id(FORCE_ID).unwrap().progress,
            progress_before
        );
        assert_eq!(
            game.player
                .condition(ConditionKind::Bandaged)
                .unwrap()
                .duration,
            40
        );
        assert!(game.logs.last().unwrap().contains("最多结算 8 小时"));
        assert_eq!(game.time_period_text(), "黄昏");

        let advanced_minutes = game.elapsed_minutes;
        game.last_saved_at_unix_seconds = Some(20_000);
        game.advance_offline_progress(19_999, None);
        assert_eq!(game.elapsed_minutes, advanced_minutes);
        assert!(game.logs.last().unwrap().contains("时钟回拨"));
    }

    #[test]
    fn m8_dead_leech_lifecycle_uses_world_time_on_ground_and_in_inventory() {
        let mut game = Game::new();
        game.location = LocationId::from("goathill.cavern1");
        game.place_item_on_ground(ItemId::from(GOATHILL_DEAD_LEECH_ID), 1);
        let deadline = game.ground_items[&game.location]
            .iter()
            .find(|item| item.item_id.as_str() == GOATHILL_DEAD_LEECH_ID)
            .unwrap()
            .expires_at_elapsed_minutes
            .unwrap();
        assert_eq!(
            deadline,
            game.elapsed_minutes + GOATHILL_DEAD_LEECH_DECAY_TICKS * GAME_MINUTES_PER_TICK
        );

        game.elapsed_minutes = deadline - GAME_MINUTES_PER_TICK;
        game.tick();
        assert!(!game.current_ground_has(GOATHILL_DEAD_LEECH_ID));
        assert!(
            game.logs
                .iter()
                .any(|log| log.contains("地上的死岩蛭已经腐坏"))
        );

        let corpse = game.add_inventory_item(ItemId::from(GOATHILL_DEAD_LEECH_ID), 1);
        let deadline = game
            .player
            .item(corpse)
            .unwrap()
            .expires_at_elapsed_minutes
            .unwrap();
        let ticks = (deadline - game.elapsed_minutes) / GAME_MINUTES_PER_TICK;
        game.last_saved_at_unix_seconds = Some(1_000);
        game.advance_offline_progress(1_000 + ticks * OFFLINE_REAL_SECONDS_PER_TICK, Some(1_000));
        assert!(game.player.item(corpse).is_none());
        assert!(
            game.logs
                .iter()
                .any(|log| log.contains("行囊中的死岩蛭已经腐坏"))
        );
    }

    #[test]
    fn m8_corpse_lifecycle_and_dust_preserve_other_ground_loot() {
        let mut game = Game::new();
        game.location = LocationId::from("city.eastdoor1");
        game.place_corpse_on_ground("守城官兵");
        let corpse_instance_id = game.ground_items[&game.location]
            .iter()
            .find(|item| item.item_id.as_str() == CORPSE_ITEM_ID)
            .unwrap()
            .instance_id;
        let deadline = game.ground_items[&game.location]
            .iter()
            .find(|item| item.instance_id == corpse_instance_id)
            .unwrap()
            .expires_at_elapsed_minutes
            .unwrap();
        assert_eq!(
            deadline,
            game.elapsed_minutes + CORPSE_FRESH_DECAY_TICKS * GAME_MINUTES_PER_TICK
        );

        game.elapsed_minutes = deadline - GAME_MINUTES_PER_TICK;
        game.tick();
        let corpse = game.ground_items[&game.location]
            .iter()
            .find(|item| item.instance_id == corpse_instance_id)
            .unwrap();
        assert_eq!(corpse.lifecycle_stage, 1);
        assert_eq!(corpse.display_name(), "腐烂的尸体");

        let deadline = corpse.expires_at_elapsed_minutes.unwrap();
        game.elapsed_minutes = deadline - GAME_MINUTES_PER_TICK;
        game.tick();
        let corpse = game.ground_items[&game.location]
            .iter()
            .find(|item| item.instance_id == corpse_instance_id)
            .unwrap();
        assert_eq!(corpse.lifecycle_stage, 2);
        assert_eq!(corpse.display_name(), "一具枯干的骸骨");

        let deadline = corpse.expires_at_elapsed_minutes.unwrap();
        game.elapsed_minutes = deadline - GAME_MINUTES_PER_TICK;
        game.tick();
        assert!(!game.current_ground_has(CORPSE_ITEM_ID));

        game.place_corpse_on_ground("守城官兵");
        game.last_saved_at_unix_seconds = Some(1_000);
        game.advance_offline_progress(1_000 + 300 * OFFLINE_REAL_SECONDS_PER_TICK, Some(1_000));
        assert!(!game.current_ground_has(CORPSE_ITEM_ID));
        assert!(game.logs.iter().any(|log| log.contains("开始腐烂")));
        assert!(game.logs.iter().any(|log| log.contains("变成一具骸骨")));
        assert!(game.logs.iter().any(|log| log.contains("化成骨灰")));

        game.place_item_on_ground(ItemId::from("obj.cloth"), 1);
        game.place_corpse_on_ground("守城官兵");
        let corpse_instance_id = game.ground_items[&game.location]
            .iter()
            .find(|item| item.item_id.as_str() == CORPSE_ITEM_ID)
            .unwrap()
            .instance_id;
        let dust_instance_id = game.add_inventory_item(ItemId::from("obj.dust"), 1);
        let dissolve = Action::DissolveCorpse {
            dust_instance_id,
            corpse_instance_id,
        };
        assert!(game.available_actions().contains(&dissolve));
        game.perform(dissolve);

        assert!(!game.current_ground_has(CORPSE_ITEM_ID));
        assert!(game.current_ground_has("obj.cloth"));
        assert!(game.player.item(dust_instance_id).is_none());
        assert!(game.logs.last().unwrap().contains("化成一滩黄水"));
    }

    #[test]
    fn m8_source_day_phases_are_visible_only_outdoors_and_only_online() {
        let phases = [
            (0, "拂晓", "东方的天空已逐渐发白"),
            (240, "日出", "太阳刚从东方的地平线升起"),
            (360, "清晨", "太阳正高挂在东方的天空中"),
            (540, "正午", "现在是正午时分，太阳高挂在你的头顶正上方"),
            (720, "午后", "太阳正高挂在西方的天空中"),
            (900, "黄昏", "一轮火红的夕阳正徘徊在西方的地平线上"),
            (1080, "夜晚", "夜幕笼罩著大地"),
            (1200, "午夜", "夜幕低垂，满天繁星"),
        ];
        let mut outdoor = Game::new();
        outdoor.location = LocationId::from("choyin.bridge2");
        assert_eq!(
            outdoor.current_location().outdoors.as_deref(),
            Some("choyin")
        );
        for (minutes, name, description) in phases {
            outdoor.elapsed_minutes = minutes;
            assert_eq!(outdoor.time_period_text(), name);
            assert_eq!(outdoor.outdoor_time_description(), Some(description));
            assert!(outdoor.location_description().contains(description));
        }

        let mut indoor = Game::new();
        assert!(indoor.current_location().outdoors.is_none());
        assert_eq!(indoor.outdoor_time_description(), None);
        assert!(!indoor.location_description().contains("东方的天空"));

        outdoor.elapsed_minutes = 230;
        outdoor.logs.clear();
        outdoor.tick();
        assert!(
            outdoor
                .logs
                .iter()
                .any(|log| log == "太阳从东方的地平线升起了。")
        );

        indoor.elapsed_minutes = 230;
        indoor.logs.clear();
        indoor.tick();
        assert!(
            !indoor
                .logs
                .iter()
                .any(|log| log == "太阳从东方的地平线升起了。")
        );

        let mut offline = Game::new();
        offline.location = LocationId::from("choyin.bridge2");
        offline.elapsed_minutes = 230;
        offline.last_saved_at_unix_seconds = Some(1_000);
        offline.logs.clear();
        offline.advance_offline_progress(1_000 + OFFLINE_REAL_SECONDS_PER_TICK, Some(1_000));
        assert_eq!(offline.elapsed_minutes, 240);
        assert_eq!(offline.time_period_text(), "日出");
        assert!(
            !offline
                .logs
                .iter()
                .any(|log| log == "太阳从东方的地平线升起了。")
        );
    }

    #[test]
    fn m8_dynamic_quest_is_source_backed_persistent_and_world_timed() {
        let mut game = Game::new();
        game.location = LocationId::from("u.cloud.god2");
        let request = Action::RequestDynamicQuest;
        assert!(game.available_actions().contains(&request));

        game.player.combat_experience = DYNAMIC_QUEST_MIN_COMBAT_EXPERIENCE;
        game.perform(request.clone());
        assert!(game.dynamic_quest.is_none());
        assert!(game.logs.last().unwrap().contains("这点本事"));

        game.player.combat_experience = 5_000;
        game.perform(request.clone());
        let active = game.dynamic_quest.clone().expect("quest must be issued");
        assert_eq!(active.tier, 5_000);
        assert!(
            game.available_dynamic_quest_targets()
                .contains(active.target.as_str())
        );
        assert!(game.quest_title().contains("朱鸿雪悬赏"));
        assert!(game.quest_objective().contains(&active.target));
        assert!(
            game.dynamic_quest_remaining_seconds()
                .is_some_and(|seconds| seconds > 0)
        );

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert_eq!(restored.dynamic_quest(), Some(&active));

        let mut offline = restored;
        let deadline = offline.elapsed_minutes + GAME_MINUTES_PER_TICK;
        offline
            .dynamic_quest
            .as_mut()
            .unwrap()
            .deadline_elapsed_minutes = deadline;
        offline.last_saved_at_unix_seconds = Some(1_000);
        offline.advance_offline_progress(1_000 + 2 * OFFLINE_REAL_SECONDS_PER_TICK, Some(1_000));
        assert!(offline.dynamic_quest_has_expired());
        assert_eq!(offline.dynamic_quest_remaining_seconds(), Some(0));
        assert!(offline.quest_title().contains("已逾期"));

        offline.dynamic_quest_finished = 7;
        let qi_before = offline.player.qi;
        offline.perform(request);
        assert_eq!(
            offline.player.qi,
            (qi_before / 2 + 1).min(offline.player.max_qi)
        );
        assert_eq!(offline.dynamic_quest_finished, 0);
        assert!(offline.dynamic_quest.is_some());
        assert!(!offline.dynamic_quest_has_expired());
    }

    #[test]
    fn m8_dynamic_quest_kill_rewards_match_source_ranges_and_cap_potential() {
        let mut game = Game::new();
        game.player.combat_experience = 5_000;
        let definition = game
            .dynamic_quest_candidates(5_000)
            .into_iter()
            .next()
            .expect("tier must contain an alive imported target")
            .clone();
        let (location, npc) = crate::content::world()
            .locations()
            .find_map(|location| {
                game.static_npcs_at(&location.id)
                    .into_iter()
                    .find(|npc| npc.name() == definition.target)
                    .map(|npc| (location.id.clone(), npc))
            })
            .expect("offered target must map to a placed NPC");
        game.location = location;
        game.elapsed_minutes = 1_000;
        game.player.learned_points = 30;
        game.player.potential = 125;
        game.player.reputation = -10;
        game.dynamic_quest = Some(DynamicQuest {
            target: definition.target.clone(),
            tier: definition.tier,
            deadline_elapsed_minutes: game.elapsed_minutes
                + definition.time_seconds * GAME_MINUTES_PER_TICK,
            exp_bonus: definition.exp_bonus,
            potential_bonus: definition.potential_bonus,
            score_bonus: definition.score_bonus,
            factor: DYNAMIC_QUEST_FACTOR,
        });
        let combat = CombatState {
            enemy: EnemyKind::Npc(npc),
            health: 1,
            max_health: 1,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        };
        let mut baseline = game.clone();
        baseline.dynamic_quest = None;
        let mut rewarded = game;
        baseline.win_combat(combat.clone());
        rewarded.win_combat(combat);

        let experience_bonus =
            rewarded.player.combat_experience - baseline.player.combat_experience;
        assert!(experience_bonus >= (definition.exp_bonus / 2) as u64);
        assert!(experience_bonus < definition.exp_bonus as u64);
        let reputation_penalty = baseline.player.reputation - rewarded.player.reputation;
        assert!(reputation_penalty >= definition.score_bonus / 2);
        assert!(reputation_penalty < definition.score_bonus);
        assert_eq!(
            rewarded.player.potential,
            rewarded.player.learned_points + 100
        );
        assert!(rewarded.dynamic_quest.is_none());
        assert_eq!(rewarded.dynamic_quest_finished, 1);
        assert!(
            rewarded
                .logs
                .iter()
                .any(|log| log.contains("朱鸿雪的悬赏完成"))
        );
    }

    #[test]
    fn m8_abandon_skill_deletes_only_the_source_skill_record() {
        let mut game = Game::new();
        let skill = SkillId::from(LIUH_KEN_ID);
        let action = Action::AbandonSkill(skill.clone());
        game.player.learned_points = 37;
        game.player.potential = 120;
        game.activity = Activity::Training(skill.clone());

        assert!(game.available_actions().contains(&action));
        assert!(
            game.player
                .skill_mappings
                .iter()
                .any(|mapping| mapping.skill.as_str() == skill.as_str())
        );
        game.perform(action);

        assert!(game.player.skill_by_id(LIUH_KEN_ID).is_none());
        assert_eq!(game.player.learned_points, 37);
        assert_eq!(game.player.potential, 120);
        assert_eq!(game.activity, Activity::Idle);
        assert!(
            game.player
                .skill_mappings
                .iter()
                .any(|mapping| mapping.skill.as_str() == LIUH_KEN_ID)
        );
        assert_eq!(
            game.player.effective_skill(UNARMED_ID),
            game.player.skill_level(UNARMED_ID) / 2
        );
        assert!(game.logs.last().unwrap().contains(skill.name()));

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(restored.player.skill_by_id(LIUH_KEN_ID).is_none());
        assert!(
            restored
                .player
                .skill_mappings
                .iter()
                .any(|mapping| mapping.skill.as_str() == LIUH_KEN_ID)
        );
    }

    #[test]
    fn m8_choyin_justice_is_online_persistent_and_source_adapted() {
        let mut bribe = Game::new();
        bribe.location = LocationId::from("choyin.bridge2");
        bribe.player.wanted = 3;
        bribe.player.set_money_value(CHOYIN_BRIBE_AMOUNT);
        bribe.tick();
        assert_eq!(bribe.choyin_justice, ChoyinJusticeState::Pursuit);
        bribe.tick();
        assert_eq!(bribe.choyin_justice, ChoyinJusticeState::Caught);
        assert!(
            bribe
                .available_actions()
                .iter()
                .all(|action| !matches!(action, Action::Move { .. }))
        );
        let ransom = Action::OfferMoney {
            amount: CHOYIN_BRIBE_AMOUNT,
            npc: NpcId::from(CHOYIN_POLICE_ID),
        };
        assert_eq!(bribe.available_actions(), vec![ransom.clone()]);

        let mut restored: Game =
            serde_json::from_str(&serde_json::to_string(&bribe).unwrap()).unwrap();
        assert_eq!(restored.choyin_justice, ChoyinJusticeState::Caught);
        restored.perform(ransom);
        assert_eq!(restored.player.money_value(), 0);
        assert_eq!(restored.player.wanted, 0);
        assert_eq!(restored.choyin_justice, ChoyinJusticeState::Free);
        assert!(
            restored
                .available_actions()
                .iter()
                .any(|action| matches!(action, Action::Move { .. }))
        );

        let mut trial = Game::new();
        trial.location = LocationId::from("choyin.bridge2");
        trial.player.wanted = 2;
        let health_before = (trial.player.essence, trial.player.qi, trial.player.spirit);
        for _ in 0..4 {
            trial.tick();
        }
        assert_eq!(trial.location.as_str(), "choyin.court1");
        assert_eq!(trial.choyin_justice, ChoyinJusticeState::Free);
        assert_eq!(trial.player.wanted, 2);
        assert!(trial.player.essence < health_before.0);
        assert!(trial.player.qi < health_before.1);
        assert!(trial.player.spirit < health_before.2);
        assert!(trial.logs.iter().any(|log| log.contains("打20大板")));
        assert!(
            trial
                .logs
                .iter()
                .any(|log| log.contains("差役将你丢在县衙门外"))
        );

        let mut offline = Game::new();
        offline.player.wanted = 1;
        offline.last_saved_at_unix_seconds = Some(1_000);
        offline.advance_offline_progress(1_000 + OFFLINE_REAL_SECONDS_PER_TICK, Some(1_000));
        assert_eq!(offline.choyin_justice, ChoyinJusticeState::Free);
        assert!(
            !offline
                .logs
                .iter()
                .any(|log| log.contains("巡捕正在赶来缉拿"))
        );
    }

    #[test]
    fn m8_ambient_chat_random_movement_is_safe_persistent_and_online_only() {
        let mut speaker = Game::new();
        speaker.location = LocationId::from("choyin.n_street1");
        speaker.rng_state = 17;
        speaker.run_npc_ambient_chat();
        assert!(speaker.logs.iter().any(|log| log.contains("卖饼大叔")));

        let mut mover = Game::new();
        mover.location = LocationId::from("choyin.bridge2");
        let scholar = NpcId::from("choyin.npc.scholar");
        assert_eq!(
            mover
                .static_npcs_at(&mover.location)
                .iter()
                .filter(|npc| npc.as_str() == scholar.as_str())
                .count(),
            2
        );
        mover.rng_state = 17;
        mover.run_npc_ambient_chat();
        assert_eq!(mover.npc_location_overrides.len(), 1);
        let moved = mover.npc_location_overrides.first().unwrap().clone();
        assert_eq!(moved.origin.as_str(), "choyin.bridge2");
        assert_eq!(moved.npc, scholar);
        assert!(matches!(
            moved.location.as_str(),
            "choyin.bridge1" | "choyin.bridge3"
        ));
        assert_eq!(
            mover
                .static_npcs_at(&mover.location)
                .iter()
                .filter(|npc| npc.as_str() == scholar.as_str())
                .count(),
            1
        );
        mover.location = moved.location.clone();
        assert!(mover.npc_is_present(&scholar));
        assert!(
            mover
                .available_actions()
                .contains(&Action::Talk(scholar.clone()))
        );

        let restored: Game = serde_json::from_str(&serde_json::to_string(&mover).unwrap()).unwrap();
        assert_eq!(
            restored.npc_location_overrides,
            mover.npc_location_overrides
        );

        let mut protected = Game::new();
        protected.location = LocationId::from("snow.school1");
        assert!(
            protected
                .run_npc_random_movement(&NpcId::from(SNOW_GUARD_ID))
                .is_none()
        );
        assert!(protected.npc_location_overrides.is_empty());

        let mut offline = Game::new();
        offline.location = LocationId::from("choyin.bridge2");
        offline.last_saved_at_unix_seconds = Some(1_000);
        offline.advance_offline_progress(1_000 + 128 * OFFLINE_REAL_SECONDS_PER_TICK, Some(1_000));
        assert!(offline.npc_location_overrides.is_empty());
    }

    #[test]
    fn m7_latemoon_teacher_clues_and_token_rewards_form_one_plot_chain() {
        let mut game = Game::new();
        game.location = LocationId::from("latemoon.latemoon1");
        let apprentice = Action::BecomeApprentice("dancer".into());
        assert!(game.available_actions().contains(&apprentice));
        game.perform(apprentice.clone());
        assert!(game.player.teacher.is_none());
        game.player.gender = Gender::Female;
        game.perform(apprentice);
        assert_eq!(game.player.teacher.as_deref(), Some("dancer"));
        assert_eq!(game.player.faction.as_deref(), Some("晚月庄"));

        game.location = LocationId::from("latemoon.miroom2");
        let bamboo = game.add_inventory_item(ItemId::from(LATEMOON_BAMBOO_IDS[0]), 1);
        game.perform(Action::GiveItem {
            instance_id: bamboo,
            npc: NpcId::from(LATEMOON_SHAOWEI_ID),
        });
        assert!(game.latemoon_dragonfly_received);
        assert!(game.player.item(bamboo).is_none());
        let dragonfly = game
            .player
            .inventory
            .iter()
            .find(|item| LATEMOON_DRAGONFLY_IDS.contains(&item.item_id.as_str()))
            .unwrap()
            .instance_id;

        game.location = LocationId::from("latemoon.latemoon6");
        game.perform(Action::GiveItem {
            instance_id: dragonfly,
            npc: NpcId::from(LATEMOON_FUNLIN_ID),
        });
        assert!(game.latemoon_bracelet_clue);
        assert!(game.player.item(dragonfly).is_none());
        game.location = LocationId::from("latemoon.latemoon2");
        let search_bracelet = Action::Interact(InteractionKind::SearchLateMoonBracelet);
        assert!(game.available_actions().contains(&search_bracelet));
        game.perform(search_bracelet.clone());
        assert!(game.latemoon_bracelet_received);
        assert!(game.player.has_item(&ItemId::from("latemoon.obj.bracelet")));
        assert!(!game.available_actions().contains(&search_bracelet));

        game.location = LocationId::from("latemoon.upstar.uproom2");
        game.perform(Action::AskNpc {
            npc: NpcId::from(LATEMOON_SHINFUN_ID),
            topic: "舞曲谱".into(),
        });
        assert!(game.latemoon_dance_book_clue);
        game.location = LocationId::from("latemoon.latemoon8");
        let search_book = Action::Interact(InteractionKind::SearchLateMoonDanceBook);
        assert!(game.available_actions().contains(&search_book));
        game.perform(search_book.clone());
        assert!(game.latemoon_dance_book_received);
        assert!(game.player.has_item(&ItemId::from("latemoon.obj.book")));
        assert!(!game.available_actions().contains(&search_book));

        game.location = LocationId::from("latemoon.room.lstudio");
        game.player.max_force = 100;
        game.player.force = 80;
        game.rng_state = 0;
        let token = game.add_inventory_item(ItemId::from(LATEMOON_TOKEN_ID), 1);
        game.perform(Action::GiveItem {
            instance_id: token,
            npc: NpcId::from(LATEMOON_OLD_ID),
        });
        assert!(game.latemoon_token_rewarded);
        assert!(game.player.item(token).is_none());
        assert!((101..=110).contains(&game.player.max_force));
        assert_eq!(game.player.force, 0);

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(restored.latemoon_bracelet_received);
        assert!(restored.latemoon_dance_book_received);
        assert!(restored.latemoon_token_rewarded);
    }

    #[test]
    fn m7_cloud_escort_letter_crossing_and_exchanges_form_one_plot_chain() {
        let mut game = Game::new();
        game.location = LocationId::from("u.cloud.biaoju");
        game.player.courage = 24;
        let join = Action::Interact(InteractionKind::JoinCloudEscort);
        game.perform(join.clone());
        assert!(!game.cloud_escort_member);
        game.player.courage = 25;
        assert!(game.available_actions().contains(&join));
        game.perform(join);
        assert!(game.cloud_escort_member);
        assert_eq!(game.player.faction.as_deref(), Some("振远镖局"));
        assert_eq!(game.player.teacher.as_deref(), Some(CLOUD_B_HEADER_ID));

        let grass = game.add_inventory_item(ItemId::from(CHOYIN_GRASS_ID), 1);
        game.perform(Action::GiveItem {
            instance_id: grass,
            npc: NpcId::from(CLOUD_B_HEADER_ID),
        });
        assert!(game.cloud_escort_letter_received);
        assert!(game.player.item(grass).is_none());
        let letter = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == CLOUD_ESCORT_LETTER_ID)
            .unwrap()
            .instance_id;

        game.location = LocationId::from("city.biaoju");
        let chen = NpcId::from(crate::npcs::CITY_MASTER_CHEN_ID);
        assert!(game.available_actions().iter().all(|action| !matches!(
            action,
            Action::LearnFromNpc { npc, .. } if npc == &chen
        )));
        game.perform(Action::GiveItem {
            instance_id: letter,
            npc: chen.clone(),
        });
        assert!(game.city_chen_letter_delivered);
        assert!(game.player.item(letter).is_none());
        assert!(game.available_actions().iter().any(|action| matches!(
            action,
            Action::LearnFromNpc { npc, .. } if npc == &chen
        )));

        game.location = LocationId::from("u.cloud.dukou");
        let fare = game.add_inventory_item(ItemId::from("obj.weapon.dagger"), 1);
        game.perform(Action::GiveItem {
            instance_id: fare,
            npc: NpcId::from(CLOUD_BOATER_ID),
        });
        assert!(game.cloud_boater_paid);
        assert!(game.player.item(fare).is_none());
        game.move_to(LocationId::from("u.cloud.sunhill.northriver"));
        let cross = game
            .available_actions()
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Move { target, .. }
                        if target.as_str() == "u.cloud.sunhill.midriver"
                )
            })
            .unwrap();
        game.perform(cross);
        assert_eq!(game.location.as_str(), "u.cloud.sunhill.midriver");
        assert!(!game.cloud_boater_paid);

        game.location = LocationId::from("u.cloud.dragonhill.hummock");
        let passage = game.add_inventory_item(ItemId::from(GREEN_WIND_SWORD_ID), 1);
        game.perform(Action::GiveItem {
            instance_id: passage,
            npc: NpcId::from(CLOUD_GANGSTER_ID),
        });
        assert!(game.cloud_gangster_pass);
        assert!(game.player.item(passage).is_none());

        game.location = LocationId::from("u.cloud.jiyuan2");
        game.player.perception = 25;
        let recognition = game.add_inventory_item(ItemId::from(LATEMOON_FIRE_ID), 1);
        game.perform(Action::GiveItem {
            instance_id: recognition,
            npc: NpcId::from(CLOUD_GIRL_ID),
        });
        assert!(game.cloud_girl_recognized);
        assert!(game.player.item(recognition).is_none());

        game.location = LocationId::from("u.cloud.duchang");
        let stake = game.add_inventory_item(ItemId::from("obj.weapon.dagger"), 1);
        let money_before = game.player.money_value();
        game.rng_state = 0;
        game.perform(Action::GiveItem {
            instance_id: stake,
            npc: NpcId::from(CLOUD_JUDGE_ID),
        });
        assert!(game.player.item(stake).is_none());
        assert!(
            matches!(game.player.money_value(), value if value == money_before || value == money_before + 100)
        );

        game.location = LocationId::from("u.cloud.monky");
        let donation = game.add_inventory_item(ItemId::from("obj.weapon.dagger"), 1);
        game.perform(Action::GiveItem {
            instance_id: donation,
            npc: NpcId::from(CLOUD_MONK_ID),
        });
        assert!(game.player.item(donation).is_none());

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(restored.cloud_escort_member);
        assert!(restored.city_chen_letter_delivered);
        assert!(restored.cloud_gangster_pass);
        assert!(restored.cloud_girl_recognized);
    }

    #[test]
    fn m7_room_events_preserve_latemoon_and_death_state_machines() {
        let mut game = Game::new();
        game.location = LocationId::from("latemoon.latemoon2");
        for _ in 0..2 {
            let take = Action::Interact(InteractionKind::TakeLateMoonCloth);
            assert!(game.available_actions().contains(&take));
            game.perform(take);
        }
        assert_eq!(game.latemoon_clothes_taken, 2);
        assert_eq!(
            game.player
                .inventory
                .iter()
                .filter(|item| item.item_id.as_str() == "latemoon.obj.skirt")
                .count(),
            2
        );
        assert!(
            !game
                .available_actions()
                .contains(&Action::Interact(InteractionKind::TakeLateMoonCloth))
        );

        game.location = LocationId::from("latemoon.latemoon8");
        game.player.max_spirit = 200;
        game.player.spirit = 200;
        game.perform(Action::Interact(InteractionKind::DanceLateMoonYuFong));
        assert_eq!(game.location.as_str(), "latemoon.miroom");
        assert_eq!(game.player.spirit, 100);
        game.perform(Action::Interact(InteractionKind::DanceLateMoonOut));
        assert_eq!(game.location.as_str(), "latemoon.bamboo");
        assert_eq!(game.player.spirit, 20);

        game.location = LocationId::from("latemoon.park.moonc");
        for _ in 0..2 {
            game.perform(Action::Interact(InteractionKind::PickLateMoonFlower));
        }
        assert_eq!(game.latemoon_flowers_picked, 2);
        assert_eq!(
            game.player
                .inventory
                .iter()
                .filter(|item| item.item_id.as_str() == "latemoon.park.npc.obj.flower")
                .map(|item| item.quantity)
                .sum::<u32>(),
            2
        );

        let mut male = Game::new();
        male.location = LocationId::from("latemoon.room.bathroom");
        male.player.qi = 100;
        male.player.spirit = 100;
        male.perform(Action::Interact(InteractionKind::BatheLateMoonPool));
        assert_eq!(
            male.player.condition(ConditionKind::RosePoison),
            Some(&ConditionState {
                kind: ConditionKind::RosePoison,
                duration: 15,
                potency: 10,
            })
        );
        male.update_conditions();
        assert_eq!(male.player.qi, 90);
        assert_eq!(male.player.spirit, 80);
        assert_eq!(
            male.player
                .condition(ConditionKind::RosePoison)
                .unwrap()
                .duration,
            14
        );
        male.location = LocationId::from("latemoon.room.bathroom1");
        male.move_to(LocationId::from("latemoon.room.flower1"));
        assert_eq!(
            male.player
                .condition(ConditionKind::RosePoison)
                .unwrap()
                .duration,
            5
        );

        let mut female = Game::new();
        female.location = LocationId::from("latemoon.room.bathroom");
        female.player.gender = Gender::Female;
        female.player.max_essence = 200;
        female.player.essence = 100;
        female.player.max_spirit = 200;
        female.player.spirit = 100;
        female.rng_state = 0;
        female.perform(Action::Interact(InteractionKind::BatheLateMoonPool));
        assert_eq!(female.player.essence, 90);
        assert!((105..=109).contains(&female.player.spirit));
        assert!(female.player.condition(ConditionKind::RosePoison).is_none());

        female.location = LocationId::from("latemoon.upstar.uproom3");
        female.player.spirit = 100;
        female.player.bellicosity = 100;
        female.rng_state = 0;
        female.perform(Action::Interact(InteractionKind::PonderLateMoonRoom));
        assert_eq!(female.player.spirit, 50);
        assert!(female.player.bellicosity <= 93);

        let mut death = Game::new();
        death.location = LocationId::from("death.gateway");
        assert!(death.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == "death.gate"
        )));
        death.location = LocationId::from("death.road2");
        for expected in 1..5 {
            death.perform(Action::Move {
                direction: "north".into(),
                target: LocationId::from("death.road3"),
            });
            assert_eq!(death.location.as_str(), "death.road2");
            assert_eq!(death.death_road_steps, expected);
        }
        death.perform(Action::Move {
            direction: "north".into(),
            target: LocationId::from("death.road3"),
        });
        assert_eq!(death.location.as_str(), "death.road3");
        assert_eq!(death.death_road_steps, 0);
        death.location = LocationId::from("death.road2");
        death.death_road_steps = 3;
        death.perform(Action::Move {
            direction: "south".into(),
            target: LocationId::from("death.road1"),
        });
        assert_eq!(death.death_road_steps, 0);

        death.location = LocationId::from("death.inn1");
        death.player.essence = 1;
        death.player.qi = 2;
        death.player.spirit = 3;
        death.player.set_condition(ConditionKind::Poison, 8, 5);
        death.perform(Action::Interact(InteractionKind::InspectDeathShadows));
        assert!(death.logs.last().unwrap().contains("一模一样"));
        death.perform(Action::Interact(InteractionKind::ReincarnateDeathInn));
        assert_eq!(death.location.as_str(), "snow.temple");
        assert_eq!(death.player.essence, death.player.max_essence);
        assert_eq!(death.player.qi, death.player.max_qi);
        assert_eq!(death.player.spirit, death.player.max_spirit);
        assert!(death.player.conditions.is_empty());
    }

    #[test]
    fn m7_item_commands_consumables_and_meat_residuals_preserve_source_behavior() {
        let mut game = Game::new();
        let book = game.add_inventory_item(ItemId::from("latemoon.npc.obj.book"), 1);
        let use_book = Action::Interact(InteractionKind::UseLateMoonDanceBook(book));
        game.location = LocationId::from("death.road1");
        game.player.spirit = 40;
        game.perform(use_book.clone());
        assert_eq!(game.location.as_str(), "death.road1");
        assert_eq!(game.player.spirit, 40);
        game.player.spirit = 100;
        game.perform(use_book);
        assert_eq!(game.location.as_str(), "latemoon.latemoon8");
        assert_eq!(game.player.spirit, 50);
        assert!(!game.available_actions().contains(&Action::DropItem(book)));
        game.perform(Action::DropItem(book));
        game.perform(Action::SellItem(book));
        assert!(game.player.item(book).is_some());

        let bracelet = game.add_inventory_item(ItemId::from("latemoon.obj.bracelet"), 1);
        game.player.spirit = 49;
        game.perform(Action::Interact(InteractionKind::PrayLateMoonBracelet(
            bracelet,
        )));
        assert_eq!(game.location.as_str(), "latemoon.latemoon8");
        game.player.spirit = 100;
        game.perform(Action::Interact(InteractionKind::PrayLateMoonBracelet(
            bracelet,
        )));
        assert_eq!(game.location.as_str(), "snow.temple");
        assert_eq!(game.player.spirit, 50);

        let letter = game.add_inventory_item(ItemId::from(LATEMOON_SECRET_LETTER_ID), 1);
        let read_letter = Action::Interact(InteractionKind::ReadLateMoonSecretLetter(letter));
        assert!(!game.available_actions().contains(&read_letter));
        let fire = game.add_inventory_item(ItemId::from(LATEMOON_FIRE_ID), 1);
        assert!(game.available_actions().contains(&read_letter));
        game.perform(read_letter);
        assert!(game.logs.last().unwrap().contains("密室藏有舞谱"));
        assert!(game.player.item(letter).is_some());
        assert!(game.player.item(fire).is_some());

        game.player.max_essence = 1_000;
        game.player.max_qi = 1_000;
        game.player.max_spirit = 1_000;
        game.player.essence = 0;
        game.player.qi = 0;
        game.player.spirit = 0;
        let bean = game.add_inventory_item(ItemId::from("latemoon.sell.bean"), 1);
        game.perform(Action::ConsumeItem(bean));
        assert_eq!(
            (game.player.essence, game.player.qi, game.player.spirit),
            (50, 100, 50)
        );
        assert!(game.player.item(bean).is_none());

        game.player.spirit = 0;
        game.player.set_condition(ConditionKind::RosePoison, 15, 10);
        let flower = game.add_inventory_item(ItemId::from("latemoon.park.npc.obj.flower"), 1);
        game.perform(Action::ConsumeItem(flower));
        assert_eq!(game.player.spirit, 50);
        assert_eq!(
            game.player
                .condition(ConditionKind::RosePoison)
                .unwrap()
                .duration,
            5
        );

        game.player.essence = 0;
        game.player.qi = 0;
        game.player.spirit = 0;
        let pill = game.add_inventory_item(ItemId::from("latemoon.sell.white_pill"), 1);
        game.perform(Action::ConsumeItem(pill));
        assert_eq!(
            (game.player.essence, game.player.qi, game.player.spirit),
            (100, 300, 100)
        );
        let wine = game.add_inventory_item(ItemId::from("latemoon.sell.wine"), 1);
        game.perform(Action::ConsumeItem(wine));
        assert_eq!((game.player.essence, game.player.spirit), (90, 120));

        game.player.max_food = 10_000;
        for (item_id, uses, residual_name, weight, value) in [
            ("u.cloud.obj.meat.beef", 3, "牛肋骨", 200, 100),
            ("u.cloud.obj.meat.dog_m", 3, "狗骨头", 250, 60),
            ("u.cloud.obj.meat.hind", 5, "牛腿骨", 300, 150),
        ] {
            let meat = game.add_inventory_item(ItemId::from(item_id), 1);
            if item_id == "u.cloud.obj.meat.hind" {
                assert_eq!(
                    game.player.item(meat).unwrap().definition().weapon_skill(),
                    Some("hammer")
                );
                game.perform(Action::EquipItem(meat));
                assert!(game.player.is_equipped(meat));
            }
            for _ in 0..uses {
                game.player.food = 0;
                game.perform(Action::ConsumeItem(meat));
            }
            let residual = game.player.item(meat).unwrap();
            assert_eq!(residual.display_name(), residual_name);
            assert_eq!(residual.total_weight(), weight);
            assert_eq!(residual.unit_value(), value);
            assert_eq!(residual.remaining_uses, Some(0));
        }

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert_eq!(
            restored
                .player
                .inventory
                .iter()
                .filter(|item| CLOUD_MEAT_IDS.contains(&item.item_id.as_str()))
                .count(),
            3
        );
    }

    #[test]
    fn m6_green_jade_and_slumber_chain_preserves_source_clues_and_exchanges() {
        let mut game = Game::new();
        game.location = LocationId::from("green.house4");
        let oldman = NpcId::from("green.npc.oldman2");
        game.perform(Action::AskNpc {
            npc: oldman,
            topic: "玉佩".into(),
        });
        assert!(game.green_elder_jade_clue);

        game.location = LocationId::from("snow.mstreet2");
        let drunk = NpcId::from("snow.npc.drunk");
        for expected_jade_clue in [true, false] {
            let wine = game.add_inventory_item(ItemId::from("chuenyu.obj.qiwine"), 1);
            let gift = Action::GiveItem {
                instance_id: wine,
                npc: drunk.clone(),
            };
            assert!(game.available_actions().contains(&gift));
            game.perform(gift);
            assert!(game.player.item(wine).is_none());
            if expected_jade_clue {
                assert!(game.green_drunk_jade_clue);
                assert!(!game.green_drunk_drug_clue);
            }
        }
        assert!(game.green_drunk_drug_clue);

        game.location = LocationId::from("green.shop0");
        let shen = NpcId::from(GREEN_SHEN_ID);
        let ask_jade = Action::AskNpc {
            npc: shen.clone(),
            topic: "玉佩".into(),
        };
        assert!(game.available_actions().contains(&ask_jade));
        game.perform(ask_jade.clone());
        assert!(game.green_jade_received);
        assert!(game.player.has_item(&ItemId::from(GREEN_JADE_ID)));
        assert!(!game.available_actions().contains(&ask_jade));

        let ask_drug = Action::AskNpc {
            npc: shen.clone(),
            topic: "蒙汗药".into(),
        };
        assert!(game.available_actions().contains(&ask_drug));
        game.perform(ask_drug.clone());
        assert!(game.green_drug_offer_unlocked);
        assert!(!game.available_actions().contains(&ask_drug));
        let jade = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == GREEN_JADE_ID)
            .unwrap()
            .instance_id;
        game.perform(Action::GiveItem {
            instance_id: jade,
            npc: shen,
        });
        assert!(game.player.item(jade).is_none());
        assert!(game.player.has_item(&ItemId::from(items::SLUMBER_DRUG_ID)));

        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(restored.green_elder_jade_clue);
        assert!(restored.green_drunk_jade_clue);
        assert!(restored.green_drunk_drug_clue);
        assert!(restored.green_jade_received);
        assert!(restored.green_drug_offer_unlocked);
    }

    #[test]
    fn m6_source_juan_rescue_and_murder_share_the_existing_liu_plot() {
        let mut rescue = Game::new();
        rescue.location = LocationId::from("chuenyu.home");
        let old_liu = NpcId::from(CHUENYU_OLD_LIU_ID);
        assert!(
            rescue
                .available_actions()
                .contains(&Action::Talk(old_liu.clone()))
        );
        rescue.perform(Action::Talk(old_liu.clone()));
        assert_eq!(rescue.quest, QuestStage::FindJuan);

        rescue.move_to(LocationId::from("chuenyu.dungeon"));
        let boss_combat = match std::mem::replace(&mut rescue.activity, Activity::Idle) {
            Activity::Fighting(combat) => combat,
            _ => panic!("chuenyu boss must attack on dungeon entry"),
        };
        assert_eq!(
            boss_combat.enemy,
            EnemyKind::Npc(NpcId::from(CHUENYU_BOSS_ID))
        );
        assert_eq!(boss_combat.mode, CombatMode::Lethal);
        rescue.win_combat(boss_combat);
        assert_eq!(rescue.quest, QuestStage::FoundJuan);
        let source_juan = NpcId::from(CHUENYU_XIAO_JUAN_PLACED_ID);
        assert!(
            rescue
                .available_actions()
                .contains(&Action::Talk(source_juan.clone()))
        );
        rescue.perform(Action::Talk(source_juan));
        assert_eq!(rescue.quest, QuestStage::ReturnHome);
        rescue.move_to(LocationId::from("chuenyu.home"));
        rescue.perform(Action::Talk(old_liu));
        assert_eq!(rescue.quest, QuestStage::Complete);
        assert!(
            rescue
                .player
                .has_item(&ItemId::from(items::HENGBING_SWORD_ID))
        );
        assert!(
            rescue
                .player
                .has_item(&ItemId::from(items::PARRY_MANUAL_ID))
        );

        let mut murder = Game::new();
        murder.quest = QuestStage::FoundJuan;
        murder.location = LocationId::from("chuenyu.dungeon");
        murder.win_combat(CombatState {
            enemy: EnemyKind::Npc(NpcId::from(CHUENYU_XIAO_JUAN_PLACED_ID)),
            health: 0,
            max_health: 1,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        });
        assert_eq!(murder.quest, QuestStage::MurderedJuan);
        murder.move_to(LocationId::from("chuenyu.home"));
        assert_eq!(murder.quest, QuestStage::Failed);
        assert!(matches!(
            murder.activity,
            Activity::Fighting(CombatState {
                enemy: EnemyKind::OldLiuRevenge,
                mode: CombatMode::Lethal,
                ..
            })
        ));
    }

    #[test]
    fn m6_entry_aggression_elite_powerup_and_green_callbacks_are_bounded() {
        let mut guard = Game::new();
        guard.move_to(LocationId::from("chuenyu.tortureroom"));
        assert!(matches!(
            guard.activity,
            Activity::Fighting(CombatState {
                enemy: EnemyKind::Npc(ref npc),
                mode: CombatMode::Lethal,
                ..
            }) if npc.as_str() == CHUENYU_GUARD_TWO_ID
        ));

        let mut elite = Game::new();
        elite.location = LocationId::from("waterfog.east_2f");
        elite.perform(Action::Fight(EnemyKind::Npc(NpcId::from(
            WATERFOG_ELITE_GUARD_ID,
        ))));
        assert!(matches!(elite.activity, Activity::Idle));
        elite.perform(Action::Kill(EnemyKind::Npc(NpcId::from(
            WATERFOG_ELITE_GUARD_ID,
        ))));
        assert!(matches!(
            elite.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Lethal,
                enemy_attack_bonus,
                ..
            }) if enemy_attack_bonus > 0
        ));

        let mut helper = Game::new();
        helper.location = LocationId::from("green.house2");
        let oldman = NpcId::from("green.npc.oldman");
        let mut combat = CombatState {
            enemy: EnemyKind::Npc(oldman.clone()),
            health: 100,
            max_health: 100,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        };
        helper.run_npc_combat_callback(
            &mut combat,
            &oldman,
            "(: this_object(), \"ask_for_help\" :)",
        );
        assert_eq!(combat.enemy_attack_bonus, 5);
        for _ in 0..10 {
            helper.run_npc_combat_callback(
                &mut combat,
                &oldman,
                "(: this_object(), \"wield_something\" :)",
            );
        }
        assert_eq!(combat.enemy_attack_bonus, 30);
    }

    #[test]
    fn m5_oldpine_combat_hooks_spawn_help_once_and_apply_snake_poison() {
        let mut game = Game::new();
        game.location = LocationId::from("oldpine.pine1");
        let fat_bandit = NpcId::from(OLDPINE_FAT_BANDIT_ID);
        let mut combat = CombatState {
            enemy: EnemyKind::Npc(fat_bandit.clone()),
            health: 50,
            max_health: 50,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        };
        game.rng_state = 20;
        assert!(!game.run_npc_combat_chat(&mut combat));
        assert_eq!(game.spawned_npc_instances.len(), 1);
        let chief = NpcId::from(OLDPINE_BANDIT_CHIEF_ID);
        assert!(
            game.available_actions()
                .contains(&Action::Talk(chief.clone()))
        );
        assert!(
            game.available_actions()
                .contains(&Action::Kill(EnemyKind::Npc(chief.clone())))
        );

        let mut restored: Game =
            serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        restored.run_npc_combat_callback(
            &mut combat,
            &fat_bandit,
            "(: this_object(), \"call_for_help\" :)",
        );
        assert_eq!(restored.spawned_npc_instances.len(), 1);
        restored.win_combat(CombatState {
            enemy: EnemyKind::Npc(chief.clone()),
            health: 0,
            max_health: 1,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        });
        assert!(!restored.npc_is_present(&chief));
        assert!(
            restored.ground_items[&restored.location]
                .iter()
                .any(|item| { item.item_id.as_str() == "oldpine.npc.obj.blade" })
        );

        let venom = EnemyKind::Npc(NpcId::from(OLDPINE_VENOM_SNAKE_ID));
        for _ in 0..10 {
            game.apply_npc_hit_hook(&venom, 100, 0);
            if game.player.condition(ConditionKind::SnakePoison).is_some() {
                break;
            }
        }
        assert_eq!(
            game.player.condition(ConditionKind::SnakePoison),
            Some(&ConditionState {
                kind: ConditionKind::SnakePoison,
                duration: 20,
                potency: 10,
            })
        );
        game.player
            .set_condition(ConditionKind::SnakePoison, 12, 10);
        for _ in 0..10 {
            game.apply_npc_hit_hook(&venom, 100, 0);
        }
        assert_eq!(
            game.player
                .condition(ConditionKind::SnakePoison)
                .unwrap()
                .duration,
            12
        );
    }

    #[test]
    fn m5_goathill_leech_corpses_are_consumable_tonics() {
        let mut game = Game::new();
        game.location = LocationId::from("goathill.cavern1");
        let lethal_state = |npc_id: &str| CombatState {
            enemy: EnemyKind::Npc(NpcId::from(npc_id)),
            health: 0,
            max_health: 1,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        };
        game.win_combat(lethal_state("goathill.npc.worm"));
        let corpse = game.ground_items[&game.location]
            .iter()
            .find(|item| item.item_id.as_str() == GOATHILL_DEAD_LEECH_ID)
            .unwrap()
            .instance_id;
        game.perform(Action::PickUpItem(corpse));
        game.player.food = 0;
        let force_before = game.player.force;
        let max_mana_before = game.player.max_mana;
        game.perform(Action::ConsumeItem(corpse));
        assert_eq!(game.player.force, force_before + 1);
        assert_eq!(game.player.max_mana, max_mana_before + 60);
        assert_eq!(game.player.item(corpse).unwrap().remaining_uses, Some(2));

        let count_corpses = |game: &Game| {
            game.ground_items
                .get(&game.location)
                .into_iter()
                .flatten()
                .filter(|item| item.item_id.as_str() == GOATHILL_DEAD_LEECH_ID)
                .count()
        };
        let corpses_before = count_corpses(&game);
        game.win_combat(lethal_state("goathill.npc.huge_worm"));
        let corpses_after = count_corpses(&game);
        assert_eq!(corpses_after, corpses_before);
    }

    #[test]
    fn m5_death_hooks_drop_grass_and_escalate_police_wanted_level() {
        let lethal_state = |npc_id: &str| CombatState {
            enemy: EnemyKind::Npc(NpcId::from(npc_id)),
            health: 0,
            max_health: 1,
            rounds: 1,
            mode: CombatMode::Lethal,
            attack_bonus: 0,
            dodge_bonus: 0,
            enemy_attack_bonus: 0,
            enemy_busy_rounds: 0,
            technique_cooldown: 0,
            power_up_active: false,
            fake_fault_active: false,
        };
        let mut game = Game::new();

        game.location = LocationId::from("choyin.lionroom");
        game.win_combat(lethal_state(CHOYIN_LION_ID));
        assert!(
            game.ground_items[&game.location]
                .iter()
                .any(|item| item.item_id.as_str() == CHOYIN_GRASS_ID)
        );

        game.location = LocationId::from("choyin.yamen");
        let wanted_before = game.player.wanted;
        game.win_combat(lethal_state(CHOYIN_POLICE_ID));
        assert_eq!(game.player.wanted, wanted_before + 6);
        assert!(game.logs.iter().any(|log| log.contains("通缉额外 +5")));
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
    fn city_commoner_accepts_source_gifts_without_reward() {
        let mut game = Game::new();
        game.location = LocationId::from("city.nroad2");
        let commoner = NpcId::from("u.cp.chater2");
        let rations = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == items::DRY_RATIONS_ID)
            .unwrap()
            .instance_id;
        let gift = Action::GiveItem {
            instance_id: rations,
            npc: commoner,
        };

        assert!(game.available_actions().contains(&gift));
        game.perform(gift);

        assert!(game.player.item(rations).is_none());
        assert!(game.logs.last().unwrap().contains("普通百姓收下"));
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
    fn m4_source_apprenticeship_dispositions_bind_npc_lessons() {
        fn npc_lessons(game: &Game, npc_id: &str) -> Vec<Action> {
            game.available_actions()
                .into_iter()
                .filter(|action| {
                    matches!(action, Action::LearnFromNpc { npc, .. } if npc.as_str() == npc_id)
                })
                .collect()
        }

        let mut game = Game::new();
        game.location = LocationId::from("snow.school2");
        assert!(npc_lessons(&game, SNOW_FIST_TRAINER_ID).is_empty());
        game.player.faction = Some("封山剑派".into());
        let fist_lessons = npc_lessons(&game, SNOW_FIST_TRAINER_ID);
        assert_eq!(fist_lessons.len(), 3);
        assert!(fist_lessons.iter().any(|action| matches!(
            action,
            Action::LearnFromNpc { skill, .. } if skill.as_str() == LIUH_KEN_ID
        )));
        let learned_before = game.player.learned_points;
        game.perform(Action::LearnFromNpc {
            skill: SkillId::from(UNARMED_ID),
            npc: NpcId::from(SNOW_FIST_TRAINER_ID),
        });
        assert_eq!(game.player.learned_points, learned_before + 1);

        game.location = LocationId::from("snow.nyard");
        assert!(npc_lessons(&game, SNOW_GIRL_ID).is_empty());
        game.player.faction = Some("封山剑派北宗".into());
        assert_eq!(npc_lessons(&game, SNOW_GIRL_ID).len(), 10);

        game.location = LocationId::from("city.biaoju");
        assert!(npc_lessons(&game, "city.npc.chen").is_empty());

        game.location = LocationId::from("temple.temple1");
        assert!(npc_lessons(&game, TEMPLE_PROTECTOR_ID).is_empty());
        assert!(npc_lessons(&game, TEMPLE_TRAINER_ID).is_empty());
        game.player.faction = Some("茅山派".into());
        assert_eq!(npc_lessons(&game, TEMPLE_PROTECTOR_ID).len(), 11);
        assert_eq!(npc_lessons(&game, TEMPLE_TRAINER_ID).len(), 12);
        assert!(
            npc_lessons(&game, TEMPLE_TRAINER_ID)
                .iter()
                .any(|action| matches!(
                    action,
                    Action::LearnFromNpc { skill, .. } if skill.as_str() == "scratching"
                ))
        );
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
    fn m4_source_fight_gates_bind_combat_to_the_current_npc() {
        fn fight_action(game: &Game, npc_id: &str) -> Action {
            game.available_actions()
                .into_iter()
                .find(|action| {
                    matches!(action, Action::Fight(EnemyKind::Npc(npc)) if npc.as_str() == npc_id)
                })
                .expect("source fight action must be available")
        }

        let mut game = Game::new();
        game.location = LocationId::from("city.bank");
        game.perform(fight_action(&game, "city.npc.microsof"));
        let combat = match std::mem::replace(&mut game.activity, Activity::Idle) {
            Activity::Fighting(combat) => combat,
            _ => panic!("city banker must force combat"),
        };
        assert_eq!(combat.mode, CombatMode::Lethal);
        assert_eq!(
            combat.enemy,
            EnemyKind::Npc(NpcId::from("city.npc.microsof"))
        );
        game.win_combat(combat);
        assert!(game.available_actions().iter().all(|action| {
            !matches!(action, Action::Talk(npc) | Action::Fight(EnemyKind::Npc(npc)) if npc.as_str() == "city.npc.microsof")
        }));
        let mut restored: Game =
            serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        restored.location = LocationId::from("city.bank");
        assert!(restored.available_actions().iter().all(|action| {
            !matches!(action, Action::Talk(npc) | Action::Fight(EnemyKind::Npc(npc)) if npc.as_str() == "city.npc.microsof")
        }));

        game.location = LocationId::from("snow.bank");
        game.perform(fight_action(&game, "snow.npc.annihir"));
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Lethal,
                enemy: EnemyKind::Npc(ref npc),
                ..
            }) if npc.as_str() == "snow.npc.annihir"
        ));
        game.activity = Activity::Idle;

        game.location = LocationId::from("snow.school2");
        game.perform(fight_action(&game, SNOW_FIST_TRAINER_ID));
        assert_eq!(game.activity, Activity::Idle);
        assert!(game.logs.last().unwrap().contains("不许和来这里的客人过招"));
        game.player.faction = Some("封山剑派".into());
        game.perform(fight_action(&game, SNOW_FIST_TRAINER_ID));
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Spar,
                ..
            })
        ));
        game.perform(Action::Surrender);

        game.location = LocationId::from("snow.nyard");
        game.perform(fight_action(&game, SNOW_GIRL_ID));
        assert_eq!(game.activity, Activity::Idle);
        assert!(game.logs.last().unwrap().contains("李教头"));
        game.perform(Action::Kill(EnemyKind::Npc(NpcId::from(SNOW_GIRL_ID))));
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Lethal,
                ..
            })
        ));
        game.activity = Activity::Idle;

        for (location, npc_id, rejection) in [
            ("city.street8", SNOW_SCAVENGER_ID, "饶命"),
            ("temple.restroom1", TEMPLE_OLD_TAOIST_ID, "年迈力衰"),
        ] {
            game.location = LocationId::from(location);
            game.perform(fight_action(&game, npc_id));
            assert_eq!(game.activity, Activity::Idle);
            assert!(game.logs.last().unwrap().contains(rejection));
        }

        game.location = LocationId::from("temple.temple1");
        game.perform(fight_action(&game, TEMPLE_PROTECTOR_ID));
        assert_eq!(game.activity, Activity::Idle);
        assert!(game.logs.last().unwrap().contains("不和别派"));
        game.player.faction = Some("茅山派".into());
        for npc_id in [TEMPLE_PROTECTOR_ID, TEMPLE_TRAINER_ID] {
            game.perform(fight_action(&game, npc_id));
            assert!(matches!(
                game.activity,
                Activity::Fighting(CombatState {
                    mode: CombatMode::Spar,
                    ..
                })
            ));
            game.perform(Action::Surrender);
        }
    }

    #[test]
    fn every_placed_m4_m5_m6_m7_source_npc_exposes_combat_actions_when_present() {
        let mut game = Game::new();
        let mut placed = HashSet::new();
        for catalog_json in [
            include_str!("../migration/catalog/city.json"),
            include_str!("../migration/catalog/snow.json"),
            include_str!("../migration/catalog/temple.json"),
            include_str!("../migration/catalog/canyon.json"),
            include_str!("../migration/catalog/oldpine.json"),
            include_str!("../migration/catalog/goathill.json"),
            include_str!("../migration/catalog/choyin.json"),
            include_str!("../migration/catalog/chuenyu.json"),
            include_str!("../migration/catalog/green.json"),
            include_str!("../migration/catalog/sanyen.json"),
            include_str!("../migration/catalog/waterfog.json"),
            include_str!("../migration/catalog/latemoon.json"),
            include_str!("../migration/catalog/death.json"),
            include_str!("../migration/catalog/graveyard.json"),
            include_str!("../migration/catalog/jail.json"),
            include_str!("../migration/catalog/cloud.json"),
        ] {
            let catalog: serde_json::Value = serde_json::from_str(catalog_json).unwrap();
            for room in catalog["rooms"].as_array().unwrap() {
                game.location = LocationId::from(room["id"].as_str().unwrap());
                if game.location.as_str() == "chuenyu.dungeon" {
                    game.quest = QuestStage::FoundJuan;
                }
                let actions = game.available_actions();
                for source_path in room["object_sources"].as_array().unwrap() {
                    let Some(npc) = source_path
                        .as_str()
                        .and_then(|source_path| npcs().id_for_source(source_path))
                    else {
                        continue;
                    };
                    placed.insert(npc.clone());
                    assert!(
                        actions.contains(&Action::Fight(EnemyKind::Npc(npc.clone()))),
                        "{} lacks fight at {}",
                        npc.as_str(),
                        game.location.as_str()
                    );
                    assert!(
                        actions.contains(&Action::Kill(EnemyKind::Npc(npc.clone()))),
                        "{} lacks kill at {}",
                        npc.as_str(),
                        game.location.as_str()
                    );
                }
            }
        }
        assert_eq!(placed.len(), 219);
    }

    #[test]
    fn m5_fight_and_kill_gates_preserve_source_outcomes() {
        let room_for = |npc_id: &str| {
            let npc = NpcId::from(npc_id);
            world()
                .locations()
                .find(|location| location.npcs.contains(&npc))
                .unwrap()
                .id
                .clone()
        };
        let mut game = Game::new();

        game.location = room_for(CHOYIN_HOTEL_GUARD_ID);
        game.perform(Action::Fight(EnemyKind::Npc(NpcId::from(
            CHOYIN_HOTEL_GUARD_ID,
        ))));
        assert_eq!(game.activity, Activity::Idle);
        assert!(game.logs.last().unwrap().contains("不准任何人"));
        game.perform(Action::Kill(EnemyKind::Npc(NpcId::from(
            CHOYIN_HOTEL_GUARD_ID,
        ))));
        assert_eq!(game.player.wanted, 1);
        assert!(game.logs.iter().any(|log| log.contains("报官缉拿")));
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Lethal,
                ..
            })
        ));

        game.activity = Activity::Idle;
        game.location = room_for(CHOYIN_MAGISTRATE_ID);
        game.perform(Action::Fight(EnemyKind::Npc(NpcId::from(
            CHOYIN_MAGISTRATE_ID,
        ))));
        assert_eq!(game.activity, Activity::Idle);
        assert!(game.logs.last().unwrap().contains("这是衙门"));

        game.location = room_for(CHOYIN_OLD_MAN_ID);
        game.perform(Action::Fight(EnemyKind::Npc(NpcId::from(
            CHOYIN_OLD_MAN_ID,
        ))));
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                mode: CombatMode::Spar,
                ..
            })
        ));
    }

    #[test]
    fn lethal_npc_defeats_preserve_same_room_counts_and_other_room_instances() {
        let mut game = Game::new();
        let guard = NpcId::from("city.npc.guard");
        game.location = LocationId::from("city.eastdoor1");
        assert_eq!(
            game.current_location()
                .npcs
                .iter()
                .filter(|npc| *npc == &guard)
                .count(),
            3
        );
        for defeated in 1..=3 {
            game.perform(Action::Kill(EnemyKind::Npc(guard.clone())));
            let combat = match std::mem::replace(&mut game.activity, Activity::Idle) {
                Activity::Fighting(combat) => combat,
                _ => panic!("city guard must enter lethal combat"),
            };
            game.win_combat(combat);
            assert_eq!(
                game.ground_items[&LocationId::from("city.eastdoor1")].len(),
                defeated * 3
            );
            assert_eq!(
                game.available_actions()
                    .contains(&Action::Talk(guard.clone())),
                defeated < 3
            );
        }
        let drops = &game.ground_items[&LocationId::from("city.eastdoor1")];
        assert_eq!(
            drops
                .iter()
                .filter(|item| item.item_id.as_str() == "obj.cloth")
                .count(),
            3
        );
        assert_eq!(
            drops
                .iter()
                .filter(|item| item.item_id.as_str() == "obj.longsword")
                .count(),
            3
        );

        assert_eq!(
            drops
                .iter()
                .filter(|item| item.item_id.as_str() == CORPSE_ITEM_ID)
                .count(),
            3
        );

        game.location = LocationId::from("city.eastdoor2");
        assert!(
            game.available_actions()
                .contains(&Action::Talk(guard.clone()))
        );
        let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(restored.available_actions().contains(&Action::Talk(guard)));
    }

    #[test]
    fn source_combat_chat_executes_for_a_hundred_percent_chat_profile() {
        let mut game = Game::new();
        game.location = LocationId::from("temple.temple1");
        game.player.essence = 10_000;
        game.player.qi = 10_000;
        game.player.spirit = 10_000;
        game.perform(Action::Kill(EnemyKind::Npc(NpcId::from(TEMPLE_TRAINER_ID))));
        let mut combat = match std::mem::replace(&mut game.activity, Activity::Idle) {
            Activity::Fighting(combat) => combat,
            _ => panic!("temple trainer must enter combat"),
        };
        let log_count = game.logs.len();
        assert!(!game.run_npc_combat_chat(&mut combat));
        assert_eq!(game.logs.len(), log_count + 1);
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
    fn old_liu_plot_rescues_juan_and_grants_only_source_rewards() {
        let mut game = Game::new();
        game.perform(Action::Talk(NpcId::from(OLD_LIU_ID)));
        assert_eq!(game.quest, QuestStage::FindJuan);

        game.location = LocationId::from(content::PINE_FOREST);
        game.player.strength = 100;
        game.perform(Action::Fight(EnemyKind::Bandit));
        while matches!(game.activity, Activity::Fighting(_)) {
            game.tick();
        }
        assert_eq!(game.quest, QuestStage::FoundJuan);
        assert!(
            game.available_actions()
                .contains(&Action::Talk(NpcId::from(XIAO_JUAN_ID)))
        );
        assert!(
            game.available_actions()
                .contains(&Action::Kill(EnemyKind::XiaoJuan))
        );

        game.perform(Action::Talk(NpcId::from(XIAO_JUAN_ID)));
        assert_eq!(game.quest, QuestStage::ReturnHome);
        let reputation = game.player.reputation;
        let insight = game.player.insight;

        game.location = LocationId::from(content::LIU_HOME);
        game.perform(Action::Talk(NpcId::from(OLD_LIU_ID)));
        assert_eq!(game.quest, QuestStage::Complete);
        assert!(
            game.player
                .has_item(&ItemId::from(items::HENGBING_SWORD_ID))
        );
        assert!(game.player.has_item(&ItemId::from(items::PARRY_MANUAL_ID)));
        assert_eq!(game.player.reputation, reputation);
        assert_eq!(game.player.insight, insight);
        assert!(
            game.available_actions().iter().all(|action| {
                !matches!(action, Action::Talk(npc) if npc.as_str() == OLD_LIU_ID)
            })
        );
    }

    #[test]
    fn murdering_juan_triggers_old_liu_source_revenge() {
        let mut game = Game::new();
        game.perform(Action::Talk(NpcId::from(OLD_LIU_ID)));
        game.location = LocationId::from(content::PINE_FOREST);
        game.player.strength = 100;
        game.perform(Action::Fight(EnemyKind::Bandit));
        while matches!(game.activity, Activity::Fighting(_)) {
            game.tick();
        }

        game.perform(Action::Kill(EnemyKind::XiaoJuan));
        while matches!(game.activity, Activity::Fighting(_)) {
            game.tick();
        }
        assert_eq!(game.quest, QuestStage::MurderedJuan);

        game.move_to(LocationId::from(content::LIU_HOME));
        assert_eq!(game.quest, QuestStage::Failed);
        assert!(matches!(
            game.activity,
            Activity::Fighting(CombatState {
                enemy: EnemyKind::OldLiuRevenge,
                mode: CombatMode::Lethal,
                ..
            })
        ));
        assert!(game.logs.iter().any(|line| line.contains("纳命来")));
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
        let Activity::Fighting(ref combat) = game.activity else {
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
    fn m4_source_room_events_control_passages_resources_and_rewards() {
        let can_reach =
            |game: &Game, target: &str| {
                game.available_actions().iter().any(|action| matches!(
                action,
                Action::Move { target: action_target, .. } if action_target.as_str() == target
            ))
            };
        let mut game = Game::new();

        game.location = LocationId::from(CITY_ALTAR);
        game.perform(Action::Interact(InteractionKind::TurnAltarForward));
        for _ in 0..3 {
            game.perform(Action::Interact(InteractionKind::TurnAltarBackward));
        }
        game.perform(Action::Interact(InteractionKind::PressAltarButton));
        assert!(can_reach(&game, CITY_ALTAR_TUNNEL));
        game.perform(Action::Move {
            direction: "down".into(),
            target: LocationId::from(CITY_ALTAR_TUNNEL),
        });
        for _ in 0..5 {
            game.tick();
        }
        assert!(can_reach(&game, CITY_ALTAR));
        game.perform(Action::Move {
            direction: "up".into(),
            target: LocationId::from(CITY_ALTAR),
        });
        for _ in 0..3 {
            game.tick();
        }
        assert!(!can_reach(&game, CITY_ALTAR_TUNNEL));

        game.location = LocationId::from(SNOW_WEAPON_STORAGE);
        for _ in 0..3 {
            game.perform(Action::Interact(InteractionKind::PushSnowShelf));
        }
        assert!(can_reach(&game, SNOW_SECRET_STORAGE));
        game.perform(Action::Move {
            direction: "down".into(),
            target: LocationId::from(SNOW_SECRET_STORAGE),
        });
        for _ in 0..12 {
            game.tick();
        }
        assert!(can_reach(&game, SNOW_WEAPON_STORAGE));

        game.location = LocationId::from(SNOW_WORKPLACE);
        game.player.essence = 30;
        game.player.spirit = 30;
        let silver_before = game.player.silver;
        game.perform(Action::Interact(InteractionKind::WorkAtSnowWorkshop));
        assert_eq!(game.player.essence, 0);
        assert_eq!(game.player.spirit, 0);
        assert_eq!(game.player.silver, silver_before + 1);

        game.location = LocationId::from(CANYON_BAMBOO_BOULDER);
        game.player.force = 560;
        game.player.max_force = 560;
        game.player
            .skills
            .iter_mut()
            .find(|skill| skill.kind.as_str() == FORCE_ID)
            .unwrap()
            .level = 40;
        game.perform(Action::Interact(InteractionKind::MoveBambooBoulder));
        assert!(can_reach(&game, CANYON_BAMBOO_TRAINING_ROOM));
        game.perform(Action::Move {
            direction: "enter".into(),
            target: LocationId::from(CANYON_BAMBOO_TRAINING_ROOM),
        });
        assert!(!game.canyon_boulder_open);
        game.perform(Action::Interact(InteractionKind::SearchBambooBookcase));
        assert!(game.player.has_item(&ItemId::from(CANYON_SLIPCASE_ID)));
        assert!(game.player.has_item(&ItemId::from(CANYON_PARRY_BOOK_ID)));
        assert!(
            !game
                .available_actions()
                .contains(&Action::Interact(InteractionKind::SearchBambooBookcase))
        );

        game.location = LocationId::from(TEMPLE_SLIPPERY_ROAD);
        game.player.spirituality = 1;
        game.player.essence = 100;
        game.player.qi = 80;
        game.player.spirit = 70;
        game.perform(Action::Move {
            direction: "northwest".into(),
            target: LocationId::from(content::TEMPLE_ROAD_TWO),
        });
        assert_eq!(game.location.as_str(), TEMPLE_SLIPPERY_ROAD);
        assert_eq!(
            (game.player.essence, game.player.qi, game.player.spirit),
            (50, 40, 35)
        );

        game.location = LocationId::from(content::TEMPLE_ROAD_TWO);
        assert!(!can_reach(&game, TEMPLE_BOOK_ROOM));
        game.player.faction = Some("茅山派".into());
        assert!(can_reach(&game, TEMPLE_BOOK_ROOM));
    }

    #[test]
    fn city_exit_token_is_found_exchanged_and_consumed_on_the_live_north_route() {
        let mut game = Game::new();
        game.location = LocationId::from(content::CITY_RUINED_GARDEN);
        game.player.spirituality = 35;
        game.perform(Action::Interact(InteractionKind::SearchCityRuinedGarden));
        let token = game
            .player
            .inventory
            .iter()
            .find(|item| item.item_id.as_str() == CITY_EXIT_TOKEN_ID)
            .unwrap()
            .instance_id;

        game.location = LocationId::from(content::CITY_NORTH_GATE);
        assert!(game.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::CITY_NORTH_ROAD
        )));
        game.perform(Action::GiveItem {
            instance_id: token,
            npc: NpcId::from(crate::npcs::CITY_GUARD_ID),
        });
        assert!(!game.player.has_item(&ItemId::from(CITY_EXIT_TOKEN_ID)));
        assert!(game.city_exit_permit);

        let encoded = serde_json::to_string(&game).unwrap();
        let mut restored: Game = serde_json::from_str(&encoded).unwrap();
        let leave = restored
            .available_actions()
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Move { target, .. } if target.as_str() == content::CITY_NORTH_ROAD
                )
            })
            .unwrap();
        restored.perform(leave);
        assert_eq!(restored.location.as_str(), content::CITY_NORTH_ROAD);
        assert!(!restored.city_exit_permit);

        restored.location = LocationId::from(content::CITY_NORTH_GATE);
        assert!(restored.available_actions().iter().all(|action| !matches!(
            action,
            Action::Move { target, .. } if target.as_str() == content::CITY_NORTH_ROAD
        )));
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
        game.perform(Action::OpenSourceDoor {
            target: LocationId::from(content::CITY_MANOR_YARD),
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
    fn manor_patrols_block_the_inner_road_until_both_are_defeated() {
        let mut game = Game::new();
        game.location = LocationId::from(content::CITY_MANOR_ROAD_TWO);
        let can_move_to = |game: &Game, target: &str| {
            game.available_actions().iter().any(
                |action| matches!(action, Action::Move { target: exit, .. } if exit.as_str() == target),
            )
        };
        assert!(can_move_to(&game, content::CITY_MANOR_YARD));
        assert!(!can_move_to(&game, "city.shangshu.road1"));
        assert!(!can_move_to(&game, "city.shangshu.kefang"));

        for npc_id in [CITY_SHANGSHU_PATROL_ID, CITY_SHANGSHU_PATROL_ELITE_ID] {
            game.defeated_npc_instances.push(DefeatedNpcInstance {
                location: LocationId::from(content::CITY_MANOR_ROAD_TWO),
                npc: NpcId::from(npc_id),
            });
        }
        assert!(can_move_to(&game, "city.shangshu.road1"));
        game.perform(Action::OpenSourceDoor {
            target: LocationId::from("city.shangshu.kefang"),
        });
        assert!(can_move_to(&game, "city.shangshu.kefang"));
    }

    #[test]
    fn temple_library_uses_a_stable_pair_from_the_source_random_guard_families() {
        let mut game = Game::new();
        game.location = LocationId::from(content::TEMPLE_ROAD_TWO);
        for npc_id in [
            crate::npcs::TEMPLE_LIBRARY_GUARD_ID,
            crate::npcs::TEMPLE_LIBRARY_GUARD_PEER_ID,
        ] {
            let npc = NpcId::from(npc_id);
            assert!(game.current_location().npcs.contains(&npc));
            assert!(
                game.available_actions()
                    .contains(&Action::Fight(EnemyKind::Npc(npc.clone())))
            );
            assert!(
                game.available_actions()
                    .contains(&Action::Kill(EnemyKind::Npc(npc)))
            );
        }
    }

    #[test]
    fn snow_temple_donations_reject_worthless_items_and_can_reduce_bellicosity() {
        let mut game = Game::new();
        game.location = LocationId::from("snow.temple");
        game.player.bellicosity = 100;
        game.player.spirituality = 10;
        game.rng_state = 0;

        let token = game.add_inventory_item(ItemId::from(CITY_EXIT_TOKEN_ID), 1);
        game.perform(Action::GiveItem {
            instance_id: token,
            npc: NpcId::from(crate::npcs::SNOW_KEEPER_ID),
        });
        assert!(game.player.item(token).is_some());
        assert_eq!(game.player.bellicosity, 100);

        let lotus = game.add_inventory_item(ItemId::from("snow.npc.obj.ebony_lotus"), 1);
        game.perform(Action::GiveItem {
            instance_id: lotus,
            npc: NpcId::from(crate::npcs::SNOW_KEEPER_ID),
        });
        assert!(game.player.item(lotus).is_none());
        assert!(game.player.bellicosity < 100);
        assert!(game.available_actions().contains(&Action::OfferMoney {
            amount: 1_000,
            npc: NpcId::from(crate::npcs::SNOW_KEEPER_ID),
        }));
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
