use anyhow::Result;
use std::collections::HashMap;

pub struct PeerState {
    pub peer_id: String,
    pub cursor_position: (usize, usize),
    pub selected_text: Option<String>,
    pub open_files: Vec<String>,
}

pub struct PeerSync {
    pub peers: HashMap<String, PeerState>,
}

impl PeerSync {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer_id: String) {
        self.peers.insert(
            peer_id.clone(),
            PeerState {
                peer_id,
                cursor_position: (0, 0),
                selected_text: None,
                open_files: Vec::new(),
            },
        );
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }

    pub fn update_peer_cursor(&mut self, peer_id: &str, line: usize, col: usize) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.cursor_position = (line, col);
        }
    }

    pub fn get_active_peers(&self) -> Vec<&PeerState> {
        self.peers.values().collect()
    }
}
