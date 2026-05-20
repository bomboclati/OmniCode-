use crate::config::Config;
use crate::tui::event::{Event, EventHandler};
use crate::tui::themes::default_theme;
use crate::tui::ui::draw;
use crate::tui::widgets::file_tree::FileTree;
use chrono::Local;
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Chat,
    Swarm,
    Reviews,
    Incidents,
    Docs,
    Deps,
    Decisions,
    Migration,
    Onboarding,
    Compliance,
    Skills,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    User,
    Agent,
    System,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub timestamp: chrono::DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct SwarmAgentStatus {
    pub name: String,
    pub role: String,
    pub status: String,
    pub current_task: String,
    pub tokens_used: u64,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub version: String,
    pub description: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct Review {
    pub id: String,
    pub title: String,
    pub author: String,
    pub status: String,
    pub created_at: chrono::DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct IncidentAlert {
    pub id: String,
    pub timestamp: chrono::DateTime<Local>,
    pub severity: String,
    pub source: String,
    pub message: String,
    pub stack_trace: String,
    pub correlated_commit: String,
    pub fix_status: String,
    pub pr_link: String,
}

#[derive(Debug, Clone)]
pub struct DependencyUpdate {
    pub package: String,
    pub current_version: String,
    pub new_version: String,
    pub breaking_change: bool,
    pub auto_fix_status: String,
    pub changelog_summary: String,
    pub affected_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub id: String,
    pub date: chrono::DateTime<Local>,
    pub title: String,
    pub summary: String,
    pub pr_links: Vec<String>,
    pub full_text: String,
}

#[derive(Debug, Clone)]
pub struct OnboardingStep {
    pub current: usize,
    pub total: usize,
    pub title: String,
    pub description: String,
    pub input: String,
}

#[derive(Debug, Clone)]
pub struct ComplianceEntry {
    pub timestamp: chrono::DateTime<Local>,
    pub action: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusPanel {
    FileTree,
    Chat,
    Diff,
    Input,
}

pub struct App {
    pub config: Config,
    pub current_screen: Screen,
    pub chat_messages: Vec<ChatMessage>,
    pub input_buffer: String,
    pub cursor_position: usize,
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
    pub file_tree: FileTree,
    pub current_file_content: Option<String>,
    pub diff_content: Option<String>,
    pub status_message: String,
    pub agent_busy: bool,
    pub swarm_agents: Vec<SwarmAgentStatus>,
    pub active_skills: Vec<Skill>,
    pub pending_reviews: Vec<Review>,
    pub incident_alerts: Vec<IncidentAlert>,
    pub dependency_updates: Vec<DependencyUpdate>,
    pub decisions: Vec<Decision>,
    pub onboarding_step: Option<OnboardingStep>,
    pub collaboration_peers: Vec<String>,
    pub voice_active: bool,
    pub compliance_log: Vec<ComplianceEntry>,
    pub terminal_scroll: u16,
    pub focus: FocusPanel,
    pub show_command_palette: bool,
    pub command_palette_input: String,
    pub show_file_tree: bool,
    pub show_diff: bool,
    pub chat_scroll_offset: usize,
    pub git_branch: String,
    pub should_quit: bool,
    pub agent_sender: Option<mpsc::UnboundedSender<Event>>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let file_tree = FileTree::scan_directory(std::env::current_dir().unwrap_or_default());
        let git_branch = get_git_branch();

        let mut app = Self {
            config,
            current_screen: Screen::Chat,
            chat_messages: Vec::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            command_history: Vec::new(),
            history_index: None,
            file_tree,
            current_file_content: None,
            diff_content: None,
            status_message: "Ready".to_string(),
            agent_busy: false,
            swarm_agents: Vec::new(),
            active_skills: Vec::new(),
            pending_reviews: Vec::new(),
            incident_alerts: Vec::new(),
            dependency_updates: Vec::new(),
            decisions: Vec::new(),
            onboarding_step: None,
            collaboration_peers: Vec::new(),
            voice_active: false,
            compliance_log: Vec::new(),
            terminal_scroll: 0,
            focus: FocusPanel::Input,
            show_command_palette: false,
            command_palette_input: String::new(),
            show_file_tree: true,
            show_diff: false,
            chat_scroll_offset: 0,
            git_branch,
            should_quit: false,
            agent_sender: None,
        };

        app.chat_messages.push(ChatMessage {
            role: Role::System,
            content: "Welcome to OmniCode. Press Ctrl+K for commands, Ctrl+Q to quit.".to_string(),
            timestamp: Local::now(),
        });

        app
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        use crossterm::event::KeyModifiers;

        if self.show_command_palette {
            self.handle_command_palette_key(key);
            return;
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.should_quit = true;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.show_command_palette = true;
                self.command_palette_input.clear();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                self.show_file_tree = !self.show_file_tree;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                self.show_diff = !self.show_diff;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.status_message = "File saved".to_string();
            }
            (KeyModifiers::CONTROL, KeyCode::Char(' ')) => {
                self.insert_at_cursor("[AI suggestion]");
            }
            (KeyModifiers::ALT, KeyCode::Enter) => {
                self.submit_to_swarm();
            }
            (KeyModifiers::NONE, KeyCode::F(1)) => {
                self.current_screen = Screen::Chat;
            }
            (KeyModifiers::NONE, KeyCode::F(2)) => {
                self.current_screen = Screen::Swarm;
            }
            (KeyModifiers::NONE, KeyCode::F(3)) => {
                self.current_screen = Screen::Reviews;
            }
            (KeyModifiers::NONE, KeyCode::F(4)) => {
                self.current_screen = Screen::Incidents;
            }
            (KeyModifiers::NONE, KeyCode::F(5)) => {
                self.current_screen = Screen::Docs;
            }
            (KeyModifiers::NONE, KeyCode::F(6)) => {
                self.current_screen = Screen::Deps;
            }
            (KeyModifiers::NONE, KeyCode::F(7)) => {
                self.current_screen = Screen::Decisions;
            }
            (KeyModifiers::NONE, KeyCode::F(8)) => {
                self.current_screen = Screen::Skills;
            }
            (KeyModifiers::NONE, KeyCode::F(9)) => {
                self.current_screen = Screen::Compliance;
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.cycle_focus();
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if self.focus == FocusPanel::Input {
                    self.submit_input();
                } else if self.focus == FocusPanel::FileTree {
                    self.file_tree.toggle_selected();
                    if let Some(path) = self.file_tree.get_selected_file() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            self.current_file_content = Some(content);
                        }
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::Escape) => {
                self.input_buffer.clear();
                self.cursor_position = 0;
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.focus == FocusPanel::Input {
                    self.navigate_history_up();
                } else {
                    self.chat_scroll_offset = self.chat_scroll_offset.saturating_sub(1);
                }
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if self.focus == FocusPanel::Input {
                    self.navigate_history_down();
                } else {
                    self.chat_scroll_offset += 1;
                }
            }
            (KeyModifiers::NONE, KeyCode::PageUp) => {
                self.chat_scroll_offset = self.chat_scroll_offset.saturating_sub(10);
            }
            (KeyModifiers::NONE, KeyCode::PageDown) => {
                self.chat_scroll_offset += 10;
            }
            (KeyModifiers::NONE, KeyCode::Char(c)) => {
                if self.focus == FocusPanel::Input {
                    self.insert_at_cursor(&c.to_string());
                } else if self.focus == FocusPanel::FileTree {
                    self.file_tree.handle_key(c);
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                if self.focus == FocusPanel::Input {
                    self.delete_before_cursor();
                }
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                if self.focus == FocusPanel::Input {
                    self.delete_after_cursor();
                }
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_command_palette_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        use crossterm::event::KeyModifiers;

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Escape) => {
                self.show_command_palette = false;
                self.command_palette_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.execute_command_palette_selection();
                self.show_command_palette = false;
                self.command_palette_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::Char(c)) => {
                self.command_palette_input.push(c);
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.command_palette_input.pop();
            }
            _ => {}
        }
    }

    fn execute_command_palette_selection(&mut self) {
        let input = self.command_palette_input.to_lowercase();
        if input.contains("swarm") {
            self.current_screen = Screen::Swarm;
        } else if input.contains("review") {
            self.current_screen = Screen::Reviews;
        } else if input.contains("incident") {
            self.current_screen = Screen::Incidents;
        } else if input.contains("doc") {
            self.current_screen = Screen::Docs;
        } else if input.contains("dep") {
            self.current_screen = Screen::Deps;
        } else if input.contains("decision") {
            self.current_screen = Screen::Decisions;
        } else if input.contains("skill") {
            self.current_screen = Screen::Skills;
        } else if input.contains("compliance") {
            self.current_screen = Screen::Compliance;
        } else if input.contains("share") {
            self.status_message = "Creating collaboration session...".to_string();
        } else if input.contains("sandbox") {
            self.status_message = "Toggling sandbox mode...".to_string();
        } else if input.contains("onboard") {
            self.current_screen = Screen::Onboarding;
        } else {
            self.submit_input();
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Input => FocusPanel::FileTree,
            FocusPanel::FileTree => FocusPanel::Chat,
            FocusPanel::Chat => FocusPanel::Diff,
            FocusPanel::Diff => FocusPanel::Input,
        };
    }

    fn submit_input(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let message = self.input_buffer.clone();
        self.command_history.push(message.clone());
        self.history_index = None;

        self.chat_messages.push(ChatMessage {
            role: Role::User,
            content: message.clone(),
            timestamp: Local::now(),
        });

        self.input_buffer.clear();
        self.cursor_position = 0;

        self.agent_busy = true;
        self.status_message = "Agent is thinking...".to_string();

        if let Some(sender) = &self.agent_sender {
            let _ = sender.send(Event::AgentMessage(message));
        }
    }

    fn submit_to_swarm(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let message = self.input_buffer.clone();
        self.chat_messages.push(ChatMessage {
            role: Role::User,
            content: format!("[Swarm] {}", message),
            timestamp: Local::now(),
        });

        self.input_buffer.clear();
        self.cursor_position = 0;
        self.current_screen = Screen::Swarm;
        self.status_message = "Swarm mode activated".to_string();
    }

    fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        match self.history_index {
            None => {
                self.history_index = Some(self.command_history.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            Some(_) => {}
        }
        if let Some(idx) = self.history_index {
            self.input_buffer = self.command_history[idx].clone();
            self.cursor_position = self.input_buffer.len();
        }
    }

    fn navigate_history_down(&mut self) {
        match self.history_index {
            None => {
                self.input_buffer.clear();
                self.cursor_position = 0;
            }
            Some(idx) if idx < self.command_history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input_buffer = self.command_history[idx + 1].clone();
                self.cursor_position = self.input_buffer.len();
            }
            Some(_) => {
                self.history_index = None;
                self.input_buffer.clear();
                self.cursor_position = 0;
            }
        }
    }

    fn insert_at_cursor(&mut self, text: &str) {
        self.input_buffer
            .insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    fn delete_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            self.input_buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    fn delete_after_cursor(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    pub fn update(&mut self) {
        if self.agent_busy && self.chat_messages.len() > 1 {
            let last = self.chat_messages.last();
            if let Some(msg) = last {
                if msg.role == Role::Agent {
                    self.agent_busy = false;
                    self.status_message = "Idle".to_string();
                }
            }
        }
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    config: Config,
) -> anyhow::Result<()> {
    let mut app = App::new(config);
    let mut events = EventHandler::new(100);
    app.agent_sender = Some(events.sender());

    loop {
        if app.should_quit {
            break;
        }

        terminal.draw(|f| draw(f, &app))?;

        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    app.handle_key(key);
                }
                Event::Resize(_, _) => {}
                Event::Tick => {
                    app.update();
                }
                Event::AgentMessage(msg) => {
                    app.chat_messages.push(ChatMessage {
                        role: Role::Agent,
                        content: format!("Processing: {}", msg),
                        timestamp: Local::now(),
                    });
                    app.agent_busy = false;
                    app.status_message = "Response received".to_string();
                }
                Event::CollaborationMessage(msg) => {
                    app.chat_messages.push(ChatMessage {
                        role: Role::System,
                        content: format!("[Collab] {}", msg),
                        timestamp: Local::now(),
                    });
                }
                Event::SentinelAlert(msg) => {
                    app.status_message = format!("[Sentinel] {}", msg);
                }
                Event::IncidentAlert(msg) => {
                    app.incident_alerts.push(IncidentAlert {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Local::now(),
                        severity: "warning".to_string(),
                        source: "production".to_string(),
                        message: msg.clone(),
                        stack_trace: String::new(),
                        correlated_commit: String::new(),
                        fix_status: "analyzing".to_string(),
                        pr_link: String::new(),
                    });
                }
                Event::GuardianUpdate(msg) => {
                    app.status_message = format!("[Guardian] {}", msg);
                }
                Event::Mouse(_) => {}
            }
        }
    }

    Ok(())
}

fn get_git_branch() -> String {
    let git_head = std::env::current_dir()
        .ok()
        .map(|d| d.join(".git").join("HEAD"));
    if let Some(path) = git_head {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
                return branch.trim().to_string();
            }
        }
    }
    "main".to_string()
}
