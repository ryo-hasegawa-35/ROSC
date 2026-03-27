const REFRESH_INTERVAL_MS = 2500;

const elements = {
  connectionStatus: document.getElementById("connection-status"),
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
  routesTable: document.getElementById("routes-table"),
  destinationsTable: document.getElementById("destinations-table"),
  overridesList: document.getElementById("overrides-list"),
  operatorHistory: document.getElementById("operator-history"),
  configHistory: document.getElementById("config-history"),
  problematicEntities: document.getElementById("problematic-entities"),
  refreshButton: document.getElementById("refresh-button"),
  sectionLinks: [...document.querySelectorAll("[data-section-link]")],
  emptyStateTemplate: document.getElementById("empty-state-template"),
};

let lastRefreshAt = null;
let lastSnapshot = null;
let refreshTimer = null;

elements.refreshButton.addEventListener("click", () => refreshDashboard());
window.addEventListener("hashchange", syncActiveSection);
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    refreshDashboard();
  }
});

document.body.addEventListener("click", async (event) => {
  const actionButton = event.target.closest("[data-action]");
  if (actionButton) {
    await handleGlobalAction(actionButton.dataset.action, actionButton.dataset.destinationId);
    return;
  }

  const routeButton = event.target.closest("[data-route-action]");
  if (routeButton) {
    await handleRouteAction(routeButton.dataset.routeAction, routeButton.dataset.routeId);
    return;
  }
});

syncActiveSection();
refreshDashboard();
refreshTimer = window.setInterval(refreshDashboard, REFRESH_INTERVAL_MS);

async function refreshDashboard() {
  try {
    const [snapshotResponse, statusResponse] = await Promise.all([
      fetchJson("/snapshot?limit=8"),
      fetchJson("/status"),
    ]);
    lastSnapshot = snapshotResponse.snapshot;
    renderDashboard(snapshotResponse.snapshot, statusResponse.status);
    setConnectionState("Connected", "healthy");
  } catch (error) {
    console.error(error);
    setConnectionState("Disconnected", "blocked");
    elements.heroTitle.textContent = "Control plane unavailable";
    elements.heroSubtitle.textContent = String(error.message || error);
  }
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    cache: "no-store",
    ...options,
  });
  const data = await response.json();
  if (!response.ok || data.ok === false) {
    throw new Error(data.error || `${response.status} ${response.statusText}`);
  }
  return data;
}

function renderDashboard(snapshot, status) {
  const { overview, readiness, diagnostics, attention, incidents } = snapshot;
  lastRefreshAt = new Date();

  const stateText = humanizeState(overview.report.state);
  elements.heroTitle.textContent = `${stateText} broker state`;
  elements.heroSubtitle.textContent = describeHero(readiness, attention);
  applyStateBadge(elements.heroState, overview.report.state, stateText);
  applyStateBadge(elements.overviewState, overview.report.state, stateText);
  elements.heroRefresh.textContent = lastRefreshAt.toLocaleTimeString("ja-JP");
  elements.heroRuntime.textContent = diagnostics.overview.runtime_summary.has_runtime_status
    ? "live"
    : "static";

  renderOverviewStats(snapshot, status);
  renderAttention(attention, incidents);
  renderReadiness(readiness);
  renderTopology(status);
  renderRoutes(status, diagnostics);
  renderDestinations(status);
  renderRecovery(snapshot);
  renderIncidents(incidents);
}

function renderOverviewStats(snapshot, status) {
  const metrics = [
    ["Routes", status.summary.active_routes, "active"],
    ["Destinations", status.summary.active_destinations, "active"],
    ["Fallback missing", status.summary.fallback_missing_routes, "needs action"],
    ["Problematic routes", snapshot.readiness.counts.problematic_routes, "runtime"],
    ["Problematic destinations", snapshot.readiness.counts.problematic_destinations, "runtime"],
    ["Recent operator actions", snapshot.diagnostics.recent_operator_actions.length, "history"],
  ];

  elements.overviewStats.replaceChildren(
    ...metrics.map(([label, value, context]) => {
      const card = document.createElement("dl");
      card.className = "stat-card";
      card.innerHTML = `<dt>${escapeHtml(label)}</dt><dd>${escapeHtml(String(value))}</dd><div class="token-list"><span class="token">${escapeHtml(context)}</span></div>`;
      return card;
    }),
  );
}

function renderAttention(attention, incidents) {
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
  fillList(elements.attentionList, items);
}

function renderReadiness(readiness) {
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
      <span class="token">fallback ready=${escapeHtml(String(readiness.flags.fallback_ready))}</span>
      <span class="token">runtime visible=${escapeHtml(String(readiness.flags.runtime_visibility_ready))}</span>
    </div>
  `;
  const reasonList = document.createElement("ul");
  reasonList.className = "callout-list";
  reasonList.replaceChildren(...reasons.map((reason) => listItem(reason)));
  wrapper.appendChild(reasonList);
  elements.readinessBlock.replaceChildren(wrapper);
}

function renderTopology(status) {
  fillEntityColumn(
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
    elements.topologyRoutes,
    status.routes.map((route) => ({
      title: route.id,
      state: route.enabled ? "healthy" : "warning",
      body: [
        `mode: ${route.mode}`,
        `class: ${route.traffic_class}`,
        `ingresses: ${route.ingress_ids.join(", ") || "*"}`,
        `destinations: ${route.destination_ids.join(", ") || "none"}`,
      ],
    })),
  );

  fillEntityColumn(
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

function renderRoutes(status, diagnostics) {
  const isolatedRoutes = new Set(status.runtime?.isolated_route_ids || []);
  const runtimeRoutes = new Map(
    Object.entries(diagnostics.overview.report.runtime_signals.routes_with_dispatch_failures || {}),
  );
  const transformFailures = new Set(
    diagnostics.overview.report.runtime_signals.routes_with_transform_failures || [],
  );

  elements.routesTable.replaceChildren(
    ...status.route_assessments.map((assessment) => {
      const route = status.routes.find((item) => item.id === assessment.route_id);
      const row = document.createElement("tr");
      const warnings = [...assessment.warnings];
      const dispatchFailures = runtimeRoutes.get(assessment.route_id);
      if (dispatchFailures) {
        warnings.push(`dispatch failures=${dispatchFailures}`);
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

function renderDestinations(status) {
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

function renderRecovery(snapshot) {
  const overrides = snapshot.overview.report.overrides;
  const overrideItems = [
    `launch profile: ${overrides.launch_profile_mode}`,
    `traffic frozen: ${overrides.traffic_frozen}`,
    `isolated routes: ${overrides.isolated_route_ids.join(", ") || "none"}`,
    `disabled capture routes: ${overrides.disabled_capture_routes.join(", ") || "none"}`,
    `disabled replay routes: ${overrides.disabled_replay_routes.join(", ") || "none"}`,
    `disabled restart rehydrate routes: ${overrides.disabled_restart_rehydrate_routes.join(", ") || "none"}`,
  ];
  fillList(elements.overridesList, overrideItems);

  fillTimeline(
    elements.operatorHistory,
    snapshot.diagnostics.recent_operator_actions.map((action) => ({
      title: action.action,
      details: action.details,
      timestamp: action.at,
    })),
  );
}

function renderIncidents(incidents) {
  fillTimeline(
    elements.configHistory,
    incidents.recent_config_issues.map((event) => ({
      title: event.kind,
      details: [
        `revision=${event.revision ?? "-"}`,
        ...(event.reason ? [event.reason] : []),
        ...(event.launch_profile_mode ? [`launch_profile=${event.launch_profile_mode}`] : []),
      ],
      timestamp: event.at,
    })),
  );

  const problematic = [
    ...incidents.problematic_routes.map((route) => `Route: ${route.route_id}`),
    ...incidents.problematic_destinations.map(
      (destination) => `Destination: ${destination.destination_id}`,
    ),
  ];
  fillList(elements.problematicEntities, problematic);
}

async function handleGlobalAction(action, destinationId) {
  const specs = {
    freeze: {
      path: "/freeze",
      method: "POST",
      prompt: "Freeze traffic now? This stops live dispatch until thaw.",
    },
    thaw: {
      path: "/thaw",
      method: "POST",
      prompt: "Thaw traffic and allow dispatch again?",
    },
    "restore-all": {
      path: "/routes/restore-all",
      method: "POST",
      prompt: "Restore every isolated route?",
    },
  };
  const spec =
    action === "rehydrate-destination" && destinationId
      ? {
          path: `/destinations/${encodeURIComponent(destinationId)}/rehydrate`,
          method: "POST",
          prompt: `Rehydrate destination ${destinationId}?`,
        }
      : specs[action];
  if (!spec) {
    return;
  }
  if (!window.confirm(spec.prompt)) {
    return;
  }
  await fetchJson(spec.path, { method: spec.method });
  await refreshDashboard();
}

async function handleRouteAction(action, routeId) {
  if (!routeId) {
    return;
  }
  if (action !== "toggle-isolation") {
    return;
  }
  const isolated = lastSnapshot?.overview?.status?.runtime?.isolated_route_ids?.includes(routeId);
  const spec = isolated
    ? {
        path: `/routes/${encodeURIComponent(routeId)}/restore`,
        prompt: `Restore route ${routeId}?`,
      }
    : {
        path: `/routes/${encodeURIComponent(routeId)}/isolate`,
        prompt: `Isolate route ${routeId}?`,
      };
  if (!window.confirm(spec.prompt)) {
    return;
  }
  await fetchJson(spec.path, { method: "POST" });
  await refreshDashboard();
}

function fillEntityColumn(container, items) {
  container.replaceChildren(
    ...emptyAware(items.map((item) => {
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
    })),
  );
}

function fillList(container, items) {
  container.replaceChildren(...emptyAware(items.map((item) => listItem(item))));
}

function fillTimeline(container, entries) {
  container.replaceChildren(
    ...emptyAware(
      entries.map((entry) => {
        const item = document.createElement("li");
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

function emptyAware(items) {
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

function setConnectionState(label, level) {
  elements.connectionStatus.textContent = label;
  elements.connectionStatus.dataset.level = level;
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

function syncActiveSection() {
  const current = window.location.hash.replace("#", "") || "overview";
  for (const link of elements.sectionLinks) {
    link.classList.toggle("is-active", link.dataset.sectionLink === current);
  }
}

function formatTimestamp(value) {
  if (!value) {
    return "time unavailable";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString("ja-JP");
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
