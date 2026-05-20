use crate::tui::app::App;
use chrono::Local;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct StatusBarWidget;

impl StatusBarWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let status_message = if app.agent_busy {
            "🔵 running"
        } else {
            "🟢 idle"
        };

        let sandbox_icon = match app.config.sandbox_mode {
            crate::config::SandboxMode::Local => "🔒",
            crate::config::SandboxMode::Docker => "🐳",
            crate::config::SandboxMode::Firecracker => "🔥",
        };

        let model_name = app
            .config
            .get_active_profile()
            .ok()
            .map(|p| p.model)
            .unwrap_or_else(|| "none".to_string());

        let time = Local::now().format("%H:%M:%S").to_string();
        let peer_count = app.collaboration_peers.len();
        let compliance_badge = if app.config.compliance_mode {
            "🛡️ compliance"
        } else {
            ""
        };
        let voice_indicator = if app.voice_active { "🎤" } else { "" };

        let left = format!(
            " {} {} {} | {}",
            app.git_branch, sandbox_icon, model_name, app.status_message
        );
        let center = status_message.to_string();
        let mut right = format!("👥 {} | {}", peer_count, time);
        if !compliance_badge.is_empty() {
            right = format!("{} | {}", compliance_badge, right);
        }
        if !voice_indicator.is_empty() {
            right = format!("{} {}", voice_indicator, right);
        }

        let total_width = area.width as usize;
        let center_start = total_width / 2 - center.len() / 2;
        let right_start = total_width.saturating_sub(right.len() + 1);

        let mut display = " ".repeat(total_width);
        let left_chars: Vec<char> = display.chars().collect();
        let mut left_chars: Vec<char> = left_chars;

        for (i, c) in left.chars().enumerate() {
            if i < total_width {
                left_chars[i] = c;
            }
        }

        for (i, c) in center.chars().enumerate() {
            let pos = center_start + i;
            if pos < total_width {
                left_chars[pos] = c;
            }
        }

        for (i, c) in right.chars().enumerate() {
            let pos = right_start + i;
            if pos < total_width {
                left_chars[pos] = c;
            }
        }

        let text: String = left_chars.into_iter().collect();

        let para = Paragraph::new(text)
            .style(Style::default().bg(theme.surface).fg(theme.text_secondary));
        frame.render_widget(para, area);
    }
}
