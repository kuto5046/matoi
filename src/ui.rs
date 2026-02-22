use crate::app::{App, ItemKind};
use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // 垂直方向レイアウト: ヘッダー(1行) / メイン(可変) / ステータスバー(1行)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    // ヘッダー
    let header = Paragraph::new(
        "matoi - tmux monitor           r:refresh  f:focus  q:quit",
    )
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(header, vertical[0]);

    // 水平方向レイアウト: 左30% / 右70%
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(vertical[1]);

    // 左パネル: ウィンドウリスト
    draw_window_list(frame, app, horizontal[0]);

    // 右パネル: プレビュー
    draw_preview(frame, app, horizontal[1]);

    // ステータスバー
    draw_status_bar(frame, app, vertical[2]);
}

fn draw_window_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .window_list
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if item.kind == ItemKind::SessionHeader {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(&item.label, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Sessions & Windows"),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_preview(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let content = if app.preview_content.is_empty() {
        "Select a window to preview".to_string()
    } else {
        app.preview_content.clone()
    };

    let preview = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Preview"))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let now = Local::now().format("%H:%M:%S").to_string();
    let status_text = if let Some(err) = &app.error_message {
        format!("Updated: {} | {}", now, err)
    } else {
        format!("Updated: {}", now)
    };

    let style = if app.error_message.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let status = Paragraph::new(status_text).style(style);
    frame.render_widget(status, area);
}
