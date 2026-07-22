use ephemeral_postgres::postgres_image::PostgresImage;

const POSTGRES_TEST_IMAGE_NAME: &str = "postgres";
const POSTGRES_TEST_IMAGE_TAG: &str =
    "18@sha256:3a82e1f56c8f0f5616a11103ac3d47e632c3938698946a7ad26da0df1334744a";

#[must_use]
pub fn postgres_test_image() -> PostgresImage {
    PostgresImage::new(POSTGRES_TEST_IMAGE_NAME, POSTGRES_TEST_IMAGE_TAG)
}
