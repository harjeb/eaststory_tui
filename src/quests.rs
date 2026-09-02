use std::sync::LazyLock;

pub const TASK_TIERS: [u64; 15] = [
    1_000, 1_500, 2_000, 3_000, 5_000, 8_000, 10_000, 13_000, 17_000, 22_000, 40_000, 50_000,
    60_000, 80_000, 100_000,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    Kill,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDefinition {
    pub tier: u64,
    pub target: String,
    pub kind: TaskKind,
    pub time_seconds: u64,
    pub exp_bonus: u32,
    pub potential_bonus: u32,
    pub score_bonus: i32,
}

const TASK_SOURCES: [(u64, &str); 15] = [
    (1_000, include_str!("../es2-utf8/mudlib/quest/qlist1000.c")),
    (1_500, include_str!("../es2-utf8/mudlib/quest/qlist1500.c")),
    (2_000, include_str!("../es2-utf8/mudlib/quest/qlist2000.c")),
    (3_000, include_str!("../es2-utf8/mudlib/quest/qlist3000.c")),
    (5_000, include_str!("../es2-utf8/mudlib/quest/qlist5000.c")),
    (8_000, include_str!("../es2-utf8/mudlib/quest/qlist8000.c")),
    (
        10_000,
        include_str!("../es2-utf8/mudlib/quest/qlist10000.c"),
    ),
    (
        13_000,
        include_str!("../es2-utf8/mudlib/quest/qlist13000.c"),
    ),
    (
        17_000,
        include_str!("../es2-utf8/mudlib/quest/qlist17000.c"),
    ),
    (
        22_000,
        include_str!("../es2-utf8/mudlib/quest/qlist22000.c"),
    ),
    (
        40_000,
        include_str!("../es2-utf8/mudlib/quest/qlist40000.c"),
    ),
    (
        50_000,
        include_str!("../es2-utf8/mudlib/quest/qlist50000.c"),
    ),
    (
        60_000,
        include_str!("../es2-utf8/mudlib/quest/qlist60000.c"),
    ),
    (
        80_000,
        include_str!("../es2-utf8/mudlib/quest/qlist80000.c"),
    ),
    (
        100_000,
        include_str!("../es2-utf8/mudlib/quest/qlist100000.c"),
    ),
];

static TASK_DEFINITIONS: LazyLock<Vec<TaskDefinition>> = LazyLock::new(|| {
    TASK_SOURCES
        .into_iter()
        .flat_map(|(tier, source)| parse_task_source(tier, source))
        .collect()
});

pub fn all_task_definitions() -> &'static [TaskDefinition] {
    TASK_DEFINITIONS.as_slice()
}

pub fn task_definitions_for_tier(
    tier: u64,
) -> impl Iterator<Item = &'static TaskDefinition> + 'static {
    all_task_definitions()
        .iter()
        .filter(move |definition| definition.tier == tier)
}

pub fn task_tier_for(combat_experience: u64, finished_streak: i32) -> u64 {
    let base_index = TASK_TIERS
        .iter()
        .rposition(|tier| *tier <= combat_experience)
        .unwrap_or(0);
    let adjustment = finished_streak / 3;
    let adjusted = (base_index as i32 + adjustment).clamp(0, TASK_TIERS.len() as i32 - 1);
    TASK_TIERS[adjusted as usize]
}

fn parse_task_source(tier: u64, source: &str) -> Vec<TaskDefinition> {
    let source = strip_block_comments(source);
    source
        .split("([")
        .skip(1)
        .filter_map(|mapping| mapping.split_once("])").map(|(mapping, _)| mapping))
        .filter(|mapping| mapping.contains("\"quest\""))
        .map(|mapping| {
            let target = quoted_field(mapping, "quest")
                .unwrap_or_else(|| panic!("quest table {tier} is missing quest target"));
            let kind = match quoted_field(mapping, "quest_type")
                .unwrap_or_else(|| panic!("quest table {tier} is missing quest type"))
            {
                "杀" => TaskKind::Kill,
                kind => panic!("quest table {tier} has unsupported quest type {kind}"),
            };
            TaskDefinition {
                tier,
                target: target.to_string(),
                kind,
                time_seconds: integer_field(mapping, "time", tier),
                exp_bonus: integer_field(mapping, "exp_bonus", tier) as u32,
                potential_bonus: integer_field(mapping, "pot_bonus", tier) as u32,
                score_bonus: integer_field(mapping, "score", tier) as i32,
            }
        })
        .collect()
}

fn strip_block_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut remaining = source;
    while let Some(start) = remaining.find("/*") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("*/") else {
            panic!("unterminated block comment in quest source");
        };
        remaining = &after_start[end + 2..];
    }
    result.push_str(remaining);
    result
}

fn quoted_field<'a>(mapping: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\"");
    let (_, after_key) = mapping.split_once(&marker)?;
    let (_, after_colon) = after_key.split_once(':')?;
    let value = after_colon.trim_start().strip_prefix('"')?;
    value.split_once('"').map(|(value, _)| value)
}

fn integer_field(mapping: &str, key: &str, tier: u64) -> u64 {
    let marker = format!("\"{key}\"");
    let (_, after_key) = mapping
        .split_once(&marker)
        .unwrap_or_else(|| panic!("quest table {tier} is missing {key}"));
    let (_, after_colon) = after_key
        .split_once(':')
        .unwrap_or_else(|| panic!("quest table {tier} has malformed {key}"));
    let digits: String = after_colon
        .trim_start()
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("quest table {tier} has malformed {key}"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn source_task_tables_are_complete() {
        const EXPECTED_COUNTS: [(u64, usize); 15] = [
            (1_000, 23),
            (1_500, 23),
            (2_000, 22),
            (3_000, 23),
            (5_000, 17),
            (8_000, 19),
            (10_000, 11),
            (13_000, 11),
            (17_000, 10),
            (22_000, 9),
            (40_000, 11),
            (50_000, 9),
            (60_000, 7),
            (80_000, 8),
            (100_000, 17),
        ];

        assert_eq!(all_task_definitions().len(), 220);
        assert_eq!(
            all_task_definitions()
                .iter()
                .map(|definition| definition.target.as_str())
                .collect::<HashSet<_>>()
                .len(),
            84
        );
        assert!(
            all_task_definitions()
                .iter()
                .all(|definition| definition.kind == TaskKind::Kill)
        );
        for (tier, expected_count) in EXPECTED_COUNTS {
            assert_eq!(task_definitions_for_tier(tier).count(), expected_count);
        }
    }

    #[test]
    fn source_tier_selection_applies_three_task_streaks() {
        assert_eq!(task_tier_for(1_001, 0), 1_000);
        assert_eq!(task_tier_for(5_000, 2), 5_000);
        assert_eq!(task_tier_for(5_000, 3), 8_000);
        assert_eq!(task_tier_for(5_000, -3), 3_000);
        assert_eq!(task_tier_for(100_000, 99), 100_000);
        assert_eq!(task_tier_for(1_001, -99), 1_000);
    }
}
