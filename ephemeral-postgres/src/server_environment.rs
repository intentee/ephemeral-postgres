use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;

use sqlx::postgres::PgConnectOptions;

use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::server_connect_options::server_connect_options;

const CONFLICTING_VARIABLES: [&str; 12] = [
    "PGAPPNAME",
    "PGDATABASE",
    "PGHOST",
    "PGHOSTADDR",
    "PGOPTIONS",
    "PGPASSWORD",
    "PGPORT",
    "PGSSLCERT",
    "PGSSLKEY",
    "PGSSLMODE",
    "PGSSLROOTCERT",
    "PGUSER",
];
const SERVER_URL_VARIABLE: &str = "EPHEMERAL_POSTGRES_URL";

pub struct ServerEnvironment {
    pub variables: BTreeMap<OsString, OsString>,
}

impl ServerEnvironment {
    #[must_use]
    pub fn current() -> Self {
        Self {
            variables: env::vars_os().collect(),
        }
    }

    pub fn server_connect_options(&self) -> Result<PgConnectOptions, EphemeralPostgresError> {
        let conflicting: Vec<&'static str> = CONFLICTING_VARIABLES
            .into_iter()
            .filter(|variable| self.variables.contains_key(OsStr::new(variable)))
            .collect();

        if !conflicting.is_empty() {
            return Err(EphemeralPostgresError::ConflictingEnvironment {
                variables: conflicting,
            });
        }

        let server_url = self
            .variables
            .get(OsStr::new(SERVER_URL_VARIABLE))
            .ok_or(EphemeralPostgresError::MissingServerUrl {
                variable: SERVER_URL_VARIABLE,
            })?
            .to_str()
            .ok_or(EphemeralPostgresError::ServerUrlNotUnicode {
                variable: SERVER_URL_VARIABLE,
            })?;

        server_connect_options(server_url)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use super::ServerEnvironment;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;

    const SERVER_URL: &str = "postgres://user@db.example:6543/db?sslmode=disable";

    #[test]
    fn reads_the_server_url() {
        let options = ServerEnvironment {
            variables: BTreeMap::from([(
                OsString::from("EPHEMERAL_POSTGRES_URL"),
                OsString::from(SERVER_URL),
            )]),
        }
        .server_connect_options()
        .unwrap();

        assert_eq!(options.get_host(), "db.example");
    }

    #[test]
    fn rejects_libpq_variables_that_would_also_describe_the_server() {
        let result = ServerEnvironment {
            variables: BTreeMap::from([
                (
                    OsString::from("EPHEMERAL_POSTGRES_URL"),
                    OsString::from(SERVER_URL),
                ),
                (OsString::from("PGPASSWORD"), OsString::from("secret")),
                (OsString::from("PGHOST"), OsString::from("elsewhere")),
            ]),
        }
        .server_connect_options();

        assert!(matches!(
            result,
            Err(EphemeralPostgresError::ConflictingEnvironment { variables })
                if variables == vec!["PGHOST", "PGPASSWORD"]
        ));
    }

    #[test]
    fn rejects_a_missing_server_url() {
        assert!(matches!(
            ServerEnvironment {
                variables: BTreeMap::new(),
            }
            .server_connect_options(),
            Err(EphemeralPostgresError::MissingServerUrl { variable })
                if variable == "EPHEMERAL_POSTGRES_URL"
        ));
    }

    #[test]
    fn rejects_a_server_url_that_is_not_unicode() {
        assert!(matches!(
            ServerEnvironment {
                variables: BTreeMap::from([(
                    OsString::from("EPHEMERAL_POSTGRES_URL"),
                    OsString::from_vec(vec![0xff]),
                )]),
            }
            .server_connect_options(),
            Err(EphemeralPostgresError::ServerUrlNotUnicode { variable })
                if variable == "EPHEMERAL_POSTGRES_URL"
        ));
    }
}
