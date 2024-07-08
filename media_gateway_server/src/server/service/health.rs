use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct HealthState {
    status: HealthStatus,
}

const HEALTHY_STATE: HealthState = HealthState {
    status: HealthStatus::Healthy,
};
pub struct HealthService {}

impl HealthService {
    pub fn new() -> Self {
        HealthService {}
    }
    pub fn current_state(&self) -> HealthState {
        HEALTHY_STATE
    }
}

#[cfg(test)]
mod tests {
    use crate::server::service::health::{HealthService, HEALTHY_STATE};

    #[test]
    pub fn current_state() {
        let service = HealthService {};
        let result = service.current_state();

        assert_eq!(result, HEALTHY_STATE);
    }
}
