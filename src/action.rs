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

    // Search input
    InputChar(char),
    InputBackspace,
    InputSubmit,
    ClearSearch,

    // Feedback
    StatusMessage(String),
    Error(String),
}
