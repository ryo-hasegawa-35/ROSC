export function collectDashboardElements() {
  return {
    connectionStatus: document.getElementById("connection-status"),
    actionFeedback: document.getElementById("action-feedback"),
    heroTitle: document.getElementById("hero-title"),
    heroSubtitle: document.getElementById("hero-subtitle"),
    heroState: document.getElementById("hero-state"),
    heroRefresh: document.getElementById("hero-refresh"),
    heroRuntime: document.getElementById("hero-runtime"),
    overviewState: document.getElementById("overview-state"),
    overviewStats: document.getElementById("overview-stats"),
    attentionList: document.getElementById("attention-list"),
    readinessBlock: document.getElementById("readiness-block"),
    topologyIngresses: document.getElementById("topology-ingresses"),
    topologyRoutes: document.getElementById("topology-routes"),
    topologyDestinations: document.getElementById("topology-destinations"),
    trafficStats: document.getElementById("traffic-stats"),
    trafficHotspots: document.getElementById("traffic-hotspots"),
    routesTable: document.getElementById("routes-table"),
    destinationsTable: document.getElementById("destinations-table"),
    overridesList: document.getElementById("overrides-list"),
    operatorHistory: document.getElementById("operator-history"),
    configSummary: document.getElementById("config-summary"),
    configRisks: document.getElementById("config-risks"),
    configHistory: document.getElementById("config-history"),
    eventTimeline: document.getElementById("event-timeline"),
    problematicEntities: document.getElementById("problematic-entities"),
    refreshButton: document.getElementById("refresh-button"),
    sectionLinks: [...document.querySelectorAll("[data-section-link]")],
    emptyStateTemplate: document.getElementById("empty-state-template"),
  };
}

export function renderDashboard(elements, dashboard, context) {
  const snapshot = dashboard.snapshot;
  const status = snapshot.overview.status;
  const { overview, readiness, diagnostics, attention, incidents } = snapshot;

  const stateText = humanizeState(overview.report.state);
  elements.heroTitle.textContent = `${stateText} broker state`;
  elements.heroSubtitle.textContent = describeHero(readiness, attention);
  applyStateBadge(elements.connectionStatus, "healthy", "Connected");
  applyStateBadge(elements.heroState, overview.report.state, stateText);
  applyStateBadge(elements.overviewState, overview.report.state, stateText);
  elements.heroRefresh.textContent = context.refreshedAt.toLocaleTimeString("ja-JP");
  elements.heroRuntime.textContent = diagnostics.overview.runtime_summary.has_runtime_status
    ? "live"
    : "static";

  renderOverviewStats(elements, snapshot, status, dashboard.traffic);
  renderAttention(elements, attention, incidents);
  renderReadiness(elements, readiness);
  renderTopology(elements, status);
  renderTraffic(elements, dashboard.traffic, context.trafficPulse, snapshot);
  renderRoutes(elements, status, diagnostics);
  renderDestinations(elements, status);
  renderRecovery(elements, snapshot);
  renderIncidents(elements, incidents);
  renderConfig(elements, dashboard, snapshot);
}

export function renderConnectionError(elements, error) {
  applyStateBadge(elements.connectionStatus, "blocked", "Disconnected");
  applyStateBadge(elements.heroState, "blocked", "Unavailable");
  applyStateBadge(elements.overviewState, "blocked", "Unavailable");
  elements.heroTitle.textContent = "Control plane unavailable";
  elements.heroSubtitle.textContent = String(error.message || error);
  elements.heroRefresh.textContent = "--";
  elements.heroRuntime.textContent = "--";
}

export function setActionFeedback(elements, level, message) {
  elements.actionFeedback.dataset.level = level;
  elements.actionFeedback.textContent = message;
}

export function syncActiveSection(elements) {
  const current = window.location.hash.replace("#", "") || "overview";
  for (const link of elements.sectionLinks) {
    link.classList.toggle("is-active", link.dataset.sectionLink === current);
  }
}

function renderOverviewStats(elements, snapshot, status, traffic) {
  const metrics = [
    ["Routes", status.summary.active_routes, "active"],
    ["Destinations", status.summary.active_destinations, "active"],
    ["Fallback missing", status.summary.fallback_missing_routes, "needs action"],
    ["Queue backlog", traffic.destination_queue_depth_total, "runtime"],
    ["Open breakers", traffic.destinations_with_open_breakers, "runtime"],
    ["Recent operator actions", snapshot.diagnostics.recent_operator_actions.length, "history"],
  ];

  elements.overviewStats.replaceChildren(
    ...metrics.map(([label, value, context]) => statCard(label, value, context)),
  );
}

function renderAttention(elements, attention, incidents) {
  const items = [];
  if (attention.override_summary.length > 0) {
    items.push(`Overrides: ${attention.override_summary.join(", ")}`);
  }
  if (attention.latest_operator_action) {
    items.push(`Latest operator action: ${attention.latest_operator_action.action}`);
  }
  if (attention.latest_config_issue) {
    items.push(`Latest config issue: ${attention.latest_config_issue.kind}`);
  }
  for (const route of incidents.problematic_routes.slice(0, 2)) {
    items.push(`Problematic route: ${route.route_id}`);
  }
  for (const destination of incidents.problematic_destinations.slice(0, 2)) {
    items.push(`Problematic destination: ${destination.destination_id}`);
  }
  fillList(elements, elements.attentionList, items);
}

function renderReadiness(elements, readiness) {
  const reasons = readiness.reasons.length > 0 ? readiness.reasons : ["No active blockers"];
  const wrapper = document.createElement("div");
  wrapper.className = "readiness-card";

  const level = humanizeState(readiness.level);
  wrapper.innerHTML = `
    <div class="panel-header">
      <div>
        <p class="panel-label">Current gate</p>
        <h4>${escapeHtml(level)}</h4>
      </div>
      <span class="entity-state" data-level="${escapeHtml(readiness.level)}">${escapeHtml(level)}</span>
    </div>
    <div class="token-list">
      <span class="token">traffic ready=${escapeHtml(String(readiness.flags.traffic_flow_ready))}</span>
      <span class="token">fallback ready=${escapeHtml(String(readiness.flags.fallback_complete))}</span>
      <span class="token">control ready=${escapeHtml(String(readiness.flags.control_plane_ready))}</span>
    </div>
  `;
  const reasonList = document.createElement("ul");
  reasonList.className = "callout-list";
  reasonList.replaceChildren(...reasons.map((reason) => listItem(reason)));
  wrapper.appendChild(reasonList);
  elements.readinessBlock.replaceChildren(wrapper);
}

function renderTopology(elements, status) {
  fillEntityColumn(
    elements,
    elements.topologyIngresses,
    status.ingresses.map((ingress) => ({
      title: ingress.id,
      state: ingress.route_ids.length > 0 ? "healthy" : "warning",
      body: [
        `bind: ${ingress.bound_local_addr || ingress.configured_bind}`,
        `routes: ${ingress.route_ids.join(", ") || "none"}`,
      ],
    })),
  );

  fillEntityColumn(
    elements,
    elements.topologyRoutes,
    status.routes.map((route) => ({
      title: route.id,
      state: route.enabled ? "healthy" : "warning",
      body: [
        `mode: ${route.mode}`,
        `class: ${route.traffic_class}`,
        `cache: ${route.cache_policy}`,
        `capture: ${route.capture_policy}`,
      ],
    })),
  );

  fillEntityColumn(
    elements,
    elements.topologyDestinations,
    status.destinations.map((destination) => ({
      title: destination.id,
      state: destination.route_ids.length > 0 ? "healthy" : "warning",
      body: [
        `bind: ${destination.bind}`,
        `target: ${destination.target}`,
        `routes: ${destination.route_ids.join(", ") || "none"}`,
      ],
    })),
  );
}

function renderTraffic(elements, traffic, trafficPulse, snapshot) {
  const rates = trafficPulse.rates;
  const metrics = [
    ["Ingress packet/s", formatRate(rates?.ingressPacketsPerSecond), `total=${traffic.ingress_packets_total}`],
    ["Route match/s", formatRate(rates?.routeMatchesPerSecond), `total=${traffic.route_matches_total}`],
    ["Egress send/s", formatRate(rates?.destinationSendPerSecond), `total=${traffic.destination_send_total}`],
    ["Failure/s", formatRate(rates?.destinationFailuresPerSecond), `send_failures=${traffic.destination_send_failures_total} drops=${traffic.destination_drops_total}`],
    ["Queue backlog", traffic.destination_queue_depth_total, `${traffic.destinations_with_backlog} destinations`],
    ["Open breakers", traffic.destinations_with_open_breakers, `half_open=${traffic.destinations_with_half_open_breakers}`],
  ];

  elements.trafficStats.replaceChildren(
    ...metrics.map(([label, value, context]) => statCard(label, value, context)),
  );

  const hotspots = [
    ...traffic.busiest_ingresses.map((entry) => `Ingress ${entry.id}: packets=${entry.total}`),
    ...traffic.busiest_routes.map((entry) => `Route ${entry.id}: matches=${entry.total}`),
    ...traffic.noisiest_destinations.map((entry) => `Destination ${entry.id}: failures+drops=${entry.total}`),
  ];
  const signals = snapshot.overview.report.runtime_signals;
  if (signals.ingresses_with_drops.length > 0) {
    hotspots.push(`Ingresses with drops: ${signals.ingresses_with_drops.join(", ")}`);
  }
  if (signals.routes_with_transform_failures.length > 0) {
    hotspots.push(`Routes with transform failures: ${signals.routes_with_transform_failures.join(", ")}`);
  }
  fillList(elements, elements.trafficHotspots, hotspots);
}

function renderRoutes(elements, status, diagnostics) {
  const isolatedRoutes = new Set(status.runtime?.isolated_route_ids || []);
  const runtimeRoutes = new Set(
    diagnostics.overview.report.runtime_signals.routes_with_dispatch_failures || [],
  );
  const transformFailures = new Set(
    diagnostics.overview.report.runtime_signals.routes_with_transform_failures || [],
  );

  elements.routesTable.replaceChildren(
    ...status.route_assessments.map((assessment) => {
      const route = status.routes.find((item) => item.id === assessment.route_id);
      const row = document.createElement("tr");
      const warnings = [...assessment.warnings];
      if (runtimeRoutes.has(assessment.route_id)) {
        warnings.push("dispatch failures observed");
      }
      if (transformFailures.has(assessment.route_id)) {
        warnings.push("transform failures observed");
      }
      row.innerHTML = `
        <td><code>${escapeHtml(assessment.route_id)}</code></td>
        <td><span class="entity-state" data-level="${escapeHtml(routeStateLevel(assessment, warnings))}">${escapeHtml(routeStateLabel(assessment, warnings))}</span></td>
        <td>${escapeHtml(route?.mode || "--")}</td>
        <td>${escapeHtml(route?.traffic_class || "--")}</td>
        <td>${escapeHtml(warnings.join(" | ") || "none")}</td>
        <td>
          <button class="mini-button" data-route-action="toggle-isolation" data-route-id="${escapeHtml(assessment.route_id)}">
            ${isolatedRoutes.has(assessment.route_id) ? "Restore" : "Isolate"}
          </button>
        </td>
      `;
      return row;
    }),
  );
}

function renderDestinations(elements, status) {
  const runtimeDestinations = new Map(
    (status.runtime?.destinations || []).map((destination) => [destination.destination_id, destination]),
  );

  elements.destinationsTable.replaceChildren(
    ...status.destinations.map((destination) => {
      const runtime = runtimeDestinations.get(destination.id);
      const breaker = runtime?.breaker_state || "closed";
      const row = document.createElement("tr");
      row.innerHTML = `
        <td><code>${escapeHtml(destination.id)}</code></td>
        <td><code>${escapeHtml(destination.target)}</code></td>
        <td>${escapeHtml(String(runtime?.queue_depth ?? destination.queue_depth))}</td>
        <td><span class="entity-state" data-level="${escapeHtml(breakerLevel(breaker))}">${escapeHtml(String(breaker))}</span></td>
        <td>${escapeHtml(String(runtime?.send_failures_total ?? 0))}</td>
        <td>
          <button class="mini-button" data-action="rehydrate-destination" data-destination-id="${escapeHtml(destination.id)}">Rehydrate</button>
        </td>
      `;
      return row;
    }),
  );
}

function renderRecovery(elements, snapshot) {
  const overrides = snapshot.overview.report.overrides;
  const overrideItems = [
    `launch profile: ${overrides.launch_profile_mode}`,
    `traffic frozen: ${overrides.traffic_frozen}`,
    `isolated routes: ${overrides.isolated_route_ids.join(", ") || "none"}`,
    `disabled capture routes: ${overrides.disabled_capture_routes.join(", ") || "none"}`,
    `disabled replay routes: ${overrides.disabled_replay_routes.join(", ") || "none"}`,
    `disabled restart rehydrate routes: ${overrides.disabled_restart_rehydrate_routes.join(", ") || "none"}`,
  ];
  fillList(elements, elements.overridesList, overrideItems);

  fillTimeline(
    elements,
    elements.operatorHistory,
    snapshot.diagnostics.recent_operator_actions.map((action) => ({
      title: action.action,
      details: action.details,
      timestamp: action.recorded_at_unix_ms,
      level: "healthy",
    })),
  );
}

function renderIncidents(elements, incidents) {
  fillTimeline(
    elements,
    elements.configHistory,
    incidents.recent_config_issues.map((event) => ({
      title: String(event.kind),
      details: [
        `revision=${event.revision ?? "-"}`,
        ...event.details,
        ...(event.launch_profile_mode ? [`launch_profile=${event.launch_profile_mode}`] : []),
      ],
      timestamp: event.recorded_at_unix_ms,
      level: "degraded",
    })),
  );

  const problematic = [
    ...incidents.problematic_routes.map((route) => `Route: ${route.route_id}`),
    ...incidents.problematic_destinations.map(
      (destination) => `Destination: ${destination.destination_id}`,
    ),
  ];
  fillList(elements, elements.problematicEntities, problematic);
}

function renderConfig(elements, dashboard, snapshot) {
  const runtime = snapshot.overview.status.runtime;
  const summary = [
    `config revision: ${runtime?.config_revision ?? "n/a"}`,
    `config rejections: ${runtime?.config_rejections_total ?? 0}`,
    `config blocked: ${runtime?.config_blocked_total ?? 0}`,
    `reload failures: ${runtime?.config_reload_failures_total ?? 0}`,
    `launch profile: ${snapshot.readiness.launch_profile_mode}`,
    `refresh interval: ${dashboard.refresh_interval_ms} ms`,
  ];
  fillList(elements, elements.configSummary, summary);

  const risks = [
    ...snapshot.readiness.blockers.map((reason) => ({ level: "blocked", text: reason })),
    ...snapshot.readiness.warnings.map((reason) => ({ level: "degraded", text: reason })),
  ];
  if (risks.length === 0) {
    risks.push({ level: "healthy", text: "No active config blockers or warnings." });
  }
  fillRiskList(elements, elements.configRisks, risks);

  fillTimeline(
    elements,
    elements.eventTimeline,
    dashboard.timeline.map((entry) => ({
      title: `${entry.category}: ${entry.label}`,
      details: entry.details,
      timestamp: entry.recorded_at_unix_ms,
      level: entry.category === "config_event" ? "degraded" : "healthy",
    })),
  );
}

function fillEntityColumn(elements, container, items) {
  container.replaceChildren(
    ...emptyAware(
      elements,
      items.map((item) => {
        const card = document.createElement("article");
        card.className = "entity-card";
        card.innerHTML = `
          <div class="panel-header">
            <h4>${escapeHtml(item.title)}</h4>
            <span class="entity-state" data-level="${escapeHtml(item.state)}">${escapeHtml(humanizeState(item.state))}</span>
          </div>
          ${item.body.map((line) => `<p>${escapeHtml(line)}</p>`).join("")}
        `;
        return card;
      }),
    ),
  );
}

function fillList(elements, container, items) {
  container.replaceChildren(...emptyAware(elements, items.map((item) => listItem(item))));
}

function fillRiskList(elements, container, items) {
  container.replaceChildren(
    ...emptyAware(
      elements,
      items.map((item) => {
        const entry = listItem(item.text);
        entry.dataset.level = item.level;
        return entry;
      }),
    ),
  );
}

function fillTimeline(elements, container, entries) {
  container.replaceChildren(
    ...emptyAware(
      elements,
      entries.map((entry) => {
        const item = document.createElement("li");
        if (entry.level) {
          item.dataset.level = entry.level;
        }
        item.innerHTML = `
          <strong>${escapeHtml(entry.title)}</strong>
          <div class="token-list">${entry.details.map((detail) => `<span class="token">${escapeHtml(detail)}</span>`).join("")}</div>
          <p>${escapeHtml(formatTimestamp(entry.timestamp))}</p>
        `;
        return item;
      }),
    ),
  );
}

function emptyAware(elements, items) {
  if (items.length > 0) {
    return items;
  }
  return [elements.emptyStateTemplate.content.firstElementChild.cloneNode(true)];
}

function listItem(content) {
  const item = document.createElement("li");
  item.textContent = content;
  return item;
}

function statCard(label, value, context) {
  const card = document.createElement("dl");
  card.className = "stat-card";
  card.innerHTML = `<dt>${escapeHtml(label)}</dt><dd>${escapeHtml(String(value))}</dd><div class="token-list"><span class="token">${escapeHtml(String(context))}</span></div>`;
  return card;
}

function describeHero(readiness, attention) {
  if (readiness.reasons.length > 0) {
    return readiness.reasons[0];
  }
  if (attention.override_summary.length > 0) {
    return `Active overrides: ${attention.override_summary.join(", ")}`;
  }
  return "No active readiness blockers or operator overrides.";
}

function routeStateLevel(assessment, warnings) {
  if (!assessment.active) {
    return "warning";
  }
  return warnings.length > 0 ? "degraded" : "healthy";
}

function routeStateLabel(assessment, warnings) {
  if (!assessment.active) {
    return "Disabled";
  }
  return warnings.length > 0 ? "Needs attention" : "Healthy";
}

function breakerLevel(state) {
  if (state === "Open") {
    return "blocked";
  }
  if (state === "HalfOpen") {
    return "degraded";
  }
  return "healthy";
}

function applyStateBadge(element, level, label) {
  element.dataset.level = String(level).toLowerCase();
  element.textContent = label;
}

function humanizeState(value) {
  const normalized = String(value || "").toLowerCase();
  switch (normalized) {
    case "healthy":
      return "Healthy";
    case "pressured":
      return "Pressured";
    case "degraded":
    case "warning":
      return "Degraded";
    case "emergency":
    case "blocked":
      return "Emergency";
    case "safemode":
    case "safe_mode":
      return "SafeMode";
    default:
      return value || "Unknown";
  }
}

function formatTimestamp(value) {
  if (value === null || value === undefined) {
    return "time unavailable";
  }
  const date = new Date(Number(value));
  if (Number.isNaN(date.getTime())) {
    return String(value);
  }
  return date.toLocaleString("ja-JP");
}

function formatRate(value) {
  if (typeof value !== "number") {
    return "--";
  }
  return `${value.toFixed(1)}/s`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
