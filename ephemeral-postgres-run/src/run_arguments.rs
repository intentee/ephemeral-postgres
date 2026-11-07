use ephemeral_postgres::postgres_image::PostgresImage;

use crate::ephemeral_postgres_run_error::EphemeralPostgresRunError;

const SEPARATOR: &str = "--";

pub struct RunArguments {
    pub command: String,
    pub command_arguments: Vec<String>,
    pub image: PostgresImage,
}

impl RunArguments {
    pub fn parse(arguments: Vec<String>) -> Result<Self, EphemeralPostgresRunError> {
        let mut arguments = arguments.into_iter();
        let image_name = arguments
            .next()
            .ok_or(EphemeralPostgresRunError::MissingImageName)?;
        let image_tag = arguments
            .next()
            .ok_or(EphemeralPostgresRunError::MissingImageTag)?;
        let separator = arguments
            .next()
            .ok_or(EphemeralPostgresRunError::MissingSeparator)?;

        if separator != SEPARATOR {
            return Err(EphemeralPostgresRunError::UnexpectedArgument { found: separator });
        }

        let command = arguments
            .next()
            .ok_or(EphemeralPostgresRunError::MissingCommand)?;

        Ok(Self {
            command,
            command_arguments: arguments.collect(),
            image: PostgresImage::new(image_name, image_tag),
        })
    }
}
