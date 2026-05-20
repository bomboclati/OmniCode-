use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct DiffViewerWidget;

impl DiffViewerWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let diff_block = Block::default()
            .title(" Diff ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = diff_block.inner(area);
        frame.render_widget(diff_block, area);

        let content = match &app.diff_content {
            Some(diff) => diff.clone(),
            None => "No diff available".to_string(),
        };

        let mut lines: Vec<Line> = Vec::new();
        let mut line_num = 1;

        for line in content.lines() {
            let (styled_line, _) = if line.starts_with('+') && !line.starts_with("+++") {
                (
                    Span::styled(
                        format!("{:>4} {}", line_num, line),
                        Style::default()
                            .fg(theme.accent_green)
                            .bg(Color::Rgb(0, 50, 0)),
                    ),
                    true,
                )
            } else if line.starts_with('-') && !line.starts_with("---") {
                (
                    Span::styled(
                        format!("{:>4} {}", line_num, line),
                        Style::default()
                            .fg(theme.error)
                            .bg(Color::Rgb(50, 0, 0)),
                    ),
                    false,
                )
            } else if line.starts_with("@@") {
                (
                    Span::styled(
                        format!("     {}", line),
                        Style::default()
                            .fg(theme.accent_purple)
                            .add_modifier(Modifier::BOLD),
                    ),
                    true,
                )
            } else {
                (
                    Span::styled(
                        format!("{:>4} {}", line_num, line),
                        Style::default().fg(theme.text_secondary),
                    ),
                    true,
                )
            };

            if _ {
                line_num += 1;
            }
            lines.push(Line::from(vec![styled_line]));
        }

        let diff_para = Paragraph::new(lines);
        frame.render_widget(diff_para, inner);
    }
}
