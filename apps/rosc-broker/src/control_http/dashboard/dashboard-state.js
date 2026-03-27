export async function fetchDashboardData(limit) {
  const response = await fetchJson(`/dashboard/data?limit=${encodeURIComponent(String(limit))}`);
  return response.dashboard;
}

export async function postControlAction(path, method = "POST") {
  return fetchJson(path, { method });
}

export function globalActionRequest(action, destinationId) {
  if (action === "rehydrate-destination" && destinationId) {
    return {
      path: `/destinations/${encodeURIComponent(destinationId)}/rehydrate`,
      method: "POST",
      prompt: `Rehydrate destination ${destinationId}?`,
      successLabel: `Destination ${destinationId} rehydrated`,
      failureLabel: `Failed to rehydrate ${destinationId}`,
    };
  }

  const specs = {
    freeze: {
      path: "/freeze",
      method: "POST",
      prompt: "Freeze traffic now? This stops live dispatch until thaw.",
      successLabel: "Traffic frozen",
      failureLabel: "Failed to freeze traffic",
    },
    thaw: {
      path: "/thaw",
      method: "POST",
      prompt: "Thaw traffic and allow dispatch again?",
      successLabel: "Traffic thawed",
      failureLabel: "Failed to thaw traffic",
    },
    "restore-all": {
      path: "/routes/restore-all",
      method: "POST",
      prompt: "Restore every isolated route?",
      successLabel: "All isolated routes restored",
      failureLabel: "Failed to restore isolated routes",
    },
  };

  return specs[action] || null;
}

export function routeActionRequest(routeAction, routeId, dashboard) {
  if (!routeId || routeAction !== "toggle-isolation") {
    return null;
  }

  const isolated = dashboard?.snapshot?.overview?.status?.runtime?.isolated_route_ids?.includes(
    routeId,
  );
  if (isolated) {
    return {
      path: `/routes/${encodeURIComponent(routeId)}/restore`,
      method: "POST",
      prompt: `Restore route ${routeId}?`,
      successLabel: `Route ${routeId} restored`,
      failureLabel: `Failed to restore route ${routeId}`,
    };
  }

  return {
    path: `/routes/${encodeURIComponent(routeId)}/isolate`,
    method: "POST",
    prompt: `Isolate route ${routeId}?`,
    successLabel: `Route ${routeId} isolated`,
    failureLabel: `Failed to isolate route ${routeId}`,
  };
}

export function normalizeFocusState(dashboard, focusState = {}) {
  return {
    routeId: normalizeFocusId(
      focusState.routeId,
      dashboard.route_details,
      dashboard.snapshot.incidents.problematic_routes.map((route) => route.route_id),
      "route_id",
    ),
    destinationId: normalizeFocusId(
      focusState.destinationId,
      dashboard.destination_details,
      dashboard.snapshot.incidents.problematic_destinations.map(
        (destination) => destination.destination_id,
      ),
      "destination_id",
    ),
  };
}

export function buildTrafficPulse(traffic, previousSample, recordedAtUnixMs) {
  const sample = {
    recordedAtUnixMs,
    ingressPacketsTotal: traffic.ingress_packets_total || 0,
    ingressDropsTotal: traffic.ingress_drops_total || 0,
    routeMatchesTotal: traffic.route_matches_total || 0,
    destinationSendTotal: traffic.destination_send_total || 0,
    destinationSendFailuresTotal: traffic.destination_send_failures_total || 0,
    destinationDropsTotal: traffic.destination_drops_total || 0,
  };

  if (!previousSample || recordedAtUnixMs <= previousSample.recordedAtUnixMs) {
    return {
      sample,
      rates: null,
    };
  }

  const seconds = (recordedAtUnixMs - previousSample.recordedAtUnixMs) / 1000;
  return {
    sample,
    rates: {
      ingressPacketsPerSecond: rate(
        sample.ingressPacketsTotal,
        previousSample.ingressPacketsTotal,
        seconds,
      ),
      ingressDropsPerSecond: rate(
        sample.ingressDropsTotal,
        previousSample.ingressDropsTotal,
        seconds,
      ),
      routeMatchesPerSecond: rate(
        sample.routeMatchesTotal,
        previousSample.routeMatchesTotal,
        seconds,
      ),
      destinationSendPerSecond: rate(
        sample.destinationSendTotal,
        previousSample.destinationSendTotal,
        seconds,
      ),
      destinationFailuresPerSecond: rate(
        sample.destinationSendFailuresTotal + sample.destinationDropsTotal,
        previousSample.destinationSendFailuresTotal + previousSample.destinationDropsTotal,
        seconds,
      ),
    },
  };
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

function rate(current, previous, seconds) {
  if (seconds <= 0) {
    return 0;
  }
  return Math.max(0, (current - previous) / seconds);
}

function normalizeFocusId(currentId, details, preferredIds, key) {
  if (!Array.isArray(details) || details.length === 0) {
    return null;
  }
  if (currentId && details.some((detail) => detail[key] === currentId)) {
    return currentId;
  }
  for (const preferredId of preferredIds) {
    if (details.some((detail) => detail[key] === preferredId)) {
      return preferredId;
    }
  }
  return details[0][key];
}
