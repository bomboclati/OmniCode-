use crate::tui::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct FileTree {
    pub children: Vec<TreeNode>,
    pub selected_index: usize,
    pub flat_list: Vec<PathBuf>,
}

impl FileTree {
    pub fn scan_directory(dir: PathBuf) -> Self {
        let mut children = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let is_dir = path.is_dir();

                if name.starts_with('.') || name == "target" {
                    continue;
                }

                let node = TreeNode {
                    name,
                    path: path.clone(),
                    is_dir,
                    children: if is_dir {
                        Self::scan_directory(path.clone()).children
                    } else {
                        Vec::new()
                    },
                    expanded: false,
                };
                children.push(node);
            }
        }

        children.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.cmp(&b.name)
            }
        });

        let mut flat_list = Vec::new();
        Self::build_flat_list(&children, &mut flat_list);

        Self {
            children,
            selected_index: 0,
            flat_list,
        }
    }

    fn build_flat_list(nodes: &[TreeNode], flat: &mut Vec<PathBuf>) {
        for node in nodes {
            flat.push(node.path.clone());
            if node.is_dir && node.expanded {
                Self::build_flat_list(&node.children, flat);
            }
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(node) = self.get_selected_mut(&mut self.children) {
            if node.is_dir {
                node.expanded = !node.expanded;
                self.flat_list.clear();
                Self::build_flat_list(&self.children, &mut self.flat_list);
            }
        }
    }

    fn get_selected_mut(&mut self, nodes: &mut Vec<TreeNode>) -> Option<&mut TreeNode> {
        let mut idx = 0;
        self._find_selected_mut(nodes, &mut idx)
    }

    fn _find_selected_mut(
        &mut self,
        nodes: &mut Vec<TreeNode>,
        idx: &mut usize,
    ) -> Option<&mut TreeNode> {
        for node in nodes.iter_mut() {
            if *idx == self.selected_index {
                return Some(node);
            }
            *idx += 1;
            if node.is_dir && node.expanded {
                if let Some(found) = self._find_selected_mut(&mut node.children, idx) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        self.flat_list.get(self.selected_index).cloned()
    }

    pub fn get_selected_path(&self) -> Option<PathBuf> {
        self.flat_list.get(self.selected_index).cloned()
    }

    pub fn handle_key(&mut self, _c: char) {}

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index < self.flat_list.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
}

pub struct FileTreeWidget;

impl FileTreeWidget {
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let theme = crate::tui::themes::default_theme();
        let items = Self::build_tree_items(&app.file_tree, 0, &app);

        let tree_block = Block::default()
            .title(" File Tree ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let list = List::new(items).block(tree_block);
        frame.render_widget(list, area);
    }

    fn build_tree_items(
        tree: &crate::tui::widgets::file_tree::FileTree,
        depth: usize,
        app: &App,
    ) -> Vec<ListItem<'static>> {
        let mut items = Vec::new();
        let theme = crate::tui::themes::default_theme();

        for node in &tree.children {
            let indent = "  ".repeat(depth);
            let icon = if node.is_dir {
                if node.expanded {
                    "📁"
                } else {
                    "📂"
                }
            } else {
                "📄"
            };

            let is_selected = app
                .file_tree
                .get_selected_path()
                .map_or(false, |p| p == node.path);

            let style = if is_selected {
                Style::default()
                    .fg(theme.accent_green)
                    .bg(theme.selection)
                    .add_modifier(Modifier::BOLD)
            } else if node.is_dir {
                Style::default().fg(theme.file_path)
            } else {
                Style::default().fg(theme.text_primary)
            };

            let text = format!("{}{} {}", indent, icon, node.name);
            items.push(ListItem::new(Line::from(Span::styled(text, style))));

            if node.is_dir && node.expanded {
                let child_items = Self::build_tree_items(node, depth + 1, app);
                items.extend(child_items);
            }
        }

        items
    }
}
