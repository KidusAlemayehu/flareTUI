use crate::cloudflare::client::{
    D1QueryResult, PagesBuildLogEntry, PagesDeploymentResult, WorkerRouteDetail, ZoneSettings,
};
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
    Logout,

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

    // Auth & OAuth input
    AuthInputChar(char),
    AuthInputBackspace,
    AuthSubmit,
    AuthSwitchMode(crate::components::auth_modal::AuthMode),
    AuthCopyOAuthUrl,
    StartOAuthLogin,
    CancelOAuthLogin,
    OAuthSuccess {
        token: String,
        account_name: Option<String>,
        account_id: Option<String>,
    },
    OAuthError(String),

    // Real-time Cloudflare Sync & Logs
    SyncStarted,
    SyncCompleted(String),
    AccountIdentified(String),
    TailLog(LogEntry),
    AttachWorkerTail(String),
    StopWorkerTail,
    TailSessionStarted {
        worker: String,
        tail_id: String,
    },
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
    D1ConsolePageUp,
    D1ConsolePageDown,
    D1ConsoleHome,
    D1ConsoleEnd,
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

    // Security & WAF Actions
    OpenSecurityInspector,
    CloseSecurityInspector,
    SecurityInspectorNextTab,
    SecurityInspectorPrevTab,
    SecurityInspectorScrollUp,
    SecurityInspectorScrollDown,
    SecurityInspectorCopyIp,
    SecurityInspectorCopyRayId,
    SecurityViewNextTab,
    SecurityViewPrevTab,
    SecurityViewCycleFilter,
    SecurityViewToggleWafRule,
    SecurityViewShiftWafPriority(i32),

    // Command Palette Actions (Ctrl+P)
    OpenCommandPalette,
    CloseCommandPalette,
    CommandPaletteInputChar(char),
    CommandPaletteInputBackspace,
    CommandPaletteClearQuery,
    CommandPaletteNavUp,
    CommandPaletteNavDown,
    CommandPaletteNavTop,
    CommandPaletteNavBottom,
    CommandPaletteNextCategory,
    CommandPalettePrevCategory,
    CommandPaletteExecute,

    // Multi-Account Profile Actions
    OpenProfileSwitcher,
    CloseProfileSwitcher,
    ProfileSwitcherNavUp,
    ProfileSwitcherNavDown,
    ProfileSwitcherSelect,
    ProfileSwitcherDelete,
    ProfileSwitched(String),

    // Domains & Registrar Actions
    OpenDomainCreateModal,
    CloseDomainCreateModal,
    DomainCreateModalNextField,
    DomainCreateModalPrevField,
    DomainCreateModalInputChar(char),
    DomainCreateModalInputBackspace,
    DomainCreateModalCyclePlan(bool),
    DomainCreateModalToggleCheckbox,
    DomainCreateModalSubmit,
    DomainCreated(String),
    ToggleDomainAutoRenew,
    ToggleDomainLock,
    ToggleDomainPrivacy,
    DomainsViewNavUp,
    DomainsViewNavDown,
    DomainsViewToggleSearch,
    RegistrarDomainsLoaded(Vec<crate::db::CachedRegistrarDomain>),

    // Fast Sync & Settings Persistence
    FastSyncActiveView,
    FastSyncCompleted {
        tab: usize,
        message: String,
    },
    ToggleFastSync,
    CycleSyncInterval,
    ToggleAutoSyncOnTabChange,
    SetTheme(String),

    // Feedback
    StatusMessage(String),
    Error(String),
}
