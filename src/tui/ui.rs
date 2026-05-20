use crate::tui::app::{App, Screen};
use crate::tui::widgets::chat_panel::ChatPanelWidget;
use crate::tui::widgets::command_palette::CommandPaletteWidget;
use crate::tui::widgets::decisions_panel::DecisionsPanelWidget;
use crate::tui::widgets::deps_panel::DepsPanelWidget;
use crate::tui::widgets::diff_viewer::DiffViewerWidget;
use crate::tui::widgets::file_tree::FileTreeWidget;
use crate::tui::widgets::incident_panel::IncidentPanelWidget;
use crate::tui::widgets::migration_dashboard::MigrationDashboardWidget;
use crate::tui::widgets::onboarding_wizard::OnboardingWizardWidget;
use crate::tui::widgets::status_bar::StatusBarWidget;
use crate::tui::widgets::swarm_panel::SwarmPanelWidget;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = crate::tui::themes::default_theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    match app.current_screen {
        Screen::Chat => {
            render_chat_screen(frame, app, main_area);
        }
        Screen::Swarm => {
            SwarmPanelWidget::render(frame, app, main_area);
        }
        Screen::Reviews => {
            render_placeholder(frame, app, main_area, "Code Reviews", "No pending reviews");
        }
        Screen::Incidents => {
            IncidentPanelWidget::render(frame, app, main_area);
        }
        Screen::Docs => {
            render_placeholder(frame, app, main_area, "Documentation", "Press Enter to regenerate docs");
        }
        Screen::Deps => {
            DepsPanelWidget::render(frame, app, main_area);
        }
        Screen::Decisions => {
            DecisionsPanelWidget::render(frame, app, main_area);
        }
        Screen::Migration => {
            MigrationDashboardWidget::render(frame, app, main_area);
        }
        Screen::Onboarding => {
            OnboardingWizardWidget::render(frame, app, main_area);
        }
        Screen::Compliance => {
            render_placeholder(frame, app, main_area, "Compliance", "Compliance mode active");
        }
        Screen::Skills => {
            render_placeholder(frame, app, main_area, "Skills", "No skills installed");
        }
    }

    StatusBarWidget::render(frame, app, status_area);

    if app.show_command_palette {
        CommandPaletteWidget::render(frame, &app.command_palette_input, frame.area());
    }
}

fn render_chat_screen(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let main_layout = if app.show_file_tree {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(0),
                Constraint::Percentage(100),
            ])
            .split(area)
    };

    let tree_area = main_layout[0];
    let center_area = main_layout[1];

    if app.show_file_tree {
        FileTreeWidget::render(frame, app, tree_area);
    }

    let center_layout = if app.show_diff {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(center_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100),
                Constraint::Length(0),
            ])
            .split(center_area)
    };

    let chat_area = center_layout[0];
    let diff_area = center_layout[1];

    let chat_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(chat_area);

    ChatPanelWidget::render(frame, app, chat_layout[0]);
    ChatPanelWidget::render_input(frame, app, chat_layout[1]);

    if app.show_diff {
        DiffViewerWidget::render(frame, app, diff_area);
    }
}

fn render_placeholder(frame: &mut Frame, app: &App, area: ratatui::layout::Rect, title: &str, message: &str) {
    let theme = crate::tui::themes::default_theme();
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let para = Paragraph::new(vec![
        Line::from(Span::styled(
            message,
            Style::default().fg(theme.text_secondary),
        )),
    ]);
    frame.render_widget(para, inner);
}
