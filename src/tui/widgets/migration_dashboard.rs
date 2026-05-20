use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub struct MigrationDashboardWidget;

impl MigrationDashboardWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let migration_block = Block::default()
            .title(" Migration Dashboard ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = migration_block.inner(area);
        frame.render_widget(migration_block, area);

        let phases = vec![
            ("Analysis", 0.85),
            ("Planning", 0.60),
            ("Transformation", 0.35),
            ("Testing", 0.15),
            ("Verification", 0.0),
        ];

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            "Migration Progress:",
            Style::default()
                .fg(theme.text_secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        for (i, (phase, progress)) in phases.iter().enumerate() {
            let progress_pct = (*progress * 100.0) as u16;
            let gauge_color = if *progress > 0.7 {
                theme.accent_green
            } else if *progress > 0.3 {
                theme.warning
            } else {
                theme.text_secondary
            };

            let gauge_area = Rect {
                x: inner.x + 2,
                y: inner.y + 3 + i as u16 * 2,
                width: inner.width.saturating_sub(4),
                height: 1,
            };

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(gauge_color))
                .ratio(*progress)
                .label(format!("{} {}%", phase, progress_pct));

            frame.render_widget(gauge, gauge_area);
        }

        let risk_area = Rect {
            x: inner.x + 2,
            y: inner.y + 13,
            width: inner.width.saturating_sub(4),
            height: 5,
        };

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Risk Heatmap:",
            Style::default()
                .fg(theme.text_secondary)
                .add_modifier(Modifier::BOLD),
        )]));

        let heatmap = vec![
            vec!["LOW", "MED", "HIGH", "LOW", "MED"],
            vec!["MED", "LOW", "LOW", "HIGH", "LOW"],
            vec!["LOW", "LOW", "MED", "LOW", "LOW"],
        ];

        for row in &heatmap {
            let mut row_spans = Vec::new();
            for cell in row {
                let color = match *cell {
                    "LOW" => theme.accent_green,
                    "MED" => theme.warning,
                    "HIGH" => theme.error,
                    _ => theme.text_secondary,
                };
                row_spans.push(Span::styled(
                    format!(" {:^6} ", cell),
                    Style::default().fg(color).bg(theme.surface),
                ));
            }
            lines.push(Line::from(row_spans));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, risk_area);
    }
}
