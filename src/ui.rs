use chrono::{Local, NaiveDate};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap, block::BorderType,
};

use crate::app::{App, AppMode, FormField, TaskForm, locale_source_label};
use crate::model::{Priority, Task, TaskStatus, today_local};

const BG: Color = Color::Rgb(7, 11, 18);
const SURFACE: Color = Color::Rgb(13, 18, 28);
const SURFACE_ALT: Color = Color::Rgb(18, 24, 36);
const SURFACE_ELEVATED: Color = Color::Rgb(22, 29, 43);
const SURFACE_SELECTED: Color = Color::Rgb(25, 39, 58);
const SURFACE_GLOW: Color = Color::Rgb(16, 44, 55);
const BORDER: Color = Color::Rgb(70, 89, 115);
const TEXT: Color = Color::Rgb(236, 241, 247);
const TEXT_MUTED: Color = Color::Rgb(174, 185, 202);
const TEXT_DIM: Color = Color::Rgb(118, 132, 154);
const ACCENT: Color = Color::Rgb(0, 196, 178);
const ACCENT_SOFT: Color = Color::Rgb(129, 235, 226);
const INFO: Color = Color::Rgb(102, 173, 255);
const WARNING: Color = Color::Rgb(245, 194, 89);
const DANGER: Color = Color::Rgb(255, 111, 97);
const SUCCESS: Color = Color::Rgb(124, 211, 169);
const EDIT_FIELD_BG: Color = Color::Rgb(22, 28, 40);
const EDIT_FIELD_ACTIVE_BG: Color = Color::Rgb(28, 40, 55);
const EDIT_BORDER: Color = Color::Rgb(79, 100, 129);
const EDIT_BORDER_ACTIVE: Color = Color::Rgb(123, 225, 212);

pub fn draw(frame: &mut Frame, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(BG)),
        frame.area(),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height(frame.area().width)),
            Constraint::Min(10),
            Constraint::Length(footer_height(app, frame.area().width)),
        ])
        .split(frame.area());

    render_top_bar(frame, app, layout[0]);
    render_dashboard(frame, app, layout[1]);
    render_footer(frame, app, layout[2]);
    render_modal(frame, app);
}

fn header_height(_width: u16) -> u16 {
    3
}

fn render_top_bar(frame: &mut Frame, app: &App, area: Rect) {
    let search = if app.search_query().trim().is_empty() {
        app.t("top.search.off")
    } else {
        preview_text(app.search_query(), 10, &app.t("top.search.off"))
    };
    let status = app.message().map(|message| {
        vec![
            Span::styled(" · ", Style::default().fg(TEXT_DIM)),
            badge(app.t("header.state.notice"), INFO, SURFACE_ALT),
            Span::raw(" "),
            Span::styled(truncate_text(message, 16), Style::default().fg(TEXT_MUTED)),
        ]
    });

    let mut spans = vec![
        badge(app.t("panel.hero"), ACCENT_SOFT, SURFACE_GLOW),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled(app.t("top.filter"), Style::default().fg(TEXT_DIM)),
        Span::raw(" "),
        Span::styled(
            app.t(app.filter().translation_key()),
            Style::default().fg(ACCENT_SOFT),
        ),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled(app.t("top.search"), Style::default().fg(TEXT_DIM)),
        Span::raw(" "),
        Span::styled(search, Style::default().fg(WARNING)),
    ];
    if let Some(status) = status {
        spans.extend(status);
    }
    spans.extend([
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        header_metric_text(app, "header.metric.active", app.active_count(), ACCENT),
        Span::raw(" "),
        header_metric_text(app, "header.metric.today", app.due_today_count(), WARNING),
        Span::raw(" "),
        header_metric_text(app, "header.metric.overdue", app.overdue_count(), DANGER),
        Span::raw(" "),
        header_metric_text(app, "header.metric.done", app.done_count(), SUCCESS),
    ]);

    let paragraph = Paragraph::new(Line::from(spans))
        .block(header_block())
        .style(Style::default().bg(SURFACE).fg(TEXT));
    frame.render_widget(paragraph, area);
}

fn header_metric_text(app: &App, label_key: &str, value: usize, accent: Color) -> Span<'static> {
    Span::styled(
        format!("{} {}", app.t(label_key), value),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )
}

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 112 && area.height >= 18 {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(56), Constraint::Length(38)])
            .split(area);
        render_task_list(frame, app, layout[0]);
        render_sidebar(frame, app, layout[1]);
    } else if area.height >= 24 {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(12), Constraint::Length(11)])
            .split(area);
        render_task_list(frame, app, layout[0]);
        render_sidebar(frame, app, layout[1]);
    } else {
        render_task_list(frame, app, area);
    }
}

fn render_task_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible_tasks = app.visible_tasks();
    let today = today_local();
    let summary = app.t_fmt(
        "list.summary",
        &[
            ("filter", app.t(app.filter().translation_key())),
            ("visible", visible_tasks.len().to_string()),
        ],
    );

    if visible_tasks.is_empty() {
        let paragraph = Paragraph::new(vec![
            Line::from(vec![
                badge(app.t("empty.badge"), TEXT_MUTED, SURFACE_ALT),
                Span::raw(" "),
                Span::styled(
                    app.t("empty.title"),
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                app.t("empty.body"),
                Style::default().fg(TEXT_MUTED),
            )),
            Line::from(""),
            Line::from(vec![
                keycap("n"),
                hint_text(&app.t("footer.new")),
                keycap("/"),
                hint_text(&app.t("footer.search")),
                keycap("f"),
                hint_text(&app.t("footer.filter")),
            ]),
        ])
        .block(
            panel_block(&app.t("panel.tasks"), ACCENT_SOFT)
                .title_bottom(Span::styled(summary, Style::default().fg(TEXT_DIM))),
        )
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    }

    let items = visible_tasks
        .iter()
        .enumerate()
        .map(|(index, task)| render_task_item(app, task, today, index))
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(app.selected_visible_index());

    let list = List::new(items)
        .block(
            panel_block(&app.t("panel.tasks"), ACCENT_SOFT)
                .title_bottom(Span::styled(summary, Style::default().fg(TEXT_DIM))),
        )
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .highlight_style(
            Style::default()
                .bg(SURFACE_SELECTED)
                .fg(TEXT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▍ ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_task_item(app: &App, task: &Task, today: NaiveDate, index: usize) -> ListItem<'static> {
    let (priority_fg, priority_bg) = priority_tone(task.priority);
    let (status_fg, status_bg) = status_tone(task.status);
    let (due_fg, due_bg) = due_tone(task, today);
    let title_style = if task.status == TaskStatus::Done {
        Style::default()
            .fg(TEXT_MUTED)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::CROSSED_OUT)
    } else {
        Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
    };

    ListItem::new(Line::from(vec![
        badge(format!(" {:02} ", index + 1), TEXT_DIM, SURFACE_ALT),
        Span::raw(" "),
        badge(
            format!(" {} ", app.t(task.status.translation_key())),
            status_fg,
            status_bg,
        ),
        Span::raw(" "),
        badge(
            format!(" {} ", priority_label(app, task.priority, true, true)),
            priority_fg,
            priority_bg,
        ),
        Span::raw(" "),
        badge(
            format!(" {} ", task_due_chip_label(app, task, today)),
            due_fg,
            due_bg,
        ),
        Span::raw(" "),
        Span::styled(truncate_text(&task.title, 54), title_style),
    ]))
    .style(Style::default().bg(SURFACE_ALT))
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let today = today_local();
    let lines = if let Some(task) = app.selected_task() {
        let (priority_fg, priority_bg) = priority_tone(task.priority);
        let (_status_fg, status_bg) = status_tone(task.status);
        let notes = if task.notes.trim().is_empty() {
            app.t("sidebar.notes.empty")
        } else {
            task.notes.clone()
        };
        let (alert_key, alert_color) = focus_alert(task, today);

        vec![
            Line::from(vec![badge(app.t(alert_key), alert_color, status_bg)]),
            Line::from(""),
            Line::from(Span::styled(
                task.title.clone(),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            stat_line(
                app.t("sidebar.priority"),
                badge(
                    format!(" {} ", priority_label(app, task.priority, false, false)),
                    priority_fg,
                    priority_bg,
                ),
            ),
            stat_line(
                app.t("sidebar.due"),
                badge(
                    format!(" {} ", task_due_label(app, task, today)),
                    WARNING,
                    SURFACE_ALT,
                ),
            ),
            stat_line(
                app.t("sidebar.updated"),
                Span::styled(
                    task.updated_at
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string(),
                    Style::default().fg(TEXT_MUTED),
                ),
            ),
            stat_line(
                app.t("focus.created"),
                Span::styled(
                    task.created_at
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string(),
                    Style::default().fg(TEXT_MUTED),
                ),
            ),
            Line::from(""),
            section_title(app.t("sidebar.notes"), ACCENT_SOFT),
            Line::from(Span::styled(notes, Style::default().fg(TEXT_MUTED))),
        ]
    } else {
        vec![
            Line::from(vec![
                badge(app.t("sidebar.idle"), TEXT_MUTED, SURFACE_ALT),
                Span::raw(" "),
                Span::styled(
                    app.t("sidebar.none_selected"),
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                app.t("sidebar.none_selected.body"),
                Style::default().fg(TEXT_MUTED),
            )),
        ]
    };

    let paragraph = Paragraph::new(lines)
        .block(panel_block(&app.t("panel.focus"), INFO))
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn section_title(text: String, accent: Color) -> Line<'static> {
    Line::from(vec![Span::styled(
        text,
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )])
}

fn stat_line(label: String, value: Span<'static>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{} ", label), Style::default().fg(TEXT_DIM)),
        value,
    ])
}

fn focus_alert(task: &Task, today: NaiveDate) -> (&'static str, Color) {
    if task.status == TaskStatus::Done {
        return ("focus.alert.done", SUCCESS);
    }
    if task.is_overdue(today) {
        return ("focus.alert.overdue", DANGER);
    }
    if task.is_due_today(today) {
        return ("focus.alert.today", WARNING);
    }
    ("focus.alert.active", ACCENT)
}

fn footer_height(app: &App, width: u16) -> u16 {
    let content_rows = match app.mode() {
        AppMode::CreateEditModal(_) => 3,
        _ => {
            if width < 110 {
                5
            } else {
                4
            }
        }
    };

    content_rows + 2
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let lines = if matches!(app.mode(), AppMode::CreateEditModal(_)) {
        render_edit_footer(app, area.width)
    } else if area.width < 110 {
        vec![
            footer_group(
                app.t("footer.group.nav"),
                vec![
                    keycap("j/k"),
                    hint_text(&app.t("footer.move")),
                    keycap("/"),
                    hint_text(&app.t("footer.search")),
                ],
            ),
            footer_group(
                app.t("footer.group.view"),
                vec![
                    keycap("f"),
                    hint_text(&app.t("footer.filter")),
                    keycap("g"),
                    hint_text(&app.t("footer.locale")),
                    keycap("?"),
                    hint_text(&app.t("footer.help")),
                ],
            ),
            footer_group(
                app.t("footer.group.task"),
                vec![
                    keycap("n"),
                    hint_text(&app.t("footer.new")),
                    keycap("e"),
                    hint_text(&app.t("footer.edit")),
                    keycap("d"),
                    hint_text(&app.t("footer.delete")),
                ],
            ),
            footer_group(
                app.t("footer.group.priority"),
                vec![
                    keycap("Space"),
                    hint_text(&app.t("footer.toggle_done")),
                    keycap("1/2/3"),
                    hint_text(&app.t("footer.priority")),
                ],
            ),
            footer_group(
                app.t("footer.group.system"),
                vec![keycap("Esc/q"), hint_text(&app.t("footer.quit"))],
            ),
        ]
    } else {
        vec![
            footer_group(
                app.t("footer.group.nav"),
                vec![
                    keycap("j/k"),
                    hint_text(&app.t("footer.move")),
                    keycap("/"),
                    hint_text(&app.t("footer.search")),
                    keycap("f"),
                    hint_text(&app.t("footer.filter")),
                ],
            ),
            footer_group(
                app.t("footer.group.task"),
                vec![
                    keycap("n"),
                    hint_text(&app.t("footer.new")),
                    keycap("e"),
                    hint_text(&app.t("footer.edit")),
                    keycap("Space"),
                    hint_text(&app.t("footer.toggle_done")),
                    keycap("d"),
                    hint_text(&app.t("footer.delete")),
                ],
            ),
            footer_group(
                app.t("footer.group.priority"),
                vec![
                    keycap("1/2/3"),
                    hint_text(&app.t("footer.priority")),
                    keycap("h/l"),
                    hint_text(&app.t("footer.priority_shift")),
                ],
            ),
            footer_group(
                app.t("footer.group.system"),
                vec![
                    keycap("g"),
                    hint_text(&app.t("footer.locale")),
                    keycap("?"),
                    hint_text(&app.t("footer.help")),
                    keycap("Esc/q"),
                    hint_text(&app.t("footer.quit")),
                ],
            ),
        ]
    };

    let paragraph = Paragraph::new(lines)
        .block(panel_block("", ACCENT))
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_edit_footer(app: &App, _width: u16) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            badge(app.t("footer.edit_mode"), ACCENT_SOFT, SURFACE_GLOW),
            Span::raw(" "),
            Span::styled(
                app.t("footer.edit_mode.body.compact"),
                Style::default().fg(TEXT),
            ),
        ]),
        footer_group(
            app.t("footer.group.nav"),
            vec![
                keycap("Tab/Up/Down"),
                hint_text(&app.t("form.footer.fields")),
                keycap("Enter"),
                hint_text(&app.t("form.footer.save")),
                keycap("Esc"),
                hint_text(&app.t("form.footer.cancel")),
            ],
        ),
        footer_group(
            app.t("footer.group.priority"),
            vec![
                keycap("Left/Right"),
                hint_text(&app.t("form.footer.adjust_priority")),
                keycap("1/2/3"),
                hint_text(&app.t("form.footer.pick_priority")),
            ],
        ),
    ]
}

fn footer_group(title: String, mut spans: Vec<Span<'static>>) -> Line<'static> {
    let mut content = vec![badge(title, INFO, SURFACE_ALT), Span::raw(" ")];
    content.append(&mut spans);
    Line::from(content)
}

fn render_modal(frame: &mut Frame, app: &App) {
    match app.mode() {
        AppMode::List => {}
        AppMode::CreateEditModal(form) => render_form_modal(frame, app, form),
        AppMode::DeleteConfirm => render_delete_confirm(frame, app),
        AppMode::Search { draft } => render_search_modal(frame, app, draft),
        AppMode::Help => render_help_modal(frame, app),
        AppMode::LocalePicker { .. } => render_locale_modal(frame, app),
    }
}

fn render_form_modal(frame: &mut Frame, app: &App, form: &TaskForm) {
    let area = centered_rect(72, 18, frame.area());
    frame.render_widget(Clear, area);
    let block = modal_block(&app.t(form.title_key()), ACCENT_SOFT);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(form_text_line(
            &form.title,
            &app.t("form.placeholder.title"),
            form.field == FormField::Title,
        ))
        .block(input_block(
            app.t("form.field.title"),
            form.field == FormField::Title,
            app.t("form.state.active"),
        ))
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .wrap(Wrap { trim: true }),
        form_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(render_priority_options(app, form, form_chunks[1].width))
            .block(input_block(
                app.t("form.field.priority"),
                form.field == FormField::Priority,
                app.t("form.state.active"),
            ))
            .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
            .wrap(Wrap { trim: true }),
        form_chunks[1],
    );
    frame.render_widget(
        Paragraph::new(form_text_line(
            &form.notes,
            &app.t("form.placeholder.notes"),
            form.field == FormField::Notes,
        ))
        .block(input_block(
            app.t("form.field.notes"),
            form.field == FormField::Notes,
            app.t("form.state.active"),
        ))
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .wrap(Wrap { trim: true }),
        form_chunks[2],
    );
    frame.render_widget(
        Paragraph::new(form_text_line(
            &form.due_date,
            &app.t("form.placeholder.due_date"),
            form.field == FormField::DueDate,
        ))
        .block(input_block(
            app.t("form.field.due_date"),
            form.field == FormField::DueDate,
            app.t("form.state.active"),
        ))
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .wrap(Wrap { trim: true }),
        form_chunks[3],
    );

    if let Some(error) = &form.error {
        frame.render_widget(
            Paragraph::new(vec![Line::from(vec![
                badge(app.t("header.state.notice"), DANGER, SURFACE_ALT),
                Span::raw(" "),
                Span::styled(error.clone(), Style::default().fg(DANGER)),
            ])])
            .block(modal_block(&app.t(form.title_key()), DANGER))
            .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
            .wrap(Wrap { trim: true }),
            centered_rect(60, 5, frame.area()),
        );
    }
}

fn input_block(title: String, active: bool, active_label: String) -> Block<'static> {
    let title = if active {
        format!(" {} · {} ", title, active_label)
    } else {
        format!(" {} ", title)
    };

    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(if active {
            EDIT_BORDER_ACTIVE
        } else {
            EDIT_BORDER
        }))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(if active {
            EDIT_FIELD_ACTIVE_BG
        } else {
            EDIT_FIELD_BG
        }))
}

fn form_text_line(text: &str, placeholder: &str, active: bool) -> Line<'static> {
    if text.trim().is_empty() {
        Line::from(Span::styled(
            placeholder.to_string(),
            Style::default()
                .fg(if active { ACCENT_SOFT } else { TEXT_DIM })
                .add_modifier(Modifier::ITALIC),
        ))
    } else {
        Line::from(Span::styled(text.to_string(), Style::default().fg(TEXT)))
    }
}

fn render_priority_options(app: &App, form: &TaskForm, width: u16) -> Vec<Line<'static>> {
    let compact = width < 44;
    let ultra_compact = width < 32;
    let priority = form.priority;
    let (fg, bg) = priority_tone(priority);

    vec![Line::from(vec![badge(
        priority_label(app, priority, compact, ultra_compact),
        fg,
        bg,
    )])]
}

fn priority_label(app: &App, priority: Priority, compact: bool, ultra_compact: bool) -> String {
    let key = if ultra_compact {
        priority.translation_key()
    } else if compact {
        match priority {
            Priority::P1 => "form.priority.p1.medium",
            Priority::P2 => "form.priority.p2.medium",
            Priority::P3 => "form.priority.p3.medium",
        }
    } else {
        match priority {
            Priority::P1 => "form.priority.p1.long",
            Priority::P2 => "form.priority.p2.long",
            Priority::P3 => "form.priority.p3.long",
        }
    };
    app.t(key)
}

fn render_delete_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(68, 12, frame.area());
    frame.render_widget(Clear, area);

    let title = app
        .selected_task()
        .map(|task| task.title.clone())
        .unwrap_or_default();
    let due = app
        .selected_task()
        .map(|task| task_due_label(app, task, today_local()))
        .unwrap_or_else(|| app.t("due.none"));
    let priority = app
        .selected_task()
        .map(|task| priority_label(app, task.priority, true, false))
        .unwrap_or_default();

    let paragraph = Paragraph::new(vec![
        Line::from(vec![
            badge(app.t("modal.delete.badge"), DANGER, SURFACE_ALT),
            Span::raw(" "),
            Span::styled(
                app.t("modal.delete.body"),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                format!("{} ", app.t("modal.delete.priority")),
                Style::default().fg(TEXT_DIM),
            ),
            badge(format!(" {} ", priority), WARNING, SURFACE_ALT),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", app.t("modal.delete.due")),
                Style::default().fg(TEXT_DIM),
            ),
            badge(format!(" {} ", due), INFO, SURFACE_ALT),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            app.t("modal.delete.controls"),
            Style::default().fg(TEXT_MUTED),
        )),
    ])
    .block(modal_block(&app.t("modal.delete.title"), DANGER))
    .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
    .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_search_modal(frame: &mut Frame, app: &App, draft: &str) {
    let area = centered_rect(72, 12, frame.area());
    frame.render_widget(Clear, area);
    let block = modal_block(&app.t("modal.search.title"), ACCENT);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(vec![Line::from(vec![
            badge(app.t("modal.search.badge"), ACCENT, SURFACE_GLOW),
            Span::raw(" "),
            Span::styled(app.t("modal.search.body"), Style::default().fg(TEXT_MUTED)),
        ])])
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .wrap(Wrap { trim: true }),
        layout[0],
    );

    let query = if draft.trim().is_empty() {
        app.t("modal.search.placeholder")
    } else {
        draft.to_string()
    };
    frame.render_widget(
        Paragraph::new(query)
            .block(input_block(
                app.t("modal.search.current"),
                true,
                app.t("form.state.active"),
            ))
            .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
            .wrap(Wrap { trim: true }),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                app.t("modal.search.controls"),
                Style::default().fg(TEXT_MUTED),
            )),
            Line::from(Span::styled(
                app.t("modal.search.clear"),
                Style::default().fg(TEXT_DIM),
            )),
        ])
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .wrap(Wrap { trim: true }),
        layout[2],
    );
}

fn render_help_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(82, 18, frame.area());
    frame.render_widget(Clear, area);

    let lines = vec![
        footer_group(
            app.t("footer.group.nav"),
            vec![
                keycap("j/k"),
                hint_text(&app.t("modal.help.move")),
                keycap("/"),
                hint_text(&app.t("modal.help.search")),
                keycap("f"),
                hint_text(&app.t("modal.help.filter")),
            ],
        ),
        footer_group(
            app.t("footer.group.task"),
            vec![
                keycap("n"),
                hint_text(&app.t("modal.help.new")),
                keycap("e"),
                hint_text(&app.t("modal.help.edit")),
                keycap("d"),
                hint_text(&app.t("modal.help.delete")),
            ],
        ),
        footer_group(
            app.t("footer.group.priority"),
            vec![
                keycap("1/2/3"),
                hint_text(&app.t("modal.help.set_priority")),
                keycap("Space"),
                hint_text(&app.t("modal.help.toggle_done")),
                keycap("g"),
                hint_text(&app.t("modal.help.locale_switch")),
            ],
        ),
        Line::from(""),
        Line::from(vec![
            badge(app.t("modal.help.form"), ACCENT_SOFT, SURFACE_GLOW),
            Span::raw(" "),
            Span::styled(
                app.t("modal.help.form.body"),
                Style::default().fg(TEXT_MUTED),
            ),
        ]),
        Line::from(Span::styled(
            app.t("modal.help.form.priority"),
            Style::default().fg(TEXT_MUTED),
        )),
        Line::from(""),
        Line::from(vec![
            badge(app.t("footer.group.system"), INFO, SURFACE_ALT),
            Span::raw(" "),
            Span::styled(app.t("modal.help.close"), Style::default().fg(TEXT)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(modal_block(&app.t("modal.help.title"), ACCENT_SOFT))
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_locale_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 14, frame.area());
    frame.render_widget(Clear, area);
    let block = modal_block(&app.t("modal.locale.title"), ACCENT);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(2)])
        .split(inner);

    let (options, selected) = app.locale_picker_options().unwrap();
    let items = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let marker = if option.is_current {
                format!(" {}", app.t("modal.locale.current"))
            } else {
                String::new()
            };
            let source = locale_source_label(app, option.source);
            ListItem::new(Line::from(vec![
                Span::styled(
                    option.locale.clone(),
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("({})", option.display_name()),
                    Style::default().fg(TEXT_MUTED),
                ),
                Span::raw(" "),
                badge(format!(" {} ", source), ACCENT, SURFACE_ALT),
                if marker.is_empty() {
                    Span::raw("")
                } else {
                    Span::styled(marker, Style::default().fg(WARNING))
                },
            ]))
            .style(if index == selected {
                Style::default().bg(SURFACE_SELECTED)
            } else {
                Style::default().bg(SURFACE_ALT)
            })
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(selected));
    let list = List::new(items)
        .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT))
        .highlight_style(Style::default().bg(SURFACE_SELECTED).fg(TEXT))
        .highlight_symbol("▍ ");
    frame.render_stateful_widget(list, layout[0], &mut state);

    frame.render_widget(
        Paragraph::new(app.t("modal.locale.controls"))
            .style(Style::default().bg(SURFACE_ELEVATED).fg(TEXT_MUTED))
            .wrap(Wrap { trim: true }),
        layout[1],
    );
}

fn panel_block(title: &str, accent: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(BORDER))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(SURFACE))
}

fn header_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .padding(Padding::new(1, 0, 0, 0))
        .style(Style::default().bg(SURFACE))
}

fn modal_block(title: &str, accent: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(accent))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(SURFACE_ELEVATED))
}

fn badge(text: impl Into<String>, fg: Color, bg: Color) -> Span<'static> {
    Span::styled(
        text.into(),
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    )
}

fn keycap(text: &str) -> Span<'static> {
    badge(format!(" {} ", text), ACCENT, SURFACE_GLOW)
}

fn hint_text(text: &str) -> Span<'static> {
    Span::styled(text.to_string(), Style::default().fg(TEXT_MUTED))
}

fn priority_tone(priority: Priority) -> (Color, Color) {
    match priority {
        Priority::P1 => (DANGER, Color::Rgb(76, 24, 34)),
        Priority::P2 => (WARNING, Color::Rgb(78, 54, 18)),
        Priority::P3 => (INFO, Color::Rgb(26, 46, 76)),
    }
}

fn status_tone(status: TaskStatus) -> (Color, Color) {
    match status {
        TaskStatus::Active => (ACCENT_SOFT, Color::Rgb(13, 56, 48)),
        TaskStatus::Done => (SUCCESS, Color::Rgb(28, 54, 44)),
    }
}

fn due_tone(task: &Task, today: NaiveDate) -> (Color, Color) {
    if task.status == TaskStatus::Done {
        return (SUCCESS, Color::Rgb(28, 54, 44));
    }
    match task.due_date {
        Some(date) if date < today => (DANGER, Color::Rgb(76, 24, 34)),
        Some(date) if date == today => (WARNING, Color::Rgb(78, 54, 18)),
        Some(_) => (INFO, Color::Rgb(26, 46, 76)),
        None => (TEXT_MUTED, SURFACE_ALT),
    }
}

fn task_due_label(app: &App, task: &Task, today: NaiveDate) -> String {
    if task.status == TaskStatus::Done {
        return task
            .due_date
            .map(|date| app.t_fmt("due.upcoming", &[("date", date.to_string())]))
            .unwrap_or_else(|| app.t("due.none"));
    }

    app.format_due_label(task, today)
}

fn task_due_chip_label(app: &App, task: &Task, today: NaiveDate) -> String {
    match task.due_date {
        Some(date) if task.status != TaskStatus::Done && date < today => app.t("list.due.overdue"),
        Some(date) if task.status != TaskStatus::Done && date == today => app.t("list.due.today"),
        Some(date) => app.t_fmt(
            "list.due.date",
            &[("date", date.format("%m-%d").to_string())],
        ),
        None => app.t("list.due.none"),
    }
}

fn preview_text(input: &str, max_chars: usize, empty_label: &str) -> String {
    let compact = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return empty_label.to_string();
    }

    let mut preview = String::new();
    let mut count = 0;
    for ch in compact.chars() {
        if count >= max_chars {
            preview.push_str("...");
            return preview;
        }
        preview.push(ch);
        count += 1;
    }
    preview
}

fn truncate_text(input: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for (index, ch) in input.chars().enumerate() {
        if index >= max_chars {
            output.push_str("...");
            break;
        }
        output.push(ch);
    }
    output
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let max_width = area.width.saturating_sub(2).max(1);
    let max_height = area.height.saturating_sub(2).max(1);
    let width = width.min(max_width);
    let height = height.min(max_height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect};
    use tempfile::tempdir;

    use crate::app::{App, FormField, TaskForm};
    use crate::i18n::I18n;
    use crate::model::{Priority, TaskDraft, TaskStore};
    use crate::storage::Config;

    use super::{
        centered_rect, footer_height, priority_label, render_footer, render_form_modal,
        render_help_modal, render_sidebar, render_task_list, render_top_bar,
    };

    fn buffer_to_string(buffer: &Buffer) -> String {
        let area = buffer.area();
        let mut output = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                output.push_str(buffer[(x, y)].symbol());
            }
            output.push('\n');
        }
        output
    }

    fn render_form_modal_to_string(app: &App, form: &TaskForm, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_form_modal(frame, app, form))
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn render_help_modal_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_help_modal(frame, app))
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn render_footer_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_footer(frame, app, frame.area()))
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn render_top_bar_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_top_bar(frame, app, frame.area()))
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn render_sidebar_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_sidebar(frame, app, frame.area()))
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn render_task_list_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_task_list(frame, app, frame.area()))
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn fixed_time(input: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(input)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn make_task_app() -> App {
        let dir = tempdir().unwrap();
        let data_file = dir.path().join("tasks.json");
        let config_path = dir.path().join("config.json");
        let i18n_dir = dir.path().join("i18n");
        let mut store = TaskStore::new();
        store
            .add_task(
                TaskDraft {
                    title: "Ship the compact list redesign before review".into(),
                    notes: "This note should stay out of the list row".into(),
                    priority: Priority::P1,
                    due_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
                },
                fixed_time("2026-03-08T10:00:00Z"),
            )
            .unwrap();
        let config = Config::default();
        let i18n = I18n::load(&config.i18n.locale, &i18n_dir).unwrap();
        App::new(store, data_file, config_path, i18n_dir, config, i18n)
    }

    fn normalize_rendered_text(input: &str) -> String {
        input.chars().filter(|ch| !ch.is_whitespace()).collect()
    }

    #[test]
    fn centered_rect_keeps_requested_size_when_terminal_is_large_enough() {
        let rect = centered_rect(60, 18, Rect::new(0, 0, 80, 24));
        assert_eq!(rect, Rect::new(10, 3, 60, 18));
    }

    #[test]
    fn centered_rect_clamps_to_available_space_on_small_terminal() {
        let rect = centered_rect(60, 18, Rect::new(0, 0, 40, 10));
        assert_eq!(rect, Rect::new(1, 1, 38, 8));
    }

    #[test]
    fn form_modal_shows_field_guide() {
        let app = App::test_app();
        let mut form = TaskForm::new();
        form.field = FormField::Priority;

        let rendered = render_form_modal_to_string(&app, &form, 100, 30);
        let normalized = normalize_rendered_text(&rendered);

        assert!(normalized.contains("NewTask"));
        assert!(normalized.contains("Priority"));
        assert!(normalized.contains("ACTIVE"));
        assert!(normalized.contains("Typeacleartasktitle"));
        assert!(normalized.contains("P2Important"));
        assert!(!normalized.contains("P1Critical"));
        assert!(!normalized.contains("P3Routine"));
    }

    #[test]
    fn help_modal_includes_language_shortcut() {
        let app = App::test_app();
        let rendered = render_help_modal_to_string(&app, 100, 30);
        let normalized = normalize_rendered_text(&rendered);

        assert!(normalized.contains("g"));
        assert!(normalized.contains("Openlanguagepicker"));
    }

    #[test]
    fn priority_labels_compact_gracefully_for_narrow_widths() {
        let app = App::test_app();
        assert_eq!(
            priority_label(&app, crate::model::Priority::P1, false, false),
            "P1 Critical"
        );
        assert_eq!(
            priority_label(&app, crate::model::Priority::P2, true, false),
            "P2 Important"
        );
        assert_eq!(
            priority_label(&app, crate::model::Priority::P3, true, true),
            "P3"
        );
    }

    #[test]
    fn footer_groups_shortcuts_in_sections() {
        let app = App::test_app();
        let height = footer_height(&app, 120);
        let rendered = render_footer_to_string(&app, 120, height);
        let normalized = normalize_rendered_text(&rendered);
        assert!(normalized.contains("NAV"));
        assert!(normalized.contains("SYSTEM"));
        assert_eq!(height, 6);
    }

    #[test]
    fn top_bar_shows_metrics() {
        let app = App::test_app();
        let rendered = render_top_bar_to_string(&app, 120, 3);
        let normalized = normalize_rendered_text(&rendered);
        assert!(normalized.contains("CONTROLDESK"));
        assert!(!normalized.contains("TODOX"));
        assert!(normalized.contains("ACTIVE"));
        assert!(normalized.contains("OVERDUE"));
        assert!(!normalized.contains("LIVECONTEXT"));
    }

    #[test]
    fn task_list_renders_single_line_without_updated_or_notes() {
        let app = make_task_app();
        let rendered = render_task_list_to_string(&app, 100, 10);
        let normalized = normalize_rendered_text(&rendered);
        assert!(normalized.contains("Shipthecompactlistredesignbeforereview"));
        assert!(normalized.contains("P1"));
        assert!(normalized.contains("03-09") || normalized.contains("Today"));
        assert!(!normalized.contains("Thisnoteshouldstayoutofthelistrow"));
        assert!(!normalized.contains("10:00"));
    }

    #[test]
    fn sidebar_renders_focus_panel() {
        let app = App::test_app();
        let rendered = render_sidebar_to_string(&app, 38, 18);
        let normalized = normalize_rendered_text(&rendered);
        assert!(normalized.contains("FOCUSPANEL"));
    }
}
