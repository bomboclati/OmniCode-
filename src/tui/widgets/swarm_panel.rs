use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub struct SwarmPanelWidget;

impl SwarmPanelWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let swarm_block = Block::default()
            .title(" Swarm Mode ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = swarm_block.inner(area);
        frame.render_widget(swarm_block, area);

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            "Agent Name          | Role              | Status     | Task",
            Style::default()
                .fg(theme.text_secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "-".repeat(80),
            Style::default().fg(theme.border),
        )]));

        for agent in &app.swarm_agents {
            let status_icon = match agent.status.as_str() {
                "idle" => "⚪",
                "running" => "🔵",
                "completed" => "🟢",
                "error" => "🔴",
                _ => "🟡",
            };

            let is_coordinator = agent.role == "coordinator";
            let style = if is_coordinator {
                Style::default()
                    .fg(theme.accent_purple)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_primary)
            };

            let text = format!(
                "{:<20}| {:<18}| {} {:<10}| {} ({} tokens)",
                agent.name,
                agent.role,
                status_icon,
                agent.status,
                agent.current_task,
                agent.tokens_used
            );
            lines.push(Line::from(vec![Span::styled(text, style)]));
        }

        if app.swarm_agents.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  No agents active. Submit a task to start swarm mode.",
                Style::default().fg(theme.text_secondary),
            )]));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}
