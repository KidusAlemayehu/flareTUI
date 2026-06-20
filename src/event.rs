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
        worker_inspector_active: bool,
        tail_log_details_active: bool,
        dns_modal_active: bool,
        delete_modal_active: bool,
        pages_deployments_active: bool,
        rollback_modal_active: bool,
        pages_logs_active: bool,
        d1_console_active: bool,
        kv_modal_active: bool,
        command_palette_active: bool,
        security_inspector_active: bool,
        profile_modal_active: bool,
        active_tab: usize,
        is_zone_pane_focused: bool,
        is_storage_kv_focused: bool,
    ) -> Option<Action> {
        if command_palette_active {
            return match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => Some(Action::CloseCommandPalette),
                (_, KeyCode::Enter) => Some(Action::CommandPaletteExecute),
                (_, KeyCode::Up) => Some(Action::CommandPaletteNavUp),
                (_, KeyCode::Down) => Some(Action::CommandPaletteNavDown),
                (_, KeyCode::Home) => Some(Action::CommandPaletteNavTop),
                (_, KeyCode::End) => Some(Action::CommandPaletteNavBottom),
                (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(Action::CommandPalettePrevCategory),
                (_, KeyCode::Tab) => Some(Action::CommandPaletteNextCategory),
                (_, KeyCode::Backspace) => Some(Action::CommandPaletteInputBackspace),
                (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Action::CommandPaletteClearQuery),
                (KeyModifiers::NONE, KeyCode::Char(c)) | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    Some(Action::CommandPaletteInputChar(c))
                }
                _ => None,
            };
        }

        if security_inspector_active {
            return match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Action::CloseSecurityInspector),
                (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(Action::SecurityInspectorPrevTab),
                (_, KeyCode::Tab) => Some(Action::SecurityInspectorNextTab),
                (_, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => Some(Action::SecurityInspectorScrollUp),
                (_, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => Some(Action::SecurityInspectorScrollDown),
                (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Action::SecurityInspectorCopyIp),
                (KeyModifiers::NONE, KeyCode::Char('r')) => Some(Action::SecurityInspectorCopyRayId),
                _ => None,
            };
        }

        if profile_modal_active {
            return match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Action::CloseProfileSwitcher),
                (_, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => Some(Action::ProfileSwitcherNavUp),
                (_, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => Some(Action::ProfileSwitcherNavDown),
                (_, KeyCode::Enter) => Some(Action::ProfileSwitcherSelect),
                (KeyModifiers::NONE, KeyCode::Char('d')) => Some(Action::ProfileSwitcherDelete),
                (KeyModifiers::NONE, KeyCode::Char('l')) | (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(Action::Logout),
                _ => None,
            };
        }

        if d1_console_active {
            return match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => Some(Action::CloseD1Console),
                (_, KeyCode::Tab) => Some(Action::D1ConsoleSwitchPane),
                (KeyModifiers::CONTROL, KeyCode::Enter) => Some(Action::D1ConsoleExecuteQuery),
                (KeyModifiers::CONTROL, KeyCode::Char('u')) | (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    Some(Action::D1ConsoleClearQuery)
                }
                (KeyModifiers::CONTROL, KeyCode::Char('1')) | (KeyModifiers::ALT, KeyCode::Char('1')) => {
                    Some(Action::D1ConsoleApplyPreset(0))
                }
                (KeyModifiers::CONTROL, KeyCode::Char('2')) | (KeyModifiers::ALT, KeyCode::Char('2')) => {
                    Some(Action::D1ConsoleApplyPreset(1))
                }
                (KeyModifiers::CONTROL, KeyCode::Char('3')) | (KeyModifiers::ALT, KeyCode::Char('3')) => {
                    Some(Action::D1ConsoleApplyPreset(2))
                }
                (KeyModifiers::CONTROL, KeyCode::Char('4')) | (KeyModifiers::ALT, KeyCode::Char('4')) => {
                    Some(Action::D1ConsoleApplyPreset(3))
                }
                (_, KeyCode::Up) => Some(Action::D1ConsoleNavUp),
                (_, KeyCode::Down) => Some(Action::D1ConsoleNavDown),
                (_, KeyCode::Left) => Some(Action::D1ConsoleNavLeft),
                (_, KeyCode::Right) => Some(Action::D1ConsoleNavRight),
                (_, KeyCode::PageUp) => Some(Action::D1ConsolePageUp),
                (_, KeyCode::PageDown) => Some(Action::D1ConsolePageDown),
                (_, KeyCode::Home) => Some(Action::D1ConsoleHome),
                (_, KeyCode::End) => Some(Action::D1ConsoleEnd),
                (_, KeyCode::Backspace) => Some(Action::D1ConsoleInputBackspace),
                (_, KeyCode::Delete) => Some(Action::D1ConsoleInputDelete),
                (_, KeyCode::Enter) => Some(Action::D1ConsoleExecuteQuery),
                (KeyModifiers::NONE, KeyCode::Char(c)) | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    Some(Action::D1ConsoleInputChar(c))
                }
                _ => None,
            };
        }

        if kv_modal_active {
            return match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => Some(Action::CloseKvModal),
                (_, KeyCode::Tab) => Some(Action::KvModalSwitchPane),
                (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Action::KvModalClearFilter),
                (_, KeyCode::Up) => Some(Action::KvModalNavUp),
                (_, KeyCode::Down) => Some(Action::KvModalNavDown),
                (_, KeyCode::Backspace) => Some(Action::KvModalInputBackspace),
                (_, KeyCode::Enter) | (KeyModifiers::NONE, KeyCode::Char(' ')) => Some(Action::KvModalFetchValue),
                (KeyModifiers::NONE, KeyCode::Char(c)) | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    Some(Action::KvModalInputChar(c))
                }
                _ => None,
            };
        }

        if tail_log_details_active {
            return match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Action::CloseTailLogDetails),
                KeyCode::Char('j') | KeyCode::Down => Some(Action::TailLogDetailsScrollDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Action::TailLogDetailsScrollUp),
                _ => None,
            };
        }

        if rollback_modal_active {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(Action::ConfirmRollback),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::CloseRollbackModal),
                _ => None,
            };
        }

        if pages_logs_active {
            return match key.code {
                KeyCode::Esc => Some(Action::ClosePagesLogs),
                KeyCode::Char('j') | KeyCode::Down => Some(Action::PagesLogsScrollDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Action::PagesLogsScrollUp),
                _ => None,
            };
        }

        if pages_deployments_active {
            return match key.code {
                KeyCode::Esc => Some(Action::ClosePagesDeployments),
                KeyCode::Char('j') | KeyCode::Down => Some(Action::PagesDeploymentsNavDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Action::PagesDeploymentsNavUp),
                KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::OpenRollbackModal),
                KeyCode::Char('l') | KeyCode::Char('L') => Some(Action::OpenPagesLogs),
                _ => None,
            };
        }

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

        if worker_inspector_active {
            return match key.code {
                KeyCode::Esc => Some(Action::CloseWorkerInspector),
                KeyCode::Char('j') | KeyCode::Down => Some(Action::WorkerInspectorNavDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Action::WorkerInspectorNavUp),
                KeyCode::Char('t') | KeyCode::Enter => Some(Action::AttachWorkerTail(String::new())),
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
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => Some(Action::OpenCommandPalette),
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Action::OpenProfileSwitcher),
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(Action::Logout),
            (KeyModifiers::NONE, KeyCode::Tab) => Some(Action::NextTab),
            (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(Action::PrevTab),
            (KeyModifiers::NONE, KeyCode::Char('1')) => Some(Action::SelectTab(0)),
            (KeyModifiers::NONE, KeyCode::Char('2')) => Some(Action::SelectTab(1)),
            (KeyModifiers::NONE, KeyCode::Char('3')) => Some(Action::SelectTab(2)),
            (KeyModifiers::NONE, KeyCode::Char('4')) => Some(Action::SelectTab(3)),
            (KeyModifiers::NONE, KeyCode::Char('5')) => Some(Action::SelectTab(4)),
            (KeyModifiers::NONE, KeyCode::Char('6')) => Some(Action::SelectTab(5)),
            (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
                if active_tab == 4 {
                    Some(Action::TailNavDown)
                } else {
                    Some(Action::NavDown)
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
                if active_tab == 4 {
                    Some(Action::TailNavUp)
                } else {
                    Some(Action::NavUp)
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('h')) | (KeyModifiers::NONE, KeyCode::Left) => Some(Action::NavLeft),
            (KeyModifiers::NONE, KeyCode::Char('l')) | (KeyModifiers::NONE, KeyCode::Right) => Some(Action::NavRight),
            (KeyModifiers::NONE, KeyCode::Char('s')) => Some(Action::SwitchPane),
            (KeyModifiers::NONE, KeyCode::Char('p')) if active_tab == 5 => Some(Action::SecurityViewToggleWafRule),
            (KeyModifiers::NONE, KeyCode::Char('+')) if active_tab == 5 => Some(Action::SecurityViewShiftWafPriority(1)),
            (KeyModifiers::NONE, KeyCode::Char('-')) if active_tab == 5 => Some(Action::SecurityViewShiftWafPriority(-1)),
            (KeyModifiers::NONE, KeyCode::Char('p')) => Some(Action::ToggleProxy),
            (KeyModifiers::NONE, KeyCode::Char('r')) => Some(Action::Refresh),
            (KeyModifiers::NONE, KeyCode::Char('?')) => Some(Action::ToggleHelp),
            (KeyModifiers::NONE, KeyCode::Char('/')) => Some(Action::ToggleSearch),
            (KeyModifiers::NONE, KeyCode::Char('a')) | (KeyModifiers::CONTROL, KeyCode::Char('a')) => Some(Action::ToggleAuth),
            (KeyModifiers::NONE, KeyCode::Char('i')) if active_tab == 0 => Some(Action::OpenZoneInspector),
            (KeyModifiers::NONE, KeyCode::Char('i')) if active_tab == 1 => Some(Action::OpenWorkerInspector),
            (KeyModifiers::NONE, KeyCode::Char('t')) if active_tab == 1 => Some(Action::AttachWorkerTail(String::new())),
            (KeyModifiers::NONE, KeyCode::Char(' ')) if active_tab == 4 => Some(Action::ToggleTailPause),
            (KeyModifiers::NONE, KeyCode::Char('f')) if active_tab == 4 => Some(Action::ToggleTailFollow),
            (KeyModifiers::NONE, KeyCode::Char('c')) if active_tab == 4 => Some(Action::ClearTailLogs),
            (KeyModifiers::NONE, KeyCode::Enter) if is_zone_pane_focused => Some(Action::OpenZoneInspector),
            (KeyModifiers::NONE, KeyCode::Enter) if active_tab == 1 => Some(Action::AttachWorkerTail(String::new())),
            (KeyModifiers::NONE, KeyCode::Enter) if active_tab == 2 => Some(Action::OpenPagesDeployments),
            (KeyModifiers::NONE, KeyCode::Enter) if active_tab == 3 => {
                if is_storage_kv_focused {
                    Some(Action::OpenKvModal)
                } else {
                    Some(Action::OpenD1Console)
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) if active_tab == 4 => Some(Action::OpenTailLogDetails),
            (KeyModifiers::NONE, KeyCode::Enter) if active_tab == 5 => Some(Action::OpenSecurityInspector),
            (KeyModifiers::NONE, KeyCode::Char('i')) if active_tab == 5 => Some(Action::OpenSecurityInspector),
            (KeyModifiers::NONE, KeyCode::Char('t')) if active_tab == 5 => Some(Action::SecurityViewNextTab),
            (KeyModifiers::NONE, KeyCode::Char('f')) if active_tab == 5 => Some(Action::SecurityViewCycleFilter),
            (KeyModifiers::NONE, KeyCode::Char('i')) if active_tab == 3 => {
                if is_storage_kv_focused {
                    Some(Action::OpenKvModal)
                } else {
                    Some(Action::OpenD1Console)
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('d')) if active_tab == 2 => Some(Action::OpenPagesDeployments),
            (KeyModifiers::NONE, KeyCode::Char('n')) => Some(Action::OpenCreateDnsModal),
            (KeyModifiers::NONE, KeyCode::Char('e')) => Some(Action::OpenEditDnsModal),
            (KeyModifiers::NONE, KeyCode::Char('d')) if active_tab == 0 => Some(Action::OpenDeleteDnsModal),
            (KeyModifiers::SHIFT, KeyCode::Char('E')) => Some(Action::ExportZoneBind),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d1_console_event_mapping() {
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            enter_key, false, false, false, false, false, false, false, false, false, false,
            true,  // d1_console_active
            false, // kv_modal_active
            false, false, false,
            3, false, false,
        );
        assert_eq!(action, Some(Action::D1ConsoleExecuteQuery));

        let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            tab_key, false, false, false, false, false, false, false, false, false, false,
            true, false, false, false, false, 3, false, false,
        );
        assert_eq!(action, Some(Action::D1ConsoleSwitchPane));

        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            esc_key, false, false, false, false, false, false, false, false, false, false,
            true, false, false, false, false, 3, false, false,
        );
        assert_eq!(action, Some(Action::CloseD1Console));

        let pgdn_key = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            pgdn_key, false, false, false, false, false, false, false, false, false, false,
            true, false, false, false, false, 3, false, false,
        );
        assert_eq!(action, Some(Action::D1ConsolePageDown));

        let home_key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            home_key, false, false, false, false, false, false, false, false, false, false,
            true, false, false, false, false, 3, false, false,
        );
        assert_eq!(action, Some(Action::D1ConsoleHome));
    }

    #[test]
    fn test_kv_modal_and_storage_view_event_mapping() {
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Tab 3 with D1 focused
        let action = EventHandler::map_key_to_action(
            enter_key, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, 3, false, false,
        );
        assert_eq!(action, Some(Action::OpenD1Console));

        // Tab 3 with KV focused
        let action = EventHandler::map_key_to_action(
            enter_key, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, 3, false, true,
        );
        assert_eq!(action, Some(Action::OpenKvModal));

        // Inside KV modal
        let action = EventHandler::map_key_to_action(
            enter_key, false, false, false, false, false, false, false, false, false, false,
            false, true, false, false, false, 3, false, true,
        );
        assert_eq!(action, Some(Action::KvModalFetchValue));
    }

    #[test]
    fn test_command_palette_event_mapping() {
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            esc_key, false, false, false, false, false, false, false, false, false, false,
            false, false,
            true,  // command_palette_active
            false, false, 0, false, false,
        );
        assert_eq!(action, Some(Action::CloseCommandPalette));

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            enter_key, false, false, false, false, false, false, false, false, false, false,
            false, false, true, false, false, 0, false, false,
        );
        assert_eq!(action, Some(Action::CommandPaletteExecute));
    }

    #[test]
    fn test_security_and_palette_global_hotkeys() {
        let ctrl_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        let action = EventHandler::map_key_to_action(
            ctrl_p, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, 0, false, false,
        );
        assert_eq!(action, Some(Action::OpenCommandPalette));

        let tab_6 = KeyEvent::new(KeyCode::Char('6'), KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            tab_6, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, 0, false, false,
        );
        assert_eq!(action, Some(Action::SelectTab(5)));
    }

    #[test]
    fn test_waf_rules_key_events() {
        let p_key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            p_key, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, 5, false, false,
        );
        assert_eq!(action, Some(Action::SecurityViewToggleWafRule));

        let plus_key = KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE);
        let action = EventHandler::map_key_to_action(
            plus_key, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, 5, false, false,
        );
        assert_eq!(action, Some(Action::SecurityViewShiftWafPriority(1)));
    }
}

