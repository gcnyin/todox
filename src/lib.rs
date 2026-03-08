pub mod app;
pub mod cli;
pub mod error;
pub mod i18n;
pub mod model;
pub mod storage;
pub mod ui;

use std::io;
use std::time::Duration;

use app::{App, translate_error};
use cli::Cli;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use error::AppResult;
use i18n::I18n;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use storage::{ConfigLoadStatus, config_file_for, i18n_dir_for};

pub fn run(cli: Cli) -> AppResult<()> {
    let data_file = storage::resolve_data_file(cli.data_file)?;
    let store = storage::load_or_create(&data_file)?;
    let config_path = config_file_for(&data_file)?;
    let i18n_dir = i18n_dir_for(&data_file)?;
    let loaded_config = storage::load_or_create_config(&config_path)?;
    std::fs::create_dir_all(&i18n_dir)?;
    let i18n = I18n::load(&loaded_config.config.i18n.locale, &i18n_dir)?;

    if std::env::var_os("TODOX_DISABLE_TUI").is_some() {
        return Ok(());
    }

    let mut app = App::new(
        store,
        data_file,
        config_path,
        i18n_dir.clone(),
        loaded_config.config,
        i18n,
    );
    if loaded_config.status == ConfigLoadStatus::ResetInvalid {
        app.set_message(app.t("message.config_reset"));
    }
    let mut terminal = TerminalSession::enter()?;

    loop {
        terminal.terminal.draw(|frame| ui::draw(frame, &app))?;

        if app.should_quit() {
            break;
        }

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Err(error) = app.handle_key_event(key) {
                        app.set_message(translate_error(
                            &I18n::load(app.locale(), &i18n_dir)?,
                            &error,
                        ));
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

pub fn localized_error(cli: &Cli, error: &error::AppError) -> String {
    let data_file = storage::resolve_data_file(cli.data_file.clone());
    if let Ok(data_file) = data_file {
        if let (Ok(config_path), Ok(i18n_dir)) =
            (config_file_for(&data_file), i18n_dir_for(&data_file))
        {
            if let Ok(loaded_config) = storage::load_or_create_config(&config_path) {
                if let Ok(i18n) = I18n::load(&loaded_config.config.i18n.locale, &i18n_dir) {
                    return format!(
                        "{}: {}",
                        i18n.t("error.prefix"),
                        translate_error(&i18n, error)
                    );
                }
            }
        }
    }

    format!("Error: {:?}", error)
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn enter() -> AppResult<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
