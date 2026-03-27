import {
  buildTrafficPulse,
  fetchDashboardData,
  globalActionRequest,
  normalizeFocusState,
  postControlAction,
  retryDelayMs,
  routeActionRequest,
} from "/dashboard/dashboard-state.js";
import {
  collectDashboardElements,
  renderConnectionError,
  renderDashboard,
  setActionFeedback,
  syncActiveSection,
} from "/dashboard/dashboard-render.js";

const HISTORY_LIMIT = 8;

const elements = collectDashboardElements();

let lastDashboard = null;
let lastTrafficSample = null;
let refreshTimer = null;
let refreshIntervalMs = 2500;
let lastRenderContext = null;
let refreshInFlight = null;
let connectionState = {
  connected: false,
  stale: false,
  retryAttempt: 0,
  nextRetryDelayMs: refreshIntervalMs,
  lastError: null,
  lastSuccessAt: null,
};
let focusState = {
  routeId: null,
  destinationId: null,
};

elements.refreshButton.addEventListener("click", () => refreshDashboard());
elements.routeFocusSelect.addEventListener("change", (event) => {
  focusState.routeId = event.target.value || null;
  if (lastDashboard) {
    renderCurrentDashboard();
  }
});
elements.destinationFocusSelect.addEventListener("change", (event) => {
  focusState.destinationId = event.target.value || null;
  if (lastDashboard) {
    renderCurrentDashboard();
  }
});
window.addEventListener("hashchange", () => syncActiveSection(elements));
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    refreshDashboard();
  }
});

document.body.addEventListener("click", async (event) => {
  const actionButton = event.target.closest("[data-action]");
  if (actionButton) {
    const request = globalActionRequest(
      actionButton.dataset.action,
      actionButton.dataset.destinationId,
    );
    await runAction(request);
    return;
  }

  const routeButton = event.target.closest("[data-route-action]");
  if (routeButton) {
    const request = routeActionRequest(
      routeButton.dataset.routeAction,
      routeButton.dataset.routeId,
      lastDashboard,
    );
    await runAction(request);
    return;
  }

  const focusRouteButton = event.target.closest("[data-focus-route-id]");
  if (focusRouteButton) {
    focusState.routeId = focusRouteButton.dataset.focusRouteId || null;
    window.location.hash = "#focus";
    renderCurrentDashboard();
    return;
  }

  const focusDestinationButton = event.target.closest("[data-focus-destination-id]");
  if (focusDestinationButton) {
    focusState.destinationId = focusDestinationButton.dataset.focusDestinationId || null;
    window.location.hash = "#focus";
    renderCurrentDashboard();
  }
});

syncActiveSection(elements);
refreshDashboard();

async function refreshDashboard() {
  if (refreshInFlight) {
    return refreshInFlight;
  }
  if (refreshTimer) {
    window.clearTimeout(refreshTimer);
    refreshTimer = null;
  }
  refreshInFlight = (async () => {
    try {
      const dashboard = await fetchDashboardData(HISTORY_LIMIT);
      const refreshedAt = new Date();
      const trafficPulse = buildTrafficPulse(
        dashboard.traffic,
        lastTrafficSample,
        refreshedAt.getTime(),
      );

      lastDashboard = dashboard;
      lastTrafficSample = trafficPulse.sample;
      refreshIntervalMs = dashboard.refresh_interval_ms || refreshIntervalMs;
      connectionState = {
        connected: true,
        stale: false,
        retryAttempt: 0,
        nextRetryDelayMs: refreshIntervalMs,
        lastError: null,
        lastSuccessAt: refreshedAt,
      };
      focusState = normalizeFocusState(dashboard, focusState);
      lastRenderContext = { refreshedAt, trafficPulse };
      renderCurrentDashboard();
      scheduleRefresh(refreshIntervalMs);
    } catch (error) {
      console.error(error);
      const retryAttempt = connectionState.retryAttempt + 1;
      const nextRetryDelayMs = retryDelayMs(refreshIntervalMs, retryAttempt);
      connectionState = {
        connected: false,
        stale: Boolean(lastDashboard),
        retryAttempt,
        nextRetryDelayMs,
        lastError: String(error.message || error),
        lastSuccessAt: connectionState.lastSuccessAt,
      };
      if (lastDashboard) {
        renderCurrentDashboard();
      } else {
        renderConnectionError(elements, error, connectionState);
      }
      scheduleRefresh(nextRetryDelayMs);
    } finally {
      refreshInFlight = null;
    }
  })();
  return refreshInFlight;
}

function renderCurrentDashboard() {
  if (!lastDashboard) {
    return;
  }
  const context = lastRenderContext || {
    refreshedAt: new Date(),
    trafficPulse: { sample: lastTrafficSample, rates: null },
  };
  renderDashboard(elements, lastDashboard, {
    ...context,
    connectionState,
    focusState,
  });
}

async function runAction(request) {
  if (!request) {
    return;
  }
  if (!window.confirm(request.prompt)) {
    return;
  }

  try {
    const response = await postControlAction(request.path, request.method);
    const details = [`applied=${response.applied}`];
    if (typeof response.dispatch_count === "number") {
      details.push(`dispatch_count=${response.dispatch_count}`);
    }
    setActionFeedback(
      elements,
      "healthy",
      `${request.successLabel} (${details.join(", ")})`,
    );
    await refreshDashboard();
  } catch (error) {
    console.error(error);
    setActionFeedback(
      elements,
      "blocked",
      `${request.failureLabel}: ${String(error.message || error)}`,
    );
  }
}

function scheduleRefresh(delayMs) {
  if (refreshTimer) {
    window.clearTimeout(refreshTimer);
  }
  const delay = typeof delayMs === "number" ? delayMs : refreshIntervalMs;
  refreshTimer = window.setTimeout(() => {
    refreshTimer = null;
    refreshDashboard();
  }, delay);
}
