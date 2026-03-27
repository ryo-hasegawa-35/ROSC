use super::types::{ControlPlaneActionResult, ControlPlaneError};
use crate::UdpProxyStatusSnapshot;

pub(super) fn status_result(
    applied: bool,
    status: UdpProxyStatusSnapshot,
) -> ControlPlaneActionResult {
    ControlPlaneActionResult {
        applied,
        dispatch_count: None,
        status,
    }
}

pub(super) fn dispatch_result(
    dispatch_count: usize,
    status: UdpProxyStatusSnapshot,
) -> ControlPlaneActionResult {
    ControlPlaneActionResult {
        applied: dispatch_count > 0,
        dispatch_count: Some(dispatch_count),
        status,
    }
}

pub(super) fn ensure_route_exists(exists: bool, route_id: &str) -> Result<(), ControlPlaneError> {
    if exists {
        Ok(())
    } else {
        Err(ControlPlaneError::UnknownRoute(route_id.to_owned()))
    }
}

pub(super) fn ensure_destination_exists(
    exists: bool,
    destination_id: &str,
) -> Result<(), ControlPlaneError> {
    if exists {
        Ok(())
    } else {
        Err(ControlPlaneError::UnknownDestination(
            destination_id.to_owned(),
        ))
    }
}
