use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct OnboardingWizardWidget;

impl OnboardingWizardWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let onboarding_block = Block::default()
            .title(" Onboarding Wizard ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let inner = onboarding_block.inner(area);
        frame.render_widget(onboarding_block, area);

        let mut lines: Vec<Line> = Vec::new();

        if let Some(step) = &app.onboarding_step {
            let progress = format!("Step {} of {}", step.current + 1, step.total);

            lines.push(Line::from(vec![
                Span::styled(
                    step.title.clone(),
                    Style::default()
                        .fg(theme.accent_green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  ({})", progress),
                    Style::default().fg(theme.text_secondary),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                step.description.clone(),
                Style::default().fg(theme.text_primary),
            )]));
            lines.push(Line::from(""));

            if !step.input.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    format!("> {}", step.input),
                    Style::default().fg(theme.accent_green),
                )]));
            } else {
                lines.push(Line::from(vec![Span::styled(
                    "> _",
                    Style::default().fg(theme.text_secondary),
                )]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Press Enter to continue, Escape to go back",
                Style::default().fg(theme.text_secondary),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "Welcome to OmniCode Onboarding!",
                Style::default()
                    .fg(theme.accent_green)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Press Enter to start the onboarding wizard.",
                Style::default().fg(theme.text_primary),
            )]));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}
