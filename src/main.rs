pub mod action;
pub mod app;
pub mod components;
pub mod db;
pub mod event;
pub mod theme;
pub mod tui;

use std::path::PathBuf;
use color_eyre::Result;
use directories::ProjectDirs;

use crate::action::Action;
use crate::app::App;
use crate::db::CacheDb;
use crate::event::{Event, EventHandler};
use crate::tui::{install_panic_hook, Tui};

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
    color_eyre::install()?;
    install_panic_hook();

    // 1. Initialize Local SQLite Cache
    let db_path = get_cache_db_path();
    let db = CacheDb::open_or_create(&db_path).unwrap_or_else(|_| {
        CacheDb::open_in_memory().expect("Failed to initialize in-memory SQLite cache")
    });

    // 2. Initialize TUI Terminal & Event Stream
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut event_handler = EventHandler::new(200); // 200ms tick for responsive UI & logs
    let mut app = App::new(db);

    // 3. Main Application Loop
    loop {
        tui.terminal.draw(|f| app.render(f))?;

        if let Some(event) = event_handler.rx.recv().await {
            match event {
                Event::Tick => {
                    app.update(Action::Tick)?;
                }
                Event::Resize(_, _) => {
                    app.update(Action::Render)?;
                }
                Event::Key(key) => {
                    let search_active = app.is_search_active();
                    if let Some(action) = EventHandler::map_key_to_action(key, search_active) {
                        app.update(action)?;
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // 4. Graceful Cleanup
    tui.exit()?;
    println!("Exited flareTUI cleanly. See you at the edge!");
    Ok(())
}
