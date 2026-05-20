use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    AgentMessage(String),
    CollaborationMessage(String),
    SentinelAlert(String),
    IncidentAlert(String),
    GuardianUpdate(String),
}

pub struct EventHandler {
    pub rx: mpsc::UnboundedReceiver<Event>,
    tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        tokio::spawn(async move {
            let loop_tx = event_tx.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(tick_rate));
                loop {
                    interval.tick().await;
                    let _ = loop_tx.send(Event::Tick);
                }
            });

            loop {
                if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                    match event::read().unwrap_or(CrosstermEvent::Resize(0, 0)) {
                        CrosstermEvent::Key(key) => {
                            let _ = event_tx.send(Event::Key(key));
                        }
                        CrosstermEvent::Mouse(mouse) => {
                            let _ = event_tx.send(Event::Mouse(mouse));
                        }
                        CrosstermEvent::Resize(w, h) => {
                            let _ = event_tx.send(Event::Resize(w, h));
                        }
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        Self { rx, tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }
}
