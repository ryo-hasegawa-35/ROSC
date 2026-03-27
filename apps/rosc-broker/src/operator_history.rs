use rosc_telemetry::{RecentConfigEvent, RecentConfigEventKind};

pub fn bounded_recent_entries<T>(entries: Vec<T>, limit: Option<usize>) -> Vec<T> {
    match limit {
        Some(limit) if entries.len() > limit => {
            let start = entries.len() - limit;
            entries.into_iter().skip(start).collect()
        }
        _ => entries,
    }
}

pub fn bounded_recent_config_issues(
    events: Vec<RecentConfigEvent>,
    limit: Option<usize>,
) -> Vec<RecentConfigEvent> {
    bounded_recent_entries(
        events
            .into_iter()
            .filter(|event| {
                !matches!(
                    event.kind,
                    RecentConfigEventKind::Applied | RecentConfigEventKind::LaunchProfileChanged
                )
            })
            .collect(),
        limit,
    )
}
