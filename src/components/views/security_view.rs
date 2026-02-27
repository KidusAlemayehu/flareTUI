use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    symbols::Marker,
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, BorderType, Borders, Cell, Chart, Dataset, GraphType,
        Paragraph, Row, Table, Tabs,
    },
    Frame,
};

use crate::db::{CacheDb, CachedSecurityEvent, CachedWafRule};
use crate::security::{SecurityAnalyticsEngine, SecurityAnalyticsReport, ThreatLevel};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityTab {
    Events = 0,
    WafRules = 1,
    Analytics = 2,
}

impl SecurityTab {
    pub fn all() -> &'static [SecurityTab] {
        &[SecurityTab::Events, SecurityTab::WafRules, SecurityTab::Analytics]
    }

    pub fn title(&self) -> &'static str {
        match self {
            SecurityTab::Events => "Events",
            SecurityTab::WafRules => "WAF Rules",
            SecurityTab::Analytics => "Analytics & Threats",
        }
    }
}

pub struct SecurityView {
    pub db: CacheDb,
    pub current_zone_id: Option<String>,
    pub events: Vec<CachedSecurityEvent>,
    pub rules: Vec<CachedWafRule>,
    pub analytics: SecurityAnalyticsReport,
    pub active_tab: SecurityTab,
    pub selected_event_idx: usize,
    pub selected_rule_idx: usize,
    pub action_filter: Option<String>,
    pub search_query: String,
    pub is_searching: bool,
}

impl SecurityView {
    pub fn new(db: CacheDb) -> Self {
        let mut view = Self {
            db,
            current_zone_id: None,
            events: Vec::new(),
            rules: Vec::new(),
            analytics: SecurityAnalyticsEngine::analyze(&[], &[]),
            active_tab: SecurityTab::Events,
            selected_event_idx: 0,
            selected_rule_idx: 0,
            action_filter: None,
            search_query: String::new(),
            is_searching: false,
        };
        view.reload(None);
        view
    }

    pub fn reload(&mut self, zone_id: Option<&str>) {
        if let Some(zid) = zone_id {
            self.current_zone_id = Some(zid.to_string());
        }

        let zid = match &self.current_zone_id {
            Some(z) => z.clone(),
            None => {
                if let Ok(zones) = self.db.get_zones() {
                    if let Some(first) = zones.first() {
                        self.current_zone_id = Some(first.id.clone());
                        first.id.clone()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
        };

        if !zid.is_empty() {
            self.events = self.db.get_security_events(&zid).unwrap_or_default();
            self.rules = self.db.get_waf_rules(&zid).unwrap_or_default();
        } else {
            self.events.clear();
            self.rules.clear();
        }

        self.analytics = SecurityAnalyticsEngine::analyze(&self.events, &self.rules);

        let filtered_len = self.filtered_events().len();
        if self.selected_event_idx >= filtered_len && filtered_len > 0 {
            self.selected_event_idx = filtered_len - 1;
        }

        if self.selected_rule_idx >= self.rules.len() && !self.rules.is_empty() {
            self.selected_rule_idx = self.rules.len() - 1;
        }
    }

    pub fn set_zone(&mut self, zone_id: &str) {
        self.current_zone_id = Some(zone_id.to_string());
        self.selected_event_idx = 0;
        self.selected_rule_idx = 0;
        self.reload(Some(zone_id));
    }

    pub fn filtered_events(&self) -> Vec<CachedSecurityEvent> {
        let act_opt = self.action_filter.as_deref();
        let q_opt = if self.search_query.trim().is_empty() {
            None
        } else {
            Some(self.search_query.as_str())
        };
        SecurityAnalyticsEngine::filter_events(&self.events, act_opt, q_opt)
    }

    pub fn selected_event(&self) -> Option<CachedSecurityEvent> {
        let filtered = self.filtered_events();
        filtered.get(self.selected_event_idx).cloned()
    }

    pub fn selected_rule(&self) -> Option<CachedWafRule> {
        self.rules.get(self.selected_rule_idx).cloned()
    }

    pub fn nav_up(&mut self) {
        match self.active_tab {
            SecurityTab::Events => {
                if self.selected_event_idx > 0 {
                    self.selected_event_idx -= 1;
                }
            }
            SecurityTab::WafRules => {
                if self.selected_rule_idx > 0 {
                    self.selected_rule_idx -= 1;
                }
            }
            SecurityTab::Analytics => {}
        }
    }

    pub fn nav_down(&mut self) {
        match self.active_tab {
            SecurityTab::Events => {
                let len = self.filtered_events().len();
                if len > 0 && self.selected_event_idx + 1 < len {
                    self.selected_event_idx += 1;
                }
            }
            SecurityTab::WafRules => {
                if !self.rules.is_empty() && self.selected_rule_idx + 1 < self.rules.len() {
                    self.selected_rule_idx += 1;
                }
            }
            SecurityTab::Analytics => {}
        }
    }

    pub fn nav_top(&mut self) {
        match self.active_tab {
            SecurityTab::Events => self.selected_event_idx = 0,
            SecurityTab::WafRules => self.selected_rule_idx = 0,
            SecurityTab::Analytics => {}
        }
    }

    pub fn nav_bottom(&mut self) {
        match self.active_tab {
            SecurityTab::Events => {
                let len = self.filtered_events().len();
                if len > 0 {
                    self.selected_event_idx = len - 1;
                }
            }
            SecurityTab::WafRules => {
                if !self.rules.is_empty() {
                    self.selected_rule_idx = self.rules.len() - 1;
                }
            }
            SecurityTab::Analytics => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            SecurityTab::Events => SecurityTab::WafRules,
            SecurityTab::WafRules => SecurityTab::Analytics,
            SecurityTab::Analytics => SecurityTab::Events,
        };
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            SecurityTab::Events => SecurityTab::Analytics,
            SecurityTab::WafRules => SecurityTab::Events,
            SecurityTab::Analytics => SecurityTab::WafRules,
        };
    }

    pub fn cycle_action_filter(&mut self) {
        self.action_filter = match self.action_filter.as_deref() {
            None => Some("block".to_string()),
            Some("block") => Some("managed_challenge".to_string()),
            Some("managed_challenge") => Some("challenge".to_string()),
            Some("challenge") => Some("js_challenge".to_string()),
            Some("js_challenge") => Some("rate_limit".to_string()),
            Some("rate_limit") => Some("log".to_string()),
            _ => None,
        };
        self.selected_event_idx = 0;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Threat metric banner
                Constraint::Length(3), // Tabs and filter bar
                Constraint::Min(5),    // Main view content
            ])
            .split(area);

        self.render_threat_banner(f, chunks[0], theme);
        self.render_tab_bar(f, chunks[1], theme);

        match self.active_tab {
            SecurityTab::Events => self.render_events_tab(f, chunks[2], theme),
            SecurityTab::WafRules => self.render_waf_rules_tab(f, chunks[2], theme),
            SecurityTab::Analytics => self.render_analytics_tab(f, chunks[2], theme),
        }
    }

    fn render_threat_banner(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let (level_color, level_bg) = match self.analytics.threat_level {
            ThreatLevel::Low => (theme.success, Color::Reset),
            ThreatLevel::Medium => (theme.warning, Color::Reset),
            ThreatLevel::High => (theme.primary, Color::Reset),
            ThreatLevel::Critical => (Color::White, theme.error),
        };

        let zone_name = self.current_zone_id.as_deref().unwrap_or("No Zone");

        let mut spans = vec![
            Span::styled("Zone: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{zone_name}  |  "), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("Threat Level: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" {} ", self.analytics.threat_level.as_str()),
                Style::default().fg(level_color).bg(level_bg).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/100", self.analytics.threat_score),
                Style::default().fg(level_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  |  Total: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} ", self.analytics.total_events), Style::default().fg(theme.text)),
            Span::styled("Mitigated: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ({:.1}%) ", self.analytics.total_mitigated, self.analytics.mitigation_rate),
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Blocked: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} ", self.analytics.threats_blocked), Style::default().fg(theme.error)),
            Span::styled("Challenged: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} ", self.analytics.threats_challenged), Style::default().fg(theme.warning)),
            Span::styled("Rate Limited: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", self.analytics.rate_limited), Style::default().fg(theme.info)),
        ];

        if let Some(peak) = &self.analytics.sparkline.peak_hour {
            spans.push(Span::styled("  |  Peak: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(peak.clone(), Style::default().fg(theme.secondary)));
        }

        let text = vec![Line::from(spans)];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" WAF & Security Overview ", theme.title()));

        let p = Paragraph::new(text).block(block).alignment(Alignment::Left);
        f.render_widget(p, area);
    }

    fn render_tab_bar(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let titles: Vec<Line> = SecurityTab::all()
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let badge = match tab {
                    SecurityTab::Events => format!(" ({})", self.filtered_events().len()),
                    SecurityTab::WafRules => format!(" ({})", self.rules.len()),
                    SecurityTab::Analytics => String::new(),
                };
                Line::from(format!("[{}] {}{}", i + 1, tab.title(), badge))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(
                        format!(
                            " Navigation [Tab/Shift+Tab]  Filter [f]: {}  Search [/]: {} ",
                            self.action_filter.as_deref().unwrap_or("ALL"),
                            if self.search_query.is_empty() {
                                "None"
                            } else {
                                &self.search_query
                            }
                        ),
                        Style::default().fg(theme.secondary),
                    )),
            )
            .select(self.active_tab as usize)
            .style(Style::default().fg(theme.text_dim))
            .highlight_style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );

        f.render_widget(tabs, area);
    }

    fn render_events_tab(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let events = self.filtered_events();

        if events.is_empty() {
            let msg = if self.events.is_empty() {
                "No security events cached for this zone. Sync with 's' or verify zone activity."
            } else {
                "No events matching current filter or search criteria."
            };
            let p = Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(theme.border))
                        .title(" Security Events Feed "),
                );
            f.render_widget(p, area);
            return;
        }

        let header_cells = [
            "Action",
            "Timestamp",
            "Source",
            "Client IP",
            "Country",
            "Method",
            "Path",
            "Rule / Description",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(theme.header_style()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = events.iter().enumerate().map(|(idx, ev)| {
            let is_selected = idx == self.selected_event_idx;
            let action_style = match ev.action.to_lowercase().as_str() {
                "block" => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
                "managed_challenge" | "challenge" | "js_challenge" => {
                    Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)
                }
                "rate_limit" => Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                _ => Style::default().fg(theme.text_dim),
            };

            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default()
            };

            let rule_disp = if !ev.rule_description.is_empty() {
                &ev.rule_description
            } else if !ev.rule_id.is_empty() {
                &ev.rule_id
            } else {
                "-"
            };

            let time_disp = if ev.timestamp.len() >= 19 {
                &ev.timestamp[11..19]
            } else {
                &ev.timestamp
            };

            Row::new(vec![
                Cell::from(ev.action.clone()).style(action_style),
                Cell::from(time_disp.to_string()),
                Cell::from(ev.source.clone()),
                Cell::from(ev.client_ip.clone()),
                Cell::from(ev.client_country.clone()),
                Cell::from(ev.http_method.clone()),
                Cell::from(ev.http_path.clone()),
                Cell::from(rule_disp.to_string()),
            ])
            .style(row_style)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(18), // Action
                Constraint::Length(10), // Timestamp
                Constraint::Length(14), // Source
                Constraint::Length(16), // Client IP
                Constraint::Length(9),  // Country
                Constraint::Length(8),  // Method
                Constraint::Length(22), // Path
                Constraint::Min(25),    // Rule
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(
                    format!(
                        " Security Events Feed [{}/{}] - Press [Enter] for Inspector ",
                        if events.is_empty() { 0 } else { self.selected_event_idx + 1 },
                        events.len()
                    ),
                    theme.title(),
                )),
        );

        f.render_widget(table, area);
    }

    fn render_waf_rules_tab(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if self.rules.is_empty() {
            let p = Paragraph::new("No WAF rules found for this zone. Sync with 's' to fetch rules.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(theme.border))
                        .title(" Configured WAF Rules "),
                );
            f.render_widget(p, area);
            return;
        }

        let header_cells = ["Pri", "Status", "Action", "Description", "Expression", "Reference"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.header_style()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = self.rules.iter().enumerate().map(|(idx, rule)| {
            let is_selected = idx == self.selected_rule_idx;
            let status_style = if rule.paused {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(theme.success)
            };
            let status_text = if rule.paused { "PAUSED" } else { "ACTIVE" };

            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(rule.priority.to_string()),
                Cell::from(status_text).style(status_style),
                Cell::from(rule.action.clone()).style(Style::default().fg(theme.primary)),
                Cell::from(rule.description.clone()),
                Cell::from(rule.expression.clone()),
                Cell::from(rule.ref_code.clone()),
            ])
            .style(row_style)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),  // Priority
                Constraint::Length(8),  // Status
                Constraint::Length(18), // Action
                Constraint::Length(30), // Description
                Constraint::Min(35),    // Expression
                Constraint::Length(16), // Reference
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(
                    format!(
                        " Configured WAF Rules [{}/{}] ",
                        self.selected_rule_idx + 1,
                        self.rules.len()
                    ),
                    theme.title(),
                )),
        );

        f.render_widget(table, area);
    }

    fn render_analytics_tab(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if area.height >= 16 {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(45), // Top: Full-width Time-Series Line Graph (like Cloudflare Dashboard)
                    Constraint::Percentage(55), // Bottom: 3-column Analytical Breakdown
                ])
                .split(area);

            self.render_threat_line_chart(f, main_chunks[0], theme);

            let bottom_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(main_chunks[1]);

            self.render_action_breakdown(f, bottom_cols[0], theme);

            let col2_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(bottom_cols[1]);
            self.render_top_countries(f, col2_chunks[0], theme);
            self.render_top_asns(f, col2_chunks[1], theme);

            let col3_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(bottom_cols[2]);
            self.render_top_paths(f, col3_chunks[0], theme);
            self.render_rule_hits(f, col3_chunks[1], theme);
        } else {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(area);

            let col1_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(columns[0]);
            self.render_action_breakdown(f, col1_chunks[0], theme);
            self.render_threat_line_chart(f, col1_chunks[1], theme);

            let col2_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(columns[1]);
            self.render_top_countries(f, col2_chunks[0], theme);
            self.render_top_asns(f, col2_chunks[1], theme);

            let col3_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(columns[2]);
            self.render_top_paths(f, col3_chunks[0], theme);
            self.render_rule_hits(f, col3_chunks[1], theme);
        }
    }

    fn render_threat_line_chart(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut title_spans = vec![
            Span::styled(" Threat Activity & Traffic Velocity ", theme.title()),
        ];
        if self.analytics.sparkline.anomaly_detected {
            title_spans.push(Span::styled(
                " [! SPIKE DETECTED] ",
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
            ));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Line::from(title_spans));

        if self.analytics.timeline.is_empty() {
            let p = Paragraph::new("No time-series security events recorded for this zone.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let inner = block.inner(area);
        f.render_widget(block, area);

        let (chart_area, table_area) = if inner.width >= 90 && inner.height >= 6 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(inner);
            (chunks[0], Some(chunks[1]))
        } else {
            (inner, None)
        };

        let buckets = &self.analytics.timeline;
        let n = buckets.len();

        let total_data: Vec<(f64, f64)> = if n == 1 {
            vec![(0.0, buckets[0].total as f64), (1.0, buckets[0].total as f64)]
        } else {
            buckets.iter().enumerate().map(|(i, b)| (i as f64, b.total as f64)).collect()
        };

        let blocked_data: Vec<(f64, f64)> = if n == 1 {
            vec![(0.0, buckets[0].blocked as f64), (1.0, buckets[0].blocked as f64)]
        } else {
            buckets.iter().enumerate().map(|(i, b)| (i as f64, b.blocked as f64)).collect()
        };

        let challenged_data: Vec<(f64, f64)> = if n == 1 {
            vec![(0.0, buckets[0].challenged as f64), (1.0, buckets[0].challenged as f64)]
        } else {
            buckets.iter().enumerate().map(|(i, b)| (i as f64, b.challenged as f64)).collect()
        };

        let max_val = self.analytics.sparkline.peak_volume.max(1) as f64;
        let y_max = (max_val * 1.15).ceil();
        let x_max = if n <= 1 { 1.0 } else { (n - 1) as f64 };

        let x_labels: Vec<Span> = if n <= 1 {
            let lbl = buckets.first().map(|b| b.label.as_str()).unwrap_or("00:00");
            vec![Span::raw(lbl)]
        } else if n == 2 {
            vec![
                Span::raw(buckets.first().unwrap().label.clone()),
                Span::raw(buckets.last().unwrap().label.clone()),
            ]
        } else {
            vec![
                Span::raw(buckets.first().unwrap().label.clone()),
                Span::raw(buckets[n / 2].label.clone()),
                Span::raw(buckets.last().unwrap().label.clone()),
            ]
        };

        let y_mid = (y_max / 2.0).round();
        let y_labels = vec![
            Span::raw("0"),
            Span::raw(format!("{y_mid:.0}")),
            Span::raw(format!("{y_max:.0}")),
        ];

        let mut datasets = vec![
            Dataset::default()
                .name(format!("Total ({})", self.analytics.total_events))
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme.info))
                .data(&total_data),
            Dataset::default()
                .name(format!("Blocked ({})", self.analytics.threats_blocked))
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme.error))
                .data(&blocked_data),
        ];

        if self.analytics.threats_challenged > 0 {
            datasets.push(
                Dataset::default()
                    .name(format!("Challenged ({})", self.analytics.threats_challenged))
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(theme.warning))
                    .data(&challenged_data),
            );
        }

        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title(Span::styled(" Time ", Style::default().fg(theme.text_dim)))
                    .style(Style::default().fg(theme.border))
                    .bounds([0.0, x_max])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled(" Requests ", Style::default().fg(theme.text_dim)))
                    .style(Style::default().fg(theme.border))
                    .bounds([0.0, y_max])
                    .labels(y_labels),
            );

        f.render_widget(chart, chart_area);

        if let Some(t_area) = table_area {
            let rows = self.analytics.timeline.iter().map(|tb| {
                Row::new(vec![
                    Cell::from(tb.label.clone()),
                    Cell::from(tb.total.to_string()),
                    Cell::from(tb.blocked.to_string()).style(Style::default().fg(theme.error)),
                    Cell::from(tb.challenged.to_string()).style(Style::default().fg(theme.warning)),
                ])
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ],
            )
            .header(
                Row::new(["Time", "Total", "Block", "Chal"])
                    .style(theme.header_style())
                    .bottom_margin(1),
            );

            f.render_widget(table, t_area);
        }
    }

    fn render_action_breakdown(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" Action Breakdown ", theme.title()));

        if self.analytics.action_breakdown.is_empty() {
            let p = Paragraph::new("No security actions recorded.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let inner = block.inner(area);
        f.render_widget(block, area);

        let (chart_area, table_area) = if inner.height >= 7 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(3)])
                .split(inner);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, inner)
        };

        if let Some(c_area) = chart_area {
            let bars: Vec<Bar> = self
                .analytics
                .action_breakdown
                .iter()
                .take(5)
                .map(|ab| {
                    let color = match ab.action.to_lowercase().as_str() {
                        "block" => theme.error,
                        "managed_challenge" | "challenge" | "js_challenge" => theme.warning,
                        "rate_limit" => theme.info,
                        _ => theme.secondary,
                    };
                    let label = match ab.action.as_str() {
                        "managed_challenge" => "m_chal",
                        "js_challenge" => "js_chal",
                        "rate_limit" => "rate_lim",
                        other => other,
                    };
                    Bar::default()
                        .value(ab.count as u64)
                        .label(label.into())
                        .style(Style::default().fg(color))
                        .value_style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
                })
                .collect();

            let bar_group = BarGroup::default().bars(&bars);
            let max_val = self
                .analytics
                .action_breakdown
                .iter()
                .map(|a| a.count as u64)
                .max()
                .unwrap_or(1);

            let bar_chart = BarChart::default()
                .data(bar_group)
                .bar_width(8)
                .bar_gap(1)
                .max(max_val);
            f.render_widget(bar_chart, c_area);
        }

        let rows = self.analytics.action_breakdown.iter().map(|ab| {
            let color = match ab.action.to_lowercase().as_str() {
                "block" => theme.error,
                "managed_challenge" | "challenge" | "js_challenge" => theme.warning,
                "rate_limit" => theme.info,
                _ => theme.text_dim,
            };
            Row::new(vec![
                Cell::from(ab.action.clone()).style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Cell::from(ab.count.to_string()),
                Cell::from(format!("{:.1}%", ab.percentage)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .header(
            Row::new(["Action", "Count", "Share"])
                .style(theme.header_style())
                .bottom_margin(1),
        );

        f.render_widget(table, table_area);
    }

    fn render_top_countries(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" Top Origin Countries ", theme.title()));

        if self.analytics.top_countries.is_empty() {
            let p = Paragraph::new("No origin countries recorded.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let rows = self.analytics.top_countries.iter().take(6).map(|cs| {
            let meter = SecurityAnalyticsEngine::format_horizontal_meter(cs.percentage / 100.0, 8);
            Row::new(vec![
                Cell::from(cs.country.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from(cs.count.to_string()),
                Cell::from(Line::from(vec![
                    Span::styled(meter, Style::default().fg(theme.primary)),
                    Span::raw(format!(" {:.0}%", cs.percentage)),
                ])),
                Cell::from(cs.blocked_count.to_string()).style(Style::default().fg(theme.error)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(22),
                Constraint::Percentage(20),
                Constraint::Percentage(38),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(["Country", "Events", "Distribution", "Blocked"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(block);

        f.render_widget(table, area);
    }

    fn render_top_asns(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" Top Origin ASNs ", theme.title()));

        if self.analytics.top_asns.is_empty() {
            let p = Paragraph::new("No origin ASNs recorded.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let rows = self.analytics.top_asns.iter().take(6).map(|as_stat| {
            let asn_label = if as_stat.asn > 0 {
                format!("AS{}", as_stat.asn)
            } else {
                "-".to_string()
            };
            let meter = SecurityAnalyticsEngine::format_horizontal_meter(as_stat.percentage / 100.0, 6);
            Row::new(vec![
                Cell::from(asn_label).style(Style::default().fg(theme.secondary)),
                Cell::from(as_stat.org.clone()),
                Cell::from(as_stat.count.to_string()),
                Cell::from(Line::from(vec![
                    Span::styled(meter, Style::default().fg(theme.secondary)),
                    Span::raw(format!(" {:.0}%", as_stat.percentage)),
                ])),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(9),
                Constraint::Min(14),
                Constraint::Length(7),
                Constraint::Length(14),
            ],
        )
        .header(
            Row::new(["ASN", "Organization", "Hits", "Share"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(block);

        f.render_widget(table, area);
    }

    fn render_top_paths(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" Targeted Endpoints ", theme.title()));

        if self.analytics.top_paths.is_empty() {
            let p = Paragraph::new("No targeted paths recorded.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let total_ev = self.analytics.total_events.max(1) as f64;
        let rows = self.analytics.top_paths.iter().take(6).map(|ps| {
            let ratio = ps.count as f64 / total_ev;
            let meter = SecurityAnalyticsEngine::format_horizontal_meter(ratio, 6);
            let action_color = match ps.top_action.to_lowercase().as_str() {
                "block" => theme.error,
                "managed_challenge" | "challenge" | "js_challenge" => theme.warning,
                "rate_limit" => theme.info,
                _ => theme.primary,
            };
            Row::new(vec![
                Cell::from(ps.path.clone()).style(Style::default().fg(theme.text)),
                Cell::from(ps.count.to_string()),
                Cell::from(Line::from(vec![
                    Span::styled(meter, Style::default().fg(theme.warning)),
                    Span::raw(format!(" {:.0}%", ratio * 100.0)),
                ])),
                Cell::from(ps.top_action.clone()).style(Style::default().fg(action_color)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Min(16),
                Constraint::Length(6),
                Constraint::Length(14),
                Constraint::Length(12),
            ],
        )
        .header(
            Row::new(["Target Path", "Hits", "Activity", "Action"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(block);

        f.render_widget(table, area);
    }

    fn render_rule_hits(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" WAF Rule Hit Leaderboard ", theme.title()));

        if self.analytics.rule_hits.is_empty() {
            let p = Paragraph::new("No WAF rule hits recorded.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let total_hits: usize = self.analytics.rule_hits.iter().map(|rh| rh.hits).sum();
        let total_base = total_hits.max(1) as f64;

        let rows = self.analytics.rule_hits.iter().take(6).map(|rh| {
            let ratio = rh.hits as f64 / total_base;
            let meter = SecurityAnalyticsEngine::format_horizontal_meter(ratio, 6);
            Row::new(vec![
                Cell::from(rh.description.clone()),
                Cell::from(rh.action.clone()).style(Style::default().fg(theme.primary)),
                Cell::from(rh.hits.to_string()).style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from(Line::from(vec![
                    Span::styled(meter, Style::default().fg(theme.primary)),
                    Span::raw(format!(" {:.0}%", ratio * 100.0)),
                ])),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Min(18),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Length(14),
            ],
        )
        .header(
            Row::new(["Rule Description", "Action", "Hits", "Impact"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(block);

        f.render_widget(table, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Zone;

    fn setup_test_db() -> (CacheDb, String) {
        let db = CacheDb::open_in_memory().expect("open test db");
        let zone_id = "zone-sec-view-test";
        db.upsert_zone(&Zone {
            id: zone_id.to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            plan: "Pro".to_string(),
            created_on: "2026-01-01".to_string(),
        })
        .expect("upsert zone");

        let events = vec![
            CachedSecurityEvent {
                ray_id: "8472a1bf001".to_string(),
                zone_id: zone_id.to_string(),
                timestamp: "2026-01-15T12:00:00Z".to_string(),
                action: "block".to_string(),
                source: "firewallCustom".to_string(),
                rule_id: "rule-sqli".to_string(),
                rule_description: "Block SQL injection".to_string(),
                client_ip: "198.51.100.42".to_string(),
                client_country: "US".to_string(),
                client_asn: 13335,
                client_as_org: "Cloudflare Inc".to_string(),
                http_method: "POST".to_string(),
                http_host: "api.example.com".to_string(),
                http_path: "/login".to_string(),
                user_agent: "sqlmap/1.7".to_string(),
            },
            CachedSecurityEvent {
                ray_id: "8472a1bf002".to_string(),
                zone_id: zone_id.to_string(),
                timestamp: "2026-01-15T12:05:00Z".to_string(),
                action: "managed_challenge".to_string(),
                source: "waf".to_string(),
                rule_id: "rule-bot".to_string(),
                rule_description: "Challenge bot".to_string(),
                client_ip: "203.0.113.19".to_string(),
                client_country: "DE".to_string(),
                client_asn: 24940,
                client_as_org: "Hetzner Online GmbH".to_string(),
                http_method: "GET".to_string(),
                http_host: "api.example.com".to_string(),
                http_path: "/catalog".to_string(),
                user_agent: "Mozilla/5.0 Bot".to_string(),
            },
        ];
        db.replace_security_events(zone_id, &events).expect("replace events");

        let rules = vec![
            CachedWafRule {
                id: "rule-sqli".to_string(),
                zone_id: zone_id.to_string(),
                description: "Block SQL injection".to_string(),
                action: "block".to_string(),
                expression: "(http.request.uri.path contains \"/login\")".to_string(),
                paused: false,
                priority: 1,
                ref_code: "SQLI_1".to_string(),
            },
            CachedWafRule {
                id: "rule-bot".to_string(),
                zone_id: zone_id.to_string(),
                description: "Challenge bot".to_string(),
                action: "managed_challenge".to_string(),
                expression: "(cf.client.bot)".to_string(),
                paused: false,
                priority: 2,
                ref_code: "BOT_2".to_string(),
            },
        ];
        db.replace_waf_rules(zone_id, &rules).expect("replace rules");

        (db, zone_id.to_string())
    }

    #[test]
    fn test_security_view_initialization_and_tabs() {
        let (db, zone_id) = setup_test_db();
        let mut view = SecurityView::new(db);
        view.set_zone(&zone_id);

        assert_eq!(view.active_tab, SecurityTab::Events);
        assert_eq!(view.events.len(), 2);
        assert_eq!(view.rules.len(), 2);
        assert_eq!(view.analytics.total_events, 2);

        view.next_tab();
        assert_eq!(view.active_tab, SecurityTab::WafRules);

        view.next_tab();
        assert_eq!(view.active_tab, SecurityTab::Analytics);

        view.next_tab();
        assert_eq!(view.active_tab, SecurityTab::Events);

        view.prev_tab();
        assert_eq!(view.active_tab, SecurityTab::Analytics);
    }

    #[test]
    fn test_security_view_navigation() {
        let (db, zone_id) = setup_test_db();
        let mut view = SecurityView::new(db);
        view.set_zone(&zone_id);

        assert_eq!(view.selected_event_idx, 0);
        view.nav_down();
        assert_eq!(view.selected_event_idx, 1);
        view.nav_down();
        assert_eq!(view.selected_event_idx, 1);
        view.nav_up();
        assert_eq!(view.selected_event_idx, 0);

        view.active_tab = SecurityTab::WafRules;
        assert_eq!(view.selected_rule_idx, 0);
        view.nav_down();
        assert_eq!(view.selected_rule_idx, 1);
        view.nav_top();
        assert_eq!(view.selected_rule_idx, 0);
        view.nav_bottom();
        assert_eq!(view.selected_rule_idx, 1);
    }

    #[test]
    fn test_security_view_filtering() {
        let (db, zone_id) = setup_test_db();
        let mut view = SecurityView::new(db);
        view.set_zone(&zone_id);

        assert_eq!(view.filtered_events().len(), 2);

        view.cycle_action_filter();
        assert_eq!(view.action_filter.as_deref(), Some("block"));
        assert_eq!(view.filtered_events().len(), 1);
        assert_eq!(view.selected_event().unwrap().action, "block");

        view.action_filter = None;
        view.search_query = "198.51.100.42".to_string();
        assert_eq!(view.filtered_events().len(), 1);
        assert_eq!(view.selected_event().unwrap().ray_id, "8472a1bf001");
    }

    #[test]
    fn test_security_view_analytics_tab_rendering() {
        let (db, zone_id) = setup_test_db();
        let mut view = SecurityView::new(db);
        view.set_zone(&zone_id);
        view.active_tab = SecurityTab::Analytics;

        let backend = ratatui::backend::TestBackend::new(140, 45);
        let mut terminal = ratatui::Terminal::new(backend).expect("create test terminal");
        let theme = Theme::default();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &theme);
            })
            .expect("draw analytics tab");

        assert_eq!(view.analytics.total_events, 2);
        assert_eq!(view.analytics.sparkline.total_sparkline.len(), 2);
        assert_eq!(view.analytics.action_breakdown.len(), 2);
    }

    #[test]
    fn test_security_view_analytics_empty_state_rendering() {
        let db = CacheDb::open_in_memory().expect("open test db");
        let mut view = SecurityView::new(db);
        view.active_tab = SecurityTab::Analytics;

        let backend = ratatui::backend::TestBackend::new(120, 40);
        let mut terminal = ratatui::Terminal::new(backend).expect("create test terminal");
        let theme = Theme::default();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &theme);
            })
            .expect("draw empty analytics tab");

        assert_eq!(view.analytics.total_events, 0);
        assert!(view.analytics.top_countries.is_empty());
        assert!(view.analytics.top_asns.is_empty());
        assert!(view.analytics.top_paths.is_empty());
        assert!(view.analytics.rule_hits.is_empty());
    }
}
