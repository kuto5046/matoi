mod app;
mod event;
mod tmux;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self as ct_event, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const POLL_TIMEOUT: Duration = Duration::from_millis(250);

fn main() -> Result<()> {
    // パニック時にターミナルを正常状態に戻す
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // ターミナル初期化
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // クリーンアップ
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    // 起動時にデータ取得
    app.refresh();

    let mut last_refresh = Instant::now();

    loop {
        // 描画
        terminal.draw(|f| ui::draw(f, &app))?;

        // キー入力待機（250ms タイムアウト）
        if ct_event::poll(POLL_TIMEOUT)? {
            if let Event::Key(key) = ct_event::read()? {
                if let Some(app_event) = event::map_key_event(key) {
                    match app_event {
                        event::AppEvent::Quit => {
                            app.should_quit = true;
                        }
                        event::AppEvent::SelectNext => {
                            app.select_next();
                        }
                        event::AppEvent::SelectPrev => {
                            app.select_prev();
                        }
                        event::AppEvent::Focus => {
                            app.focus_selected();
                        }
                        event::AppEvent::Refresh => {
                            app.refresh();
                            last_refresh = Instant::now();
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }

        // 5秒ごとに自動リフレッシュ
        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh();
            last_refresh = Instant::now();
        }
    }

    Ok(())
}
