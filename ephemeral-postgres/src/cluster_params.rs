use std::time::Duration;

use crate::postgres_image::PostgresImage;

const READINESS_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ClusterParams {
    pub image: PostgresImage,
    pub readiness_timeout: Duration,
}

impl ClusterParams {
    #[must_use]
    pub fn new(image: PostgresImage) -> Self {
        Self {
            image,
            readiness_timeout: READINESS_TIMEOUT,
        }
    }
}
