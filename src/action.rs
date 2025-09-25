use crate::components::views::tail_view::LogEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Tick,
    Render,
    Quit,

    // Navigation
    NextTab,
    PrevTab,
    SelectTab(usize),
    NavUp,
    NavDown,
    NavLeft,
    NavRight,
    SwitchPane,

    // Operations
    ToggleProxy,
    Refresh,
    ToggleHelp,
    ToggleSearch,
    ToggleAuth,

    // Search input
    InputChar(char),
    InputBackspace,
    InputSubmit,
    ClearSearch,

    // Auth input
    AuthInputChar(char),
    AuthInputBackspace,
    AuthSubmit,

    // Real-time Cloudflare Sync & Logs
    SyncStarted,
    SyncCompleted(String),
    TailLog(LogEntry),

    // Feedback
    StatusMessage(String),
    Error(String),
}
