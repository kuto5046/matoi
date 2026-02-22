use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    SelectNext,
    SelectPrev,
    Focus,
    Refresh,
    Quit,
}

pub fn map_key_event(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(AppEvent::SelectNext),
        KeyCode::Char('k') | KeyCode::Up => Some(AppEvent::SelectPrev),
        KeyCode::Char('f') | KeyCode::Enter => Some(AppEvent::Focus),
        KeyCode::Char('r') => Some(AppEvent::Refresh),
        KeyCode::Char('q') => Some(AppEvent::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(AppEvent::Quit),
        _ => None,
    }
}
