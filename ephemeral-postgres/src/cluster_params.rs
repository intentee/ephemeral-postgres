use std::time::Duration;

const POSTGRES_IMAGE_TAG: &str =
    "17@sha256:2a0d0fe14825b0939f78a8cad5cd4e6aa68bf94d0e5dd96e24b6d23af4315545";

const READINESS_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ClusterParams {
    pub image_tag: String,
    pub readiness_timeout: Duration,
}

impl Default for ClusterParams {
    fn default() -> Self {
        Self {
            image_tag: POSTGRES_IMAGE_TAG.to_owned(),
            readiness_timeout: READINESS_TIMEOUT,
        }
    }
}
