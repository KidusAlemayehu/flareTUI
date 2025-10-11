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

    pub fn map_key_to_action(
        key: KeyEvent,
        search_active: bool,
        auth_active: bool,
        inspector_active: bool,
        dns_modal_active: bool,
        delete_modal_active: bool,
        is_zone_pane_focused: bool,
    ) -> Option<Action> {
        if delete_modal_active {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(Action::ConfirmDeleteDns),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::CloseDeleteDnsModal),
                _ => None,
            };
        }

        if dns_modal_active {
            return match key.code {
                KeyCode::Esc => Some(Action::CloseDnsModal),
                KeyCode::Tab => Some(Action::DnsModalNextField),
                KeyCode::BackTab => Some(Action::DnsModalPrevField),
                KeyCode::Left => Some(Action::DnsModalCycleType(false)),
                KeyCode::Right => Some(Action::DnsModalCycleType(true)),
                KeyCode::Enter => Some(Action::DnsModalSubmit),
                KeyCode::Backspace => Some(Action::DnsModalInputBackspace),
                KeyCode::Char(c) => Some(Action::DnsModalInputChar(c)),
                _ => None,
            };
        }

        if inspector_active {
            return match key.code {
                KeyCode::Esc => Some(Action::CloseZoneInspector),
                KeyCode::Char('s') => Some(Action::CycleZoneSsl),
                KeyCode::Char('d') => Some(Action::ToggleZoneDevMode),
                KeyCode::Char('u') => Some(Action::ToggleZoneUnderAttack),
                KeyCode::Char('p') => Some(Action::PurgeZoneCache),
                _ => None,
            };
        }

        if auth_active {
            return match key.code {
                KeyCode::Esc => Some(Action::ToggleAuth),
                KeyCode::Enter => Some(Action::AuthSubmit),
                KeyCode::Backspace => Some(Action::AuthInputBackspace),
                KeyCode::Char(c) => Some(Action::AuthInputChar(c)),
                _ => None,
            };
        }

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
            (KeyModifiers::NONE, KeyCode::Char('a')) | (KeyModifiers::CONTROL, KeyCode::Char('a')) => Some(Action::ToggleAuth),
            (KeyModifiers::NONE, KeyCode::Char('i')) => Some(Action::OpenZoneInspector),
            (KeyModifiers::NONE, KeyCode::Enter) if is_zone_pane_focused => Some(Action::OpenZoneInspector),
            (KeyModifiers::NONE, KeyCode::Char('n')) => Some(Action::OpenCreateDnsModal),
            (KeyModifiers::NONE, KeyCode::Char('e')) => Some(Action::OpenEditDnsModal),
            (KeyModifiers::NONE, KeyCode::Char('d')) => Some(Action::OpenDeleteDnsModal),
            (KeyModifiers::SHIFT, KeyCode::Char('E')) => Some(Action::ExportZoneBind),
            _ => None,
        }
    }
}
