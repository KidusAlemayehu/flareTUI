use crate::cloudflare::client::{D1QueryResult, PagesBuildLogEntry, PagesDeploymentResult, WorkerRouteDetail, ZoneSettings};
use crate::components::views::tail_view::LogEntry;
use crate::db::CachedWorkerBinding;

#[derive(Debug, Clone, PartialEq)]
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
    AttachWorkerTail(String),
    StopWorkerTail,
    TailSessionStarted { worker: String, tail_id: String },
    ToggleTailPause,
    ClearTailLogs,
    ToggleTailFollow,
    OpenTailLogDetails,
    CloseTailLogDetails,
    TailLogDetailsScrollUp,
    TailLogDetailsScrollDown,
    TailNavUp,
    TailNavDown,

    // Worker Inspector Actions
    OpenWorkerInspector,
    CloseWorkerInspector,
    WorkerInspectorNavUp,
    WorkerInspectorNavDown,
    WorkerDetailsLoaded {
        flags: Vec<String>,
        usage_model: String,
        handlers: Vec<String>,
        routes: Vec<WorkerRouteDetail>,
        bindings: Vec<CachedWorkerBinding>,
    },

    // Storage Actions (D1 SQL Console & Workers KV Inspector)
    OpenD1Console,
    CloseD1Console,
    D1ConsoleSwitchPane,
    D1ConsoleNavUp,
    D1ConsoleNavDown,
    D1ConsoleNavLeft,
    D1ConsoleNavRight,
    D1ConsoleInputChar(char),
    D1ConsoleInputBackspace,
    D1ConsoleInputDelete,
    D1ConsoleClearQuery,
    D1ConsoleApplyPreset(usize),
    D1ConsoleExecuteQuery,
    D1QueryResultLoaded(D1QueryResult),
    D1QueryExecutionFailed(String),

    OpenKvModal,
    CloseKvModal,
    KvModalSwitchPane,
    KvModalNavUp,
    KvModalNavDown,
    KvModalInputChar(char),
    KvModalInputBackspace,
    KvModalClearFilter,
    KvModalFetchValue,
    KvModalValueLoaded(String),
    KvModalValueFailed(String),

    // Feedback
    StatusMessage(String),
    Error(String),
}
