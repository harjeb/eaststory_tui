use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::Game;

pub struct App {
    pub game: Game,
    pub selected_action: usize,
    pub show_help: bool,
    pub should_quit: bool,
    save_requested: bool,
}

impl App {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            selected_action: 0,
            show_help: false,
            should_quit: false,
            save_requested: false,
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

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
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
        self.clamp_selection();
    }

    pub fn take_save_request(&mut self) -> bool {
        std::mem::take(&mut self.save_requested)
    }

    fn perform_selected(&mut self) {
        let actions = self.game.available_actions();
        if let Some(action) = actions.get(self.selected_action).cloned() {
            self.game.perform(action);
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
}
