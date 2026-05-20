use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct DecisionsPanelWidget;

impl DecisionsPanelWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let decisions_block = Block::default()
            .title(" Architectural Decisions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = decisions_block.inner(area);
        frame.render_widget(decisions_block, area);

        let mut lines: Vec<Line> = Vec::new();

        for decision in &app.decisions {
            let date = decision.date.format("%Y-%m-%d").to_string();
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", date),
                    Style::default().fg(theme.text_secondary),
                ),
                Span::styled(
                    decision.title.clone(),
                    Style::default()
                        .fg(theme.accent_purple)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", decision.summary),
                Style::default().fg(theme.text_primary),
            )]));

            if !decision.pr_links.is_empty() {
                let pr_text = format!("  PRs: {}", decision.pr_links.join(", "));
                lines.push(Line::from(vec![Span::styled(
                    pr_text,
                    Style::default().fg(theme.file_path),
                )]));
            }

            lines.push(Line::from(""));
        }

        if app.decisions.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  No decisions recorded yet.",
                Style::default().fg(theme.text_secondary),
            )]));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}
