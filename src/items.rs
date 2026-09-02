use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Deserializer, Serialize, de::Error};

const CATALOG_JSON: &str = include_str!("../migration/catalog/items.json");

pub const CLOTH_ID: &str = "obj.cloth";
pub const DRY_RATIONS_ID: &str = "solo.dry_rations";
pub const HENGBING_SWORD_ID: &str = "village.npc.obj.hengbing";
pub const PARRY_MANUAL_ID: &str = "village.npc.obj.parrybook";
pub const WOLF_PELT_ID: &str = "solo.wolf_pelt";
pub const WATER_MELON_ID: &str = "village.obj.melon";
pub const BANDAGE_ID: &str = "obj.bandage";
pub const WOUND_MEDICINE_ID: &str = "obj.drug.hurt_drug";
pub const SNAKE_MEDICINE_ID: &str = "obj.drug.snake_drug";
pub const SLUMBER_DRUG_ID: &str = "obj.slumber_drug";
pub const POISON_DUST_ID: &str = "obj.toy.poison_dust";
pub const CHUENYU_PIGMEAT_IDS: [&str; 2] = ["chuenyu.npc.obj.pigmeat", "chuenyu.obj.pigmeat"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TalismanKind {
    Haunt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalismanInscription {
    pub kind: TalismanKind,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId(String);
impl ItemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<&str> for ItemId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemCategory {
    Armor,
    Combined,
    Food,
    Item,
    Liquid,
    Medicine,
    Money,
    Weapon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Head,
    Torso,
    Feet,
    Waist,
    Neck,
    Surcoat,
    Shield,
    Wrists,
    Finger,
    Hands,
    Accessory,
}
impl EquipmentSlot {
    pub fn name(self) -> &'static str {
        match self {
            Self::Weapon => "武器",
            Self::Head => "头部",
            Self::Torso => "躯干",
            Self::Feet => "脚部",
            Self::Waist => "腰部",
            Self::Neck => "颈部",
            Self::Surcoat => "外袍",
            Self::Shield => "盾牌",
            Self::Wrists => "腕部",
            Self::Finger => "手指",
            Self::Hands => "手部",
            Self::Accessory => "饰品",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    pub id: ItemId,
    pub source_path: String,
    pub status: String,
    pub category: ItemCategory,
    pub inherited: Vec<String>,
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub material: Option<String>,
    pub weight: Option<i32>,
    pub value: Option<i32>,
    pub weapon_damage: Option<i32>,
    pub armor: Option<i32>,
    pub food_supply: Option<i32>,
    pub food_remaining: Option<i32>,
    pub water_supply: Option<i32>,
    pub max_liquid: Option<i32>,
    pub liquid_remaining: Option<i32>,
    pub liquid_type: Option<String>,
    pub liquid_name: Option<String>,
    pub drunk_apply: Option<i32>,
    pub study_skill: Option<String>,
    pub study_exp_required: Option<i32>,
    pub study_spirit_cost: Option<i32>,
    pub study_difficulty: Option<i32>,
    pub study_max_level: Option<i32>,
    pub behavior_flags: Vec<String>,
}
impl ItemDefinition {
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            self.id.as_str()
        } else {
            &self.name
        }
    }
    pub fn equipment_slot(&self) -> Option<EquipmentSlot> {
        if self.category == ItemCategory::Weapon || self.is_food_weapon() {
            return Some(EquipmentSlot::Weapon);
        }
        self.inherited
            .iter()
            .find_map(|marker| match marker.as_str() {
                "HEAD" => Some(EquipmentSlot::Head),
                "CLOTH" | "ARMOR" => Some(EquipmentSlot::Torso),
                "BOOTS" => Some(EquipmentSlot::Feet),
                "WAIST" => Some(EquipmentSlot::Waist),
                "NECK" => Some(EquipmentSlot::Neck),
                "SURCOAT" => Some(EquipmentSlot::Surcoat),
                "SHIELD" => Some(EquipmentSlot::Shield),
                "WRISTS" => Some(EquipmentSlot::Wrists),
                "FINGER" => Some(EquipmentSlot::Finger),
                "HANDS" => Some(EquipmentSlot::Hands),
                "EQUIP" => Some(EquipmentSlot::Accessory),
                _ => None,
            })
    }
    pub fn unit_weight(&self) -> u32 {
        self.weight.unwrap_or(0).max(0) as u32
    }
    pub fn stackable(&self) -> bool {
        matches!(self.category, ItemCategory::Money | ItemCategory::Combined)
            || self.id.as_str() == DRY_RATIONS_ID
    }
    pub fn max_durability(&self) -> Option<u32> {
        self.equipment_slot().map(|_| 100)
    }
    pub fn weapon_skill(&self) -> Option<&'static str> {
        if self.category != ItemCategory::Weapon && !self.is_food_weapon() {
            return None;
        }
        self.inherited
            .iter()
            .find_map(|marker| match marker.as_str() {
                "AXE" => Some("axe"),
                "BLADE" => Some("blade"),
                "DAGGER" => Some("dagger"),
                "FORK" => Some("fork"),
                "HAMMER" => Some("hammer"),
                "STAFF" => Some("staff"),
                "SWORD" => Some("sword"),
                "THROWING" => Some("throwing"),
                "WHIP" => Some("whip"),
                _ => None,
            })
            .or(Some("sword"))
    }
    fn is_food_weapon(&self) -> bool {
        self.category == ItemCategory::Food
            && self.weapon_damage.is_some()
            && self.inherited.iter().any(|marker| {
                matches!(
                    marker.as_str(),
                    "AXE"
                        | "BLADE"
                        | "DAGGER"
                        | "FORK"
                        | "HAMMER"
                        | "STAFF"
                        | "SWORD"
                        | "THROWING"
                        | "WHIP"
                )
            })
    }

    pub fn initial_uses(&self) -> Option<u32> {
        if self.stackable() && self.id.as_str() == DRY_RATIONS_ID {
            return None;
        }
        if self.food_supply.is_some() {
            return Some(self.food_remaining.unwrap_or(1).max(1) as u32);
        }
        if self.category == ItemCategory::Liquid {
            return Some(
                self.liquid_remaining
                    .or(self.max_liquid)
                    .unwrap_or(1)
                    .max(0) as u32,
            );
        }
        match self.id.as_str() {
            "obj.bandage" => Some(2),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    source_commit: String,
    items: Vec<ItemDefinition>,
}
static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    let catalog: Catalog =
        serde_json::from_str(CATALOG_JSON).expect("invalid embedded item catalog");
    assert_eq!(catalog.schema_version, 3, "unsupported item catalog schema");
    assert_eq!(catalog.items.len(), 451, "unexpected item catalog count");
    catalog
});

pub struct ItemRepository {
    definitions: HashMap<ItemId, ItemDefinition>,
    source_count: usize,
    source_commit: String,
}
impl ItemRepository {
    fn load() -> Self {
        let mut definitions = CATALOG
            .items
            .iter()
            .cloned()
            .map(|item| (item.id.clone(), item))
            .collect::<HashMap<_, _>>();
        for (id, name) in [
            ("choyin.npc.obj.book", "「嘉德文选」"),
            ("choyin.npc.obj.book1", "「笑傲江湖」"),
            ("choyin.npc.obj.book2", "「秘传八步赶蝉」"),
            ("choyin.obj.book", "「嘉德文选」"),
            ("choyin.npc.obj.denotation", "功德箱"),
            ("choyin.obj.denotation", "功德箱"),
        ] {
            definitions
                .get_mut(&ItemId::from(id))
                .expect("M5 item override must exist")
                .name = name.into();
        }
        definitions.insert(
            ItemId::from(DRY_RATIONS_ID),
            ItemDefinition {
                id: ItemId::from(DRY_RATIONS_ID),
                source_path: "adapted".into(),
                status: "adapted".into(),
                category: ItemCategory::Food,
                inherited: vec!["ITEM".into()],
                name: "干粮".into(),
                description: None,
                unit: Some("份".into()),
                material: None,
                weight: Some(500),
                value: Some(15),
                weapon_damage: None,
                armor: None,
                food_supply: Some(20),
                food_remaining: Some(1),
                water_supply: None,
                max_liquid: None,
                liquid_remaining: None,
                liquid_type: None,
                liquid_name: None,
                drunk_apply: None,
                study_skill: None,
                study_exp_required: None,
                study_spirit_cost: None,
                study_difficulty: None,
                study_max_level: None,
                behavior_flags: vec![],
            },
        );
        definitions.insert(
            ItemId::from(WOLF_PELT_ID),
            ItemDefinition {
                id: ItemId::from(WOLF_PELT_ID),
                source_path: "adapted".into(),
                status: "adapted".into(),
                category: ItemCategory::Item,
                inherited: vec!["ITEM".into()],
                name: "狼皮".into(),
                description: None,
                unit: Some("张".into()),
                material: None,
                weight: Some(1800),
                value: Some(80),
                weapon_damage: None,
                armor: None,
                food_supply: None,
                food_remaining: None,
                water_supply: None,
                max_liquid: None,
                liquid_remaining: None,
                liquid_type: None,
                liquid_name: None,
                drunk_apply: None,
                study_skill: None,
                study_exp_required: None,
                study_spirit_cost: None,
                study_difficulty: None,
                study_max_level: None,
                behavior_flags: vec![],
            },
        );
        Self {
            definitions,
            source_count: CATALOG.items.len(),
            source_commit: CATALOG.source_commit.clone(),
        }
    }
    pub fn definition(&self, id: &ItemId) -> Option<&ItemDefinition> {
        self.definitions.get(id)
    }
    pub fn id_for_source(&self, source_path: &str) -> Option<&ItemId> {
        self.definitions
            .values()
            .find(|item| item.source_path == source_path)
            .map(|item| &item.id)
    }
    pub fn contains(&self, id: &ItemId) -> bool {
        self.definitions.contains_key(id)
    }
    pub fn len(&self) -> usize {
        self.definitions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
    pub fn source_item_count(&self) -> usize {
        self.source_count
    }
    pub fn source_commit(&self) -> &str {
        &self.source_commit
    }
}
static ITEMS: LazyLock<ItemRepository> = LazyLock::new(ItemRepository::load);
pub fn items() -> &'static ItemRepository {
    &ITEMS
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemInstance {
    pub instance_id: u64,
    pub item_id: ItemId,
    pub quantity: u32,
    pub durability: Option<u32>,
    pub remaining_uses: Option<u32>,
    pub transformed_name: Option<String>,
    pub transformed_weight: Option<u32>,
    pub transformed_value: Option<i32>,
    pub slumber_effect: u32,
    pub filled_with_water: bool,
    pub expires_at_elapsed_minutes: Option<u64>,
    pub lifecycle_stage: u8,
    #[serde(default)]
    pub contents: Vec<ItemInstance>,
    #[serde(default)]
    pub container_capacity: Option<u32>,
    #[serde(default)]
    pub talisman: Option<TalismanInscription>,
}
impl ItemInstance {
    pub fn new(instance_id: u64, item_id: ItemId, quantity: u32) -> Self {
        assert!(quantity >= 1, "item quantity must be positive");
        let definition = items().definition(&item_id);
        let durability = definition.and_then(ItemDefinition::max_durability);
        let remaining_uses = definition.and_then(ItemDefinition::initial_uses);
        Self {
            instance_id,
            item_id,
            quantity,
            durability,
            remaining_uses,
            transformed_name: None,
            transformed_weight: None,
            transformed_value: None,
            slumber_effect: 0,
            filled_with_water: false,
            expires_at_elapsed_minutes: None,
            lifecycle_stage: 0,
            contents: Vec::new(),
            container_capacity: None,
            talisman: None,
        }
    }
    pub fn definition(&self) -> &'static ItemDefinition {
        items()
            .definition(&self.item_id)
            .expect("runtime item ID must exist in the item catalog")
    }
    pub fn display_name(&self) -> &str {
        self.transformed_name
            .as_deref()
            .unwrap_or_else(|| self.definition().display_name())
    }
    pub fn total_weight(&self) -> u32 {
        let own_weight = self
            .transformed_weight
            .unwrap_or_else(|| self.definition().unit_weight())
            .saturating_mul(self.quantity);
        own_weight.saturating_add(self.contents_weight())
    }
    pub fn contents_weight(&self) -> u32 {
        self.contents.iter().map(ItemInstance::total_weight).sum()
    }
    pub fn container_capacity(&self) -> Option<u32> {
        self.container_capacity
            .or_else(|| match self.item_id.as_str() {
                "canyon.bamboo.obj.slipcase" => Some(5_000),
                "choyin.npc.obj.silk_bag" | "choyin.obj.silk_bag" => Some(3_000),
                "latemoon.room.npc.obj.token" => Some(3_000),
                "choyin.npc.obj.denotation"
                | "choyin.obj.denotation"
                | "snow.npc.obj.denotation"
                | "snow.obj.denotation" => Some(10_000),
                _ => None,
            })
    }
    pub fn is_container(&self) -> bool {
        self.container_capacity().is_some()
    }
    pub fn contains_instance_id(&self, instance_id: u64) -> bool {
        self.instance_id == instance_id
            || self
                .contents
                .iter()
                .any(|item| item.contains_instance_id(instance_id))
    }
    pub fn max_nested_instance_id(&self) -> u64 {
        self.contents
            .iter()
            .map(ItemInstance::max_nested_instance_id)
            .fold(self.instance_id, u64::max)
    }
    pub fn find_nested(&self, instance_id: u64) -> Option<&ItemInstance> {
        if self.instance_id == instance_id {
            return Some(self);
        }
        self.contents
            .iter()
            .find_map(|item| item.find_nested(instance_id))
    }
    pub fn find_nested_mut(&mut self, instance_id: u64) -> Option<&mut ItemInstance> {
        if self.instance_id == instance_id {
            return Some(self);
        }
        self.contents
            .iter_mut()
            .find_map(|item| item.find_nested_mut(instance_id))
    }
    pub fn split_off(&mut self, instance_id: u64, quantity: u32) -> Option<ItemInstance> {
        if quantity == 0
            || quantity >= self.quantity
            || !self.contents.is_empty()
            || self.talisman.is_some()
        {
            return None;
        }
        self.quantity -= quantity;
        let mut split = self.clone();
        split.instance_id = instance_id;
        split.quantity = quantity;
        Some(split)
    }
    pub fn unit_value(&self) -> i32 {
        self.transformed_value
            .unwrap_or_else(|| self.definition().value.unwrap_or(0))
    }
    pub fn is_broken(&self) -> bool {
        self.durability == Some(0)
    }
    pub fn has_uses_left(&self) -> bool {
        self.remaining_uses != Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) enum LegacyItemKind {
    Cloth,
    DryRations,
    HengbingSword,
    ParryManual,
    WolfPelt,
    WaterMelon,
}
impl LegacyItemKind {
    pub(crate) fn item_id(&self) -> ItemId {
        ItemId::from(match self {
            Self::Cloth => CLOTH_ID,
            Self::DryRations => DRY_RATIONS_ID,
            Self::HengbingSword => HENGBING_SWORD_ID,
            Self::ParryManual => PARRY_MANUAL_ID,
            Self::WolfPelt => WOLF_PELT_ID,
            Self::WaterMelon => WATER_MELON_ID,
        })
    }
}

impl<'de> Deserialize<'de> for ItemInstance {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Current {
            instance_id: u64,
            item_id: ItemId,
            quantity: u32,
            durability: Option<u32>,
            #[serde(default)]
            remaining_uses: Option<u32>,
            #[serde(default)]
            transformed_name: Option<String>,
            #[serde(default)]
            transformed_weight: Option<u32>,
            #[serde(default)]
            transformed_value: Option<i32>,
            #[serde(default)]
            slumber_effect: u32,
            #[serde(default)]
            filled_with_water: bool,
            #[serde(default)]
            expires_at_elapsed_minutes: Option<u64>,
            #[serde(default)]
            lifecycle_stage: u8,
            #[serde(default)]
            contents: Vec<ItemInstance>,
            #[serde(default)]
            container_capacity: Option<u32>,
            #[serde(default)]
            talisman: Option<TalismanInscription>,
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input {
            Current(Current),
            Legacy(LegacyItemKind),
        }
        match Input::deserialize(deserializer)? {
            Input::Current(v) => {
                if v.quantity < 1 {
                    return Err(D::Error::custom("item quantity must be positive"));
                }
                Ok(Self {
                    instance_id: v.instance_id,
                    item_id: v.item_id,
                    quantity: v.quantity,
                    durability: v.durability,
                    remaining_uses: v.remaining_uses,
                    transformed_name: v.transformed_name,
                    transformed_weight: v.transformed_weight,
                    transformed_value: v.transformed_value,
                    slumber_effect: v.slumber_effect,
                    filled_with_water: v.filled_with_water,
                    expires_at_elapsed_minutes: v.expires_at_elapsed_minutes,
                    lifecycle_stage: v.lifecycle_stage,
                    contents: v.contents,
                    container_capacity: v.container_capacity,
                    talisman: v.talisman,
                })
            }
            Input::Legacy(v) => Ok(Self::new(0, v.item_id(), 1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_complete() {
        assert_eq!(items().source_item_count(), 451);
        assert_eq!(items().definitions.len(), 453);
        let ids = CATALOG
            .items
            .iter()
            .map(|x| x.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), 451);
        for id in [CLOTH_ID, HENGBING_SWORD_ID, PARRY_MANUAL_ID, WATER_MELON_ID] {
            assert!(items().contains(&ItemId::from(id)));
        }
    }
    #[test]
    fn slots_and_adaptations() {
        assert_eq!(
            items()
                .definition(&ItemId::from(HENGBING_SWORD_ID))
                .unwrap()
                .equipment_slot(),
            Some(EquipmentSlot::Weapon)
        );
        assert_eq!(
            items()
                .definition(&ItemId::from(DRY_RATIONS_ID))
                .unwrap()
                .food_supply,
            Some(20)
        );
        assert_eq!(
            items()
                .definition(&ItemId::from(WOLF_PELT_ID))
                .unwrap()
                .weight,
            Some(1800)
        );
    }
    #[test]
    fn shared_behavior_ledger_covers_every_dynamic_obj_item() {
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("../migration/overrides/items.json")).unwrap();
        assert_eq!(ledger["source_commit"], CATALOG.source_commit);
        let actual = ledger["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let mut expected = CATALOG
            .items
            .iter()
            .filter(|item| {
                item.source_path.starts_with("mudlib/obj/") && !item.behavior_flags.is_empty()
            })
            .map(|item| item.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        expected.insert(SLUMBER_DRUG_ID);
        assert_eq!(actual, expected);
        for status in ["verified", "adapted", "deferred", "excluded"] {
            assert_eq!(
                ledger["entries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|entry| entry["status"] == status)
                    .count() as u64,
                ledger["summary"][status].as_u64().unwrap()
            );
        }
        assert!(ledger["entries"].as_array().unwrap().iter().all(|entry| {
            matches!(
                entry["status"].as_str(),
                Some("verified" | "adapted" | "deferred" | "excluded")
            )
        }));
    }

    #[test]
    fn instance_behavior_and_legacy() {
        let x = ItemInstance::new(4, ItemId::from(DRY_RATIONS_ID), 3);
        assert_eq!(x.total_weight(), 1500);
        assert_eq!(x.durability, None);
        assert_eq!(x.remaining_uses, None);
        assert_eq!(x.expires_at_elapsed_minutes, None);
        assert_eq!(x.lifecycle_stage, 0);
        let melon = ItemInstance::new(5, ItemId::from(WATER_MELON_ID), 1);
        assert_eq!(melon.remaining_uses, Some(8));
        let wineskin = ItemInstance::new(6, ItemId::from("obj.example.wineskin"), 1);
        assert_eq!(wineskin.remaining_uses, Some(15));
        let y: ItemInstance = serde_json::from_str("\"Cloth\"").unwrap();
        assert_eq!(y.item_id.as_str(), CLOTH_ID);
        assert_eq!(y.instance_id, 0);
    }

    #[test]
    fn nested_container_contents_round_trip_and_contribute_weight() {
        let mut container = ItemInstance::new(40, ItemId::from(CLOTH_ID), 1);
        container.container_capacity = Some(10_000);
        let rations = ItemInstance::new(41, ItemId::from(DRY_RATIONS_ID), 2);
        let expected_weight = container.definition().unit_weight() + rations.total_weight();
        container.contents.push(rations);

        let encoded = serde_json::to_string(&container).unwrap();
        let restored: ItemInstance = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.total_weight(), expected_weight);
        assert_eq!(restored.contents.len(), 1);
        assert_eq!(restored.contents[0].quantity, 2);
        assert_eq!(restored.max_nested_instance_id(), 41);
    }
}
