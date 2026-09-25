use std::time::Duration;

use crate::readiness_timeout::READINESS_TIMEOUT;

pub struct AttachParams {
    pub base_url: String,
    pub readiness_timeout: Duration,
}

impl AttachParams {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            readiness_timeout: READINESS_TIMEOUT,
        }
    }
}
