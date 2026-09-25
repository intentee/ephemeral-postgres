use crate::postgres_container::PostgresContainer;

#[doc(hidden)]
pub enum ClusterServer {
    Container(Box<PostgresContainer>),
    External,
}
