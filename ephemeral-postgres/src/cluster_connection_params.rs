use std::time::Duration;

use crate::readiness_timeout::READINESS_TIMEOUT;

pub struct ClusterConnectionParams {
    pub base_url: String,
    pub readiness_timeout: Duration,
}

impl ClusterConnectionParams {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            readiness_timeout: READINESS_TIMEOUT,
        }
    }
}
