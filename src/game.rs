use serde::{Deserialize, Serialize};

use crate::content::{self, world};

const LOG_LIMIT: usize = 80;
const SAVE_VERSION: u32 = 2;

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
pub enum ItemKind {
    Cloth,
    DryRations,
    HengbingSword,
    ParryManual,
    WolfPelt,
    WaterMelon,
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
            Self::SettleMelonDebt => "支付 6 两瓜钱，瓜农便会让开道路。",
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
    pub skills: Vec<Skill>,
    pub inventory: Vec<ItemKind>,
    pub weapon: Option<ItemKind>,
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
            skills: vec![
                Skill::new(SkillKind::Unarmed, 8),
                Skill::new(SkillKind::Sword, 3),
                Skill::new(SkillKind::Dodge, 6),
                Skill::new(SkillKind::Breathing, 5),
                Skill::new(SkillKind::Parry, 4),
            ],
            inventory: vec![ItemKind::Cloth, ItemKind::DryRations],
            weapon: None,
        }
    }
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
            Action::Surrender => self.surrender(),
        }
    }

    pub fn tick(&mut self) {
        self.elapsed_minutes += 10;
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
        self.player
            .inventory
            .iter()
            .map(|item| {
                let marker = if self.player.weapon == Some(*item) {
                    " [装备]"
                } else {
                    ""
                };
                format!("• {}{}", item.name(), marker)
            })
            .collect()
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

        self.player.inventory.push(ItemKind::WaterMelon);
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
        const MELON_PRICE: u32 = 6;
        if self.player.silver >= MELON_PRICE {
            self.player.silver -= MELON_PRICE;
            self.melon_debt = false;
            self.push_log("你付了 6 两瓜钱。瓜农收钱后让开道路，返回了瓜棚。".into());
        } else if let Some(index) = self
            .player
            .inventory
            .iter()
            .position(|item| *item == ItemKind::WaterMelon)
        {
            self.player.inventory.remove(index);
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
                    self.player.inventory.push(ItemKind::HengbingSword);
                    self.player.inventory.push(ItemKind::ParryManual);
                    self.player.weapon = Some(ItemKind::HengbingSword);
                    self.gain_skill_progress(SkillKind::Parry, 40);
                    self.push_log("刘老农：多谢搭救小女。这口衡柄剑与格挡要诀便赠予少侠。".into());
                    self.push_log("任务完成：评价 +20，领悟 +30，已装备衡柄剑。".into());
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
        let attack_skill = if self.player.weapon.is_some() {
            SkillKind::Sword
        } else {
            SkillKind::Unarmed
        };
        let skill_level = self.player.skill(attack_skill).level;
        let weapon_bonus = if self.player.weapon == Some(ItemKind::HengbingSword) {
            7
        } else {
            0
        };
        let attack = self.player.strength as i32
            + skill_level as i32 / 2
            + weapon_bonus
            + self.random(7) as i32;
        let damage = (attack - combat.enemy.defense()).max(2);
        combat.health -= damage;
        self.player.qi = (self.player.qi - 2).max(0);
        self.gain_skill_progress(attack_skill, 2);
        self.gain_skill_progress(SkillKind::Dodge, 1);
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

        let defense = self.player.skill(SkillKind::Dodge).level as i32 / 2
            + self.player.skill(SkillKind::Parry).level as i32 / 3;
        let enemy_attack = combat.enemy.attack() + self.random(6) as i32;
        let received = (enemy_attack - defense).max(2);
        self.player.essence -= received;
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
                self.player.silver += 12;
                self.push_log("你赶走山贼，在林间找到了受惊的娟儿，并护送她离开松林。".into());
                self.push_log("任务更新：回刘家小房报平安。碎银 +12。".into());
            }
            EnemyKind::Wolf => {
                if !self.player.inventory.contains(&ItemKind::WolfPelt) {
                    self.player.inventory.push(ItemKind::WolfPelt);
                    self.push_log("你收下一张完整的狼皮。".into());
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
        let lost = self.player.silver.min(5);
        self.player.silver -= lost;
        self.player.essence = self.player.max_essence / 2;
        self.player.qi = self.player.max_qi / 2;
        self.player.spirit = self.player.max_spirit / 2;
        self.push_log(format!(
            "你被{}击昏。醒来时已被路人送回刘家小房，遗失碎银{}两。",
            enemy.name(),
            lost
        ));
    }

    fn recover(&mut self, essence: i32, qi: i32, spirit: i32) {
        self.player.essence = (self.player.essence + essence).min(self.player.max_essence);
        self.player.qi = (self.player.qi + qi).min(self.player.max_qi);
        self.player.spirit = (self.player.spirit + spirit).min(self.player.max_spirit);
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

impl ItemKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Cloth => "粗布衣",
            Self::DryRations => "干粮",
            Self::HengbingSword => "衡柄剑",
            Self::ParryManual => "格挡要诀",
            Self::WolfPelt => "狼皮",
            Self::WaterMelon => "西瓜",
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
        let silver_before = game.player.silver;

        game.perform(Action::Interact(InteractionKind::PickMelon));
        assert!(game.player.inventory.contains(&ItemKind::WaterMelon));
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
        assert_eq!(game.player.silver, silver_before - 6);
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
        assert_eq!(game.player.weapon, Some(ItemKind::HengbingSword));
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
