use crate::cloudflare::client::{PagesBuildLogEntry, PagesDeploymentResult, ZoneSettings};
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

    // Zone Inspector Actions
    OpenZoneInspector,
    CloseZoneInspector,
    ZoneSettingsLoaded(ZoneSettings),
    CycleZoneSsl,
    ToggleZoneDevMode,
    ToggleZoneUnderAttack,
    PurgeZoneCache,

    // DNS Modal Actions
    OpenCreateDnsModal,
    OpenEditDnsModal,
    CloseDnsModal,
    DnsModalNextField,
    DnsModalPrevField,
    DnsModalCycleType(bool),
    DnsModalInputChar(char),
    DnsModalInputBackspace,
    DnsModalSubmit,

    // Delete Confirmation Guard
    OpenDeleteDnsModal,
    CloseDeleteDnsModal,
    ConfirmDeleteDns,

    // Pages Deployments, Rollback, and Logs
    OpenPagesDeployments,
    ClosePagesDeployments,
    PagesDeploymentsLoaded(Vec<PagesDeploymentResult>),
    PagesDeploymentsNavUp,
    PagesDeploymentsNavDown,
    OpenRollbackModal,
    CloseRollbackModal,
    ConfirmRollback,
    OpenPagesLogs,
    ClosePagesLogs,
    PagesLogsLoaded(Vec<PagesBuildLogEntry>),
    PagesLogsScrollUp,
    PagesLogsScrollDown,

    // Export Zone
    ExportZoneBind,

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
