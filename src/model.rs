use std::cmp::Ordering;
use std::fmt;

use chrono::{DateTime, Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    P1,
    P2,
    P3,
}

impl Priority {
    pub const ALL: [Priority; 3] = [Priority::P1, Priority::P2, Priority::P3];

    pub fn code(self) -> &'static str {
        match self {
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Priority::P1 => "form.priority.p1.short",
            Priority::P2 => "form.priority.p2.short",
            Priority::P3 => "form.priority.p3.short",
        }
    }

    pub fn rank(self) -> u8 {
        match self {
            Priority::P1 => 0,
            Priority::P2 => 1,
            Priority::P3 => 2,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Priority::P1 => Priority::P2,
            Priority::P2 => Priority::P3,
            Priority::P3 => Priority::P1,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Priority::P1 => Priority::P3,
            Priority::P2 => Priority::P1,
            Priority::P3 => Priority::P2,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::P2
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskStatus {
    Active,
    Done,
}

impl TaskStatus {
    pub fn translation_key(self) -> &'static str {
        match self {
            TaskStatus::Active => "status.active",
            TaskStatus::Done => "status.done",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub notes: String,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn matches_query(&self, query: &str) -> bool {
        if query.trim().is_empty() {
            return true;
        }

        let query = query.to_lowercase();
        self.title.to_lowercase().contains(&query) || self.notes.to_lowercase().contains(&query)
    }

    pub fn is_due_today(&self, today: NaiveDate) -> bool {
        self.due_date == Some(today)
    }

    pub fn is_overdue(&self, today: NaiveDate) -> bool {
        self.status == TaskStatus::Active && self.due_date.is_some_and(|due| due < today)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Done,
    DueToday,
    Overdue,
}

impl Filter {
    pub const ALL: [Filter; 5] = [
        Filter::All,
        Filter::Active,
        Filter::Done,
        Filter::DueToday,
        Filter::Overdue,
    ];

    pub fn translation_key(self) -> &'static str {
        match self {
            Filter::All => "filter.all",
            Filter::Active => "filter.active",
            Filter::Done => "filter.done",
            Filter::DueToday => "filter.due_today",
            Filter::Overdue => "filter.overdue",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Filter::All => Filter::Active,
            Filter::Active => Filter::Done,
            Filter::Done => Filter::DueToday,
            Filter::DueToday => Filter::Overdue,
            Filter::Overdue => Filter::All,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub next_id: u64,
    pub tasks: Vec<Task>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, draft: TaskDraft, now: DateTime<Utc>) -> AppResult<u64> {
        let task = Task {
            id: self.next_id,
            title: normalize_title(&draft.title)?,
            notes: draft.notes.trim().to_string(),
            priority: draft.priority,
            due_date: draft.due_date,
            status: TaskStatus::Active,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };
        self.next_id += 1;
        let id = task.id;
        self.tasks.push(task);
        Ok(id)
    }

    pub fn update_task(&mut self, id: u64, draft: TaskDraft, now: DateTime<Utc>) -> AppResult<()> {
        let task = self
            .tasks
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("task {id} not found"),
            )))?;

        task.title = normalize_title(&draft.title)?;
        task.notes = draft.notes.trim().to_string();
        task.priority = draft.priority;
        task.due_date = draft.due_date;
        task.updated_at = now;
        Ok(())
    }

    pub fn delete_task(&mut self, id: u64) -> bool {
        let original_len = self.tasks.len();
        self.tasks.retain(|task| task.id != id);
        self.tasks.len() != original_len
    }

    pub fn toggle_task(&mut self, id: u64, now: DateTime<Utc>) -> bool {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) else {
            return false;
        };

        match task.status {
            TaskStatus::Active => {
                task.status = TaskStatus::Done;
                task.completed_at = Some(now);
            }
            TaskStatus::Done => {
                task.status = TaskStatus::Active;
                task.completed_at = None;
            }
        }
        task.updated_at = now;
        true
    }

    pub fn set_priority(&mut self, id: u64, priority: Priority, now: DateTime<Utc>) -> bool {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) else {
            return false;
        };

        task.priority = priority;
        task.updated_at = now;
        true
    }

    pub fn get_task(&self, id: u64) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    pub fn visible_tasks(&self, filter: Filter, query: &str, today: NaiveDate) -> Vec<&Task> {
        let mut tasks = self
            .tasks
            .iter()
            .filter(|task| task.matches_query(query))
            .filter(|task| match filter {
                Filter::All => true,
                Filter::Active => task.status == TaskStatus::Active,
                Filter::Done => task.status == TaskStatus::Done,
                Filter::DueToday => task.is_due_today(today),
                Filter::Overdue => task.is_overdue(today),
            })
            .collect::<Vec<_>>();

        tasks.sort_by(|left, right| compare_tasks(left, right, today));
        tasks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDraft {
    pub title: String,
    pub notes: String,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
}

pub fn today_local() -> NaiveDate {
    Local::now().date_naive()
}

pub fn parse_due_date_input(input: &str) -> AppResult<Option<NaiveDate>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| AppError::InvalidDueDate)
}

fn normalize_title(input: &str) -> AppResult<String> {
    let title = input.trim();
    if title.is_empty() {
        return Err(AppError::EmptyTitle);
    }

    Ok(title.to_string())
}

fn compare_tasks(left: &Task, right: &Task, today: NaiveDate) -> Ordering {
    left.status
        .cmp(&right.status)
        .then_with(|| priority_bucket(left, today).cmp(&priority_bucket(right, today)))
        .then_with(|| left.priority.rank().cmp(&right.priority.rank()))
        .then_with(|| left.due_date.cmp(&right.due_date))
        .then_with(|| right.updated_at.cmp(&left.updated_at))
        .then_with(|| left.id.cmp(&right.id))
}

fn priority_bucket(task: &Task, today: NaiveDate) -> u8 {
    match (task.status, task.due_date) {
        (TaskStatus::Active, Some(date)) if date < today => 0,
        (TaskStatus::Active, Some(date)) if date == today => 1,
        (TaskStatus::Active, Some(_)) => 2,
        (TaskStatus::Active, None) => 3,
        (TaskStatus::Done, _) => 4,
    }
}
