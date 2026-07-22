use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ContainerRequest;
use testcontainers_modules::testcontainers::ImageExt;

pub struct PostgresImage {
    pub name: String,
    pub tag: String,
}

impl PostgresImage {
    #[must_use]
    pub fn new(name: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tag: tag.into(),
        }
    }

    #[doc(hidden)]
    pub fn into_container_request(self) -> ContainerRequest<Postgres> {
        Postgres::default()
            .with_host_auth()
            .with_name(self.name)
            .with_tag(self.tag)
    }
}
