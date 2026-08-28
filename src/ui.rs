use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    app::App,
    game::{Activity, CombatMode, Game},
};

const BORDER: Color = Color::Rgb(74, 89, 80);
const JADE: Color = Color::Rgb(84, 184, 138);
const GOLD: Color = Color::Rgb(226, 180, 85);
const PAPER: Color = Color::Rgb(216, 220, 210);
const MUTED: Color = Color::Rgb(145, 155, 148);
const DANGER: Color = Color::Rgb(214, 93, 93);
const WATER: Color = Color::Rgb(90, 169, 196);

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    frame.render_widget(
        Block::new().style(Style::default().bg(Color::Rgb(17, 22, 20))),
        area,
    );

    if area.width < 58 || area.height < 20 {
        render_too_small(frame, area);
        return;
    }

    if area.width >= 105 && area.height >= 27 {
        render_wide(frame, area, app);
    } else {
        render_compact(frame, area, app);
    }

    if app.show_help {
        render_help(frame, area);
    }
}

fn render_wide(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(15),
            Constraint::Length(8),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, rows[0], &app.game);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Min(42),
            Constraint::Length(34),
        ])
        .split(rows[1]);
    render_stats(frame, columns[0], &app.game, false);
    render_location(frame, columns[1], app, false);
    render_side(frame, columns[2], &app.game);
    render_log(frame, rows[2], &app.game);
    render_footer(frame, rows[3]);
}

fn render_compact(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, rows[0], &app.game);
    render_stats(frame, rows[1], &app.game, true);
    render_location(frame, rows[2], app, true);
    render_quest_compact(frame, rows[3], &app.game);
    render_log(frame, rows[4], &app.game);
    render_footer(frame, rows[5]);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, game: &Game) {
    let title = Line::from(vec![
        Span::styled(
            " 东方故事 ",
            Style::default()
                .fg(Color::Black)
                .bg(GOLD)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " 独行 ",
            Style::default()
                .fg(Color::Black)
                .bg(JADE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            game.current_location().zone.as_str(),
            Style::default().fg(PAPER),
        ),
        Span::styled("  ·  ", Style::default().fg(MUTED)),
        Span::styled(game.time_text(), Style::default().fg(WATER)),
    ]);
    frame.render_widget(Paragraph::new(title).alignment(Alignment::Left), area);
}

fn panel<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ))
}

fn render_stats(frame: &mut Frame<'_>, area: Rect, game: &Game, compact: bool) {
    let player = &game.player;
    if compact {
        let lines = vec![
            Line::from(vec![
                stat_span("精", player.essence, player.max_essence, JADE),
                Span::raw("   "),
                stat_span("气", player.qi, player.max_qi, GOLD),
                Span::raw("   "),
                stat_span("神", player.spirit, player.max_spirit, WATER),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("{}  ", game.activity_text()),
                    Style::default().fg(JADE),
                ),
                Span::styled(
                    format!("钱 {}  ", player.money_text()),
                    Style::default().fg(GOLD),
                ),
                Span::styled(
                    format!(
                        "潜 {}/{}  杀 {}  缉 {}",
                        player.potential.saturating_sub(player.learned_points),
                        player.potential,
                        player.bellicosity,
                        player.wanted
                    ),
                    Style::default().fg(PAPER),
                ),
            ]),
        ];
        frame.render_widget(Paragraph::new(lines).block(panel("人物")), area);
        return;
    }

    let faction = player.faction.as_deref().unwrap_or("无门无派");
    let mut lines = vec![
        bar_line("精", player.essence, player.max_essence, JADE),
        bar_line("气", player.qi, player.max_qi, GOLD),
        bar_line("神", player.spirit, player.max_spirit, WATER),
        Line::from(vec![
            Span::styled(
                format!("饱 {}/{}", player.food, player.max_food),
                Style::default().fg(GOLD),
            ),
            Span::styled(
                format!("  饮 {}/{}", player.water, player.max_water),
                Style::default().fg(WATER),
            ),
        ]),
        Line::from(vec![
            Span::styled("状态  ", Style::default().fg(MUTED)),
            Span::styled(game.activity_text(), Style::default().fg(JADE)),
            Span::styled(
                format!("  {}", player.conditions_text()),
                Style::default().fg(DANGER),
            ),
        ]),
        Line::from(vec![
            Span::styled("师门  ", Style::default().fg(MUTED)),
            Span::styled(faction, Style::default().fg(PAPER)),
        ]),
        Line::from(vec![
            Span::styled("实战  ", Style::default().fg(MUTED)),
            Span::styled(
                format!(
                    "{}  潜能 {}/{}",
                    player.combat_experience,
                    player.potential.saturating_sub(player.learned_points),
                    player.potential
                ),
                Style::default().fg(WATER),
            ),
        ]),
        Line::from(vec![
            Span::styled("储备  ", Style::default().fg(MUTED)),
            Span::styled(
                format!(
                    "内 {}/{}  法 {}/{}  灵 {}/{}",
                    player.force,
                    player.max_force,
                    player.mana,
                    player.max_mana,
                    player.atman,
                    player.max_atman
                ),
                Style::default().fg(GOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("江湖  ", Style::default().fg(MUTED)),
            Span::styled(
                format!(
                    "评价 {:+}  杀气 {}  通缉 {}",
                    player.reputation, player.bellicosity, player.wanted
                ),
                Style::default().fg(PAPER),
            ),
        ]),
        Line::from(vec![
            Span::styled("钱财  ", Style::default().fg(MUTED)),
            Span::styled(player.money_text(), Style::default().fg(GOLD)),
        ]),
        Line::styled(
            "当前武学",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
    ];
    let mut shown = Vec::new();
    for mapping in &player.skill_mappings {
        if shown.contains(&mapping.skill) {
            continue;
        }
        shown.push(mapping.skill.clone());
        if let Some(skill) = player.skill_by_id(mapping.skill.as_str()) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<12}", skill.kind.name()),
                    Style::default().fg(PAPER),
                ),
                Span::styled(format!("{:>3}层", skill.level), Style::default().fg(JADE)),
            ]));
        }
        if shown.len() == 3 {
            break;
        }
    }
    frame.render_widget(Paragraph::new(lines).block(panel("人物")), area);
}

fn stat_span(label: &str, value: i32, max: i32, color: Color) -> Span<'static> {
    Span::styled(
        format!("{label} {}/{}", value.max(0), max),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn bar_line(label: &str, value: i32, max: i32, color: Color) -> Line<'static> {
    let filled = ((value.max(0) as usize * 10) / max.max(1) as usize).min(10);
    let bar = format!("{}{}", "━".repeat(filled), "─".repeat(10 - filled));
    Line::from(vec![
        Span::styled(
            format!("{label}  "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(bar, Style::default().fg(color)),
        Span::styled(
            format!(" {:>3}/{:<3}", value.max(0), max),
            Style::default().fg(PAPER),
        ),
    ])
}

fn render_location(frame: &mut Frame<'_>, area: Rect, app: &App, compact: bool) {
    let place = app.game.current_location();
    let block = panel(&place.name);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if compact {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(43), Constraint::Percentage(57)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(place.description.as_str())
                .style(Style::default().fg(PAPER))
                .wrap(Wrap { trim: true }),
            columns[0],
        );
        render_actions(frame, columns[1], app, false);
        return;
    }

    let combat_height = if matches!(app.game.activity, Activity::Fighting(_)) {
        3
    } else {
        0
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(combat_height),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(inner);
    frame.render_widget(
        Paragraph::new(place.description.as_str())
            .style(Style::default().fg(PAPER))
            .wrap(Wrap { trim: true }),
        rows[0],
    );
    if combat_height > 0 {
        render_combat(frame, rows[1], &app.game);
    }
    render_actions(frame, rows[2], app, true);
    let detail = app
        .game
        .available_actions()
        .get(app.selected_action)
        .map_or("", |action| action.detail());
    frame.render_widget(
        Paragraph::new(detail)
            .style(Style::default().fg(MUTED))
            .wrap(Wrap { trim: true }),
        rows[3],
    );
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, app: &App, titled: bool) {
    let actions = app.game.available_actions();
    let items: Vec<ListItem<'_>> = actions
        .iter()
        .map(|action| {
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(action.label(&app.game)),
            ]))
        })
        .collect();
    let mut list = List::new(items).highlight_symbol("› ").highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(JADE)
            .add_modifier(Modifier::BOLD),
    );
    if titled {
        list = list.block(Block::new().title(Span::styled("可用行动", Style::default().fg(GOLD))));
    }
    let mut state = ListState::default().with_selected(Some(app.selected_action));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_combat(frame: &mut Frame<'_>, area: Rect, game: &Game) {
    let Activity::Fighting(combat) = &game.activity else {
        return;
    };
    let ratio = (combat.health.max(0) as f64 / combat.max_health as f64).clamp(0.0, 1.0);
    let (mode, color) = match combat.mode {
        CombatMode::Spar => ("比试", GOLD),
        CombatMode::Lethal => ("死斗", DANGER),
    };
    let gauge = Gauge::default()
        .block(Block::new().title(Span::styled(
            format!("{} · {} · 第{}合", combat.enemy.name(), mode, combat.rounds),
            Style::default().fg(color),
        )))
        .gauge_style(Style::default().fg(color).bg(Color::Rgb(48, 36, 36)))
        .ratio(ratio)
        .label(format!("{}/{}", combat.health.max(0), combat.max_health));
    frame.render_widget(gauge, area);
}

fn render_side(frame: &mut Frame<'_>, area: Rect, game: &Game) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);
    let quest = vec![
        Line::styled(
            game.quest_title(),
            Style::default().fg(JADE).add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::styled(game.quest_objective(), Style::default().fg(PAPER)),
    ];
    frame.render_widget(
        Paragraph::new(quest)
            .block(panel("当前见闻"))
            .wrap(Wrap { trim: true }),
        rows[0],
    );
    let inventory = game.inventory_lines();
    let lines = if inventory.is_empty() {
        vec![Line::styled("空", Style::default().fg(MUTED))]
    } else {
        inventory
            .into_iter()
            .map(|line| Line::styled(line, Style::default().fg(PAPER)))
            .collect()
    };
    frame.render_widget(Paragraph::new(lines).block(panel("行囊")), rows[1]);
}

fn render_quest_compact(frame: &mut Frame<'_>, area: Rect, game: &Game) {
    let text = Line::from(vec![
        Span::styled(
            format!("{}  ", game.quest_title()),
            Style::default().fg(JADE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(game.quest_objective(), Style::default().fg(PAPER)),
    ]);
    frame.render_widget(
        Paragraph::new(text)
            .block(panel("当前见闻"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_log(frame: &mut Frame<'_>, area: Rect, game: &Game) {
    let count = area.height.saturating_sub(2) as usize;
    let start = game.logs.len().saturating_sub(count);
    let lines: Vec<Line<'_>> = game.logs[start..]
        .iter()
        .map(|entry| {
            Line::from(vec![
                Span::styled("· ", Style::default().fg(GOLD)),
                Span::styled(entry, Style::default().fg(PAPER)),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel("江湖见闻"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let footer = Line::from(vec![
        Span::styled(" ↑↓/jk ", Style::default().fg(Color::Black).bg(MUTED)),
        Span::raw(" 选择  "),
        Span::styled(" Enter ", Style::default().fg(Color::Black).bg(JADE)),
        Span::raw(" 执行  "),
        Span::styled(" s ", Style::default().fg(Color::Black).bg(GOLD)),
        Span::raw(" 保存  "),
        Span::styled(" ? ", Style::default().fg(Color::Black).bg(WATER)),
        Span::raw(" 帮助  "),
        Span::styled(" q ", Style::default().fg(Color::Black).bg(DANGER)),
        Span::raw(" 退出"),
    ]);
    frame.render_widget(
        Paragraph::new(footer).style(Style::default().fg(PAPER)),
        area,
    );
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered_rect(
        area,
        64.min(area.width.saturating_sub(4)),
        16.min(area.height.saturating_sub(2)),
    );
    frame.render_widget(Clear, popup);
    let lines = vec![
        Line::styled(
            "行动",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Line::raw("  ↑/k、↓/j    选择行动"),
        Line::raw("  Enter/Space  执行行动"),
        Line::raw("  Esc          战斗中认输"),
        Line::raw(""),
        Line::styled(
            "系统",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Line::raw("  s            立即保存"),
        Line::raw("  q / Ctrl+C   保存并退出"),
        Line::raw("  ? / Esc      关闭帮助"),
        Line::raw(""),
        Line::styled(
            "修炼、休息和战斗会随现实时间自动推进。",
            Style::default().fg(MUTED),
        ),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel("帮助"))
            .style(Style::default().fg(PAPER))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn render_too_small(frame: &mut Frame<'_>, area: Rect) {
    let message = Paragraph::new(vec![
        Line::styled(
            "东方故事 · 独行",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::styled("终端空间不足", Style::default().fg(DANGER)),
        Line::styled("请将窗口调整到至少 58×20。", Style::default().fg(PAPER)),
        Line::styled("按 q 仍可保存并退出。", Style::default().fg(MUTED)),
    ])
    .alignment(Alignment::Center)
    .block(panel("提示"));
    let popup = centered_rect(area, 42.min(area.width), 9.min(area.height));
    frame.render_widget(message, popup);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::game::{Action, EnemyKind, LocationId};

    fn draw(app: &App, width: u16, height: u16) {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
    }

    #[test]
    fn renders_supported_terminal_sizes() {
        let app = App::new(Game::new());
        for (width, height) in [(120, 30), (80, 24), (58, 20), (40, 12)] {
            draw(&app, width, height);
        }
    }

    #[test]
    fn renders_combat_and_help_states() {
        let mut app = App::new(Game::new());
        app.game.location = LocationId::from(crate::content::PINE_FOREST);
        app.game.perform(Action::Fight(EnemyKind::Bandit));
        draw(&app, 120, 30);

        app.show_help = true;
        draw(&app, 80, 24);
    }
}
