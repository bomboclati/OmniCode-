use crate::tui::app::{App, Role};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub struct ChatPanelWidget;

impl ChatPanelWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let chat_block = Block::default()
            .title(" Chat ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = chat_block.inner(area);
        frame.render_widget(chat_block, area);

        let mut lines: Vec<Line> = Vec::new();
        let visible_height = inner.height as usize;

        for msg in &app.chat_messages {
            let time = msg.timestamp.format("%H:%M").to_string();
            let role_icon = match msg.role {
                Role::User => "🧑",
                Role::Agent => "🤖",
                Role::System => "⚙️",
                Role::Tool => "🔧",
            };

            let role_color = match msg.role {
                Role::User => theme.accent_green,
                Role::Agent => theme.accent_purple,
                Role::System => Color::White,
                Role::Tool => theme.file_path,
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", time),
                    Style::default().fg(theme.text_secondary),
                ),
                Span::styled(
                    format!("{} ", role_icon),
                    Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                ),
            ]));

            for line in msg.content.lines() {
                let styled_line = if line.starts_with("```") {
                    Span::styled(
                        format!("  {}", line),
                        Style::default()
                            .fg(theme.text_secondary)
                            .add_modifier(Modifier::DIM),
                    )
                } else {
                    Span::styled(
                        format!("  {}", line),
                        Style::default().fg(theme.text_primary),
                    )
                };
                lines.push(Line::from(vec![styled_line]));
            }
            lines.push(Line::from(""));
        }

        let total_lines = lines.len();
        let scroll = if app.chat_scroll_offset == 0
            && total_lines > visible_height.saturating_sub(2)
        {
            total_lines.saturating_sub(visible_height.saturating_sub(2))
        } else {
            app.chat_scroll_offset
        };

        let chat_para = Paragraph::new(lines)
            .scroll((scroll as u16, 0))
            .wrap(Wrap { trim: false });

        frame.render_widget(chat_para, inner);
    }

    pub fn render_input(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let input_block = Block::default()
            .title(" Input ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = input_block.inner(area);
        frame.render_widget(input_block, area);

        let prompt = "$ ";
        let full_text = format!("{}{}", prompt, app.input_buffer);

        let input_para = Paragraph::new(full_text).style(Style::default().fg(theme.text_primary));
        frame.render_widget(input_para, inner);

        frame.set_cursor(
            inner.x + prompt.len() as u16 + app.cursor_position as u16,
            inner.y,
        );
    }
}
