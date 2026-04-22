use std::path::Path;

use anyhow::{Result, bail};

use super::super::ProxyCommandOptions;
use super::super::common::{
    launch_profile_mode, load_config, print_json_pretty, safety_policy, status_from_config,
};

pub(crate) async fn check_config(path: &Path) -> Result<()> {
    let config = load_config(path)?;
    println!(
        "valid config: schema_version={} route(s)={}",
        config.schema_version,
        config.routes.len()
    );
    Ok(())
}

pub(crate) async fn proxy_status(
    path: &Path,
    resolve_bindings: bool,
    safe_mode: bool,
) -> Result<()> {
    let config = load_config(path)?;
    let status = status_from_config(
        &config,
        resolve_bindings,
        launch_profile_mode(ProxyCommandOptions {
            fail_on_warnings: false,
            require_fallback_ready: false,
            safe_mode,
            start_frozen: false,
        }),
    )
    .await?;
    print_json_pretty(&status)?;
    Ok(())
}

pub(crate) async fn proxy_overview(
    path: &Path,
    resolve_bindings: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let overview = rosc_broker::proxy_operator_overview(&status, safety_policy(options));
    print_json_pretty(&overview)?;
    Ok(())
}

pub(crate) async fn proxy_readiness(
    path: &Path,
    resolve_bindings: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let readiness = rosc_broker::proxy_operator_readiness(&status, safety_policy(options));
    print_json_pretty(&readiness)?;
    Ok(())
}

pub(crate) async fn proxy_snapshot(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    print_json_pretty(&snapshot)?;
    Ok(())
}

pub(crate) async fn proxy_assert_ready(
    path: &Path,
    resolve_bindings: bool,
    allow_degraded: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let readiness = rosc_broker::proxy_operator_readiness(&status, safety_policy(options));
    let acceptable = readiness.is_acceptable(allow_degraded);
    print_json_pretty(&readiness)?;
    if acceptable {
        Ok(())
    } else {
        bail!(
            "proxy readiness gate failed: level={} allow_degraded={}",
            serde_json::to_string(&readiness.level)?,
            allow_degraded
        )
    }
}

pub(crate) async fn proxy_diagnostics(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let diagnostics =
        rosc_broker::proxy_operator_diagnostics(&status, safety_policy(options), history_limit);
    print_json_pretty(&diagnostics)?;
    Ok(())
}

pub(crate) async fn proxy_attention(
    path: &Path,
    resolve_bindings: bool,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let report = rosc_broker::proxy_operator_report(&status, safety_policy(options));
    let attention = rosc_broker::proxy_operator_attention(&report);
    print_json_pretty(&attention)?;
    Ok(())
}

pub(crate) async fn proxy_incidents(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let report = rosc_broker::proxy_operator_report(&status, safety_policy(options));
    let (recent_operator_actions, recent_config_events) = status
        .runtime
        .as_ref()
        .map(|runtime| {
            (
                runtime.recent_operator_actions.clone(),
                runtime.recent_config_events.clone(),
            )
        })
        .unwrap_or_default();
    let incidents = rosc_broker::proxy_operator_incidents_from_histories(
        &report,
        recent_operator_actions,
        recent_config_events,
        history_limit,
    );
    print_json_pretty(&incidents)?;
    Ok(())
}

pub(crate) async fn proxy_handoff(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    options: ProxyCommandOptions,
) -> Result<()> {
    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    print_json_pretty(&snapshot.handoff)?;
    Ok(())
}

pub(crate) async fn proxy_timeline(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-timeline accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    let mut timeline = rosc_broker::proxy_operator_timeline(&snapshot);

    if let Some(route_id) = route_id {
        let Some(route_timeline) = timeline
            .routes
            .iter()
            .find(|timeline| timeline.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route timeline `{route_id}`");
        };
        timeline.routes = vec![route_timeline];
        timeline.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_timeline) = timeline
            .destinations
            .iter()
            .find(|timeline| timeline.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination timeline `{destination_id}`");
        };
        timeline.routes.clear();
        timeline.destinations = vec![destination_timeline];
    }

    print_json_pretty(&timeline)?;
    Ok(())
}

pub(crate) async fn proxy_triage(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-triage accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    let mut triage = snapshot.triage;

    if let Some(route_id) = route_id {
        let Some(route_triage) = triage
            .route_triage
            .iter()
            .find(|triage| triage.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route triage `{route_id}`");
        };
        triage.route_triage = vec![route_triage];
        triage.destination_triage.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_triage) = triage
            .destination_triage
            .iter()
            .find(|triage| triage.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination triage `{destination_id}`");
        };
        triage.route_triage.clear();
        triage.destination_triage = vec![destination_triage];
    }

    print_json_pretty(&triage)?;
    Ok(())
}

pub(crate) async fn proxy_casebook(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-casebook accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    let mut casebook = snapshot.casebook;

    if let Some(route_id) = route_id {
        let Some(route_casebook) = casebook
            .route_casebooks
            .iter()
            .find(|casebook| casebook.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route casebook `{route_id}`");
        };
        casebook.route_casebooks = vec![route_casebook];
        casebook.destination_casebooks.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_casebook) = casebook
            .destination_casebooks
            .iter()
            .find(|casebook| casebook.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination casebook `{destination_id}`");
        };
        casebook.route_casebooks.clear();
        casebook.destination_casebooks = vec![destination_casebook];
    }

    print_json_pretty(&casebook)?;
    Ok(())
}

pub(crate) async fn proxy_board(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-board accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let snapshot =
        rosc_broker::proxy_operator_snapshot(&status, safety_policy(options), history_limit);
    let mut board = snapshot.board;

    if let Some(route_id) = route_id {
        board.blocked_items.retain(|item| {
            item.scope == rosc_broker::ProxyOperatorBoardScope::Global
                || item.route_id.as_deref() == Some(route_id)
        });
        board.degraded_items.retain(|item| {
            item.scope == rosc_broker::ProxyOperatorBoardScope::Global
                || item.route_id.as_deref() == Some(route_id)
        });
        board.watch_items.retain(|item| {
            item.scope == rosc_broker::ProxyOperatorBoardScope::Global
                || item.route_id.as_deref() == Some(route_id)
        });
    } else if let Some(destination_id) = destination_id {
        board.blocked_items.retain(|item| {
            item.scope == rosc_broker::ProxyOperatorBoardScope::Global
                || item.destination_id.as_deref() == Some(destination_id)
        });
        board.degraded_items.retain(|item| {
            item.scope == rosc_broker::ProxyOperatorBoardScope::Global
                || item.destination_id.as_deref() == Some(destination_id)
        });
        board.watch_items.retain(|item| {
            item.scope == rosc_broker::ProxyOperatorBoardScope::Global
                || item.destination_id.as_deref() == Some(destination_id)
        });
    }

    print_json_pretty(&board)?;
    Ok(())
}

pub(crate) async fn proxy_focus(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-focus accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut focus = dashboard.focus;

    if let Some(route_id) = route_id {
        let Some(route_focus) = focus
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route focus `{route_id}`");
        };
        focus.routes = vec![route_focus];
        focus.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_focus) = focus
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination focus `{destination_id}`");
        };
        focus.routes.clear();
        focus.destinations = vec![destination_focus];
    }

    print_json_pretty(&focus)?;
    Ok(())
}

pub(crate) async fn proxy_lens(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-lens accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut lens = dashboard.lens;

    if let Some(route_id) = route_id {
        let Some(route_lens) = lens
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route lens `{route_id}`");
        };
        lens.routes = vec![route_lens];
        lens.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_lens) = lens
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination lens `{destination_id}`");
        };
        lens.routes.clear();
        lens.destinations = vec![destination_lens];
    }

    print_json_pretty(&lens)?;
    Ok(())
}

pub(crate) async fn proxy_brief(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-brief accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut brief = dashboard.brief;

    if let Some(route_id) = route_id {
        let Some(route_brief) = brief
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route brief `{route_id}`");
        };
        brief.routes = vec![route_brief];
        brief.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_brief) = brief
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination brief `{destination_id}`");
        };
        brief.routes.clear();
        brief.destinations = vec![destination_brief];
    }

    print_json_pretty(&brief)?;
    Ok(())
}

pub(crate) async fn proxy_dossier(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-dossier accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut dossier = dashboard.dossier;

    if let Some(route_id) = route_id {
        let Some(route_dossier) = dossier
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route dossier `{route_id}`");
        };
        dossier.routes = vec![route_dossier];
        dossier.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_dossier) = dossier
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination dossier `{destination_id}`");
        };
        dossier.routes.clear();
        dossier.destinations = vec![destination_dossier];
    }

    print_json_pretty(&dossier)?;
    Ok(())
}

pub(crate) async fn proxy_runbook(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-runbook accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut runbook = dashboard.runbook;

    if let Some(route_id) = route_id {
        let Some(route_runbook) = runbook
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route runbook `{route_id}`");
        };
        runbook.routes = vec![route_runbook];
        runbook.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_runbook) = runbook
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination runbook `{destination_id}`");
        };
        runbook.routes.clear();
        runbook.destinations = vec![destination_runbook];
    }

    print_json_pretty(&runbook)?;
    Ok(())
}

pub(crate) async fn proxy_mission(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-mission accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut mission = dashboard.mission;

    if let Some(route_id) = route_id {
        let Some(route_mission) = mission
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route mission `{route_id}`");
        };
        mission.routes = vec![route_mission];
        mission.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_mission) = mission
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination mission `{destination_id}`");
        };
        mission.routes.clear();
        mission.destinations = vec![destination_mission];
    }

    print_json_pretty(&mission)?;
    Ok(())
}

pub(crate) async fn proxy_workspace(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-workspace accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut workspace = dashboard.workspace;

    if let Some(route_id) = route_id {
        let Some(route_workspace) = workspace
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route workspace `{route_id}`");
        };
        workspace.routes = vec![route_workspace];
        workspace.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_workspace) = workspace
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination workspace `{destination_id}`");
        };
        workspace.routes.clear();
        workspace.destinations = vec![destination_workspace];
    }

    print_json_pretty(&workspace)?;
    Ok(())
}

pub(crate) async fn proxy_cockpit(
    path: &Path,
    resolve_bindings: bool,
    history_limit: Option<usize>,
    route_id: Option<&str>,
    destination_id: Option<&str>,
    options: ProxyCommandOptions,
) -> Result<()> {
    if route_id.is_some() && destination_id.is_some() {
        bail!("proxy-cockpit accepts only one of --route-id or --destination-id");
    }

    let config = load_config(path)?;
    let status =
        status_from_config(&config, resolve_bindings, launch_profile_mode(options)).await?;
    let dashboard =
        rosc_broker::proxy_operator_dashboard(&status, safety_policy(options), history_limit);
    let mut cockpit = dashboard.cockpit;

    if let Some(route_id) = route_id {
        let Some(route_cockpit) = cockpit
            .routes
            .iter()
            .find(|packet| packet.route_id == route_id)
            .cloned()
        else {
            bail!("unknown route cockpit `{route_id}`");
        };
        cockpit.routes = vec![route_cockpit];
        cockpit.destinations.clear();
    } else if let Some(destination_id) = destination_id {
        let Some(destination_cockpit) = cockpit
            .destinations
            .iter()
            .find(|packet| packet.destination_id == destination_id)
            .cloned()
        else {
            bail!("unknown destination cockpit `{destination_id}`");
        };
        cockpit.routes.clear();
        cockpit.destinations = vec![destination_cockpit];
    }

    print_json_pretty(&cockpit)?;
    Ok(())
}
