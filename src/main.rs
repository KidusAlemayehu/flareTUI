//! flareTUI: Production-grade Terminal User Interface for Cloudflare.
//!
//! This module serves as the binary entry point. It orchestrates:
//! - Terminal initialization and raw mode lifecycle management.
//! - Panic hook registration to guarantee terminal screen restoration.
//! - Offline-first SQLite local cache initialization.
//! - Asynchronous event loops driven by Tokio MPSC channels and Crossterm inputs.
//! - Centralized state dispatching and prioritized modal event handling.

pub mod action;
pub mod app;
pub mod cloudflare;
pub mod components;
pub mod db;
pub mod event;
pub mod palette;
pub mod security;
pub mod system;
pub mod theme;
pub mod tui;

use color_eyre::Result;
use directories::ProjectDirs;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::action::Action;
use crate::app::App;
use crate::db::CacheDb;
use crate::event::{Event, EventHandler};
use crate::tui::{install_panic_hook, Tui};

/// Resolves the filesystem path for the persistent SQLite cache database.
///
/// Follows the XDG Base Directory specification on Linux (`~/.cache/flaretui/cache.db`)
/// and standard user cache directories on macOS and Windows. If directory
/// resolution fails, defaults to a local file in the working directory.
fn get_cache_db_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "flare", "flareTUI") {
        let cache_dir = proj_dirs.cache_dir();
        let _ = std::fs::create_dir_all(cache_dir);
        cache_dir.join("cache.db")
    } else {
        PathBuf::from("flare-cache.db")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize diagnostic reporting and terminal panic restoration hooks.
    color_eyre::install()?;
    install_panic_hook();

    // 2. Initialize the local SQLite cache database.
    // Falls back to an in-memory database if filesystem permissions or disk I/O fail.
    let db_path = get_cache_db_path();
    let db = CacheDb::open_or_create(&db_path).unwrap_or_else(|_| {
        CacheDb::open_in_memory().expect("Failed to initialize in-memory SQLite cache")
    });

    // 3. Allocate an unbounded MPSC action channel for asynchronous background tasks
    // (e.g., Cloudflare API synchronization, OAuth callbacks, WebSocket log streams).
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    // 4. Initialize the alternate screen terminal interface in raw mode.
    let mut tui = Tui::new()?;
    tui.enter()?;

    // 5. Start the event ingestion loop with a 200ms periodic tick for UI responsiveness.
    let mut event_handler = EventHandler::new(200);
    let mut app = App::new(db, action_tx.clone());

    // 6. Central execution loop: render frames, poll events, and dispatch actions.
    loop {
        tui.terminal.draw(|f| app.render(f))?;

        tokio::select! {
            Some(event) = event_handler.rx.recv() => {
                match event {
                    Event::Tick => {
                        app.update(Action::Tick)?;
                    }
                    Event::Resize(_, _) => {
                        app.update(Action::Render)?;
                    }
                    Event::Key(key) => {
                        // Query active modal hierarchy to route keyboard events to the focused layer.
                        let search_active = app.is_search_active();
                        let auth_active = app.is_auth_active();
                        let inspector_active = app.is_inspector_active();
                        let worker_inspector_active = app.is_worker_inspector_active();
                        let tail_log_details_active = app.is_tail_log_details_active();
                        let dns_modal_active = app.is_dns_modal_active();
                        let delete_modal_active = app.is_delete_modal_active();
                        let pages_deployments_active = app.is_pages_deployments_active();
                        let rollback_modal_active = app.is_rollback_modal_active();
                        let pages_logs_active = app.is_pages_logs_active();
                        let d1_console_active = app.is_d1_console_active();
                        let kv_modal_active = app.is_kv_modal_active();
                        let command_palette_active = app.is_command_palette_active();
                        let security_inspector_active = app.is_security_inspector_active();
                        let profile_modal_active = app.is_profile_switcher_active();
                        let domain_modal_active = app.is_domain_modal_active();
                        let active_tab = app.header.active_tab;
                        let is_zone_pane_focused = app.is_zone_pane_focused();
                        let is_storage_kv_focused = app.is_storage_kv_focused();

                        if let Some(action) = EventHandler::map_key_to_action(
                            key,
                            search_active,
                            auth_active,
                            inspector_active,
                            worker_inspector_active,
                            tail_log_details_active,
                            dns_modal_active,
                            delete_modal_active,
                            pages_deployments_active,
                            rollback_modal_active,
                            pages_logs_active,
                            d1_console_active,
                            kv_modal_active,
                            command_palette_active,
                            security_inspector_active,
                            profile_modal_active,
                            domain_modal_active,
                            active_tab,
                            is_zone_pane_focused,
                            is_storage_kv_focused,
                        ) {
                            app.update(action)?;
                        }
                    }
                }
            }
            Some(action) = action_rx.recv() => {
                // Process background actions emitted by network tasks or WebSocket streams.
                app.update(action)?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    // 7. Restore the terminal cursor, leave the alternate screen, and perform clean exit.
    tui.exit()?;
    println!("Exited flareTUI cleanly.");
    Ok(())
}
