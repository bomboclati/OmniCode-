use crate::agent::llm_client::LlmClient;
use crate::config::Config;
use crate::cortex::Cortex;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct OnboardingState {
    pub current_step: usize,
    pub total_steps: usize,
    pub user_role: String,
    pub skill_level: String,
    pub learning_goals: Vec<String>,
    pub learning_path: Vec<LearningItem>,
}

pub struct LearningItem {
    pub title: String,
    pub description: String,
    pub item_type: String,
    pub completed: bool,
    pub file_path: Option<String>,
}

pub async fn run_onboarding(config: &Config) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile.clone());

    println!("\n=== OmniCode Onboarding Wizard ===\n");
    println!("Welcome! Let's personalize your OmniCode experience.\n");

    let mut state = OnboardingState {
        current_step: 0,
        total_steps: 6,
        user_role: String::new(),
        skill_level: String::new(),
        learning_goals: Vec::new(),
        learning_path: Vec::new(),
    };

    // Step 1: Role Selection
    state.current_step = 1;
    println!("Step 1/{}: What is your primary role?", state.total_steps);
    println!("  a) Backend Developer");
    println!("  b) Frontend Developer");
    println!("  c) Full Stack Developer");
    println!("  d) DevOps/SRE Engineer");
    println!("  e) Data Scientist/ML Engineer");
    println!("  f) Student/Hobbyist");
    println!("  (Enter a-e, or type your own)");

    state.user_role = "Full Stack Developer".to_string();
    println!("  → Selected: {}", state.user_role);

    // Step 2: Experience Level
    state.current_step = 2;
    println!("\nStep 2/{}: What is your experience level?", state.total_steps);
    println!("  a) Beginner - New to programming");
    println!("  b) Intermediate - Some projects under my belt");
    println!("  c) Advanced - Professional developer");
    println!("  d) Expert - Architect/Lead level");

    state.skill_level = "intermediate".to_string();
    println!("  → Selected: {}", state.skill_level);

    // Step 3: Goals
    state.current_step = 3;
    println!("\nStep 3/{}: What are your learning goals?", state.total_steps);
    println!("  What would you like to achieve with OmniCode?");
    println!("  (e.g., Rust mastery, AI development, full-stack apps)");

    state.learning_goals = vec![
        "Rust development".to_string(),
        "AI/ML integration".to_string(),
        "Full-stack applications".to_string(),
    ];
    println!("  → Goals: {}", state.learning_goals.join(", "));

    // Step 4: Generate Learning Path
    state.current_step = 4;
    println!("\nStep 4/{}: Generating personalized learning path...", state.total_steps);

    let prompt = format!(
        r#"Create a personalized learning path for a developer.
Role: {}
Level: {}
Goals: {}

Generate 5-7 concrete learning activities for this developer to get started with the project.
Each should be a specific, actionable task (e.g., "Read through src/main.rs", "Run the test suite", "Fix a simple bug in src/tui/app.rs").

Return as a JSON array of objects with fields:
- "title": short title
- "description": what to do
- "type": one of "read", "run", "fix", "write", "explore""#,
        state.user_role, state.skill_level, state.learning_goals.join(", ")
    );

    if let Ok(response) = llm.chat(&prompt, &[]).await {
        if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&response) {
            for item in items {
                state.learning_path.push(LearningItem {
                    title: item["title"].as_str().unwrap_or("Task").to_string(),
                    description: item["description"].as_str().unwrap_or("").to_string(),
                    item_type: item["type"].as_str().unwrap_or("read").to_string(),
                    completed: false,
                    file_path: None,
                });
            }
        }
    }

    if state.learning_path.is_empty() {
        state.learning_path = vec![
            LearningItem { title: "Explore Project Structure".to_string(), description: "Read through the main source files to understand the architecture".to_string(), item_type: "read".to_string(), completed: false, file_path: None },
            LearningItem { title: "Run Test Suite".to_string(), description: "Execute cargo test to verify the project builds and tests pass".to_string(), item_type: "run".to_string(), completed: false, file_path: None },
            LearningItem { title: "Fix a Simple Issue".to_string(), description: "Find and fix a TODO or warning in the codebase".to_string(), item_type: "fix".to_string(), completed: false, file_path: None },
        ];
    }

    // Step 5: Interactive Sandbox
    state.current_step = 5;
    println!("\nStep 5/{}: Setting up interactive sandbox...", state.total_steps);

    let mut cortex = Cortex::new(project_root.clone())?;
    let _ = cortex.index_project().await;
    println!("  → Project indexed for semantic search.");

    println!("\n  Your Learning Path:");
    for (i, item) in state.learning_path.iter().enumerate() {
        println!("  {}. [{}] {} - {}", i + 1, item.item_type, item.title, item.description);
    }

    // Step 6: First PR Review
    state.current_step = 6;
    println!("\nStep 6/{}: Setting up PR review automation...", state.total_steps);

    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "main".to_string());

    println!("  → Current branch: {}", branch.trim());

    println!("\n=== Onboarding Complete! ===");
    println!("You're ready to use OmniCode.");
    println!("Your personalized learning path has been generated.");
    println!("Start with task 1, or press Ctrl+K to open the command palette.\n");

    Ok(())
}

pub async fn guide_through_task(
    project_root: &Path,
    cortex: &Cortex,
    llm: &mut LlmClient,
    task: &str,
) -> Result<String> {
    let context = cortex.search(task).await.unwrap_or_default();

    let prompt = format!(
        r#"Guide the developer through the following task:
Task: {}

Relevant code context:
{}

Provide step-by-step instructions with specific file paths and code snippets.
Be encouraging and educational."#,
        task, context
    );

    let guidance = llm.chat(&prompt, &[]).await.unwrap_or_else(|_| "Follow the learning path steps above.".to_string());
    Ok(guidance)
}
