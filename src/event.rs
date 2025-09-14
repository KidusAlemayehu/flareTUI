use std::time::Duration;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::interval;

use crate::action::Action;

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    pub rx: UnboundedReceiver<Event>,
    _tx: UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_timer = interval(Duration::from_millis(tick_rate_ms));

            loop {
                let tick_delay = tick_timer.tick();
                let crossterm_event = reader.next();

                tokio::select! {
                    _ = tick_delay => {
                        if event_tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if event_tx.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            CrosstermEvent::Resize(w, h) => {
                                if event_tx.send(Event::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    pub fn map_key_to_action(key: KeyEvent, search_active: bool) -> Option<Action> {
        if search_active {
            return match key.code {
                KeyCode::Esc => Some(Action::ToggleSearch),
                KeyCode::Enter => Some(Action::InputSubmit),
                KeyCode::Backspace => Some(Action::InputBackspace),
                KeyCode::Char(c) => Some(Action::InputChar(c)),
                _ => None,
            };
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
            (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Action::Quit),
            (KeyModifiers::NONE, KeyCode::Tab) => Some(Action::NextTab),
            (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(Action::PrevTab),
            (KeyModifiers::NONE, KeyCode::Char('1')) => Some(Action::SelectTab(0)),
            (KeyModifiers::NONE, KeyCode::Char('2')) => Some(Action::SelectTab(1)),
            (KeyModifiers::NONE, KeyCode::Char('3')) => Some(Action::SelectTab(2)),
            (KeyModifiers::NONE, KeyCode::Char('4')) => Some(Action::SelectTab(3)),
            (KeyModifiers::NONE, KeyCode::Char('5')) => Some(Action::SelectTab(4)),
            (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => Some(Action::NavDown),
            (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => Some(Action::NavUp),
            (KeyModifiers::NONE, KeyCode::Char('h')) | (KeyModifiers::NONE, KeyCode::Left) => Some(Action::NavLeft),
            (KeyModifiers::NONE, KeyCode::Char('l')) | (KeyModifiers::NONE, KeyCode::Right) => Some(Action::NavRight),
            (KeyModifiers::NONE, KeyCode::Char('s')) => Some(Action::SwitchPane),
            (KeyModifiers::NONE, KeyCode::Char('p')) => Some(Action::ToggleProxy),
            (KeyModifiers::NONE, KeyCode::Char('r')) => Some(Action::Refresh),
            (KeyModifiers::NONE, KeyCode::Char('?')) => Some(Action::ToggleHelp),
            (KeyModifiers::NONE, KeyCode::Char('/')) => Some(Action::ToggleSearch),
            _ => None,
        }
    }
}
