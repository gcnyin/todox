use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::Value;

use crate::error::{AppError, AppResult};

pub const DEFAULT_LOCALE: &str = "en_US";
pub const BUILTIN_LOCALES: [&str; 2] = ["en_US", "zh_CN"];

pub type Params<'a> = [(&'a str, String)];

#[derive(Debug, Clone)]
pub struct I18n {
    locale: String,
    messages: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleOption {
    pub locale: String,
    pub source: LocaleSource,
    pub is_current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleSource {
    Builtin,
    Custom,
}

impl LocaleOption {
    pub fn display_name(&self) -> String {
        locale_display_name(&self.locale).to_string()
    }
}

impl I18n {
    pub fn load(locale: &str, i18n_dir: &Path) -> AppResult<Self> {
        let locale = normalize_locale(locale);
        let mut messages = builtin_messages(DEFAULT_LOCALE);

        if locale != DEFAULT_LOCALE && BUILTIN_LOCALES.iter().any(|item| *item == locale) {
            messages.extend(builtin_messages(&locale));
        }

        let custom_path = i18n_dir.join(format!("{locale}.json"));
        if custom_path.exists() {
            messages.extend(load_custom_messages(&custom_path)?);
        }

        Ok(Self { locale, messages })
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn t(&self, key: &str) -> String {
        self.messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn t_fmt(&self, key: &str, params: &[(&str, String)]) -> String {
        let mut text = self.t(key);
        for (name, value) in params {
            let pattern = format!("{{{name}}}");
            text = text.replace(&pattern, value);
        }
        text
    }
}

pub fn available_locales(i18n_dir: &Path, current_locale: &str) -> AppResult<Vec<LocaleOption>> {
    let mut locales = Vec::new();
    for locale in BUILTIN_LOCALES {
        locales.push(LocaleOption {
            locale: locale.to_string(),
            source: LocaleSource::Builtin,
            is_current: locale == current_locale,
        });
    }

    let mut seen = BUILTIN_LOCALES
        .iter()
        .map(|item| item.to_string())
        .collect::<BTreeSet<_>>();

    if i18n_dir.exists() {
        let mut custom = std::fs::read_dir(i18n_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .filter_map(|path| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
            })
            .collect::<Vec<_>>();
        custom.sort();

        for locale in custom {
            if seen.insert(locale.clone()) {
                locales.push(LocaleOption {
                    is_current: locale == current_locale,
                    locale,
                    source: LocaleSource::Custom,
                });
            }
        }
    }

    Ok(locales)
}

pub fn locale_display_name(locale: &str) -> &'static str {
    match locale {
        "en_US" => "English (United States)",
        "zh_CN" => "简体中文 (中国)",
        _ => "Custom language",
    }
}

pub fn normalize_locale(locale: &str) -> String {
    let trimmed = locale.trim();
    if trimmed.is_empty() {
        DEFAULT_LOCALE.to_string()
    } else {
        trimmed.to_string()
    }
}

fn load_custom_messages(path: &Path) -> AppResult<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let value: Value =
        serde_json::from_reader(reader).map_err(|source| AppError::InvalidLocaleFile {
            path: path.to_path_buf(),
            source,
        })?;

    let object = value
        .as_object()
        .ok_or_else(|| AppError::InvalidLocaleFile {
            path: path.to_path_buf(),
            source: serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "locale file must be a flat JSON object",
            )),
        })?;

    let mut messages = HashMap::new();
    for (key, value) in object {
        if let Some(text) = value.as_str() {
            messages.insert(key.clone(), text.to_string());
        }
    }

    Ok(messages)
}

fn builtin_messages(locale: &str) -> HashMap<String, String> {
    let entries = match locale {
        "zh_CN" => zh_cn_messages(),
        _ => en_us_messages(),
    };

    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn en_us_messages() -> Vec<(&'static str, &'static str)> {
    vec![
        ("app.title", "TODOX"),
        (
            "header.subtitle",
            "A calm command desk for the tasks that matter",
        ),
        ("header.state.ready", " READY "),
        ("header.state.notice", " NOTICE "),
        ("header.focus", "Focus"),
        ("header.focus.none", "none"),
        ("header.context.file", "File"),
        ("header.metric.active", "ACTIVE"),
        ("header.metric.today", "TODAY"),
        ("header.metric.overdue", "OVERDUE"),
        ("header.metric.done", "DONE"),
        ("top.filter", "Filter"),
        ("top.search", "Search"),
        ("top.search.off", "OFF"),
        ("top.visible_count", "{visible} / total {total}"),
        ("panel.hero", "CONTROL DESK"),
        ("panel.context", "LIVE CONTEXT"),
        ("panel.system", "SYSTEM PANEL"),
        ("panel.tasks", "TASK FLOW"),
        ("panel.tasks.help", " ? Help "),
        ("panel.sidebar", "FOCUS DETAILS"),
        ("panel.focus", "FOCUS PANEL"),
        ("list.summary", "{filter} · {visible} visible"),
        ("empty.badge", " EMPTY "),
        ("empty.title", "No tasks match the current view"),
        (
            "empty.body",
            "Press n to create a task, or adjust search / filter.",
        ),
        ("sidebar.focus", " FOCUS "),
        ("sidebar.idle", " IDLE "),
        ("sidebar.none_selected", "No task selected"),
        (
            "sidebar.none_selected.body",
            "Create a task or change filters to see details, due info, and notes here.",
        ),
        ("sidebar.priority", "Priority"),
        ("sidebar.due", "Due"),
        ("sidebar.updated", "Updated"),
        ("sidebar.notes", "Notes"),
        ("sidebar.notes.empty", "No notes yet"),
        ("focus.created", "Created"),
        ("focus.alert.overdue", " OVERDUE "),
        ("focus.alert.today", " TODAY "),
        ("focus.alert.active", " ACTIVE "),
        ("focus.alert.done", " DONE "),
        ("footer.edit_mode", " EDIT MODE "),
        (
            "footer.edit_mode.body",
            "You are editing a task. See the modal for actions.",
        ),
        (
            "footer.edit_mode.body.compact",
            "You are editing a task; list shortcuts are temporarily replaced.",
        ),
        (
            "footer.edit_mode.hint",
            "Once closed, list shortcuts return here.",
        ),
        ("footer.move", " Move  "),
        ("footer.new", " New  "),
        ("footer.edit", " Edit  "),
        ("footer.delete", " Delete  "),
        ("footer.help", " Help  "),
        ("footer.search", " Search  "),
        ("footer.filter", " Filter"),
        ("footer.priority", " Set priority  "),
        ("footer.priority_shift", " Adjust  "),
        ("footer.toggle_done", " Toggle done  "),
        ("footer.locale", " Language  "),
        ("footer.quit", " Quit"),
        ("footer.group.nav", " NAV "),
        ("footer.group.view", " VIEW "),
        ("footer.group.task", " TASK "),
        ("footer.group.priority", " PRIORITY "),
        ("footer.group.system", " SYSTEM "),
        ("modal.form.new", "New Task"),
        ("modal.form.edit", "Edit Task"),
        ("form.field.title", "Title"),
        ("form.field.notes", "Notes"),
        ("form.field.priority", "Priority"),
        ("form.field.due_date", "Due date"),
        ("form.state.active", "ACTIVE"),
        ("form.priority.p1.long", "P1 Critical"),
        ("form.priority.p2.long", "P2 Important"),
        ("form.priority.p3.long", "P3 Routine"),
        ("form.priority.p1.medium", "P1 Critical"),
        ("form.priority.p2.medium", "P2 Important"),
        ("form.priority.p3.medium", "P3 Routine"),
        ("form.priority.p1.short", "P1"),
        ("form.priority.p2.short", "P2"),
        ("form.priority.p3.short", "P3"),
        ("form.placeholder.title", "Type a clear task title"),
        (
            "form.placeholder.notes",
            "Add details, context, or next steps",
        ),
        ("form.placeholder.due_date", "YYYY-MM-DD"),
        ("form.footer.fields", "Fields"),
        ("form.footer.save", "Save"),
        ("form.footer.cancel", "Cancel"),
        ("form.footer.adjust_priority", "Adjust"),
        ("form.footer.pick_priority", "Pick"),
        ("modal.delete.title", "Delete confirmation"),
        ("modal.delete.badge", " DANGER "),
        ("modal.delete.body", "This task will be permanently deleted"),
        ("modal.delete.priority", "Priority"),
        ("modal.delete.due", "Due"),
        ("modal.delete.controls", "Enter / y confirm, Esc / n cancel"),
        ("modal.search.title", "Search"),
        ("modal.search.badge", " SEARCH "),
        (
            "modal.search.body",
            "Match title and notes. Submit empty text to clear search.",
        ),
        (
            "modal.search.placeholder",
            "Type a keyword and press Enter to apply search",
        ),
        ("modal.search.current", "Current text"),
        ("modal.search.controls", "Enter apply, Esc cancel"),
        (
            "modal.search.clear",
            "Submit an empty query to clear the current search.",
        ),
        ("modal.help.title", "Help"),
        ("modal.help.nav", " NAV "),
        ("modal.help.form", " FORM "),
        ("modal.help.view", " VIEW "),
        ("modal.help.locale", " LANG "),
        ("modal.help.move", "j/k or Up/Down"),
        ("modal.help.new", " New task  "),
        ("modal.help.edit", " Edit task"),
        ("modal.help.set_priority", " Set priority  "),
        ("modal.help.toggle_done", " Toggle done"),
        ("modal.help.search", " Search  "),
        ("modal.help.filter", " Cycle filter"),
        ("modal.help.delete", " Delete  "),
        ("modal.help.quit", " Quit"),
        ("modal.help.locale_switch", " Open language picker"),
        (
            "modal.help.form.body",
            "Tab / Shift+Tab / Up / Down switch fields, Enter saves, Esc cancels.",
        ),
        (
            "modal.help.form.priority",
            "In the form modal, Left/Right, h/l, and 1/2/3 only work on the priority field.",
        ),
        (
            "modal.help.view.body",
            "On wide terminals, the right panel shows focus details.",
        ),
        ("modal.help.close", "Press Esc, Enter, or ? to close help."),
        ("modal.locale.title", "Language"),
        ("modal.locale.badge", " LOCALE "),
        ("modal.locale.current", "Current"),
        ("modal.locale.builtin", "Built-in"),
        ("modal.locale.custom", "Custom"),
        ("modal.locale.controls", "Enter apply, Esc cancel"),
        ("filter.all", "All"),
        ("filter.active", "Active"),
        ("filter.done", "Done"),
        ("filter.due_today", "Due today"),
        ("filter.overdue", "Overdue"),
        ("status.active", "Active"),
        ("status.done", "Done"),
        ("due.overdue", "Overdue {date}"),
        ("due.today", "Today {date}"),
        ("due.upcoming", "Due {date}"),
        ("due.none", "No due date"),
        ("list.due.overdue", "Overdue"),
        ("list.due.today", "Today"),
        ("list.due.date", "{date}"),
        ("list.due.none", "No due"),
        (
            "message.config_reset",
            "Config was invalid and has been reset to English (United States).",
        ),
        ("error.prefix", "Error"),
        ("error.io", "I/O error: {details}"),
        ("error.json", "JSON parse error: {details}"),
        (
            "error.missing_home",
            "Could not resolve the home directory for the default data path",
        ),
        ("error.invalid_due_date", "Invalid due date. Use YYYY-MM-DD"),
        ("error.empty_title", "Title cannot be empty"),
        (
            "error.corrupted_data",
            "Data file is corrupted and cannot be read: {path}",
        ),
        (
            "error.invalid_locale_file",
            "Locale file is invalid: {path}",
        ),
        (
            "error.invalid_config",
            "Config file is invalid and was reset: {path}",
        ),
    ]
}

fn zh_cn_messages() -> Vec<(&'static str, &'static str)> {
    vec![
        ("app.title", "TODOX"),
        ("header.subtitle", "把重要任务收进一个安静清晰的指挥台"),
        ("header.state.ready", " 就绪 "),
        ("header.state.notice", " 提示 "),
        ("header.focus", "焦点"),
        ("header.focus.none", "无"),
        ("header.context.file", "文件"),
        ("header.metric.active", "未完成"),
        ("header.metric.today", "今日到期"),
        ("header.metric.overdue", "已逾期"),
        ("header.metric.done", "已完成"),
        ("top.filter", "过滤"),
        ("top.search", "搜索"),
        ("top.search.off", "OFF"),
        ("top.visible_count", "{visible} / 共 {total}"),
        ("panel.hero", "任务总览"),
        ("panel.context", "当前上下文"),
        ("panel.system", "系统面板"),
        ("panel.tasks", "任务流"),
        ("panel.tasks.help", " ? 帮助 "),
        ("panel.sidebar", "焦点详情"),
        ("panel.focus", "焦点面板"),
        ("list.summary", "{filter} · 可见 {visible} 条"),
        ("empty.badge", " 空列表 "),
        ("empty.title", "当前没有符合条件的任务"),
        ("empty.body", "按 n 创建新任务，或调整搜索 / 过滤条件。"),
        ("sidebar.focus", " 焦点 "),
        ("sidebar.idle", " 空闲 "),
        ("sidebar.none_selected", "当前没有选中任务"),
        (
            "sidebar.none_selected.body",
            "创建或筛选出任务后，这里会显示焦点详情、截止信息和备注内容。",
        ),
        ("sidebar.priority", "优先级"),
        ("sidebar.due", "截止"),
        ("sidebar.updated", "更新时间"),
        ("sidebar.notes", "备注"),
        ("sidebar.notes.empty", "暂无备注"),
        ("focus.created", "创建时间"),
        ("focus.alert.overdue", " 已逾期 "),
        ("focus.alert.today", " 今天到期 "),
        ("focus.alert.active", " 进行中 "),
        ("focus.alert.done", " 已完成 "),
        ("footer.edit_mode", " 编辑模式 "),
        (
            "footer.edit_mode.body",
            "当前正在编辑任务，操作提示见弹窗。",
        ),
        (
            "footer.edit_mode.body.compact",
            "当前正在编辑任务；列表快捷键已临时切换为表单操作。",
        ),
        (
            "footer.edit_mode.hint",
            "关闭弹窗后，这里会恢复列表视图快捷键。",
        ),
        ("footer.move", " 移动  "),
        ("footer.new", " 新建  "),
        ("footer.edit", " 编辑  "),
        ("footer.delete", " 删除  "),
        ("footer.help", " 帮助  "),
        ("footer.search", " 搜索  "),
        ("footer.filter", " 过滤"),
        ("footer.priority", " 设优先级  "),
        ("footer.priority_shift", " 调整  "),
        ("footer.toggle_done", " 切换完成  "),
        ("footer.locale", " 语言  "),
        ("footer.quit", " 退出"),
        ("footer.group.nav", " 导航 "),
        ("footer.group.view", " 视图 "),
        ("footer.group.task", " 任务 "),
        ("footer.group.priority", " 优先级 "),
        ("footer.group.system", " 系统 "),
        ("modal.form.new", "新建任务"),
        ("modal.form.edit", "编辑任务"),
        ("form.field.title", "标题"),
        ("form.field.notes", "备注"),
        ("form.field.priority", "优先级"),
        ("form.field.due_date", "截止日期"),
        ("form.state.active", "当前"),
        ("form.priority.p1.long", "P1 紧急重要"),
        ("form.priority.p2.long", "P2 重要不急"),
        ("form.priority.p3.long", "P3 日常任务"),
        ("form.priority.p1.medium", "P1 紧急"),
        ("form.priority.p2.medium", "P2 重要"),
        ("form.priority.p3.medium", "P3 日常"),
        ("form.priority.p1.short", "P1"),
        ("form.priority.p2.short", "P2"),
        ("form.priority.p3.short", "P3"),
        ("form.placeholder.title", "输入清晰的任务标题"),
        ("form.placeholder.notes", "补充背景、细节或下一步"),
        ("form.placeholder.due_date", "YYYY-MM-DD"),
        ("form.footer.fields", "字段"),
        ("form.footer.save", "保存"),
        ("form.footer.cancel", "取消"),
        ("form.footer.adjust_priority", "调整"),
        ("form.footer.pick_priority", "选择"),
        ("modal.delete.title", "删除确认"),
        ("modal.delete.badge", " 危险 "),
        ("modal.delete.body", "这条任务将被永久删除"),
        ("modal.delete.priority", "优先级"),
        ("modal.delete.due", "截止"),
        ("modal.delete.controls", "Enter / y 确认，Esc / n 取消"),
        ("modal.search.title", "搜索"),
        ("modal.search.badge", " 搜索 "),
        (
            "modal.search.body",
            "匹配标题与备注，留空后回车可清除搜索。",
        ),
        ("modal.search.placeholder", "输入关键字后按 Enter 应用搜索"),
        ("modal.search.current", "当前内容"),
        ("modal.search.controls", "Enter 应用，Esc 取消"),
        (
            "modal.search.clear",
            "输入空内容后提交，即可清除当前搜索条件。",
        ),
        ("modal.help.title", "帮助"),
        ("modal.help.nav", " 导航 "),
        ("modal.help.form", " 表单 "),
        ("modal.help.view", " 视图 "),
        ("modal.help.locale", " 语言 "),
        ("modal.help.move", "j/k 或 上下方向键"),
        ("modal.help.new", " 新建任务  "),
        ("modal.help.edit", " 编辑任务"),
        ("modal.help.set_priority", " 设优先级  "),
        ("modal.help.toggle_done", " 切换完成"),
        ("modal.help.search", " 搜索  "),
        ("modal.help.filter", " 切换过滤"),
        ("modal.help.delete", " 删除  "),
        ("modal.help.quit", " 退出"),
        ("modal.help.locale_switch", " 打开语言选择"),
        (
            "modal.help.form.body",
            "Tab / Shift+Tab / 上下键 切字段，Enter 保存，Esc 取消。",
        ),
        (
            "modal.help.form.priority",
            "编辑弹窗中，左右键、h/l、1/2/3 仅在优先级字段生效。",
        ),
        ("modal.help.view.body", "宽终端会显示右侧焦点详情面板。"),
        ("modal.help.close", "按 Esc、Enter 或 ? 关闭帮助。"),
        ("modal.locale.title", "语言切换"),
        ("modal.locale.badge", " 语言 "),
        ("modal.locale.current", "当前"),
        ("modal.locale.builtin", "内置"),
        ("modal.locale.custom", "自定义"),
        ("modal.locale.controls", "Enter 应用，Esc 取消"),
        ("filter.all", "全部"),
        ("filter.active", "未完成"),
        ("filter.done", "已完成"),
        ("filter.due_today", "今天到期"),
        ("filter.overdue", "已逾期"),
        ("status.active", "未完成"),
        ("status.done", "已完成"),
        ("due.overdue", "逾期 {date}"),
        ("due.today", "今天 {date}"),
        ("due.upcoming", "截止 {date}"),
        ("due.none", "无截止日期"),
        ("list.due.overdue", "逾期"),
        ("list.due.today", "今天"),
        ("list.due.date", "{date}"),
        ("list.due.none", "无截止"),
        (
            "message.config_reset",
            "配置文件已损坏，已重建为默认英文设置。",
        ),
        ("error.prefix", "错误"),
        ("error.io", "I/O 错误: {details}"),
        ("error.json", "JSON 解析错误: {details}"),
        (
            "error.missing_home",
            "无法解析用户主目录，无法确定默认数据路径",
        ),
        (
            "error.invalid_due_date",
            "截止日期格式无效，请使用 YYYY-MM-DD",
        ),
        ("error.empty_title", "标题不能为空"),
        ("error.corrupted_data", "数据文件内容损坏，无法读取: {path}"),
        ("error.invalid_locale_file", "语言文件无效: {path}"),
        ("error.invalid_config", "配置文件无效，已重建: {path}"),
    ]
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{DEFAULT_LOCALE, I18n, available_locales};

    #[test]
    fn missing_keys_fall_back_to_builtin_english() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join("ja_JP.json"), r#"{"top.filter":"Filta"}"#).unwrap();

        let i18n = I18n::load("ja_JP", dir.path()).unwrap();
        assert_eq!(i18n.t("top.filter"), "Filta");
        assert_eq!(i18n.t("top.search"), "Search");
    }

    #[test]
    fn available_locales_lists_builtin_and_custom_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("ja_JP.json"), "{}").unwrap();
        let locales = available_locales(dir.path(), DEFAULT_LOCALE).unwrap();
        let ids = locales
            .into_iter()
            .map(|item| item.locale)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["en_US", "zh_CN", "ja_JP"]);
    }
}
