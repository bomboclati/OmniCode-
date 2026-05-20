use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn generate_architecture_diagram() -> Result<String> {
    let mut diagram = String::new();
    diagram.push_str("# Architecture Diagram\n\n");
    diagram.push_str("```mermaid\n");
    diagram.push_str("graph TD\n");
    diagram.push_str("    CLI[CLI / Main] --> TUI[TUI Terminal]\n");
    diagram.push_str("    CLI --> Server[Web Server]\n");
    diagram.push_str("    CLI --> Agent[Agent Engine]\n\n");
    diagram.push_str("    Agent --> LLM[LLM Client]\n");
    diagram.push_str("    Agent --> Loop[Agent Loop]\n");
    diagram.push_str("    Agent --> Planner[Planner]\n");
    diagram.push_str("    Agent --> Tools[Tools System]\n");
    diagram.push_str("    Agent --> Sandbox[Sandbox]\n");
    diagram.push_str("    Agent --> Swarm[Swarm Manager]\n\n");
    diagram.push_str("    Agent --> Sentinel[CI Sentinel]\n");
    diagram.push_str("    Agent --> Review[PR Review]\n");
    diagram.push_str("    Agent --> Heal[Self Heal]\n");
    diagram.push_str("    Agent --> Release[Release Pipeline]\n");
    diagram.push_str("    Agent --> Archeologist[Legacy Migration]\n");
    diagram.push_str("    Agent --> Incident[Incident Whisperer]\n");
    diagram.push_str("    Agent --> Onboarding[Onboarding Angel]\n");
    diagram.push_str("    Agent --> Guardian[Dependency Guardian]\n");
    diagram.push_str("    Agent --> Historian[Decision Historian]\n\n");
    diagram.push_str("    Server --> HTTP[HTTP Routes]\n");
    diagram.push_str("    Server --> WS[WebSocket]\n");
    diagram.push_str("    Server --> Sync[Sync Engine]\n\n");
    diagram.push_str("    Cortex[Cortex Engine] --> Indexer[Code Indexer]\n");
    diagram.push_str("    Cortex --> Embeddings[Embeddings]\n");
    diagram.push_str("    Cortex --> Search[Semantic Search]\n");
    diagram.push_str("    Cortex --> KG[Knowledge Graph]\n\n");
    diagram.push_str("    Collaboration --> Tunnel[Tunnel]\n");
    diagram.push_str("    Collaboration --> PeerSync[Peer Sync]\n\n");
    diagram.push_str("    Deploy --> Docker[Docker]\n");
    diagram.push_str("    Deploy --> K8s[Kubernetes]\n");
    diagram.push_str("    Deploy --> Cloud[Cloud Providers]\n\n");
    diagram.push_str("    TUI --> Widgets[Widget System]\n");
    diagram.push_str("    Widgets --> Chat[Chat Panel]\n");
    diagram.push_str("    Widgets --> FileTree[File Tree]\n");
    diagram.push_str("    Widgets --> Diff[Diff Viewer]\n");
    diagram.push_str("    Widgets --> SwarmUI[Swarm Panel]\n");
    diagram.push_str("    Widgets --> IncidentUI[Incident Panel]\n");
    diagram.push_str("    Widgets --> Deps[Deps Panel]\n");
    diagram.push_str("    Widgets --> Decisions[Decisions Panel]\n");
    diagram.push_str("    Widgets --> Migration[Migration Dashboard]\n");
    diagram.push_str("    Widgets --> OnboardUI[Onboarding Wizard]\n");
    diagram.push_str("    Widgets --> Palette[Command Palette]\n");
    diagram.push_str("    Widgets --> StatusBar[Status Bar]\n");
    diagram.push_str("```\n");

    Ok(diagram)
}

pub fn generate_sequence_diagram() -> Result<String> {
    let mut diagram = String::new();
    diagram.push_str("# Agent Task Sequence\n\n");
    diagram.push_str("```mermaid\n");
    diagram.push_str("sequenceDiagram\n");
    diagram.push_str("    participant User\n");
    diagram.push_str("    participant CLI\n");
    diagram.push_str("    participant Agent\n");
    diagram.push_str("    participant LLM\n");
    diagram.push_str("    participant Sandbox\n");
    diagram.push_str("    participant Git\n\n");
    diagram.push_str("    User->>CLI: omni \"implement feature\"\n");
    diagram.push_str("    CLI->>Agent: run_single_task()\n");
    diagram.push_str("    Agent->>Agent: Create sandbox\n");
    diagram.push_str("    Agent->>LLM: Generate plan\n");
    diagram.push_str("    LLM-->>Agent: Plan steps\n");
    diagram.push_str("    loop For each step\n");
    diagram.push_str("        Agent->>LLM: Request action\n");
    diagram.push_str("        LLM-->>Agent: Tool call\n");
    diagram.push_str("        Agent->>Sandbox: Execute tool\n");
    diagram.push_str("        Sandbox-->>Agent: Result\n");
    diagram.push_str("        Agent->>LLM: Report result\n");
    diagram.push_str("    end\n");
    diagram.push_str("    Agent->>Git: Commit changes\n");
    diagram.push_str("    Agent-->>CLI: Summary + diff\n");
    diagram.push_str("    CLI-->>User: Task complete\n");
    diagram.push_str("```\n\n");

    diagram.push_str("# Collaboration Sequence\n\n");
    diagram.push_str("```mermaid\n");
    diagram.push_str("sequenceDiagram\n");
    diagram.push_str("    participant Peer1\n");
    diagram.push_str("    participant Server\n");
    diagram.push_str("    participant Peer2\n\n");
    diagram.push_str("    Peer1->>Server: Create session\n");
    diagram.push_str("    Server-->>Peer1: Session code\n");
    diagram.push_str("    Peer2->>Server: Join with code\n");
    diagram.push_str("    Server-->>Peer2: Connected\n");
    diagram.push_str("    loop Real-time sync\n");
    diagram.push_str("        Peer1->>Server: File edit\n");
    diagram.push_str("        Server->>Peer2: Broadcast edit\n");
    diagram.push_str("        Peer2->>Server: Cursor update\n");
    diagram.push_str("        Server->>Peer1: Broadcast cursor\n");
    diagram.push_str("    end\n");
    diagram.push_str("```\n");

    Ok(diagram)
}

pub fn generate_flowchart(title: &str, nodes: &HashMap<String, Vec<String>>) -> Result<String> {
    let mut diagram = String::new();
    diagram.push_str(&format!("```mermaid\nflowchart LR\n"));

    for (parent, children) in nodes {
        for child in children {
            diagram.push_str(&format!("    {}[{}] --> {}[{}]\n", parent, parent, child, child));
        }
    }

    diagram.push_str("```\n");
    Ok(diagram)
}
