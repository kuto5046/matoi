use anyhow::{Context, Result};
use chrono::Local;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TmuxWindow {
    pub session_name: String,
    pub window_index: u32,
    pub window_name: String,
    pub pane_current_command: String,
    pub pane_last_activity: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
    pub windows: Vec<TmuxWindow>,
}

#[derive(Debug, Clone)]
pub enum ActivityStatus {
    Active,
    RecentlyActive,
    Idle(Duration),
}

impl TmuxWindow {
    pub fn activity_status(&self) -> ActivityStatus {
        let now = Local::now().timestamp();
        let elapsed = (now - self.pane_last_activity).max(0) as u64;
        let duration = Duration::from_secs(elapsed);

        if elapsed <= 10 {
            ActivityStatus::Active
        } else if elapsed <= 60 {
            ActivityStatus::RecentlyActive
        } else {
            ActivityStatus::Idle(duration)
        }
    }

    pub fn elapsed_display(&self) -> String {
        let now = Local::now().timestamp();
        let elapsed = (now - self.pane_last_activity).max(0) as u64;

        if elapsed < 60 {
            format!("{}s ago", elapsed)
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else {
            format!("{}h ago", elapsed / 3600)
        }
    }

    pub fn target(&self) -> String {
        format!("{}:{}", self.session_name, self.window_index)
    }
}

pub fn list_sessions() -> Result<Vec<TmuxSession>> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .context("Failed to run tmux list-sessions")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux list-sessions failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let session_names: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    let mut sessions = Vec::new();
    for name in session_names {
        let windows = list_windows(&name).unwrap_or_default();
        sessions.push(TmuxSession { name, windows });
    }

    Ok(sessions)
}

pub fn list_windows(session: &str) -> Result<Vec<TmuxWindow>> {
    let format = "#{window_index}\t#{window_name}\t#{pane_current_command}\t#{pane_last_activity}\t#{window_active}";
    let output = Command::new("tmux")
        .args(["list-windows", "-t", session, "-F", format])
        .output()
        .context("Failed to run tmux list-windows")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux list-windows failed for session '{}': {}", session, stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();

    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(5, '\t').collect();
        if parts.len() < 5 {
            continue;
        }

        let window_index: u32 = parts[0].parse().unwrap_or(0);
        let window_name = parts[1].to_string();
        let pane_current_command = parts[2].to_string();
        let pane_last_activity: i64 = parts[3].parse().unwrap_or(0);
        let is_active = parts[4] == "1";

        windows.push(TmuxWindow {
            session_name: session.to_string(),
            window_index,
            window_name,
            pane_current_command,
            pane_last_activity,
            is_active,
        });
    }

    Ok(windows)
}

pub fn capture_pane(target: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["capture-pane", "-t", target, "-p", "-e"])
        .output()
        .context("Failed to run tmux capture-pane")?;

    if !output.status.success() {
        anyhow::bail!("tmux capture-pane failed for target '{}'", target);
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(strip_ansi_codes(&content))
}

pub fn switch_client(target: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["switch-client", "-t", target])
        .output()
        .context("Failed to run tmux switch-client")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux switch-client failed for target '{}': {}", target, stderr.trim());
    }

    Ok(())
}

pub fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // CSI sequence: consume until final byte (0x40-0x7E)
                for ch in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&ch) {
                        break;
                    }
                }
            } else if chars.peek() == Some(&']') {
                chars.next();
                // OSC sequence: consume until ST (ESC \ or BEL)
                while let Some(ch) = chars.next() {
                    if ch == '\x07' {
                        break;
                    }
                    if ch == '\x1b' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            } else {
                // Other escape sequences: skip next char
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result
}
