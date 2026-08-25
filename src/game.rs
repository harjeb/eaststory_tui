use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    content::{self, world},
    items::{self, EquipmentSlot, ItemId, ItemInstance, LegacyItemKind, items},
};

const LOG_LIMIT: usize = 80;
const SAVE_VERSION: u32 = 4;
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
pub enum SkillKind {
    Unarmed,
    Sword,
    Dodge,
    Breathing,
    Parry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyKind {
    Bandit,
    Wolf,
    TempleDisciple,
    Rat,
    IceDragon,
    Meloner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcKind {
    OldLiu,
    TeaSeller,
    TempleMaster,
    Fisher,
    FlowerGirl,
    FarmWoman,
    Meloner,
    Trader,
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
}

impl ConditionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Bandaged => "包扎",
            Self::SnakePoison => "蛇毒",
            Self::Poison => "中毒",
            Self::Drunk => "醉酒",
            Self::Slumber => "蒙汗药",
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Move {
        direction: String,
        target: LocationId,
    },
    Interact(InteractionKind),
    Talk(NpcKind),
    Train(SkillKind),
    Rest,
    Fight(EnemyKind),
    BuyItem(ItemId),
    SellItem(u64),
    GiveItem {
        instance_id: u64,
        npc: NpcKind,
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
    fn label(self) -> String {
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
            Self::Interact(interaction) => interaction.label(),
            Self::Talk(npc) => format!("与{}交谈", npc.name()),
            Self::Train(skill) => {
                if game.activity == Activity::Training(*skill) {
                    format!("停止修炼{}", skill.name())
                } else {
                    format!("修炼{}", skill.name())
                }
            }
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
            Self::BuyItem(item_id) => {
                let definition = items()
                    .definition(item_id)
                    .expect("shop item must exist in catalog");
                format!(
                    "购买{} · {}",
                    definition.display_name(),
                    format_money(shop_price(item_id))
                )
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
            Self::Interact(interaction) => interaction.detail(),
            Self::Talk(_) => "交谈可能带来线索、奖励或新的武学见闻。",
            Self::Train(_) => "时间会自动推进，持续积累熟练度并消耗精力。",
            Self::Rest => "逐步恢复精、气、神，全部恢复后自动结束。",
            Self::Fight(_) => "比试以一方失去战力为止，不会造成永久死亡。",
            Self::BuyItem(_) => "按原物品价值付款；钱、银、金会自动换算。",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    Resting,
    Training(SkillKind),
    Fighting(CombatState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatState {
    pub enemy: EnemyKind,
    pub health: i32,
    pub max_health: i32,
    pub rounds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub kind: SkillKind,
    pub level: u32,
    pub progress: u32,
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
    pub reputation: i32,
    pub insight: u32,
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
            reputation: 0,
            insight: 0,
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
                Skill::new(SkillKind::Unarmed, 8),
                Skill::new(SkillKind::Sword, 3),
                Skill::new(SkillKind::Dodge, 6),
                Skill::new(SkillKind::Breathing, 5),
                Skill::new(SkillKind::Parry, 4),
            ],
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

fn default_food_capacity() -> i32 {
    DEFAULT_FOOD_CAPACITY
}

fn default_water_capacity() -> i32 {
    DEFAULT_WATER_CAPACITY
}

impl Skill {
    fn new(kind: SkillKind, level: u32) -> Self {
        Self {
            kind,
            level,
            progress: 0,
        }
    }

    pub fn required_progress(&self) -> u32 {
        18 + self.level * 4
    }
}

impl Player {
    pub fn skill(&self, kind: SkillKind) -> &Skill {
        self.skills
            .iter()
            .find(|skill| skill.kind == kind)
            .expect("all core skills exist")
    }

    fn skill_mut(&mut self, kind: SkillKind) -> &mut Skill {
        self.skills
            .iter_mut()
            .find(|skill| skill.kind == kind)
            .expect("all core skills exist")
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
        if matches!(self.activity, Activity::Fighting(_)) {
            return vec![Action::Surrender];
        }

        let mut actions = Vec::new();
        let current = self.current_location();

        if current.id.as_str() == content::MELON_FARM && self.melon_debt {
            return vec![
                Action::Interact(InteractionKind::SettleMelonDebt),
                Action::Fight(EnemyKind::Meloner),
            ];
        }

        for exit in &current.exits {
            let interaction_only = matches!(
                (current.id.as_str(), exit.target.as_str()),
                (content::LAKESIDE, content::LAKE) | (content::LAKE, content::LAKESIDE)
            );
            let dynamic_exit_closed = exit.dynamic
                && !(current.id.as_str() == content::ROAD6 && self.hidden_grass_path_ticks > 0);
            let closed_door = door_for_transition(&current.id, &exit.target)
                .is_some_and(|door| !self.is_door_open(door));
            if world().contains(&exit.target)
                && !interaction_only
                && !dynamic_exit_closed
                && !closed_door
            {
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
            _ => {}
        }

        if let Some(npc) = current.npc {
            actions.push(Action::Talk(npc));
            for &(item_id, _) in shop_stock(npc) {
                actions.push(Action::BuyItem(ItemId::from(item_id)));
            }
        }
        if let Some(skill) = current.training {
            actions.push(Action::Train(skill));
        }
        if current.can_rest {
            actions.push(Action::Rest);
        }
        if let Some(enemy) = current.enemy {
            actions.push(Action::Fight(enemy));
        }

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
            if definition.equipment_slot().is_some()
                && !item.is_broken()
                && !self.player.is_equipped(item.instance_id)
            {
                actions.push(Action::EquipItem(item.instance_id));
            }
            if !self.player.is_equipped(item.instance_id) {
                if current.npc == Some(NpcKind::Trader) && item.unit_value() > 0 {
                    actions.push(Action::SellItem(item.instance_id));
                }
                if let Some(npc) = current.npc {
                    actions.push(Action::GiveItem {
                        instance_id: item.instance_id,
                        npc,
                    });
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
            Action::Interact(interaction) => self.interact(interaction),
            Action::Talk(npc) => self.talk(npc),
            Action::Train(skill) => self.toggle_training(skill),
            Action::Rest => self.toggle_rest(),
            Action::Fight(enemy) => self.start_combat(enemy),
            Action::BuyItem(item_id) => self.buy_item(item_id),
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
        match self.activity {
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
        match self.activity {
            Activity::Idle => "整装待发".into(),
            Activity::Resting => "正在休息".into(),
            Activity::Training(skill) => format!("修炼{}中", skill.name()),
            Activity::Fighting(combat) => format!("与{}交手", combat.enemy.name()),
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

    fn buy_item(&mut self, item_id: ItemId) {
        let Some(npc) = self.current_location().npc else {
            return;
        };
        let Some(&(_, price)) = shop_stock(npc)
            .iter()
            .find(|(stock_id, _)| *stock_id == item_id.as_str())
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

    fn give_item_to_npc(&mut self, instance_id: u64, npc: NpcKind) {
        if self.current_location().npc != Some(npc) || self.player.is_equipped(instance_id) {
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
        let item = self.player.inventory.remove(index);
        let name = item.display_name().to_string();
        let value = item.unit_value().max(0) as u64 * item.quantity as u64;
        if value >= 100 {
            self.player.reputation += 1;
            self.push_log(format!("你把{name}交给{}。评价 +1。", npc.name()));
        } else {
            self.push_log(format!("你把{name}交给{}。", npc.name()));
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
        self.location = target;
        let place = self.current_location();
        self.push_log(format!("你来到{}。{}", place.name, place.arrival));
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

    fn talk(&mut self, npc: NpcKind) {
        self.activity = Activity::Idle;
        match npc {
            NpcKind::OldLiu => match self.quest {
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
                    self.gain_skill_progress(SkillKind::Parry, 40);
                    self.push_log(
                        "刘老农：多谢搭救小女。这口銮鱼衡冰与过招要旨便赠予少侠。".into(),
                    );
                    self.push_log("任务完成：评价 +20，领悟 +30，已装备銮鱼衡冰。".into());
                }
                QuestStage::Complete => {
                    self.push_log("空屋桌上压着一张字条：救命之恩，刘某没齿难忘。".into());
                }
            },
            NpcKind::TeaSeller => {
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
            NpcKind::TempleMaster => {
                let progress = 8 + self.player.perception / 3;
                self.gain_skill_progress(SkillKind::Breathing, progress);
                self.push_log("玄智和尚点出你吐纳中的滞涩之处，内息运转顺畅了许多。".into());
            }
            NpcKind::Fisher => {
                self.push_log("渔夫压低声音：湖里近来常有白光，已经没人敢下水捕鱼了。".into());
            }
            NpcKind::FlowerGirl => {
                self.push_log("采花妞：要找小娟的话，得去黑松山那边问问。".into());
            }
            NpcKind::FarmWoman => {
                self.push_log("农妇叹道：我那七岁的孩子也不见了，听说黑松山常抓小孩子。".into());
            }
            NpcKind::Meloner => {
                self.push_log("瓜农警惕地看着你：想吃瓜就去镇上买，可别打瓜田的主意。".into());
            }
            NpcKind::Trader => {
                self.push_log("关外商人拱手道：北边道路不太平，带足干粮再上路。".into());
            }
        }
    }

    fn toggle_training(&mut self, skill: SkillKind) {
        if self.activity == Activity::Training(skill) {
            self.activity = Activity::Idle;
            self.push_log(format!("你收势调息，结束了{}修炼。", skill.name()));
            return;
        }

        self.activity = Activity::Training(skill);
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

    fn training_tick(&mut self, skill: SkillKind) {
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

    fn start_combat(&mut self, enemy: EnemyKind) {
        if self.player.essence < 20 || self.player.qi < 15 {
            self.push_log("你当前状态太差，无法贸然出手。".into());
            return;
        }

        let max_health = enemy.max_health();
        self.activity = Activity::Fighting(CombatState {
            enemy,
            health: max_health,
            max_health,
            rounds: 0,
        });
        self.push_log(format!("你向{}抱拳示意，双方随即交手。", enemy.name()));
    }

    fn surrender(&mut self) {
        let Activity::Fighting(combat) = self.activity else {
            return;
        };
        self.activity = Activity::Idle;
        self.player.reputation -= 1;
        self.push_log(format!(
            "你跳出战圈，向{}认输。评价 -1。",
            combat.enemy.name()
        ));
    }

    fn combat_tick(&mut self, mut combat: CombatState) {
        combat.rounds += 1;
        let (has_weapon, weapon_bonus) =
            self.player
                .equipped(EquipmentSlot::Weapon)
                .map_or((false, 0), |item| {
                    (
                        true,
                        item.definition().weapon_damage.unwrap_or(0).max(0) / 10,
                    )
                });
        let attack_skill = if has_weapon {
            SkillKind::Sword
        } else {
            SkillKind::Unarmed
        };
        let skill_level = self.player.skill(attack_skill).level;
        let attack = self.player.strength as i32
            + skill_level as i32 / 2
            + weapon_bonus
            + self.random(7) as i32;
        let damage = (attack - combat.enemy.defense()).max(2);
        combat.health -= damage;
        self.player.qi = (self.player.qi - 2).max(0);
        self.gain_skill_progress(attack_skill, 2);
        self.gain_skill_progress(SkillKind::Dodge, 1);
        if has_weapon {
            self.degrade_equipment(EquipmentSlot::Weapon);
        }
        self.push_log(format!(
            "第{}合：你击中{}，造成{}点伤势。",
            combat.rounds,
            combat.enemy.name(),
            damage
        ));

        if combat.health <= 0 {
            self.win_combat(combat.enemy);
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
        let defense = self.player.skill(SkillKind::Dodge).level as i32 / 2
            + self.player.skill(SkillKind::Parry).level as i32 / 3
            + armor_bonus;
        let enemy_attack = combat.enemy.attack() + self.random(6) as i32;
        let received = (enemy_attack - defense).max(2);
        self.player.essence -= received;
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
            "{}反击，你损失{}点精。",
            combat.enemy.name(),
            received
        ));

        if self.player.essence <= 0 {
            self.lose_combat(combat.enemy);
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

    fn win_combat(&mut self, enemy: EnemyKind) {
        self.activity = Activity::Idle;
        let insight = enemy.insight_reward();
        self.player.insight += insight;
        self.player.reputation += enemy.reputation_reward();
        self.push_log(format!(
            "{}失去战力，抱拳认输。领悟 +{}。",
            enemy.name(),
            insight
        ));

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
            _ => {}
        }
    }

    fn lose_combat(&mut self, enemy: EnemyKind) {
        self.activity = Activity::Idle;
        self.location = LocationId::from(content::LIU_HOME);
        let lost = self.player.money_value().min(500);
        self.player
            .set_money_value(self.player.money_value() - lost);
        self.player.essence = self.player.max_essence / 2;
        self.player.qi = self.player.max_qi / 2;
        self.player.spirit = self.player.max_spirit / 2;
        self.push_log(format!(
            "你被{}击昏。醒来时已被路人送回刘家小房，遗失{}。",
            enemy.name(),
            format_money(lost)
        ));
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

    fn gain_skill_progress(&mut self, kind: SkillKind, amount: u32) {
        let mut level_up = None;
        {
            let skill = self.player.skill_mut(kind);
            skill.progress += amount;
            while skill.progress >= skill.required_progress() {
                skill.progress -= skill.required_progress();
                skill.level += 1;
                level_up = Some(skill.level);
            }
        }
        if let Some(level) = level_up {
            self.push_log(format!("你的{}提升到{}层。", kind.name(), level));
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

    pub fn push_log(&mut self, message: String) {
        self.logs.push(message);
        if self.logs.len() > LOG_LIMIT {
            self.logs.drain(0..self.logs.len() - LOG_LIMIT);
        }
    }
}

impl SkillKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Unarmed => "刘氏拳",
            Self::Sword => "六出纷飞剑",
            Self::Dodge => "飘火身法",
            Self::Breathing => "吐纳术",
            Self::Parry => "招架",
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
        }
    }
}

impl NpcKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::OldLiu => "刘老农",
            Self::TeaSeller => "茶摊老板",
            Self::TempleMaster => "玄智和尚",
            Self::Fisher => "渔夫",
            Self::FlowerGirl => "采花妞",
            Self::FarmWoman => "农妇",
            Self::Meloner => "瓜农",
            Self::Trader => "关外商人",
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
    pub npc: Option<NpcKind>,
    pub training: Option<SkillKind>,
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
        npc: Option<NpcKind>,
        training: Option<SkillKind>,
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
            npc,
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

const TRADER_STOCK: [(&str, u64); 3] = [
    (items::DRY_RATIONS_ID, 15),
    ("obj.weapon.dagger", 50),
    ("obj.weapon.longsword", 400),
];
const MELONER_STOCK: [(&str, u64); 1] = [(items::WATER_MELON_ID, 60)];

fn shop_stock(npc: NpcKind) -> &'static [(&'static str, u64)] {
    match npc {
        NpcKind::Trader => &TRADER_STOCK,
        NpcKind::Meloner => &MELONER_STOCK,
        _ => &[],
    }
}

fn shop_price(item_id: &ItemId) -> u64 {
    TRADER_STOCK
        .iter()
        .chain(MELONER_STOCK.iter())
        .find(|(stock_id, _)| *stock_id == item_id.as_str())
        .map_or(0, |(_, price)| *price)
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
    fn trader_buys_and_sells_catalog_items() {
        let mut game = Game::new();
        game.location = LocationId::from("village.road2");
        let before = game.player.money_value();
        let dagger_id = ItemId::from("obj.weapon.dagger");

        game.perform(Action::BuyItem(dagger_id.clone()));
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
            npc: NpcKind::OldLiu,
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
        game.perform(Action::Train(SkillKind::Breathing));
        let before = game.player.skill(SkillKind::Breathing).progress;
        game.tick();
        assert!(game.player.essence < game.player.max_essence);
        assert!(game.player.skill(SkillKind::Breathing).progress > before);
    }

    #[test]
    fn quest_advances_through_conversation_and_rescue() {
        let mut game = Game::new();
        game.perform(Action::Talk(NpcKind::OldLiu));
        assert_eq!(game.quest, QuestStage::FindJuan);

        game.location = LocationId::from(content::PINE_FOREST);
        game.player.strength = 100;
        game.perform(Action::Fight(EnemyKind::Bandit));
        while matches!(game.activity, Activity::Fighting(_)) {
            game.tick();
        }
        assert_eq!(game.quest, QuestStage::ReturnHome);

        game.location = LocationId::from(content::LIU_HOME);
        game.perform(Action::Talk(NpcKind::OldLiu));
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
    fn surrender_ends_combat() {
        let mut game = Game::new();
        game.location = LocationId::from(content::PINE_FOREST);
        game.perform(Action::Fight(EnemyKind::Bandit));
        game.perform(Action::Surrender);
        assert_eq!(game.activity, Activity::Idle);
        assert_eq!(game.player.reputation, -1);
    }
}
