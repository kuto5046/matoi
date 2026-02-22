use crate::tmux::{self, TmuxSession, TmuxWindow};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    SessionHeader,
    Window,
}

#[derive(Debug, Clone)]
pub struct WindowListItem {
    pub kind: ItemKind,
    pub label: String,
    pub session_name: Option<String>,
    pub window: Option<TmuxWindow>,
}

impl WindowListItem {
    pub fn session_header(name: &str) -> Self {
        Self {
            kind: ItemKind::SessionHeader,
            label: format!("[{}]", name),
            session_name: Some(name.to_string()),
            window: None,
        }
    }

    pub fn window_item(win: &TmuxWindow) -> Self {
        let active_marker = if win.is_active { "▶" } else { " " };
        let elapsed = win.elapsed_display();
        let label = format!(
            "  {} {}: {} ({})",
            active_marker, win.window_index, win.window_name, elapsed
        );
        Self {
            kind: ItemKind::Window,
            label,
            session_name: Some(win.session_name.clone()),
            window: Some(win.clone()),
        }
    }
}

pub struct App {
    pub sessions: Vec<TmuxSession>,
    pub window_list: Vec<WindowListItem>,
    pub selected_index: usize,
    pub preview_content: String,
    pub last_updated: Instant,
    pub error_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            window_list: Vec::new(),
            selected_index: 0,
            preview_content: String::new(),
            last_updated: Instant::now(),
            error_message: None,
            should_quit: false,
        }
    }

    pub fn refresh(&mut self) {
        match tmux::list_sessions() {
            Ok(sessions) => {
                self.sessions = sessions;
                self.error_message = None;
                self.rebuild_window_list();
                self.clamp_selection();
                self.last_updated = Instant::now();
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
            }
        }
        self.refresh_preview();
    }

    fn rebuild_window_list(&mut self) {
        let mut list = Vec::new();
        for session in &self.sessions {
            list.push(WindowListItem::session_header(&session.name));
            for win in &session.windows {
                list.push(WindowListItem::window_item(win));
            }
        }
        self.window_list = list;
    }

    pub fn refresh_preview(&mut self) {
        if let Some(item) = self.window_list.get(self.selected_index) {
            if let Some(win) = &item.window {
                let target = win.target();
                match tmux::capture_pane(&target) {
                    Ok(content) => {
                        self.preview_content = content;
                    }
                    Err(_) => {
                        self.preview_content = "No preview available".to_string();
                    }
                }
                return;
            }
        }
        self.preview_content = String::new();
    }

    pub fn select_next(&mut self) {
        let len = self.window_list.len();
        if len == 0 {
            return;
        }
        let mut next = (self.selected_index + 1) % len;
        // セッションヘッダーをスキップ
        let start = next;
        loop {
            if self.window_list[next].kind == ItemKind::Window {
                break;
            }
            next = (next + 1) % len;
            if next == start {
                break;
            }
        }
        if self.window_list[next].kind == ItemKind::Window {
            self.selected_index = next;
            self.refresh_preview();
        }
    }

    pub fn select_prev(&mut self) {
        let len = self.window_list.len();
        if len == 0 {
            return;
        }
        let mut prev = if self.selected_index == 0 {
            len - 1
        } else {
            self.selected_index - 1
        };
        // セッションヘッダーをスキップ
        let start = prev;
        loop {
            if self.window_list[prev].kind == ItemKind::Window {
                break;
            }
            prev = if prev == 0 { len - 1 } else { prev - 1 };
            if prev == start {
                break;
            }
        }
        if self.window_list[prev].kind == ItemKind::Window {
            self.selected_index = prev;
            self.refresh_preview();
        }
    }

    pub fn focus_selected(&mut self) {
        if let Some(item) = self.window_list.get(self.selected_index) {
            if let Some(win) = &item.window {
                let target = win.target();
                if let Err(e) = tmux::switch_client(&target) {
                    self.error_message = Some(format!("Focus failed: {}", e));
                }
            }
        }
    }

    fn clamp_selection(&mut self) {
        let len = self.window_list.len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index >= len {
            self.selected_index = len - 1;
        }
        // 選択がヘッダーになっていたら次のウィンドウへ
        if self.window_list[self.selected_index].kind == ItemKind::SessionHeader {
            self.select_next();
        }
    }
}
