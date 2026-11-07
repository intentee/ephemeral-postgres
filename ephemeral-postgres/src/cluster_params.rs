use std::time::Duration;

use crate::constants::READINESS_TIMEOUT;
use crate::postgres_image::PostgresImage;

pub struct ClusterParams {
    pub image: PostgresImage,
    pub readiness_timeout: Duration,
}

impl ClusterParams {
    #[must_use]
    pub const fn new(image: PostgresImage) -> Self {
        Self {
            image,
            readiness_timeout: READINESS_TIMEOUT,
        }
    }
}
