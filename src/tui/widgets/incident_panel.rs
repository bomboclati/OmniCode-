use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub struct IncidentPanelWidget;

impl IncidentPanelWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let incident_block = Block::default()
            .title(" Incident Alerts ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = incident_block.inner(area);
        frame.render_widget(incident_block, area);

        let mut lines: Vec<Line> = Vec::new();

        for alert in &app.incident_alerts {
            let severity_icon = match alert.severity.as_str() {
                "critical" => "🔴",
                "warning" => "🟠",
                "info" => "🟡",
                _ => "⚪",
            };

            let severity_color = match alert.severity.as_str() {
                "critical" => theme.error,
                "warning" => theme.warning,
                "info" => theme.file_path,
                _ => theme.text_secondary,
            };

            let time = alert.timestamp.format("%H:%M:%S").to_string();
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} [{}] ", severity_icon, time),
                    Style::default().fg(severity_color),
                ),
                Span::styled(
                    format!("[{}] {}", alert.source, alert.message),
                    Style::default().fg(theme.text_primary),
                ),
            ]));

            if !alert.fix_status.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    format!("    Fix: {}", alert.fix_status),
                    Style::default().fg(theme.text_secondary),
                )]));
            }

            lines.push(Line::from(""));
        }

        if app.incident_alerts.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  No incidents. All systems operational.",
                Style::default().fg(theme.text_secondary),
            )]));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}
