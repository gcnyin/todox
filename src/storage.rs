use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::i18n::DEFAULT_LOCALE;
use crate::model::TaskStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub i18n: I18nConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nConfig {
    pub locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigLoadStatus {
    Loaded,
    Created,
    ResetInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    pub config: Config,
    pub status: ConfigLoadStatus,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            i18n: I18nConfig {
                locale: DEFAULT_LOCALE.to_string(),
            },
        }
    }
}

pub fn resolve_data_file(data_file: Option<PathBuf>) -> AppResult<PathBuf> {
    match data_file {
        Some(path) => Ok(path),
        None => default_data_file(),
    }
}

pub fn default_data_file() -> AppResult<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(AppError::MissingHomeDir)?;
    Ok(home_dir.join(".todox").join("tasks.json"))
}

pub fn config_file_for(data_file: &Path) -> AppResult<PathBuf> {
    let parent = data_file.parent().ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid data file path: {}", data_file.display()),
        ))
    })?;
    Ok(parent.join("config.json"))
}

pub fn i18n_dir_for(data_file: &Path) -> AppResult<PathBuf> {
    let parent = data_file.parent().ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid data file path: {}", data_file.display()),
        ))
    })?;
    Ok(parent.join("i18n"))
}

pub fn load_or_create(path: &Path) -> AppResult<TaskStore> {
    ensure_parent_dir(path)?;

    if !path.exists() {
        let store = TaskStore::new();
        save(path, &store)?;
        return Ok(store);
    }

    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() == 0 {
        let store = TaskStore::new();
        save(path, &store)?;
        return Ok(store);
    }

    let reader = BufReader::new(file);
    let mut store: TaskStore =
        serde_json::from_reader(reader).map_err(|_| AppError::CorruptedData {
            path: path.to_path_buf(),
        })?;

    if store.next_id == 0 {
        store.next_id = next_id_from_tasks(&store);
    }

    Ok(store)
}

pub fn load_or_create_config(path: &Path) -> AppResult<LoadedConfig> {
    ensure_parent_dir(path)?;

    if !path.exists() {
        let config = Config::default();
        save_config(path, &config)?;
        return Ok(LoadedConfig {
            config,
            status: ConfigLoadStatus::Created,
        });
    }

    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() == 0 {
        let config = Config::default();
        save_config(path, &config)?;
        return Ok(LoadedConfig {
            config,
            status: ConfigLoadStatus::Created,
        });
    }

    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(config) => Ok(LoadedConfig {
            config,
            status: ConfigLoadStatus::Loaded,
        }),
        Err(_) => {
            let config = Config::default();
            save_config(path, &config)?;
            Ok(LoadedConfig {
                config,
                status: ConfigLoadStatus::ResetInvalid,
            })
        }
    }
}

pub fn save(path: &Path, store: &TaskStore) -> AppResult<()> {
    save_pretty_json(path, store)
}

pub fn save_config(path: &Path, config: &Config) -> AppResult<()> {
    save_pretty_json(path, config)
}

fn save_pretty_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    ensure_parent_dir(path)?;

    let tmp_path = path.with_extension("json.tmp");
    let file = File::create(&tmp_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    drop(writer);

    replace_file(&tmp_path, path)?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> AppResult<()> {
    let parent = path.parent().ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid data file path: {}", path.display()),
        ))
    })?;

    fs::create_dir_all(parent)?;
    Ok(())
}

fn next_id_from_tasks(store: &TaskStore) -> u64 {
    store
        .tasks
        .iter()
        .map(|task| task.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn replace_file(tmp_path: &Path, path: &Path) -> AppResult<()> {
    fs::rename(tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use chrono::{DateTime, NaiveDate, Utc};
    use tempfile::tempdir;

    use crate::error::AppError;
    use crate::model::{Priority, TaskDraft, TaskStore};

    use super::{
        ConfigLoadStatus, config_file_for, default_data_file, i18n_dir_for, load_or_create,
        load_or_create_config, save, save_config,
    };

    fn fixed_time(input: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(input)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn default_path_uses_hidden_todox_dir() {
        let resolved = default_data_file().unwrap();
        let components = resolved
            .iter()
            .map(|part| part.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(
            components
                .windows(2)
                .any(|window| window == [".todox", "tasks.json"])
        );
    }

    #[test]
    fn config_and_i18n_paths_follow_data_file_parent() {
        let data = PathBuf::from("/tmp/demo/tasks.json");
        assert_eq!(
            config_file_for(&data).unwrap(),
            PathBuf::from("/tmp/demo/config.json")
        );
        assert_eq!(
            i18n_dir_for(&data).unwrap(),
            PathBuf::from("/tmp/demo/i18n")
        );
    }

    #[test]
    fn load_or_create_initializes_empty_store_and_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("tasks.json");

        let store = load_or_create(&path).unwrap();

        assert_eq!(store, TaskStore::new());
        assert!(path.exists());
    }

    #[test]
    fn load_or_create_reports_corrupted_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        fs::write(&path, b"{not valid json").unwrap();

        let error = load_or_create(&path).unwrap_err();
        assert!(matches!(error, AppError::CorruptedData { .. }));
    }

    #[test]
    fn load_or_create_config_creates_default_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let loaded = load_or_create_config(&path).unwrap();
        assert_eq!(loaded.status, ConfigLoadStatus::Created);
        assert_eq!(loaded.config.i18n.locale, "en_US");
    }

    #[test]
    fn load_or_create_config_resets_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, b"{bad json").unwrap();
        let loaded = load_or_create_config(&path).unwrap();
        assert_eq!(loaded.status, ConfigLoadStatus::ResetInvalid);
        assert_eq!(loaded.config.i18n.locale, "en_US");
    }

    #[test]
    fn save_persists_multiple_updates() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        let mut store = TaskStore::new();
        store
            .add_task(
                TaskDraft {
                    title: "第一次保存".into(),
                    notes: String::new(),
                    priority: Priority::P2,
                    due_date: Some(NaiveDate::from_ymd_opt(2026, 3, 8).unwrap()),
                },
                fixed_time("2026-03-08T10:00:00Z"),
            )
            .unwrap();

        save(&path, &store).unwrap();

        store
            .add_task(
                TaskDraft {
                    title: "第二次保存".into(),
                    notes: "保持存在".into(),
                    priority: Priority::P1,
                    due_date: None,
                },
                fixed_time("2026-03-08T11:00:00Z"),
            )
            .unwrap();

        save(&path, &store).unwrap();

        let reloaded = load_or_create(&path).unwrap();
        assert_eq!(reloaded.tasks.len(), 2);
        assert_eq!(reloaded.tasks[1].title, "第二次保存");
    }

    #[test]
    fn save_config_persists_locale() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut loaded = load_or_create_config(&path).unwrap();
        loaded.config.i18n.locale = "zh_CN".to_string();
        save_config(&path, &loaded.config).unwrap();
        let loaded = load_or_create_config(&path).unwrap();
        assert_eq!(loaded.config.i18n.locale, "zh_CN");
    }
}
