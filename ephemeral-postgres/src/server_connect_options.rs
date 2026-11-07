use std::borrow::Cow;
use std::str::Utf8Error;

use percent_encoding::percent_decode_str;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use url::Url;

use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::ssl_mode_spelling::SslModeSpelling;

const SCHEME: &str = "postgres";
const SSL_MODE_PARAMETER: &str = "sslmode";
const SSL_MODE_SPELLINGS: [SslModeSpelling; 6] = [
    SslModeSpelling {
        mode: PgSslMode::Allow,
        spelling: "allow",
    },
    SslModeSpelling {
        mode: PgSslMode::Disable,
        spelling: "disable",
    },
    SslModeSpelling {
        mode: PgSslMode::Prefer,
        spelling: "prefer",
    },
    SslModeSpelling {
        mode: PgSslMode::Require,
        spelling: "require",
    },
    SslModeSpelling {
        mode: PgSslMode::VerifyCa,
        spelling: "verify-ca",
    },
    SslModeSpelling {
        mode: PgSslMode::VerifyFull,
        spelling: "verify-full",
    },
];

fn decoded(encoded: &str) -> Result<Cow<'_, str>, Utf8Error> {
    percent_decode_str(encoded).decode_utf8()
}

fn database(server_url: &Url) -> Result<Cow<'_, str>, EphemeralPostgresError> {
    let segments: Vec<&str> = server_url
        .path_segments()
        .map_or_else(Vec::new, Iterator::collect);

    match segments.as_slice() {
        [] | [""] => Err(EphemeralPostgresError::MissingDatabase),
        [segment] => decoded(segment)
            .map_err(|source| EphemeralPostgresError::DatabaseNameNotUnicode { source }),
        _ => Err(EphemeralPostgresError::AmbiguousDatabasePath {
            path: server_url.path().to_owned(),
        }),
    }
}

fn host(server_url: &Url) -> Result<&str, EphemeralPostgresError> {
    match server_url.host_str() {
        None | Some("") => Err(EphemeralPostgresError::MissingHost),
        Some(host) => Ok(host),
    }
}

fn password(server_url: &Url) -> Result<Option<Cow<'_, str>>, EphemeralPostgresError> {
    server_url
        .password()
        .map(|encoded| {
            decoded(encoded).map_err(|source| EphemeralPostgresError::PasswordNotUnicode { source })
        })
        .transpose()
}

fn ssl_mode(server_url: &Url) -> Result<PgSslMode, EphemeralPostgresError> {
    let mut values = Vec::new();

    for (name, value) in server_url.query_pairs() {
        if name != SSL_MODE_PARAMETER {
            return Err(EphemeralPostgresError::UnexpectedQueryParameter {
                name: name.into_owned(),
            });
        }

        values.push(value);
    }

    match values.as_slice() {
        [] => Err(EphemeralPostgresError::MissingSslMode),
        [value] => SSL_MODE_SPELLINGS
            .into_iter()
            .find(|SslModeSpelling { spelling, .. }| spelling == value)
            .map(|SslModeSpelling { mode, .. }| mode)
            .ok_or_else(|| EphemeralPostgresError::UnknownSslMode {
                value: value.clone().into_owned(),
            }),
        _ => Err(EphemeralPostgresError::RepeatedSslMode),
    }
}

fn username(server_url: &Url) -> Result<Cow<'_, str>, EphemeralPostgresError> {
    match server_url.username() {
        "" => Err(EphemeralPostgresError::MissingUsername),
        encoded => {
            decoded(encoded).map_err(|source| EphemeralPostgresError::UsernameNotUnicode { source })
        }
    }
}

pub fn server_connect_options(
    server_url: &str,
) -> Result<PgConnectOptions, EphemeralPostgresError> {
    let parsed = Url::parse(server_url)
        .map_err(|source| EphemeralPostgresError::UnparsableServerUrl { source })?;

    if parsed.scheme() != SCHEME {
        return Err(EphemeralPostgresError::UnsupportedScheme {
            scheme: parsed.scheme().to_owned(),
        });
    }

    if parsed.fragment().is_some() {
        return Err(EphemeralPostgresError::UnexpectedFragment);
    }

    let options = PgConnectOptions::new_without_pgpass()
        .host(host(&parsed)?)
        .port(parsed.port().ok_or(EphemeralPostgresError::MissingPort)?)
        .username(&username(&parsed)?)
        .database(&database(&parsed)?)
        .ssl_mode(ssl_mode(&parsed)?);

    Ok(match password(&parsed)? {
        Some(password) => options.password(&password),
        None => options,
    })
}

#[cfg(test)]
mod tests {
    use std::mem::discriminant;

    use sqlx::ConnectOptions;
    use sqlx::postgres::PgSslMode;
    use url::ParseError;

    use super::server_connect_options;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;

    #[test]
    fn takes_every_part_from_the_url() {
        let options = server_connect_options(
            "postgres://us%40er@db.example:6543/my%20db?sslmode=verify-full",
        )
        .unwrap();

        assert_eq!(options.get_host(), "db.example");
        assert_eq!(options.get_port(), 6543);
        assert_eq!(options.get_username(), "us@er");
        assert_eq!(options.get_database(), Some("my db"));
        assert_eq!(
            discriminant(&options.get_ssl_mode()),
            discriminant(&PgSslMode::VerifyFull)
        );
    }

    #[test]
    fn decodes_the_password() {
        let options =
            server_connect_options("postgres://user:se%2Fcret@db.example:6543/db?sslmode=disable")
                .unwrap();

        assert_eq!(options.to_url_lossy().password(), Some("se%2Fcret"));
    }

    #[test]
    fn rejects_an_unparsable_url() {
        assert!(matches!(
            server_connect_options("not a url"),
            Err(EphemeralPostgresError::UnparsableServerUrl { source })
                if source == ParseError::RelativeUrlWithoutBase
        ));
    }

    #[test]
    fn rejects_another_scheme() {
        assert!(matches!(
            server_connect_options("postgresql://user@db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::UnsupportedScheme { scheme }) if scheme == "postgresql"
        ));
    }

    #[test]
    fn rejects_a_fragment() {
        assert_eq!(
            discriminant(
                &server_connect_options("postgres://user@db.example:6543/db?sslmode=disable#part")
                    .err()
                    .unwrap()
            ),
            discriminant(&EphemeralPostgresError::UnexpectedFragment)
        );
    }

    #[test]
    fn rejects_a_missing_host() {
        assert_eq!(
            discriminant(
                &server_connect_options("postgres:///db?sslmode=disable")
                    .err()
                    .unwrap()
            ),
            discriminant(&EphemeralPostgresError::MissingHost)
        );
    }

    #[test]
    fn rejects_a_missing_port() {
        assert_eq!(
            discriminant(
                &server_connect_options("postgres://user@db.example/db?sslmode=disable")
                    .err()
                    .unwrap()
            ),
            discriminant(&EphemeralPostgresError::MissingPort)
        );
    }

    #[test]
    fn rejects_a_missing_username() {
        assert_eq!(
            discriminant(
                &server_connect_options("postgres://db.example:6543/db?sslmode=disable")
                    .err()
                    .unwrap()
            ),
            discriminant(&EphemeralPostgresError::MissingUsername)
        );
    }

    #[test]
    fn rejects_a_username_that_is_not_utf8() {
        assert!(matches!(
            server_connect_options("postgres://%FF@db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::UsernameNotUnicode { source }) if source.valid_up_to() == 0
        ));
    }

    #[test]
    fn rejects_a_password_that_is_not_utf8() {
        assert!(matches!(
            server_connect_options("postgres://user:%FF@db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::PasswordNotUnicode { source }) if source.valid_up_to() == 0
        ));
    }

    #[test]
    fn rejects_a_missing_database() {
        assert_eq!(
            discriminant(
                &server_connect_options("postgres://user@db.example:6543/?sslmode=disable")
                    .err()
                    .unwrap()
            ),
            discriminant(&EphemeralPostgresError::MissingDatabase)
        );
    }

    #[test]
    fn rejects_a_path_with_several_segments() {
        assert!(matches!(
            server_connect_options("postgres://user@db.example:6543/db/extra?sslmode=disable"),
            Err(EphemeralPostgresError::AmbiguousDatabasePath { path }) if path == "/db/extra"
        ));
    }

    #[test]
    fn rejects_a_database_name_that_is_not_utf8() {
        assert!(matches!(
            server_connect_options("postgres://user@db.example:6543/%FF?sslmode=disable"),
            Err(EphemeralPostgresError::DatabaseNameNotUnicode { source }) if source.valid_up_to() == 0
        ));
    }

    #[test]
    fn rejects_an_unexpected_query_parameter() {
        assert!(matches!(
            server_connect_options(
                "postgres://user@db.example:6543/db?sslmode=disable&application_name=tests"
            ),
            Err(EphemeralPostgresError::UnexpectedQueryParameter { name }) if name == "application_name"
        ));
    }

    #[test]
    fn rejects_a_missing_ssl_mode() {
        assert_eq!(
            discriminant(
                &server_connect_options("postgres://user@db.example:6543/db")
                    .err()
                    .unwrap()
            ),
            discriminant(&EphemeralPostgresError::MissingSslMode)
        );
    }

    #[test]
    fn rejects_a_repeated_ssl_mode() {
        assert_eq!(
            discriminant(
                &server_connect_options(
                    "postgres://user@db.example:6543/db?sslmode=disable&sslmode=disable"
                )
                .err()
                .unwrap()
            ),
            discriminant(&EphemeralPostgresError::RepeatedSslMode)
        );
    }

    #[test]
    fn rejects_an_ssl_mode_libpq_does_not_spell_that_way() {
        assert!(matches!(
            server_connect_options("postgres://user@db.example:6543/db?sslmode=DISABLE"),
            Err(EphemeralPostgresError::UnknownSslMode { value }) if value == "DISABLE"
        ));
    }
}
