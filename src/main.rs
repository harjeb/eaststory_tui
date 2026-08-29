use std::{
    io::{self, Stdout},
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use dongfang_tui::{
    app::App,
    game::Game,
    save::{default_save_path, load_game, save_game},
    ui,
};
use ratatui::{Terminal, backend::CrosstermBackend};

const FRAME_RATE: Duration = Duration::from_millis(100);
const GAME_TICK: Duration = Duration::from_secs(1);
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    let save_path = default_save_path();
    let (game, load_message) = match load_game(&save_path) {
        Ok(Some(game)) => (game, Some("已载入本地存档。".to_string())),
        Ok(None) => (Game::new(), None),
        Err(error) => (
            Game::new(),
            Some(format!("存档载入失败，已开始新旅程：{error}")),
        ),
    };

    let mut app = App::new(game);
    if let Some(message) = load_message {
        app.game.push_log(message);
    }

    let mut terminal = TerminalSession::start().context("无法初始化终端界面")?;
    let run_result = run(&mut terminal.terminal, &mut app, &save_path);
    if app.take_delete_save_request() {
        if let Err(error) = std::fs::remove_file(&save_path)
            && error.kind() != io::ErrorKind::NotFound
        {
            return Err(error).with_context(|| format!("无法删除存档 {}", save_path.display()));
        }
    } else {
        save_game(&save_path, &app.game).context("退出时保存失败")?;
    }
    run_result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    save_path: &Path,
) -> Result<()> {
    let mut last_game_tick = Instant::now();
    let mut last_autosave = Instant::now();

    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(FRAME_RATE)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
        }
        if app.should_quit {
            break;
        }

        if last_game_tick.elapsed() >= GAME_TICK {
            app.tick();
            last_game_tick = Instant::now();
        }

        if app.take_save_request() || last_autosave.elapsed() >= AUTOSAVE_INTERVAL {
            match save_game(save_path, &app.game) {
                Ok(()) => app.game.push_log("进度已保存。".into()),
                Err(error) => app.game.push_log(format!("保存失败：{error}")),
            }
            last_autosave = Instant::now();
        }
    }

    Ok(())
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn start() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                Err(error.into())
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
