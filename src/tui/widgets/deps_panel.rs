use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct DepsPanelWidget;

impl DepsPanelWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let deps_block = Block::default()
            .title(" Dependencies ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = deps_block.inner(area);
        frame.render_widget(deps_block, area);

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            "Package                          Current     New         Breaking  Status",
            Style::default()
                .fg(theme.text_secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "-".repeat(90),
            Style::default().fg(theme.border),
        )]));

        for dep in &app.dependency_updates {
            let breaking_badge = if dep.breaking_change {
                "⚠️ YES"
            } else {
                "   no"
            };

            let breaking_color = if dep.breaking_change {
                theme.warning
            } else {
                theme.text_secondary
            };

            let text = format!(
                "{:<34} {:<11} {:<11} {}  {}",
                dep.package,
                dep.current_version,
                dep.new_version,
                breaking_badge,
                dep.auto_fix_status
            );
            lines.push(Line::from(vec![
                Span::styled(text, Style::default().fg(theme.text_primary)),
            ]));
        }

        if app.dependency_updates.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  All dependencies are up to date.",
                Style::default().fg(theme.text_secondary),
            )]));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}
