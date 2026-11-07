use std::sync::Arc;

use crate::postgres_container::PostgresContainer;

#[doc(hidden)]
#[derive(Clone)]
pub enum ClusterServer {
    Container(Arc<PostgresContainer>),
    External,
}
