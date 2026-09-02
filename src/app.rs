use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    game::{Action, Game, SuicideKind},
    skills::SkillId,
};

pub struct App {
    pub game: Game,
    pub selected_action: usize,
    pub show_help: bool,
    show_skills: bool,
    pub should_quit: bool,
    save_requested: bool,
    skill_scroll: usize,
    pending_skill_abandonment: Option<SkillId>,
    show_identity: bool,
    show_combat_settings: bool,
    combat_setting_index: u8,
    pending_suicide: Option<SuicideKind>,
    delete_save_confirmation_armed: bool,
    delete_save_requested: bool,
}

impl App {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            selected_action: 0,
            show_help: false,
            show_skills: false,
            should_quit: false,
            save_requested: false,
            skill_scroll: 0,
            pending_skill_abandonment: None,
            show_identity: false,
            show_combat_settings: false,
            combat_setting_index: 0,
            pending_suicide: None,
            delete_save_confirmation_armed: false,
            delete_save_requested: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        if self.show_help {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
            ) {
                self.show_help = false;
            }
            return;
        }

        if self.show_skills {
            match key.code {
                KeyCode::Esc | KeyCode::Char('v') | KeyCode::Char('q') => {
                    self.show_skills = false;
                    self.skill_scroll = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.skill_scroll = self.skill_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.skill_scroll = self
                        .skill_scroll
                        .saturating_add(1)
                        .min(self.max_skill_scroll());
                }
                KeyCode::PageUp => {
                    self.skill_scroll = self.skill_scroll.saturating_sub(8);
                }
                KeyCode::PageDown => {
                    self.skill_scroll = self
                        .skill_scroll
                        .saturating_add(8)
                        .min(self.max_skill_scroll());
                }
                KeyCode::Home => self.skill_scroll = 0,
                KeyCode::End => self.skill_scroll = self.max_skill_scroll(),
                _ => {}
            }
            return;
        }

        if self.show_identity {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('q')
            ) {
                self.show_identity = false;
            }
            return;
        }

        if self.show_combat_settings {
            match key.code {
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Enter => {
                    self.show_combat_settings = false;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.combat_setting_index = self.combat_setting_index.saturating_sub(1);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.combat_setting_index = (self.combat_setting_index + 1).min(2);
                }
                KeyCode::Up | KeyCode::Char('k') => self.adjust_combat_setting(1),
                KeyCode::Down | KeyCode::Char('j') => self.adjust_combat_setting(-1),
                KeyCode::PageUp => self.adjust_combat_setting(5),
                KeyCode::PageDown => self.adjust_combat_setting(-5),
                _ => {}
            }
            return;
        }

        if let Some(kind) = self.pending_suicide {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    match kind {
                        SuicideKind::Reincarnate => {
                            self.game.perform(Action::Suicide(SuicideKind::Reincarnate));
                            self.pending_suicide = None;
                        }
                        SuicideKind::EraseSave if self.delete_save_confirmation_armed => {
                            self.delete_save_requested = true;
                            self.pending_suicide = None;
                            self.should_quit = true;
                        }
                        SuicideKind::EraseSave => {
                            self.delete_save_confirmation_armed = true;
                            self.game.push_log("再次确认后将永久删除本地存档。".into());
                        }
                    }
                    self.selected_action = 0;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.pending_suicide = None;
                    self.delete_save_confirmation_armed = false;
                    self.selected_action = 0;
                    self.game.push_log("你取消了自尽或删档操作。".into());
                }
                _ => {}
            }
            self.clamp_selection();
            return;
        }

        if let Some(skill) = self.pending_skill_abandonment.clone() {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.game.perform(Action::AbandonSkill(skill));
                    self.pending_skill_abandonment = None;
                    self.selected_action = 0;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.game
                        .push_log(format!("你暂时没有放弃继续学习{}。", skill.name()));
                    self.pending_skill_abandonment = None;
                    self.selected_action = 0;
                }
                _ => {}
            }
            self.clamp_selection();
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('v') => {
                self.show_skills = true;
                self.skill_scroll = 0;
            }
            KeyCode::Char('i') => self.show_identity = true,
            KeyCode::Char('c') => {
                self.show_combat_settings = true;
                self.combat_setting_index = 0;
            }
            KeyCode::Char('s') => {
                self.save_requested = true;
                self.game.push_log("正在保存当前进度……".into());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_action = self.selected_action.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let last = self.game.available_actions().len().saturating_sub(1);
                self.selected_action = (self.selected_action + 1).min(last);
            }
            KeyCode::Home => self.selected_action = 0,
            KeyCode::End => {
                self.selected_action = self.game.available_actions().len().saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.perform_selected(),
            KeyCode::Esc if matches!(self.game.activity, crate::game::Activity::Fighting(_)) => {
                self.game.perform(crate::game::Action::Surrender);
                self.selected_action = 0;
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.game.tick();
        self.skill_scroll = self.skill_scroll.min(self.max_skill_scroll());
        self.clamp_selection();
    }

    pub fn take_save_request(&mut self) -> bool {
        std::mem::take(&mut self.save_requested)
    }

    pub fn take_delete_save_request(&mut self) -> bool {
        std::mem::take(&mut self.delete_save_requested)
    }

    pub fn showing_identity(&self) -> bool {
        self.show_identity
    }

    pub fn showing_combat_settings(&self) -> bool {
        self.show_combat_settings
    }

    pub fn combat_setting_index(&self) -> u8 {
        self.combat_setting_index
    }

    pub fn pending_suicide(&self) -> Option<SuicideKind> {
        self.pending_suicide
    }

    pub fn delete_save_confirmation_armed(&self) -> bool {
        self.delete_save_confirmation_armed
    }

    pub fn showing_skills(&self) -> bool {
        self.show_skills
    }

    pub fn skill_scroll(&self) -> usize {
        self.skill_scroll
    }

    fn adjust_combat_setting(&mut self, delta: i32) {
        let increase = |value: u32| {
            if delta >= 0 {
                value.saturating_add(delta as u32)
            } else {
                value.saturating_sub(delta.saturating_abs() as u32)
            }
        };
        match self.combat_setting_index {
            0 => self
                .game
                .set_force_factor(increase(self.game.player.force_factor)),
            1 => self
                .game
                .set_mana_factor(increase(self.game.player.mana_factor)),
            _ => {
                let current = self.game.wimpy_percent();
                let requested = if delta >= 0 {
                    current.saturating_add(delta as u8)
                } else {
                    current.saturating_sub(delta.saturating_abs() as u8)
                };
                self.game.set_wimpy_percent(requested);
            }
        }
    }

    fn max_skill_scroll(&self) -> usize {
        self.game
            .player
            .skills
            .len()
            .saturating_mul(2)
            .saturating_sub(1)
    }

    pub fn pending_skill_abandonment(&self) -> Option<&SkillId> {
        self.pending_skill_abandonment.as_ref()
    }

    fn perform_selected(&mut self) {
        let actions = self.game.available_actions();
        if let Some(action) = actions.get(self.selected_action).cloned() {
            match action {
                Action::AbandonSkill(skill) => {
                    self.pending_skill_abandonment = Some(skill);
                    self.selected_action = 0;
                    return;
                }
                Action::ConfigureCombat => {
                    self.show_combat_settings = true;
                    self.combat_setting_index = 0;
                    self.selected_action = 0;
                    return;
                }
                Action::Suicide(kind) => {
                    self.pending_suicide = Some(kind);
                    self.delete_save_confirmation_armed = false;
                    self.selected_action = 0;
                    return;
                }
                action => self.game.perform(action),
            }
            self.selected_action = 0;
        }
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        let last = self.game.available_actions().len().saturating_sub(1);
        self.selected_action = self.selected_action.min(last);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_selection_stays_in_bounds() {
        let mut app = App::new(Game::new());
        for _ in 0..50 {
            app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        }
        assert!(app.selected_action < app.game.available_actions().len());
    }

    #[test]
    fn abandoning_a_skill_requires_confirmation() {
        let mut app = App::new(Game::new());
        let skill = SkillId::from(crate::skills::LIUH_KEN_ID);
        let action = Action::AbandonSkill(skill.clone());
        app.selected_action = app
            .game
            .available_actions()
            .iter()
            .position(|candidate| candidate == &action)
            .unwrap();
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.pending_skill_abandonment(), Some(&skill));
        assert!(app.game.player.skill_by_id(skill.as_str()).is_some());

        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.pending_skill_abandonment().is_none());
        assert!(app.game.player.skill_by_id(skill.as_str()).is_some());

        app.selected_action = app
            .game
            .available_actions()
            .iter()
            .position(|candidate| candidate == &action)
            .unwrap();
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
        assert!(app.game.player.skill_by_id(skill.as_str()).is_none());
    }

    #[test]
    fn skill_overview_opens_scrolls_and_closes() {
        let mut app = App::new(Game::new());
        assert!(!app.showing_skills());

        app.handle_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE));
        assert!(app.showing_skills());
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.skill_scroll(), 1);

        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.showing_skills());
        assert_eq!(app.skill_scroll(), 0);
    }

    #[test]
    fn m8_panels_and_delete_require_explicit_confirmation() {
        let mut app = App::new(Game::new());
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        assert!(app.showing_combat_settings());
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.game.wimpy_percent(), 1);
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        assert!(!app.showing_combat_settings());

        app.handle_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        assert!(app.showing_identity());
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.showing_identity());

        let delete = Action::Suicide(SuicideKind::EraseSave);
        app.selected_action = app
            .game
            .available_actions()
            .iter()
            .position(|candidate| candidate == &delete)
            .unwrap();
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.pending_suicide(), Some(SuicideKind::EraseSave));
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
        assert!(app.delete_save_confirmation_armed());
        assert!(!app.take_delete_save_request());
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
        assert!(app.should_quit);
        assert!(app.take_delete_save_request());
    }
}
