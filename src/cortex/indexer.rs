use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct CodeIndexer {
    pub project_root: PathBuf,
    pub file_index: HashMap<String, FileInfo>,
}

pub struct FileInfo {
    pub path: String,
    pub content_hash: String,
    pub language: String,
    pub symbols: Vec<Symbol>,
    pub line_count: usize,
}

pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: usize,
}

impl CodeIndexer {
    pub fn new(project_root: PathBuf) -> Result<Self> {
        Ok(Self {
            project_root,
            file_index: HashMap::new(),
        })
    }

    pub async fn index_all(&mut self) -> Result<usize> {
        let mut count = 0;

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| self.should_index(e))
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(path) = entry.path().to_str() {
                    if let Err(e) = self.index_file(path).await {
                        tracing::warn!("Failed to index {}: {}", path, e);
                    } else {
                        count += 1;
                    }
                }
            }
        }

        println!("Indexed {} files", count);
        Ok(count)
    }

    async fn index_file(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let hash = format!("{:x}", md5_hash(&content));
        let language = detect_language(path);
        let symbols = extract_symbols(&content, &language);
        let line_count = content.lines().count();

        let info = FileInfo {
            path: path.to_string(),
            content_hash: hash,
            language,
            symbols,
            line_count,
        };

        self.file_index.insert(path.to_string(), info);
        Ok(())
    }

    fn should_index(&self, entry: &walkdir::DirEntry) -> bool {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                return false;
            }
        }
        true
    }
}

fn md5_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

fn detect_language(path: &str) -> String {
    if let Some(ext) = std::path::Path::new(path).extension() {
        match ext.to_str().unwrap_or("") {
            "rs" => "rust".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "js" | "jsx" => "javascript".to_string(),
            "py" => "python".to_string(),
            "go" => "go".to_string(),
            "java" => "java".to_string(),
            "c" | "cpp" | "h" | "hpp" => "cpp".to_string(),
            "rb" => "ruby".to_string(),
            "html" => "html".to_string(),
            "css" => "css".to_string(),
            "json" => "json".to_string(),
            "yaml" | "yml" => "yaml".to_string(),
            "toml" => "toml".to_string(),
            "md" => "markdown".to_string(),
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}

fn extract_symbols(content: &str, language: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        match language {
            "rust" => {
                if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                    if let Some(name) = extract_function_name(trimmed) {
                        symbols.push(Symbol {
                            name,
                            kind: "function".to_string(),
                            line: line_num + 1,
                        });
                    }
                }
                if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                    if let Some(name) = extract_struct_name(trimmed) {
                        symbols.push(Symbol {
                            name,
                            kind: "struct".to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
            "typescript" | "javascript" => {
                if trimmed.starts_with("function ") || trimmed.starts_with("export function ") {
                    if let Some(name) = extract_function_name(trimmed) {
                        symbols.push(Symbol {
                            name,
                            kind: "function".to_string(),
                            line: line_num + 1,
                        });
                    }
                }
                if trimmed.starts_with("class ") || trimmed.starts_with("export class ") {
                    if let Some(name) = extract_class_name(trimmed) {
                        symbols.push(Symbol {
                            name,
                            kind: "class".to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
            "python" => {
                if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                    if let Some(name) = extract_function_name(trimmed) {
                        symbols.push(Symbol {
                            name,
                            kind: "function".to_string(),
                            line: line_num + 1,
                        });
                    }
                }
                if trimmed.starts_with("class ") {
                    if let Some(name) = extract_class_name(trimmed) {
                        symbols.push(Symbol {
                            name,
                            kind: "class".to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    symbols
}

fn extract_function_name(line: &str) -> Option<String> {
    let start = line.find("fn ").or_else(|| line.find("function ")).or_else(|| line.find("def "))?;
    let rest = &line[start..];
    let name_start = rest.find(|c: char| c.is_alphabetic() || c == '_')?;
    let rest = &rest[name_start..];
    let name_end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    Some(rest[..name_end].to_string())
}

fn extract_struct_name(line: &str) -> Option<String> {
    let start = line.find("struct ")?;
    let rest = &line[start + 7..];
    let name_end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    Some(rest[..name_end].to_string())
}

fn extract_class_name(line: &str) -> Option<String> {
    let start = line.find("class ")?;
    let rest = &line[start + 6..];
    let name_end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    Some(rest[..name_end].to_string())
}
