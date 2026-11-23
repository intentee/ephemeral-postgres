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
    use sqlx::ConnectOptions;
    use sqlx::postgres::PgSslMode;
    use url::ParseError;

    use ephemeral_postgres_tests::test_result::TestResult;
    use ephemeral_postgres_tests::verify::verify;

    use super::server_connect_options;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;

    #[test]
    fn takes_every_part_from_the_url() -> TestResult {
        let options = server_connect_options(
            "postgres://us%40er@db.example:6543/my%20db?sslmode=verify-full",
        )?;

        verify(options.get_host() == "db.example")?;
        verify(options.get_port() == 6543)?;
        verify(options.get_username() == "us@er")?;
        verify(options.get_database() == Some("my db"))?;
        verify(matches!(options.get_ssl_mode(), PgSslMode::VerifyFull))?;

        Ok(())
    }

    #[test]
    fn decodes_the_password() -> TestResult {
        let options =
            server_connect_options("postgres://user:se%2Fcret@db.example:6543/db?sslmode=disable")?;

        verify(options.to_url_lossy().password() == Some("se%2Fcret"))?;

        Ok(())
    }

    #[test]
    fn rejects_an_unparsable_url() -> TestResult {
        verify(matches!(
            server_connect_options("not a url"),
            Err(EphemeralPostgresError::UnparsableServerUrl { source })
                if source == ParseError::RelativeUrlWithoutBase
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_another_scheme() -> TestResult {
        verify(matches!(
            server_connect_options("postgresql://user@db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::UnsupportedScheme { scheme }) if scheme == "postgresql"
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_fragment() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/db?sslmode=disable#part"),
            Err(EphemeralPostgresError::UnexpectedFragment)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_missing_host() -> TestResult {
        verify(matches!(
            server_connect_options("postgres:///db?sslmode=disable"),
            Err(EphemeralPostgresError::MissingHost)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_missing_port() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example/db?sslmode=disable"),
            Err(EphemeralPostgresError::MissingPort)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_missing_username() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::MissingUsername)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_username_that_is_not_utf8() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://%FF@db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::UsernameNotUnicode { source }) if source.valid_up_to() == 0
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_password_that_is_not_utf8() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user:%FF@db.example:6543/db?sslmode=disable"),
            Err(EphemeralPostgresError::PasswordNotUnicode { source }) if source.valid_up_to() == 0
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_missing_database() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/?sslmode=disable"),
            Err(EphemeralPostgresError::MissingDatabase)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_path_with_several_segments() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/db/extra?sslmode=disable"),
            Err(EphemeralPostgresError::AmbiguousDatabasePath { path }) if path == "/db/extra"
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_database_name_that_is_not_utf8() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/%FF?sslmode=disable"),
            Err(EphemeralPostgresError::DatabaseNameNotUnicode { source }) if source.valid_up_to() == 0
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_an_unexpected_query_parameter() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/db?sslmode=disable&application_name=tests"),
            Err(EphemeralPostgresError::UnexpectedQueryParameter { name }) if name == "application_name"
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_missing_ssl_mode() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/db"),
            Err(EphemeralPostgresError::MissingSslMode)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_a_repeated_ssl_mode() -> TestResult {
        verify(matches!(
            server_connect_options(
                "postgres://user@db.example:6543/db?sslmode=disable&sslmode=disable"
            ),
            Err(EphemeralPostgresError::RepeatedSslMode)
        ))?;

        Ok(())
    }

    #[test]
    fn rejects_an_ssl_mode_libpq_does_not_spell_that_way() -> TestResult {
        verify(matches!(
            server_connect_options("postgres://user@db.example:6543/db?sslmode=DISABLE"),
            Err(EphemeralPostgresError::UnknownSslMode { value }) if value == "DISABLE"
        ))?;

        Ok(())
    }
}
