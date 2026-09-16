# Changelog

All notable changes to the `flareTUI` project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/) and adheres to [Semantic Versioning](https://semver.org/).

**Guidelines**

> **Types of changes**
>
> * `Added` for new features.
> * `Changed` for changes in existing functionality.
> * `Deprecated` for soon-to-be removed features.
> * `Removed` for now removed features.
> * `Fixed` for any bug fixes.
>
> **Versioning Format**
>
> * **Version Number** : X.Y.Z
> * Major Version (X): Indicates a significant update with potentially non-backward-compatible changes.
> * Minor Version (Y): Indicates additional features or enhancements that are backward-compatible.
> * Patch Version (Z): Indicates bug fixes or patches that maintain backward compatibility.
> * We start from **0.0.0**.
>
> **Changelog Entry Format**
>
> ```text
> #VERSION_NUMBER (#DATE)
>
> #TYPE_OF_CHANGE
>
> - Description- (#COMMIT_LINK (Optional))
> - Additional details...
> ```

---

## 0.8.3 (2026-08-06)

### Added

- Centralized responsive popup calculation (`centered_rect`) in `src/components/mod.rs` with automated coverage across standard (80x24), widescreen/4K (160x50, 240x80), and compact/tmux split (60x18, 35x14) viewports.

### Changed

- Migrated all 16 modals and inspector dialogs across flareTUI from rigid, magic-number ceilings to dynamically scaled viewports:
  - D1 SQL Console (`D1ConsoleModal`): expanded to 92% width / 88% height (up to 145x48) for wide SQL results grids and schema exploration
  - Workers KV Browser (`KvModal`): scaled to 90% width / 86% height (up to 140x46) with responsive split between keys and value viewer
  - Security Event Inspector (`SecurityInspectorModal`): scaled to 88% width / 85% height (up to 130x42) with multi-tab wrapping
  - Live Tail Log Inspector (`TailLogModal`): scaled to 88% width / 85% height (up to 130x40) with wrapped payload details and traces
  - Worker Script Inspector (`WorkerInspectorModal`): scaled to 85% width / 80% height (up to 120x36)
  - Pages Deployments Drawer (`PagesDeploymentsModal`): scaled to 85% width / 80% height (up to 125x36)
  - Pages Build Logs Viewer (`PagesLogsModal`): scaled to 88% width / 85% height (up to 130x40)
  - Pages Rollback Confirmation (`ConfirmRollbackModal`): scaled to 68% width / 50% height with text-wrapped commit notices
  - Command Palette (`CommandPaletteModal`): scaled to 80% width / 80% height (up to 110x36)
  - Keyboard Shortcuts Overlay (`HelpModal`): scaled to 75% width / 85% height (up to 105x32), displaying all 25 shortcuts without truncation
  - Zone Inspector (`ZoneInspectorModal`): scaled to 78% width / 75% height (up to 110x28) with wrapped actions and SSL cards
  - Domain Creation & Registration (`DomainCreateModal`), DNS Modal (`DnsModal`), Auth Modal (`AuthModal`), Confirm Modal (`ConfirmDeleteModal`), and Profile Modal (`ProfileModal`)
- Added text wrapping (`Wrap { trim: true }`) across all modal footers, action prompts, and operational metadata.

---

## 0.8.2 (2026-07-30)

### Fixed

- Modal dialog responsiveness and text rendering across variable terminal dimensions
  - Domain Registration & Zone Onboarding modal (`DomainCreateModal`):
    - Refactored configuration options (Jump Start DNS scan, Auto-Renew, WHOIS Privacy) from an overflowing 92-column single-line layout into stacked, dedicated rows with active field cursor indicators (`➔`) and full descriptions
    - Dynamic responsive width scaling (68 to 86 columns) and height constraints preventing overflow on compact displays
    - Responsive subtitle, input placeholders, plan tier hints, and keybindings footer
    - Added word-wrapping (`Wrap { trim: true }`) for validation errors, status banners, and multi-line descriptions
  - DNS Record creation and editing modal (`DnsModal`):
    - Responsive two-column row adapting Type and Proxy status indicators dynamically on narrow viewports
    - Added text wrapping for validation errors and footer instructions
  - Confirmation and delete modal (`ConfirmDeleteModal`):
    - Added text wrapping for long target FQDNs, CNAMEs, and IPv6 record destinations
  - Authentication and account profile modal (`AuthModal`):
    - Responsive URL truncation for OAuth callback links based on modal container width
    - Responsive column stacking for Account ID and token credentials in authenticated profile view
  - Command palette & log inspector modals:
    - Added text wrapping to command description pane and worker log traces preventing horizontal clipping

---

## 0.8.1 (2026-07-25)

### Added

- Persistent system settings & theme configuration across application restarts
  - Automatic persistence of selected theme (`Cloudflare Dark`, `Catppuccin Mocha`, `Tokyo Night`, `Nord`) to `config.toml`
  - Restoration of active theme preset immediately during application bootstrap
  - Default landing tab persistence remembering the last active view across sessions
  - Configurable fast sync settings with automatic TOML serialization: `fast_sync`, `sync_interval_secs`, and `auto_sync_on_tab_change`
- Fast targeted view-level revalidation engine (`fast_sync_active_tab`)
  - Sub-second targeted synchronization querying Cloudflare API v4 only for the visible active view (DNS, Workers, Pages, D1/KV, Security/WAF, or Registrar)
  - Automatic stale-while-revalidate execution on tab navigation switches
  - Instant live revalidation upon resource mutations (DNS records, proxy toggles, WAF rules, domain lock/renew)
  - Non-blocking background live polling timer (configurable default 15s interval) in the main event loop
  - Dynamic top header status badge showing live polling status (`⚡ FastSync: 15s Poll`) or offline cache mode
- Command Palette system actions
  - Commands to trigger instant fast sync, toggle live polling, cycle polling frequency (5s, 10s, 15s, 30s, 60s), and toggle auto-sync on tab switch

---

## 0.8.0 (2026-07-23)

### Added

- Cloudflare Domain Registrar dashboard and inspector view (Tab 7) - ([288ebc9](https://github.com/KidusAlemayehu/flareTUI/commit/288ebc9), [5e32072](https://github.com/KidusAlemayehu/flareTUI/commit/5e32072))
  - Dual-pane layout featuring an interactive registered domains table and deep-dive domain inspector panel
  - Real-time display of domain statuses, expiration countdowns, assigned nameservers, and security flags
  - Instant hotkey actions: toggle auto-renew (`a`), transfer lock (`l`), and WHOIS privacy protection (`p`)
  - Dedicated search filter (`/`) with real-time fuzzy matching across domain names and registry states
- Interactive Domain Registration & Zone Onboarding modal - ([77ddb7f](https://github.com/KidusAlemayehu/flareTUI/commit/77ddb7f))
  - Modal form dialog with real-time FQDN validation, plan tier selector (`Free`, `Pro`, `Business`), and jump-start DNS scan toggle
  - Automated write-through to local SQLite cache and asynchronous propagation to Cloudflare Edge API v4
- Cloudflare Registrar API client methods & SQLite persistence - ([834edbe](https://github.com/KidusAlemayehu/flareTUI/commit/834edbe), [ac540f7](https://github.com/KidusAlemayehu/flareTUI/commit/ac540f7))
  - Typed client methods `get_registrar_domains`, `update_registrar_domain`, and `create_zone`
  - SQLite table schema `registrar_domains` with full CRUD support, transactions, and background sync integration in `CloudflareSync`
- Command Palette domain management commands and global keybindings - ([0ec74c8](https://github.com/KidusAlemayehu/flareTUI/commit/0ec74c8))
  - Global hotkey `7` mapped to switch directly to Domains & Registrar view
  - Registered `domain_register`, `domain_toggle_autorenew`, `domain_toggle_lock`, and `domain_toggle_privacy` commands under the `Domains & Registrar` palette category

---

## 0.7.0 (2026-06-16)

### Added

- R2 Object Storage explorer and virtual directory browser - ([ab73110](https://github.com/KidusAlemayehu/flareTUI/commit/ab73110), [7ef56de](https://github.com/KidusAlemayehu/flareTUI/commit/7ef56de), [96499d3](https://github.com/KidusAlemayehu/flareTUI/commit/96499d3))
  - Embedded SQLite cache schema and migration for R2 buckets and objects
  - Typed Cloudflare API v4 client methods `list_r2_buckets` and `list_r2_objects` with delimiter support
  - 3-pane Storage View explorer (`D1 Databases`, `Workers KV`, `R2 Buckets`) with folder hierarchy navigation and search
- Interactive WAF rule control engine - ([07b4686](https://github.com/KidusAlemayehu/flareTUI/commit/07b4686))
  - Hotkeys to toggle rules active/paused (`Space`/`t`) and shift rule execution priorities (`+`/`-`)
  - Integration with Cloudflare Rulesets API to persist rule state modifications directly to the edge
  - Local database cache write-through and status bar dispatch notifications
- Runtime Universal Theme Engine - ([38a9222](https://github.com/KidusAlemayehu/flareTUI/commit/38a9222))
  - Dynamic palette switching across 4 built-in presets: Cloudflare Dark, Catppuccin Mocha, Tokyo Night, and Nord
  - Integrated theme selector commands registered directly in the Universal Command Palette (`Ctrl+P`)
  - Full propagation of updated palette styles across all views, modal dialogs, headers, and status bars
- Browser-based OAuth login redirect flow and session logout - ([871bf2c](https://github.com/KidusAlemayehu/flareTUI/commit/871bf2c), [3436dc9](https://github.com/KidusAlemayehu/flareTUI/commit/3436dc9), [765866f](https://github.com/KidusAlemayehu/flareTUI/commit/765866f), [223951e](https://github.com/KidusAlemayehu/flareTUI/commit/223951e), [3dcee1f](https://github.com/KidusAlemayehu/flareTUI/commit/3dcee1f))
  - Automatic browser launching via `xdg-open`/`open`/`start` with loopback callback server on `127.0.0.1`
  - Cryptographically secure state parameter verification and manual token fallback
  - Dedicated authentication modal with two selectable modes (`Browser Login` vs `API Token`)
  - Instant session logout via `Ctrl+L` and Command Palette `auth_logout` command

### Fixed

- Browser-based OAuth client authentication, PKCE RFC 7636 challenge flow, and terminal output isolation - ([2453587](https://github.com/KidusAlemayehu/flareTUI/commit/2453587), [c2299d0](https://github.com/KidusAlemayehu/flareTUI/commit/c2299d0), [06de90c](https://github.com/KidusAlemayehu/flareTUI/commit/06de90c))
  - Resolved `invalid_client` authorization error using Cloudflare public client ID (`54d11594-84e4-41aa-b438-e81b8fa78ee7`) and dedicated `/oauth/callback` loopback route
  - Implemented RFC 7636 PKCE (Proof Key for Code Exchange) with `S256` code challenge generation, verifier exchange, and Appendix B test vector validation
  - Suppressed child browser and `xdg-open` standard output and error streams via `Stdio::null()` to protect TUI terminal screen buffers from theme warnings and GTK logs
  - Added Cloudflare v4 `/user` endpoint model and client query to dynamically identify user profile email and display real account identity on header navigation

---

## 0.6.0 (2026-02-27)

### Changed

- Reverted Security Analytics view to clean tabular multi-column dashboard
  - Restored 3-column tabular layout for Action Breakdown, Threat Activity Timeline, Top Origin Countries, Top ASNs, Targeted Endpoints, and WAF Rule Hit Leaderboard
  - Streamlined terminal rendering to avoid graph rendering artifacts and maintain clean tabular presentation

### Added

- Visual distribution meters and mitigation cards in Security Analytics - ([99f0f81](https://github.com/KidusAlemayehu/flareTUI/commit/99f0f81))
  - Rendered Unicode horizontal density meters across Top Origin Countries, Origin ASNs, Targeted Endpoints, and WAF Rule Hit Leaderboard
  - Enhanced WAF Overview banner with Total Mitigated count, mitigation percentage rate, and peak velocity indicator

- Ratatui Sparkline and BarChart widgets in Security Analytics tab - ([0a2112a](https://github.com/KidusAlemayehu/flareTUI/commit/0a2112a))
  - Implemented responsive `BarChart` widget displaying action breakdown (Block, Managed Challenge, JS Challenge, Rate Limit)
  - Added dual-layer `Sparkline` widgets rendering Total Activity and Blocked Threats trends side by side
  - Added traffic spike badge (`[! SPIKE]`) in the timeline block header when an anomaly is detected

- Hourly bucket aggregation and sparkline anomaly detection engine - ([f952eb3](https://github.com/KidusAlemayehu/flareTUI/commit/f952eb3))
  - Added `TimelineSparklineData` and `generate_timeline_and_sparklines` for condensing security events into at most 16 uniform display buckets
  - Algorithmic traffic spike detection alerting when peak bucket volume exceeds twice the mean baseline rate
  - Formatting helpers `format_horizontal_meter` and `format_sparkline_text` for terminal density visualization

---

## 0.5.0 (2026-02-14)

### Added

- Core security wiring, Command Palette (Ctrl+P), and profile switcher integration - ([13e38b2](https://github.com/KidusAlemayehu/flareTUI/commit/13e38b2))
  - Connected interactive Security & WAF dashboard view to central application state as tab 6 (`6`)
  - Integrated nucleo-matcher powered universal Command Palette overlay (`Ctrl+P`) with category badges and action execution
  - Added runtime multi-account profile switcher (`Ctrl+U`) with instant API client reinitialization and cache refresh

- Multi-account profile configuration and runtime switcher modal - ([dc1a6ba](https://github.com/KidusAlemayehu/flareTUI/commit/dc1a6ba))
  - Added multi-profile TOML configuration schema supporting distinct API tokens, account IDs, and friendly names
  - Implemented interactive profile selector modal displaying masked tokens, active indicators, and deletion safeguards

- Universal Command Palette modal with category badges and actions - ([b5c76df](https://github.com/KidusAlemayehu/flareTUI/commit/b5c76df))
  - Full-featured command palette modal with dynamic query search and match character highlighting
  - Category filter cycling (`Tab`/`Shift+Tab`) across Navigation, Cloudflare, Security, Storage, Workers, Profiles, and System

- Nucleo-matcher powered fuzzy command palette engine - ([4071c79](https://github.com/KidusAlemayehu/flareTUI/commit/4071c79))
  - High-performance fuzzy matching engine registering application commands with title, description, category, and shortcuts
  - Exact match character indexing for visual highlight styling in the terminal

- Security event inspector modal with IP lookup and rule details - ([c135cc2](https://github.com/KidusAlemayehu/flareTUI/commit/c135cc2))
  - Deep-dive inspector modal featuring sub-tabs for Event Details, WAF Rule Expression, and formatted Raw JSON
  - In-memory clipboard buffer copy triggers for client IP (`c`) and Cloudflare Ray ID (`r`)

- Interactive Security & WAF Analytics dashboard view - ([9962d35](https://github.com/KidusAlemayehu/flareTUI/commit/9962d35))
  - Interactive Security dashboard with Events, WAF Rules, and Analytics & Threats sub-tabs (`t`)
  - Real-time action filter cycling (`f`: Block, Managed Challenge, JS Challenge, Log) and zone-wide threat level gauge

- Security analytics data aggregation and threat level calculators - ([ebbcab3](https://github.com/KidusAlemayehu/flareTUI/commit/ebbcab3))
  - Aggregation engine computing top targeted paths, top attacking IPs and ASNs, action distributions, and 24-hour threat timelines
  - Algorithmic threat score calculator (0-100) classifying zone threat levels into Low, Guarded, Elevated, High, and Critical

- Security events and WAF rules SQLite caching layer - ([beb84e6](https://github.com/KidusAlemayehu/flareTUI/commit/beb84e6))
  - Added `security_events` and `waf_rules` tables to local SQLite cache database with zone foreign key indexing
  - Extended cache models and synchronization routines in `CloudflareSync::sync_all`

- Cloudflare WAF, security events, and firewall rules client models - ([5b22e11](https://github.com/KidusAlemayehu/flareTUI/commit/5b22e11))
  - Added typed data models for security events, firewall rulesets, and analytics summaries
  - Implemented Cloudflare v4 client endpoints: `get_security_events`, `get_waf_rules`, `get_security_summary`

---

## 0.4.0 (2025-12-05)

### Added

- Core storage wiring and main loop integration - ([38bbcb8](https://github.com/KidusAlemayehu/flareTUI/commit/38bbcb8))
  - Connected D1 console modal and Workers KV modal lifecycles into the central application state machine
  - Added event routing for Tab 3 (`Enter` and `i`) to launch SQL console or KV inspector based on pane focus
  - Implemented asynchronous execution dispatchers for live Cloudflare edge API queries and value fetching
  - Added offline simulation fallbacks for query execution and key browsing when unauthenticated

- Live StorageView table integration with SQLite cache - ([8831558](https://github.com/KidusAlemayehu/flareTUI/commit/8831558))
  - Connected `StorageView` to `CacheDb` for live D1 databases and Workers KV namespaces tables
  - Added dynamic row count, database file size formatter (`format_bytes` for B, KB, MB, GB), and URL encoding support flags
  - Added pane switching (`s`) and bounded vertical navigation across database and namespace rows

- Workers KV inspector modal with prefix filter and JSON syntax viewer - ([a3d2e3a](https://github.com/KidusAlemayehu/flareTUI/commit/a3d2e3a))
  - Implemented dual-pane KV inspector with key browser on the left and value viewer on the right
  - Real-time prefix/key filtering (`/`) with debounce and live match count badge
  - Automatic JSON detection, multi-line indentation formatting, syntax highlighting, and line numbering

- D1 SQL console query history buffer and execution metrics - ([8341be7](https://github.com/KidusAlemayehu/flareTUI/commit/8341be7))
  - Added persistent in-memory query history buffer with `Up` and `Down` navigation preserving draft input
  - Word jumping (`Alt+Left`/`Alt+Right`) and line clearing (`Ctrl+U`/`Ctrl+K`)
  - Execution metrics status pill (`⚡ ms`, rows read, rows written, changes count, last row ID) and cell selection indicator

- Interactive D1 SQL console modal with schema explorer and query presets - ([f537d25](https://github.com/KidusAlemayehu/flareTUI/commit/f537d25))
  - Dual-pane layout featuring schema table explorer on the left and SQL editor + tabular result grid on the right
  - 4 quick query templates: Scan table (`LIMIT 25`), Count rows (`COUNT(*)`), Table schema (`PRAGMA table_info`), and Table list (`sqlite_schema`)
  - Dynamic tabular results parser handling arbitrary JSON objects, nulls, numbers, booleans, and nested structures

- SQLite schema cache and sync integration for D1 and KV - ([234b463](https://github.com/KidusAlemayehu/flareTUI/commit/234b463))
  - Added tables `d1_databases`, `d1_tables_cache`, and `kv_namespaces` to local SQLite schema
  - Implemented caching models `CachedD1Database`, `CachedD1Table`, and `CachedKvNamespace` with upsert/query methods
  - Extended `CloudflareSync::sync_all` to fetch and persist D1 databases and KV namespaces in parallel

- Cloudflare API v4 client endpoints and models for D1 and KV - ([f3553b6](https://github.com/KidusAlemayehu/flareTUI/commit/f3553b6))
  - Added data models: `D1DatabaseResult`, `D1QueryRequest`, `D1QueryResult`, `D1QueryMeta`, `KvNamespaceResult`, `KvKeyResult`
  - Implemented client methods: `get_d1_databases`, `execute_d1_query`, `get_kv_namespaces`, `get_kv_keys`, `get_kv_value`

### Fixed

- SQL tabular results clipping and invisible cursor when navigating large result sets - ([09bef43](https://github.com/KidusAlemayehu/flareTUI/commit/09bef43))
  - Implemented dynamic viewport windowing in D1 SQL console table to display all records beyond 18 rows
  - Added vertical scrolling (`result_scroll_y`) and horizontal column windowing (`result_scroll_x`) with active cursor auto-following
  - Added paging keys (`PageUp`/`PageDown`), boundary navigation (`Home`/`End`), and vim motions (`j`/`k`/`h`/`l`/`g`/`G`) in result grid and sidebar
  - Added dynamic viewport range indicator to table header title `(Showing X-Y of Z)` and windowed sidebar schema tables list

---

## 0.3.0 (2025-11-09)

### Added

- Workers core wiring and main loop lifecycle integration - ([8bcbeec](https://github.com/KidusAlemayehu/flareTUI/commit/8bcbeec))
  - Connected Worker Inspector drawer and live tail session switcher to main event loop
  - Wired keyboard shortcut `t` to attach real-time tail WebSocket to the selected worker script
  - Implemented automatic reconnection and cancellation token management

- Real-time log in-memory filtering, follow mode, and buffer clear - ([d621034](https://github.com/KidusAlemayehu/flareTUI/commit/d621034))
  - Interactive search and filter buffer (`/`) for live logs with instant match highlighting
  - Auto-follow tail toggle (`f`) maintaining viewport pinned to incoming log stream
  - Rapid log buffer clearing (`c`) and pause/resume stream toggle (`Space`)

- WebSocket tail session switcher and live log dispatcher - ([bb4e765](https://github.com/KidusAlemayehu/flareTUI/commit/bb4e765))
  - Interactive modal dialog to dynamically select and switch active worker tail target
  - Asynchronous WebSocket channel dispatcher translating Cloudflare tail frames into structured `LogEntry` records

- Worker Inspector drawer with route details and bindings - ([ab3ff2f](https://github.com/KidusAlemayehu/flareTUI/commit/ab3ff2f))
  - Dual-column inspector modal displaying worker runtime flags, compatibility dates, and usage model
  - Route pattern table and resource bindings explorer (KV namespaces, D1 databases, R2 buckets, environment variables)

- Cloudflare Workers metadata and tail client endpoints - ([a7f758a](https://github.com/KidusAlemayehu/flareTUI/commit/a7f758a))
  - Implemented client endpoints: `get_worker_routes`, `get_worker_bindings`, `create_worker_tail`, `delete_worker_tail`
  - Added models: `WorkerRouteDetail`, `WorkerBindingDetail`, `TailSessionResponse`

### Changed

- Documentation and README overhaul - ([b83e1b7](https://github.com/KidusAlemayehu/flareTUI/commit/b83e1b7))
  - Redesigned README with clean edge ASCII art banner and anonymized sample configuration
  - Removed emojis from documentation headings for a clean aesthetic
  - Added dual MIT / Apache-2.0 license files

### Fixed

- API response deserialization for v4 DNS endpoints - ([65e0f9e](https://github.com/KidusAlemayehu/flareTUI/commit/65e0f9e))
  - Handled Cloudflare v4 nested result envelopes and graceful error degradation on restricted zone permissions

---

## 0.2.0 (2025-10-22)

### Added

- Pages core wiring, rollback actions, and event handlers - ([1a61373](https://github.com/KidusAlemayehu/flareTUI/commit/1a61373))
  - Connected Pages deployments drawer and rollback action modals into application main loop
  - Added key triggers for viewing deployments (`Enter`/`d`), rollback (`r`), and build logs (`l`)

- Pages navigation hints and help modal updates - ([b14ac60](https://github.com/KidusAlemayehu/flareTUI/commit/b14ac60))
  - Added contextual status bar key hints and updated help modal overlay for Pages operations

- One-keystroke production rollback and build log viewer - ([d7857fe](https://github.com/KidusAlemayehu/flareTUI/commit/d7857fe))
  - Confirmation guard modal for production rollbacks (`r`) with target deployment hash verification
  - Scrollable modal viewer (`l`) displaying live and historical build logs with timestamps

- Pages deployments drawer with environment badges - ([749c06b](https://github.com/KidusAlemayehu/flareTUI/commit/749c06b))
  - Modal overlay listing project deployments, branch names, commit hashes, URLs, and status badges (Success, Building, Failed)

- Cloudflare Pages API endpoints and models - ([9f53121](https://github.com/KidusAlemayehu/flareTUI/commit/9f53121))
  - Added endpoints: `get_pages_deployments`, `rollback_pages_deployment`, `get_pages_build_logs`
  - Added models: `PagesDeploymentResult`, `PagesBuildLogResult`, `PagesBuildLogEntry`

### Fixed

- Error propagation and token permission reporting - ([2be078c](https://github.com/KidusAlemayehu/flareTUI/commit/2be078c))
  - Improved error messages when API tokens lack permissions for Pages or Zone resources

---

## 0.1.0 (2025-10-11)

### Added

- Domain modals and event mappings wiring - ([03d147c](https://github.com/KidusAlemayehu/flareTUI/commit/03d147c))
  - Connected Zone Inspector, DNS record creation/editing modal, and delete confirmation dialog to main loop

- DNS navigation and status toast system - ([ee8c073](https://github.com/KidusAlemayehu/flareTUI/commit/ee8c073))
  - Added fast single-key proxy toggling (`p`), record filtering (`/`), and status bar toasts

- Zone Inspector modal with edge controls - ([ce13592](https://github.com/KidusAlemayehu/flareTUI/commit/ce13592))
  - Edge control panel: SSL/TLS encryption cycling (`s`: Off, Flexible, Full, Strict), Development Mode toggle (`d`), Under Attack toggle (`u`), and Instant Cache Purge (`p`)

- DNS record editor modal and destructive delete guard - ([2ee4535](https://github.com/KidusAlemayehu/flareTUI/commit/2ee4535))
  - Modal editor supporting record types (A, AAAA, CNAME, TXT, MX, NS, SRV), name, content, TTL, and proxy status
  - Destructive delete confirmation dialog (`d`) preventing accidental DNS record removal

- Cloudflare Zone Settings and DNS CRUD endpoints - ([58969ad](https://github.com/KidusAlemayehu/flareTUI/commit/58969ad))
  - Added client endpoints: `get_zone_settings`, `update_zone_ssl`, `toggle_dev_mode`, `toggle_under_attack`, `purge_cache`, `create_dns_record`, `update_dns_record`, `delete_dns_record`

---

## 0.0.0 (2025-09-27)

### Added

- Cloudflare v4 REST client and local SQLite sync - ([9ec97eb](https://github.com/KidusAlemayehu/flareTUI/commit/9ec97eb))
  - High-performance asynchronous HTTP client for Cloudflare API v4 with bearer token authorization
  - Background synchronization engine pulling zones, records, workers, and pages projects into SQLite

- Real-time worker log streaming and auth modal - ([55c959b](https://github.com/KidusAlemayehu/flareTUI/commit/55c959b))
  - WebSocket tail log consumer and interactive in-app token configuration modal (`a`)

- Central state machine and main application loop - ([8357336](https://github.com/KidusAlemayehu/flareTUI/commit/8357336))
  - Implemented `App` state container managing tabs, active modals, theme, and action dispatcher

- Explorers for Workers, Pages, Storage, and Tail views - ([2c7cc94](https://github.com/KidusAlemayehu/flareTUI/commit/2c7cc94))
  - Created modular tab views: `WorkersView`, `PagesView`, `StorageView`, and `TailView`

- Dual-pane zone explorer and interactive DNS table - ([d15c6bf](https://github.com/KidusAlemayehu/flareTUI/commit/d15c6bf))
  - Left pane zone list and right pane DNS records table with live proxy indicator pill

- Header navigation, status bar toast system, and help overlay - ([c755411](https://github.com/KidusAlemayehu/flareTUI/commit/c755411))
  - Top tab bar (1: DNS, 2: Workers, 3: Pages, 4: Storage, 5: Live Tail), status bar, and quick reference overlay (`?`)

- SQLite local cache layer and schema migration - ([4b10fb5](https://github.com/KidusAlemayehu/flareTUI/commit/4b10fb5))
  - Embedded SQLite database engine (`rusqlite`) for offline caching and fast startup

- Terminal lifecycle manager, panic guard, and async event loop - ([a856e28](https://github.com/KidusAlemayehu/flareTUI/commit/a856e28))
  - Crossterm raw mode manager with terminal restoration panic hook and tokio mpsc event loop

- Cloudflare brand palette theme and action event primitives - ([ddb7118](https://github.com/KidusAlemayehu/flareTUI/commit/ddb7118))
  - Cloudflare color palette (`#F38020` orange, cyan, amber, emerald) and unified `Action` enum

- Project scaffolding - ([b7de004](https://github.com/KidusAlemayehu/flareTUI/commit/b7de004))
  - Initialized Rust workspace with Cargo dependencies and binary entrypoint

### Changed

- Purged mock seed data in favor of live sync - ([fc63ed8](https://github.com/KidusAlemayehu/flareTUI/commit/fc63ed8))
  - Replaced hardcoded dummy data with clean empty states prompting user to sync via `r`## [0.8.4] - 2026-09-16

### Documentation
- Published comprehensive documentation suite under `docs/` ready for static site generator subdomain deployment.
- Revamped `README.md` with responsive terminal preview, 7-tab layout, and complete keyboard shortcuts reference.
- Added comprehensive guides for Cloudflare Registrar, D1 SQL Console, WAF ruleset controls, and WebSocket live tail log filtering.
- Fully documented `src/main.rs` event loop, lifecycle management, and panic recovery hooks with zero informal language.

### Maintenance
- Added `.VSCodeCounter/` to `.gitignore`.
- Version bump to `v0.8.4`.

---


