use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::error::{AppError, AppResult};
use crate::i18n::{I18n, LocaleOption, LocaleSource, available_locales};
use crate::model::{
    Filter, Priority, Task, TaskDraft, TaskStatus, TaskStore, parse_due_date_input, today_local,
};
use crate::storage::{self, Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Title,
    Notes,
    Priority,
    DueDate,
}

impl FormField {
    pub fn translation_key(self) -> &'static str {
        match self {
            FormField::Title => "form.field.title",
            FormField::Notes => "form.field.notes",
            FormField::Priority => "form.field.priority",
            FormField::DueDate => "form.field.due_date",
        }
    }

    pub fn next(self) -> Self {
        match self {
            FormField::Title => FormField::Priority,
            FormField::Priority => FormField::Notes,
            FormField::Notes => FormField::DueDate,
            FormField::DueDate => FormField::Title,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            FormField::Title => FormField::DueDate,
            FormField::Priority => FormField::Title,
            FormField::Notes => FormField::Priority,
            FormField::DueDate => FormField::Notes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskForm {
    pub task_id: Option<u64>,
    pub title: String,
    pub notes: String,
    pub priority: Priority,
    pub due_date: String,
    pub field: FormField,
    pub error: Option<String>,
}

impl TaskForm {
    pub fn new() -> Self {
        Self {
            task_id: None,
            title: String::new(),
            notes: String::new(),
            priority: Priority::default(),
            due_date: String::new(),
            field: FormField::Title,
            error: None,
        }
    }

    pub fn from_task(task: &Task) -> Self {
        Self {
            task_id: Some(task.id),
            title: task.title.clone(),
            notes: task.notes.clone(),
            priority: task.priority,
            due_date: task
                .due_date
                .map(|date| date.to_string())
                .unwrap_or_default(),
            field: FormField::Title,
            error: None,
        }
    }

    pub fn title_key(&self) -> &'static str {
        if self.task_id.is_some() {
            "modal.form.edit"
        } else {
            "modal.form.new"
        }
    }

    fn into_draft(&self) -> AppResult<TaskDraft> {
        Ok(TaskDraft {
            title: self.title.clone(),
            notes: self.notes.clone(),
            priority: self.priority,
            due_date: parse_due_date_input(&self.due_date)?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum AppMode {
    List,
    CreateEditModal(TaskForm),
    DeleteConfirm,
    Search {
        draft: String,
    },
    Help,
    LocalePicker {
        selected: usize,
        options: Vec<LocaleOption>,
    },
}

#[derive(Debug)]
pub struct App {
    store: TaskStore,
    data_file: PathBuf,
    config_path: PathBuf,
    i18n_dir: PathBuf,
    config: Config,
    i18n: I18n,
    filter: Filter,
    search_query: String,
    mode: AppMode,
    selected_task_id: Option<u64>,
    should_quit: bool,
    message: Option<String>,
}

impl App {
    pub fn new(
        store: TaskStore,
        data_file: PathBuf,
        config_path: PathBuf,
        i18n_dir: PathBuf,
        config: Config,
        i18n: I18n,
    ) -> Self {
        let mut app = Self {
            store,
            data_file,
            config_path,
            i18n_dir,
            config,
            i18n,
            filter: Filter::All,
            search_query: String::new(),
            mode: AppMode::List,
            selected_task_id: None,
            should_quit: false,
            message: None,
        };
        app.sync_selection();
        app
    }

    pub fn test_app() -> Self {
        let data_file = PathBuf::from("test.json");
        let config_path = PathBuf::from("config.json");
        let i18n_dir = PathBuf::from("i18n");
        let config = Config::default();
        let i18n = I18n::load(&config.i18n.locale, Path::new(".")).unwrap();
        Self::new(
            TaskStore::new(),
            data_file,
            config_path,
            i18n_dir,
            config,
            i18n,
        )
    }

    pub fn data_file(&self) -> &Path {
        &self.data_file
    }

    pub fn filter(&self) -> Filter {
        self.filter
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn mode(&self) -> &AppMode {
        &self.mode
    }

    pub fn locale(&self) -> &str {
        self.i18n.locale()
    }

    pub fn t(&self, key: &str) -> String {
        self.i18n.t(key)
    }

    pub fn t_fmt(&self, key: &str, params: &[(&str, String)]) -> String {
        self.i18n.t_fmt(key, params)
    }

    pub fn tr_error(&self, error: &AppError) -> String {
        translate_error(&self.i18n, error)
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn total_count(&self) -> usize {
        self.store.tasks.len()
    }

    pub fn active_count(&self) -> usize {
        self.store
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Active)
            .count()
    }

    pub fn done_count(&self) -> usize {
        self.store
            .tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Done)
            .count()
    }

    pub fn due_today_count(&self) -> usize {
        let today = today_local();
        self.store
            .tasks
            .iter()
            .filter(|task| task.is_due_today(today))
            .count()
    }

    pub fn overdue_count(&self) -> usize {
        let today = today_local();
        self.store
            .tasks
            .iter()
            .filter(|task| task.is_overdue(today))
            .count()
    }

    pub fn visible_tasks(&self) -> Vec<&Task> {
        self.store
            .visible_tasks(self.filter, &self.search_query, today_local())
    }

    pub fn selected_visible_index(&self) -> Option<usize> {
        let selected = self.selected_task_id?;
        self.visible_tasks()
            .iter()
            .position(|task| task.id == selected)
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let selected = self.selected_task_id?;
        self.store.get_task(selected)
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<()> {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return Ok(());
        }

        match self.mode {
            AppMode::List => self.handle_list_key(key),
            AppMode::CreateEditModal(_) => {
                self.handle_form_key(key);
                Ok(())
            }
            AppMode::DeleteConfirm => self.handle_delete_confirm_key(key),
            AppMode::Search { .. } => {
                self.handle_search_key(key);
                Ok(())
            }
            AppMode::Help => {
                self.handle_help_key(key);
                Ok(())
            }
            AppMode::LocalePicker { .. } => self.handle_locale_picker_key(key),
        }
    }

    pub fn select_next(&mut self) {
        let visible_ids = self.visible_task_ids();
        if visible_ids.is_empty() {
            self.selected_task_id = None;
            return;
        }

        let next_index = match self
            .selected_task_id
            .and_then(|selected| visible_ids.iter().position(|task_id| *task_id == selected))
        {
            Some(index) => (index + 1) % visible_ids.len(),
            None => 0,
        };
        self.selected_task_id = Some(visible_ids[next_index]);
    }

    pub fn select_previous(&mut self) {
        let visible_ids = self.visible_task_ids();
        if visible_ids.is_empty() {
            self.selected_task_id = None;
            return;
        }

        let prev_index = match self
            .selected_task_id
            .and_then(|selected| visible_ids.iter().position(|task_id| *task_id == selected))
        {
            Some(0) | None => visible_ids.len() - 1,
            Some(index) => index - 1,
        };
        self.selected_task_id = Some(visible_ids[prev_index]);
    }

    fn visible_task_ids(&self) -> Vec<u64> {
        self.visible_tasks()
            .into_iter()
            .map(|task| task.id)
            .collect()
    }

    fn sync_selection(&mut self) {
        let visible_ids = self.visible_task_ids();
        if visible_ids.is_empty() {
            self.selected_task_id = None;
            return;
        }

        if self
            .selected_task_id
            .is_some_and(|selected| visible_ids.contains(&selected))
        {
            return;
        }

        self.selected_task_id = Some(visible_ids[0]);
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('k') if key.modifiers.is_empty() => self.select_previous(),
            KeyCode::Char('j') if key.modifiers.is_empty() => self.select_next(),
            KeyCode::Char('n') if key.modifiers.is_empty() => {
                self.mode = AppMode::CreateEditModal(TaskForm::new());
                self.clear_message();
            }
            KeyCode::Char('e') if key.modifiers.is_empty() => {
                if let Some(task) = self.selected_task().cloned() {
                    self.mode = AppMode::CreateEditModal(TaskForm::from_task(&task));
                    self.clear_message();
                }
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                if self.selected_task().is_some() {
                    self.mode = AppMode::DeleteConfirm;
                }
            }
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.filter = self.filter.next();
                self.sync_selection();
            }
            KeyCode::Char('/') if key.modifiers.is_empty() => {
                self.mode = AppMode::Search {
                    draft: self.search_query.clone(),
                };
            }
            KeyCode::Char('?') if key.modifiers.is_empty() => {
                self.mode = AppMode::Help;
            }
            KeyCode::Char('g') if key.modifiers.is_empty() => {
                self.open_locale_picker()?;
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                self.should_quit = true;
            }
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                if let Some(selected) = self.selected_task_id {
                    self.store.toggle_task(selected, Utc::now());
                    self.save_store()?;
                    self.sync_selection();
                }
            }
            KeyCode::Left => self.shift_selected_priority(false)?,
            KeyCode::Right => self.shift_selected_priority(true)?,
            KeyCode::Char('h') if key.modifiers.is_empty() => {
                self.shift_selected_priority(false)?
            }
            KeyCode::Char('l') if key.modifiers.is_empty() => self.shift_selected_priority(true)?,
            KeyCode::Char('1') if key.modifiers.is_empty() => {
                self.set_selected_priority(Priority::P1)?
            }
            KeyCode::Char('2') if key.modifiers.is_empty() => {
                self.set_selected_priority(Priority::P2)?
            }
            KeyCode::Char('3') if key.modifiers.is_empty() => {
                self.set_selected_priority(Priority::P3)?
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_delete_confirm_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::List;
            }
            KeyCode::Char('n') if key.modifiers.is_empty() => {
                self.mode = AppMode::List;
            }
            KeyCode::Char('y') if key.modifiers.is_empty() => {
                self.confirm_delete()?;
            }
            KeyCode::Enter => {
                self.confirm_delete()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn confirm_delete(&mut self) -> AppResult<()> {
        if let Some(selected) = self.selected_task_id {
            self.store.delete_task(selected);
            self.save_store()?;
            self.sync_selection();
        }
        self.mode = AppMode::List;
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?') => self.mode = AppMode::List,
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        let mut apply_query = None;
        let mut close = false;

        if let AppMode::Search { draft } = &mut self.mode {
            match key.code {
                KeyCode::Esc => close = true,
                KeyCode::Enter => {
                    apply_query = Some(draft.trim().to_string());
                    close = true;
                }
                KeyCode::Backspace => {
                    draft.pop();
                }
                KeyCode::Char(c) if is_plain_text_input(key) => {
                    draft.push(c);
                }
                _ => {}
            }
        }

        if let Some(query) = apply_query {
            self.search_query = query;
            self.sync_selection();
        }
        if close {
            self.mode = AppMode::List;
        }
    }

    fn handle_form_key(&mut self, key: KeyEvent) {
        let mut submit_requested = false;
        let mut cancel_requested = false;

        if let AppMode::CreateEditModal(form) = &mut self.mode {
            match key.code {
                KeyCode::Esc => cancel_requested = true,
                KeyCode::Tab | KeyCode::Down => form.field = form.field.next(),
                KeyCode::BackTab | KeyCode::Up => form.field = form.field.previous(),
                KeyCode::Enter => submit_requested = true,
                KeyCode::Left if form.field == FormField::Priority => {
                    form.priority = form.priority.previous();
                }
                KeyCode::Right if form.field == FormField::Priority => {
                    form.priority = form.priority.next();
                }
                KeyCode::Char('h')
                    if key.modifiers.is_empty() && form.field == FormField::Priority =>
                {
                    form.priority = form.priority.previous();
                }
                KeyCode::Char('l')
                    if key.modifiers.is_empty() && form.field == FormField::Priority =>
                {
                    form.priority = form.priority.next();
                }
                KeyCode::Char('1') if form.field == FormField::Priority => {
                    form.priority = Priority::P1
                }
                KeyCode::Char('2') if form.field == FormField::Priority => {
                    form.priority = Priority::P2
                }
                KeyCode::Char('3') if form.field == FormField::Priority => {
                    form.priority = Priority::P3
                }
                KeyCode::Backspace => match form.field {
                    FormField::Title => {
                        form.title.pop();
                    }
                    FormField::Notes => {
                        form.notes.pop();
                    }
                    FormField::Priority => {}
                    FormField::DueDate => {
                        form.due_date.pop();
                    }
                },
                KeyCode::Char(c) if is_plain_text_input(key) => match form.field {
                    FormField::Title => form.title.push(c),
                    FormField::Notes => form.notes.push(c),
                    FormField::Priority => {}
                    FormField::DueDate => form.due_date.push(c),
                },
                _ => {}
            }

            form.error = None;
        }

        if cancel_requested {
            self.mode = AppMode::List;
            return;
        }

        if submit_requested {
            self.submit_form();
        }
    }

    fn open_locale_picker(&mut self) -> AppResult<()> {
        let options = available_locales(&self.i18n_dir, self.locale())?;
        let selected = options.iter().position(|item| item.is_current).unwrap_or(0);
        self.mode = AppMode::LocalePicker { selected, options };
        Ok(())
    }

    fn handle_locale_picker_key(&mut self, key: KeyEvent) -> AppResult<()> {
        let mut apply = None;
        let mut close = false;
        if let AppMode::LocalePicker { selected, options } = &mut self.mode {
            match key.code {
                KeyCode::Esc => close = true,
                KeyCode::Up => {
                    if *selected == 0 {
                        *selected = options.len().saturating_sub(1);
                    } else {
                        *selected -= 1;
                    }
                }
                KeyCode::Down => {
                    *selected = (*selected + 1) % options.len().max(1);
                }
                KeyCode::Char('k') if key.modifiers.is_empty() => {
                    if *selected == 0 {
                        *selected = options.len().saturating_sub(1);
                    } else {
                        *selected -= 1;
                    }
                }
                KeyCode::Char('j') if key.modifiers.is_empty() => {
                    *selected = (*selected + 1) % options.len().max(1);
                }
                KeyCode::Enter => {
                    apply = options.get(*selected).cloned();
                }
                _ => {}
            }
        }

        if close {
            self.mode = AppMode::List;
        }

        if let Some(option) = apply {
            match self.apply_locale(&option.locale) {
                Ok(()) => {
                    self.mode = AppMode::List;
                }
                Err(error) => {
                    self.set_message(self.tr_error(&error));
                }
            }
        }

        Ok(())
    }

    pub fn locale_picker_options(&self) -> Option<(&[LocaleOption], usize)> {
        match &self.mode {
            AppMode::LocalePicker { selected, options } => Some((options.as_slice(), *selected)),
            _ => None,
        }
    }

    fn apply_locale(&mut self, locale: &str) -> AppResult<()> {
        let new_i18n = I18n::load(locale, &self.i18n_dir)?;
        self.config.i18n.locale = locale.to_string();
        storage::save_config(&self.config_path, &self.config)?;
        self.i18n = new_i18n;
        Ok(())
    }

    fn shift_selected_priority(&mut self, increase: bool) -> AppResult<()> {
        let Some(selected) = self.selected_task_id else {
            return Ok(());
        };
        let Some(task) = self.store.get_task(selected) else {
            return Ok(());
        };

        let next = if increase {
            task.priority.next()
        } else {
            task.priority.previous()
        };
        self.set_selected_priority(next)
    }

    fn set_selected_priority(&mut self, priority: Priority) -> AppResult<()> {
        let Some(selected) = self.selected_task_id else {
            return Ok(());
        };

        if self.store.set_priority(selected, priority, Utc::now()) {
            self.save_store()?;
            self.sync_selection();
        }

        Ok(())
    }

    fn submit_form(&mut self) {
        let AppMode::CreateEditModal(form) = &self.mode else {
            return;
        };

        let form = form.clone();
        match form.into_draft() {
            Ok(draft) => {
                let now = Utc::now();
                let result = match form.task_id {
                    Some(task_id) => self.store.update_task(task_id, draft, now).map(|_| task_id),
                    None => self.store.add_task(draft, now),
                }
                .and_then(|task_id| {
                    self.save_store()?;
                    Ok(task_id)
                });

                match result {
                    Ok(task_id) => {
                        self.selected_task_id = Some(task_id);
                        self.sync_selection();
                        self.mode = AppMode::List;
                    }
                    Err(error) => self.set_form_error(self.tr_error(&error)),
                }
            }
            Err(error) => self.set_form_error(self.tr_error(&error)),
        }
    }

    fn set_form_error(&mut self, message: String) {
        if let AppMode::CreateEditModal(form) = &mut self.mode {
            form.error = Some(message);
        }
    }

    fn save_store(&self) -> AppResult<()> {
        storage::save(&self.data_file, &self.store)
    }

    pub fn format_due_label(&self, task: &Task, today: NaiveDate) -> String {
        match task.due_date {
            Some(date) if date < today => self.t_fmt("due.overdue", &[("date", date.to_string())]),
            Some(date) if date == today => self.t_fmt("due.today", &[("date", date.to_string())]),
            Some(date) => self.t_fmt("due.upcoming", &[("date", date.to_string())]),
            None => self.t("due.none"),
        }
    }
}

pub fn translate_error(i18n: &I18n, error: &AppError) -> String {
    match error {
        AppError::Io(source) => i18n.t_fmt("error.io", &[("details", source.to_string())]),
        AppError::Serde(source) => i18n.t_fmt("error.json", &[("details", source.to_string())]),
        AppError::MissingHomeDir => i18n.t("error.missing_home"),
        AppError::InvalidDueDate => i18n.t("error.invalid_due_date"),
        AppError::EmptyTitle => i18n.t("error.empty_title"),
        AppError::CorruptedData { path } => i18n.t_fmt(
            "error.corrupted_data",
            &[("path", path.display().to_string())],
        ),
        AppError::InvalidLocaleFile { path, .. } => i18n.t_fmt(
            "error.invalid_locale_file",
            &[("path", path.display().to_string())],
        ),
        AppError::InvalidConfig { path } => i18n.t_fmt(
            "error.invalid_config",
            &[("path", path.display().to_string())],
        ),
    }
}

fn is_plain_text_input(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char(_))
        && (key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT)
}

pub fn locale_source_label(app: &App, source: LocaleSource) -> String {
    match source {
        LocaleSource::Builtin => app.t("modal.locale.builtin"),
        LocaleSource::Custom => app.t("modal.locale.custom"),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use crossterm::event::KeyEvent;
    use tempfile::tempdir;

    use crate::i18n::I18n;
    use crate::model::{Priority, TaskDraft, TaskStore};
    use crate::storage::{self, Config};

    use super::{App, AppMode, FormField};

    fn fixed_time(input: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(input)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn make_app(store: TaskStore, path: std::path::PathBuf) -> App {
        let config_path = path.parent().unwrap().join("config.json");
        let i18n_dir = path.parent().unwrap().join("i18n");
        let config = Config::default();
        let i18n = I18n::load(&config.i18n.locale, &i18n_dir).unwrap();
        App::new(store, path, config_path, i18n_dir, config, i18n)
    }

    #[test]
    fn editing_priority_updates_store_and_json_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        let mut store = TaskStore::new();
        store
            .add_task(
                TaskDraft {
                    title: "准备写一个 rust 程序".into(),
                    notes: String::new(),
                    priority: Priority::P2,
                    due_date: None,
                },
                fixed_time("2026-03-08T10:00:00Z"),
            )
            .unwrap();
        storage::save(&path, &store).unwrap();

        let mut app = make_app(store, path.clone());
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Char('e')))
            .unwrap();
        assert!(matches!(app.mode(), AppMode::CreateEditModal(_)));

        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Tab))
            .unwrap();

        match app.mode() {
            AppMode::CreateEditModal(form) => assert_eq!(form.field, FormField::Priority),
            _ => panic!("expected edit modal"),
        }

        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Char('1')))
            .unwrap();
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Enter))
            .unwrap();

        assert_eq!(app.selected_task().unwrap().priority, Priority::P1);

        let reloaded = storage::load_or_create(&path).unwrap();
        assert_eq!(reloaded.get_task(1).unwrap().priority, Priority::P1);
    }

    #[test]
    fn right_arrow_only_cycles_priority_when_priority_field_is_focused() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        let mut store = TaskStore::new();
        store
            .add_task(
                TaskDraft {
                    title: "任务 A".into(),
                    notes: String::new(),
                    priority: Priority::P2,
                    due_date: None,
                },
                fixed_time("2026-03-08T10:00:00Z"),
            )
            .unwrap();

        let mut app = make_app(store, path);
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Char('e')))
            .unwrap();
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Right))
            .unwrap();

        match app.mode() {
            AppMode::CreateEditModal(form) => {
                assert_eq!(form.field, FormField::Title);
                assert_eq!(form.priority, Priority::P2);
            }
            _ => panic!("expected edit modal"),
        }

        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Tab))
            .unwrap();
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Right))
            .unwrap();

        match app.mode() {
            AppMode::CreateEditModal(form) => {
                assert_eq!(form.field, FormField::Priority);
                assert_eq!(form.priority, Priority::P3);
            }
            _ => panic!("expected edit modal"),
        }
    }

    #[test]
    fn list_view_numeric_shortcuts_update_priority_and_persist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        let mut store = TaskStore::new();
        store
            .add_task(
                TaskDraft {
                    title: "任务 B".into(),
                    notes: String::new(),
                    priority: Priority::P3,
                    due_date: None,
                },
                fixed_time("2026-03-08T10:00:00Z"),
            )
            .unwrap();
        storage::save(&path, &store).unwrap();

        let mut app = make_app(store, path.clone());
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Char('1')))
            .unwrap();

        assert_eq!(app.selected_task().unwrap().priority, Priority::P1);
        assert_eq!(
            storage::load_or_create(&path)
                .unwrap()
                .get_task(1)
                .unwrap()
                .priority,
            Priority::P1
        );
    }

    #[test]
    fn up_and_down_move_between_form_fields() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        let mut store = TaskStore::new();
        store
            .add_task(
                TaskDraft {
                    title: "任务 C".into(),
                    notes: String::new(),
                    priority: Priority::P2,
                    due_date: None,
                },
                fixed_time("2026-03-08T10:00:00Z"),
            )
            .unwrap();

        let mut app = make_app(store, path);
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Char('e')))
            .unwrap();
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Down))
            .unwrap();

        match app.mode() {
            AppMode::CreateEditModal(form) => assert_eq!(form.field, FormField::Priority),
            _ => panic!("expected edit modal"),
        }

        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Down))
            .unwrap();
        match app.mode() {
            AppMode::CreateEditModal(form) => assert_eq!(form.field, FormField::Notes),
            _ => panic!("expected edit modal"),
        }

        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Up))
            .unwrap();
        match app.mode() {
            AppMode::CreateEditModal(form) => assert_eq!(form.field, FormField::Priority),
            _ => panic!("expected edit modal"),
        }
    }

    #[test]
    fn g_opens_locale_picker() {
        let mut app = App::test_app();
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Char('g')))
            .unwrap();
        assert!(matches!(app.mode(), AppMode::LocalePicker { .. }));
    }

    #[test]
    fn esc_quits_from_list_view() {
        let mut app = App::test_app();
        assert!(!app.should_quit());
        app.handle_key_event(KeyEvent::from(crossterm::event::KeyCode::Esc))
            .unwrap();
        assert!(app.should_quit());
    }
}
