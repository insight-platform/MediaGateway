use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum HealthStatus {
    HEALTHY,
}

#[derive(Debug, Serialize)]
pub struct HealthState {
    status: HealthStatus,
}

const HEALTHY_STATE: HealthState = HealthState {
    status: HealthStatus::HEALTHY,
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
