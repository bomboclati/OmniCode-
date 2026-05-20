use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

const COMMANDS: &[&str] = &[
    "Agent Chat",
    "Swarm Mode",
    "Code Review",
    "Deploy",
    "Release",
    "Find in Codebase",
    "Open Docs",
    "Toggle Sandbox",
    "Manage Skills",
    "Share Session",
    "Toggle Compliance",
    "Start Onboarding",
    "Dependency Guardian",
    "View Decisions",
    "Check Incidents",
    "Run Tests",
    "Heal Failures",
    "Semantic Search",
    "Migration Wizard",
    "Quit",
];

pub struct CommandPaletteWidget;

impl CommandPaletteWidget {
    pub fn render(frame: &mut Frame, input: &str, area: Rect) {
        let theme = crate::tui::themes::default_theme();

        let palette_area = Rect {
            x: area.width.saturating_sub(60) / 2,
            y: 2,
            width: 60.min(area.width),
            height: 15.min(area.height.saturating_sub(4)),
        };

        let palette_block = Block::default()
            .title(" Command Palette ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_purple));

        frame.render_widget(palette_block, palette_area);

        let inner = palette_block.inner(palette_area);

        let filtered: Vec<&str> = COMMANDS
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&input.to_lowercase()))
            .copied()
            .collect();

        let input_para = Paragraph::new(format!("> {}", input))
            .style(Style::default().fg(theme.accent_green));
        let input_area = Rect {
            x: inner.x + 1,
            y: inner.y,
            width: inner.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(input_para, input_area);

        let items: Vec<ListItem> = filtered
            .iter()
            .take(inner.height.saturating_sub(2) as usize)
            .map(|cmd| {
                ListItem::new(Line::from(Span::styled(
                    format!("  {}", cmd),
                    Style::default().fg(theme.text_primary),
                )))
            })
            .collect();

        let list_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };

        let list = List::new(items);
        frame.render_widget(list, list_area);
    }
}
