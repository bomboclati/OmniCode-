use std::collections::HashMap;

pub struct KnowledgeGraph {
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
}

pub struct Node {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: HashMap<String, String>,
}

pub struct Edge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: String, label: String, node_type: String) {
        self.nodes.insert(
            id.clone(),
            Node {
                id,
                label,
                node_type,
                properties: HashMap::new(),
            },
        );
    }

    pub fn add_edge(&mut self, from: String, to: String, relation: String) {
        self.edges.push(Edge { from, to, relation });
    }

    pub fn get_related(&self, node_id: &str) -> Vec<&Node> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id || e.to == node_id)
            .filter_map(|e| {
                let other = if e.from == node_id {
                    &e.to
                } else {
                    &e.from
                };
                self.nodes.get(other)
            })
            .collect()
    }

    pub fn find_path(&self, from: &str, to: &str) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(vec![from.to_string()]);

        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();
            if current == to {
                return path;
            }

            if visited.contains(current) {
                continue;
            }
            visited.insert(current.clone());

            for edge in &self.edges {
                if edge.from == *current && !visited.contains(&edge.to) {
                    let mut new_path = path.clone();
                    new_path.push(edge.to.clone());
                    queue.push_back(new_path);
                }
            }
        }

        Vec::new()
    }
}
